//! Database Mimicry Transport
//!
//! Wraps traffic in PostgreSQL Wire Protocol frames to bypass DPI.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// State of the PostgreSQL handshake simulation
#[derive(Debug, PartialEq, Clone, Copy)]
enum PgState {
    Initial,
    StartupReceived,
    AuthChallengeSent,
    PasswordReceived,
    AuthOkSent,
    ReadyForQuerySent,
    Established,
}

/// Wraps an underlying stream with PostgreSQL framing
pub struct DbMimicStream<S> {
    inner: S,
    state: PgState,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
    decrypted_buf: Vec<u8>, // Buffer for unwrapped data awaiting read
}

impl<S> DbMimicStream<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            state: PgState::Initial,
            read_buf: Vec::with_capacity(4096),
            write_buf: Vec::with_capacity(4096),
            decrypted_buf: Vec::with_capacity(4096),
        }
    }

    /// Wraps data in a PostgreSQL DataRow frame (Type 'D')
    fn wrap_data_row(payload: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(1 + 4 + 2 + 4 + payload.len());
        frame.push(b'D'); // Message Type: DataRow

        let total_len = 4 + 2 + 4 + payload.len() as i32;
        frame.extend_from_slice(&total_len.to_be_bytes()); // Length

        frame.extend_from_slice(&1i16.to_be_bytes()); // Column Count: 1

        frame.extend_from_slice(&(payload.len() as i32).to_be_bytes()); // Column Length
        frame.extend_from_slice(payload); // Column Data

        frame
    }

    /// Unwraps a PostgreSQL DataRow frame
    fn unwrap_data_row(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
        if buf.len() < 5 {
            return None;
        }

        if buf[0] != b'D' {
            return None;
        }

        let len_bytes: [u8; 4] = buf[1..5].try_into().unwrap();
        let msg_len = i32::from_be_bytes(len_bytes) as usize;

        if buf.len() < 1 + msg_len {
            return None; // Incomplete frame
        }

        // Column 1 Length
        let col_len_bytes: [u8; 4] = buf[7..11].try_into().unwrap();
        let col_len = i32::from_be_bytes(col_len_bytes);

        let payload = if col_len == -1 {
            Vec::new() // NULL
        } else {
            let start = 11;
            let end = 11 + col_len as usize;
            buf[start..end].to_vec()
        };

        // Consume frame
        buf.drain(0..(1 + msg_len));

        Some(payload)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for DbMimicStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // 1. Drain any previously decrypted data
        if !self.decrypted_buf.is_empty() {
            let len = std::cmp::min(buf.remaining(), self.decrypted_buf.len());
            buf.put_slice(&self.decrypted_buf[0..len]);
            self.decrypted_buf.drain(0..len);
            return Poll::Ready(Ok(()));
        }

        let mut inner_buf = [0u8; 4096];
        let mut read_buf = ReadBuf::new(&mut inner_buf);

        // 2. Poll inner stream
        match Pin::new(&mut self.inner).poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                let filled = read_buf.filled();
                if filled.is_empty() {
                    return Poll::Ready(Ok(())); // EOF
                }

                self.read_buf.extend_from_slice(filled);

                // Process buffer based on state
                match self.state {
                    PgState::Established => {
                        // Attempt to unwrap DataRows
                        while let Some(payload) = Self::unwrap_data_row(&mut self.read_buf) {
                            self.decrypted_buf.extend_from_slice(&payload);
                        }
                    }
                    _ => {
                        // Handle Handshake - simulated consumption
                        self.state = PgState::Established;

                        if let Some(pos) = self.read_buf.iter().position(|&c| c == b'D') {
                            self.read_buf.drain(0..pos);
                            while let Some(payload) = Self::unwrap_data_row(&mut self.read_buf) {
                                self.decrypted_buf.extend_from_slice(&payload);
                            }
                        } else {
                            self.read_buf.clear();
                        }

                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                }

                // If we have data now, fill buffer
                if !self.decrypted_buf.is_empty() {
                    let len = std::cmp::min(buf.remaining(), self.decrypted_buf.len());
                    buf.put_slice(&self.decrypted_buf[0..len]);
                    self.decrypted_buf.drain(0..len);
                } else {
                    // We read data from inner, processed it (maybe consumed partial frame),
                    // but produced no output yet.
                    // Return Pending to read more?
                    // If inner returned Ready(Ok(n>0)), we should probably signal we are ready for more.
                    // But standard AsyncRead says: "If n=0, EOF. If n>0, data."
                    // If we return Ready(Ok(())) with empty buf, it's EOF.
                    // So we MUST return Pending if we have no data but inner is not EOF.
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }

                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for DbMimicStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let frame = Self::wrap_data_row(buf);
        match Pin::new(&mut self.inner).poll_write(cx, &frame) {
            Poll::Ready(Ok(n)) => {
                if n == frame.len() {
                    Poll::Ready(Ok(buf.len()))
                } else {
                    Poll::Ready(Ok(0))
                }
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
