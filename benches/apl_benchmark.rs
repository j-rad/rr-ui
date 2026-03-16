// benches/apl_benchmark.rs
//
// Benchmark for Atomic Persistence Layer (APL)
// Measures performance overhead of atomic writes vs standard writes

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::fs;
use tempfile::TempDir;

#[cfg(feature = "server")]
use rr_ui::adapters::atomic_config::AtomicConfigWriter;

#[cfg(feature = "server")]
fn benchmark_atomic_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_write");

    // Test with different payload sizes
    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    for (name, size) in sizes {
        let payload: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

        // Benchmark standard write
        group.bench_with_input(BenchmarkId::new("standard", name), &payload, |b, data| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let path = temp_dir.path().join("test.bin");
                fs::write(&path, black_box(data)).unwrap();
            });
        });

        // Benchmark atomic write
        group.bench_with_input(BenchmarkId::new("atomic", name), &payload, |b, data| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let writer = AtomicConfigWriter::new(temp_dir.path());
                let path = temp_dir.path().join("test.bin");
                writer.write_atomic(&path, black_box(data)).unwrap();
            });
        });

        // Benchmark atomic write with backup
        group.bench_with_input(
            BenchmarkId::new("atomic_backup", name),
            &payload,
            |b, data| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let writer = AtomicConfigWriter::new(temp_dir.path());
                    let path = temp_dir.path().join("test.bin");

                    // Create initial file
                    fs::write(&path, b"initial").unwrap();

                    // Write with backup
                    writer.write_with_backup(&path, black_box(data)).unwrap();
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "server")]
fn benchmark_json_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_write");

    // Create realistic Xray config-like JSON
    let small_config = serde_json::json!({
        "log": {"level": "warning"},
        "inbounds": [
            {"port": 443, "protocol": "vless", "settings": {}}
        ],
        "outbounds": [
            {"protocol": "freedom"}
        ]
    });

    let large_config = serde_json::json!({
        "log": {"level": "warning"},
        "inbounds": (0..50).map(|i| serde_json::json!({
            "port": 10000 + i,
            "protocol": "vless",
            "tag": format!("inbound-{}", i),
            "settings": {
                "clients": (0..20).map(|j| serde_json::json!({
                    "id": format!("uuid-{}-{}", i, j),
                    "email": format!("user-{}-{}@example.com", i, j)
                })).collect::<Vec<_>>()
            }
        })).collect::<Vec<_>>(),
        "outbounds": [{"protocol": "freedom"}]
    });

    // Benchmark small config
    group.bench_function("small_standard", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join("config.json");
            let json = serde_json::to_string_pretty(black_box(&small_config)).unwrap();
            fs::write(&path, json).unwrap();
        });
    });

    group.bench_function("small_atomic", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let writer = AtomicConfigWriter::new(temp_dir.path());
            let path = temp_dir.path().join("config.json");
            writer.write_json(&path, black_box(&small_config)).unwrap();
        });
    });

    // Benchmark large config
    group.bench_function("large_standard", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join("config.json");
            let json = serde_json::to_string_pretty(black_box(&large_config)).unwrap();
            fs::write(&path, json).unwrap();
        });
    });

    group.bench_function("large_atomic", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let writer = AtomicConfigWriter::new(temp_dir.path());
            let path = temp_dir.path().join("config.json");
            writer.write_json(&path, black_box(&large_config)).unwrap();
        });
    });

    group.finish();
}

#[cfg(feature = "server")]
fn benchmark_rollback(c: &mut Criterion) {
    use rr_ui::adapters::atomic_config::TransactionLog;

    c.bench_function("transaction_log_rollback", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let log_path = temp_dir.path().join("transaction.log");
            let mut log = TransactionLog::new(&log_path);

            // Simulate 10 writes
            for i in 0..10 {
                let path = temp_dir.path().join(format!("file-{}.txt", i));
                let backup_path = temp_dir.path().join(format!("file-{}.bak", i));

                fs::write(&backup_path, format!("backup-{}", i)).unwrap();
                fs::write(&path, format!("current-{}", i)).unwrap();

                log.record_write(&path, Some(&backup_path)).unwrap();
            }

            // Rollback last 5
            black_box(log.rollback(5).unwrap());
        });
    });
}

#[cfg(feature = "server")]
criterion_group!(
    benches,
    benchmark_atomic_write,
    benchmark_json_write,
    benchmark_rollback
);

#[cfg(not(feature = "server"))]
fn empty_benchmark(_c: &mut Criterion) {}

#[cfg(not(feature = "server"))]
criterion_group!(benches, empty_benchmark);

criterion_main!(benches);
