use actix::prelude::*;
use actix_web::web::Bytes;
use actix_web_actors::ws;
use log::warn;
use std::time::{Duration, Instant};
use tokio::sync::broadcast::Receiver;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsTelemetrySession {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,
    pub rx: Option<Receiver<Bytes>>,
}

impl WsTelemetrySession {
    pub fn new(rx: Receiver<Bytes>) -> Self {
        Self {
            hb: Instant::now(),
            rx: Some(rx),
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                warn!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsTelemetrySession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        if let Some(rx) = self.rx.take() {
            use tokio_stream::wrappers::BroadcastStream;
            let stream = BroadcastStream::new(rx);
            ctx.add_stream(stream);
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}

/// Handle messages from the broadcast stream
impl StreamHandler<Result<Bytes, tokio_stream::wrappers::errors::BroadcastStreamRecvError>>
    for WsTelemetrySession
{
    fn handle(
        &mut self,
        msg: Result<Bytes, tokio_stream::wrappers::errors::BroadcastStreamRecvError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(bytes) => {
                // Try to send as text if UTF-8, otherwise binary
                if let Ok(text) = std::str::from_utf8(&bytes) {
                    ctx.text(text);
                } else {
                    ctx.binary(bytes);
                }
            }
            Err(e) => warn!("Broadcast stream error: {}", e),
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsTelemetrySession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
