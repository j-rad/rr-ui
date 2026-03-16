use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rr_ui::models::{InboundModel, ProtocolSettings, Sniffing, StreamSettings, VlessSettings};
use rr_ui::xray_config::XrayConfigBuilder;
use std::borrow::Cow;

fn bench_config_allocations(c: &mut Criterion) {
    // Construct a representative inbound model using Cow::Borrowed where possible
    // to simulate the Zero-Copy optimization
    let inbound = InboundModel {
        tag: Cow::Borrowed("inbound-benchmark"),
        port: 443,
        // listen removed
        protocol: Cow::Borrowed("vless"),
        settings: ProtocolSettings::Vless(VlessSettings {
            clients: vec![],
            decryption: Some(Cow::Borrowed("none")),
            fallbacks: None,
        }),
        stream_settings: StreamSettings {
            network: Cow::Borrowed("tcp"),
            security: Cow::Borrowed("none"),
            tls_settings: None,
            reality_settings: None,
            tcp_settings: None,
            kcp_settings: None,
            ws_settings: None,
            grpc_settings: None,
            http_settings: None,
            mqtt_settings: None, // Added
        },
        sniffing: Sniffing::default(),
        enable: true,
        expiry: 0,
        remark: Cow::Borrowed("Benchmark Remark"),
        id: None,
        up: 0,
        down: 0,
        total: 0,
    };

    let mut group = c.benchmark_group("allocation_check");

    group.bench_function("config_serialization", |b| {
        b.iter(|| {
            let inbounds = vec![inbound.clone()];
            let outbounds = vec![];

            // Build the XrayConfig (Zero-Copy internal references)
            let config =
                XrayConfigBuilder::build_from_models(black_box(&inbounds), black_box(&outbounds));

            // Serialize to Bytes (this triggers the format calls, but should be minimal allocs for structure)
            let _bytes = serde_json::to_vec(&config).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_config_allocations);
criterion_main!(benches);
