#![cfg(feature = "server")]

use rr_ui::transport::db_mimic::DbMimicStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};

#[tokio::test]
async fn test_db_mimic_datarow_wrapping() {
    let (client, mut server) = duplex(1024);
    let mut mimic_client = DbMimicStream::new(client);

    tokio::spawn(async move {
        mimic_client.write_all(b"HELLO").await.unwrap();
    });

    // 1 (Type) + 4 (Len) + 2 (Cols) + 4 (ColLen) + 5 (Data) = 16
    let mut buf = [0u8; 16];
    server.read_exact(&mut buf).await.unwrap();

    assert_eq!(buf[0], b'D');
    assert_eq!(buf[4], 15);
    assert_eq!(&buf[11..], b"HELLO");
}

#[tokio::test]
async fn test_db_mimic_datarow_unwrapping() {
    let (client, mut server) = duplex(1024);
    let mut mimic_client = DbMimicStream::new(client);

    let frame = vec![
        0x44, // 'D'
        0x00, 0x00, 0x00, 0x0F, // Length 15
        0x00, 0x01, // 1 Column
        0x00, 0x00, 0x00, 0x05, // Column Length 5
        b'W', b'O', b'R', b'L', b'D', // Data
    ];

    tokio::spawn(async move {
        // 1. Send dummy handshake to transition state
        server.write_all(b"handshake").await.unwrap();
        // 2. Send DataRow
        server.write_all(&frame).await.unwrap();
    });

    // Wait for the simulated handshake consumption. The stream will yield the wrapped data immediately.

    // 2. Read actual data
    let mut buf = [0u8; 5];
    mimic_client.read_exact(&mut buf).await.unwrap();

    assert_eq!(&buf, b"WORLD");
}

#[tokio::test]
async fn test_grpc_health_check_integration() {
    use rr_ui::rustray_client::RustRayClient;
    use rr_ui::rustray_client::grpc_health::{
        HealthCheckRequest, HealthCheckResponse,
        health_check_response::ServingStatus,
        health_server::{Health, HealthServer},
    };
    use std::net::SocketAddr;
    use tonic::{Request, Response, Status, transport::Server};

    struct MockHealth;

    #[tonic::async_trait]
    impl Health for MockHealth {
        type WatchStream = tokio_stream::wrappers::ReceiverStream<
            std::result::Result<HealthCheckResponse, Status>,
        >;

        async fn check(
            &self,
            _request: Request<HealthCheckRequest>,
        ) -> std::result::Result<Response<HealthCheckResponse>, Status> {
            Ok(Response::new(HealthCheckResponse {
                status: ServingStatus::Serving as i32,
            }))
        }

        async fn watch(
            &self,
            _request: Request<HealthCheckRequest>,
        ) -> std::result::Result<Response<Self::WatchStream>, Status> {
            Err(Status::unimplemented("Not implemented"))
        }
    }

    // Start a mock server on a random port
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = std::net::TcpListener::bind(addr).unwrap();
    let local_addr = listener.local_addr().unwrap();
    drop(listener); // Close it so tonic can bind

    let server_handle = tokio::spawn(async move {
        Server::builder()
            .add_service(HealthServer::new(MockHealth))
            .serve(local_addr)
            .await
            .unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = RustRayClient::new(local_addr.port());
    client
        .connect_with_retry(3)
        .await
        .expect("Failed to connect to mock health server");

    // Perform health check
    let result = client.check_health().await;
    assert!(
        result.is_ok(),
        "Health check should succeed: {:?}",
        result.err()
    );

    server_handle.abort();
}
