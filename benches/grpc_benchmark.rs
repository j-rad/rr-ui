// benches/grpc_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;

// Benchmark JSON serialization of different system state sizes
fn benchmark_system_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_serialization");

    // Small response (minimal data)
    let small_status = json!({
        "cpu": 12.5,
        "mem": { "current": 512_000_000u64, "total": 1_073_741_824u64 },
        "disk": { "current": 10_737_418_240u64, "total": 107_374_182_400u64 },
        "uptime": 123456,
        "loads": [0.5, 0.3, 0.1]
    });

    // Large response (with many inbounds)
    let mut large_status = json!({
        "cpu": 12.5,
        "mem": { "current": 512_000_000u64, "total": 1_073_741_824u64 },
        "disk": { "current": 10_737_418_240u64, "total": 107_374_182_400u64 },
        "uptime": 123456,
        "loads": [0.5, 0.3, 0.1],
        "inbounds": []
    });

    // Add 100 mock inbounds
    if let Some(inbounds) = large_status
        .get_mut("inbounds")
        .and_then(|v| v.as_array_mut())
    {
        for i in 0..100 {
            inbounds.push(json!({
                "tag": format!("inbound-{}", i),
                "port": 10000 + i,
                "protocol": "vless",
                "up": 1024 * 1024 * i,
                "down": 1024 * 1024 * i * 2
            }));
        }
    }

    group.bench_function("small_response", |b| {
        b.iter(|| serde_json::to_string(black_box(&small_status)).unwrap())
    });

    group.bench_function("large_response", |b| {
        b.iter(|| serde_json::to_string(black_box(&large_status)).unwrap())
    });

    group.finish();
}

// Simulate traffic stat processing with different volumes
#[derive(Clone)]
struct MockStat {
    name: String,
    value: i64,
}

fn process_traffic_stats(stats: &[MockStat]) -> (Vec<(String, i64, i64)>, Vec<(String, i64, i64)>) {
    use regex::Regex;

    let traffic_regex =
        Regex::new(r"(inbound|outbound)>>>([^>]+)>>>traffic>>>(downlink|uplink)").unwrap();
    let user_regex = Regex::new(r"user>>>([^>]+)>>>traffic>>>(downlink|uplink)").unwrap();

    let mut inbound_results = Vec::new();
    let mut user_results = Vec::new();

    for stat in stats {
        if let Some(caps) = traffic_regex.captures(&stat.name) {
            let tag = caps[2].to_string();
            let is_up = &caps[3] == "uplink";
            inbound_results.push((
                tag,
                if is_up { stat.value } else { 0 },
                if !is_up { stat.value } else { 0 },
            ));
        } else if let Some(caps) = user_regex.captures(&stat.name) {
            let email = caps[1].to_string();
            let is_up = &caps[2] == "uplink";
            user_results.push((
                email,
                if is_up { stat.value } else { 0 },
                if !is_up { stat.value } else { 0 },
            ));
        }
    }

    (inbound_results, user_results)
}

fn benchmark_traffic_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic_processing");

    for size in [10, 50, 100, 500].iter() {
        let mut stats = Vec::new();
        for i in 0..*size {
            stats.push(MockStat {
                name: format!("inbound>>>tag{}>>>traffic>>>downlink", i),
                value: 1000 + i as i64,
            });
            stats.push(MockStat {
                name: format!("inbound>>>tag{}>>>traffic>>>uplink", i),
                value: 500 + i as i64,
            });
            stats.push(MockStat {
                name: format!("user>>>user{}@example.com>>>traffic>>>downlink", i),
                value: 2000 + i as i64,
            });
            stats.push(MockStat {
                name: format!("user>>>user{}@example.com>>>traffic>>>uplink", i),
                value: 1000 + i as i64,
            });
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), &stats, |b, stats| {
            b.iter(|| process_traffic_stats(black_box(stats)))
        });
    }

    group.finish();
}

// Benchmark regex compilation (should be done once, not per-request)
fn benchmark_regex_compilation(c: &mut Criterion) {
    c.bench_function("regex_compile_traffic", |b| {
        b.iter(|| {
            regex::Regex::new(black_box(
                r"(inbound|outbound)>>>([^>]+)>>>traffic>>>(downlink|uplink)",
            ))
            .unwrap()
        })
    });
}

criterion_group!(
    benches,
    benchmark_system_serialization,
    benchmark_traffic_processing,
    benchmark_regex_compilation
);
criterion_main!(benches);
