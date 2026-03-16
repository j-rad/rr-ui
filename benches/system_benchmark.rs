// benches/system_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

fn benchmark_system_serialization(c: &mut Criterion) {
    let status = json!({
        "cpu": 12.5,
        "mem": { "current": 1024 * 1024 * 512, "total": 1024 * 1024 * 1024 },
        "disk": { "current": 1024u64 * 1024 * 1024 * 10, "total": 1024u64 * 1024 * 1024 * 100 },
        "uptime": 123456,
        "loads": [0.5, 0.3, 0.1]
    });

    c.bench_function("serialize_system_state", |b| {
        b.iter(|| {
            serde_json::to_string(black_box(&status)).unwrap()
        })
    });
}

// Mocking a traffic update processing
#[derive(Clone)]
struct MockStat {
    name: String,
    value: i64,
}

fn process_traffic_stats(stats: &[MockStat]) -> Vec<(String, i64, i64)> {
    let mut results = Vec::new();
    // Simulate parsing "inbound>>>tag>>>traffic>>>downlink"
    for stat in stats {
        if stat.name.contains(">>>") {
            let parts: Vec<&str> = stat.name.split(">>>").collect();
            if parts.len() >= 4 {
                results.push((parts[1].to_string(), stat.value, 0));
            }
        }
    }
    results
}

fn benchmark_traffic_processing(c: &mut Criterion) {
    let mut stats = Vec::new();
    for i in 0..100 {
        stats.push(MockStat { name: format!("inbound>>>tag{}>>>traffic>>>downlink", i), value: 1000 });
        stats.push(MockStat { name: format!("inbound>>>tag{}>>>traffic>>>uplink", i), value: 500 });
    }

    c.bench_function("process_traffic_stats", |b| {
        b.iter(|| {
            process_traffic_stats(black_box(&stats))
        })
    });
}

criterion_group!(benches, benchmark_system_serialization, benchmark_traffic_processing);
criterion_main!(benches);
