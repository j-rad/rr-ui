use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rr_ui::models::{
    InboundModel, OutboundModel, OutboundSettings, ProtocolSettings, Sniffing, StreamSettings,
    TailscaleSettings, TunConfig, VlessSettings,
};
use rr_ui::xray_config::XrayConfigBuilder;

fn bench_vless_serialization(c: &mut Criterion) {
    let vless = ProtocolSettings::Vless(VlessSettings {
        clients: vec![],
        decryption: Some("none".to_string().into()),
        fallbacks: None,
    });

    c.bench_function("serialize_vless", |b| {
        b.iter(|| serde_json::to_string(black_box(&vless)).unwrap())
    });
}

fn bench_tun_serialization(c: &mut Criterion) {
    let tun = ProtocolSettings::Tun(TunConfig {
        enable: true,
        interface: "tun0".to_string().into(),
        mtu: 1500,
        strict_route: true,
        stack: "system".to_string().into(),
        ..Default::default()
    });

    c.bench_function("serialize_tun", |b| {
        b.iter(|| serde_json::to_string(black_box(&tun)).unwrap())
    });
}

fn bench_full_config_generation(c: &mut Criterion) {
    let tun_inbound = InboundModel {
        id: None,
        up: 0,
        down: 0,
        total: 0,
        remark: "Test Tun".to_string().into(),
        enable: true,
        expiry: 0,
        port: 0,
        protocol: "tun".to_string().into(),
        settings: ProtocolSettings::Tun(TunConfig {
            enable: true,
            interface: "tun0".to_string().into(),
            mtu: 1500,
            ..Default::default()
        }),
        stream_settings: StreamSettings::default(),
        tag: "tun-in".to_string().into(),
        sniffing: Sniffing::default(),
    };

    let tailscale_outbound = OutboundModel {
        id: None,
        remark: "Test Tailscale".to_string().into(),
        enable: true,
        protocol: "tailscale".to_string().into(),
        settings: OutboundSettings::Tailscale(TailscaleSettings {
            auth_key: "tskey-123".to_string().into(),
            hostname: "restray-node".to_string().into(),
            ephemeral: true,
            exit_node: false,
            accept_routes: true,
        }),
        stream_settings: StreamSettings::default(),
        tag: "tailscale-out".to_string().into(),
        mux: None,
    };

    // We benchmark the construction AND serialization of the full config
    c.bench_function("build_and_serialize_full_config", |b| {
        b.iter(|| {
            let inbounds = black_box(vec![tun_inbound.clone()]);
            let outbounds = black_box(vec![tailscale_outbound.clone()]);
            let config = XrayConfigBuilder::build_from_models(&inbounds, &outbounds);
            serde_json::to_string(&config).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_vless_serialization,
    bench_tun_serialization,
    bench_full_config_generation
);
criterion_main!(benches);
