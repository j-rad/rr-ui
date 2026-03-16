use criterion::{criterion_group, criterion_main, Criterion};
use tokio::sync::broadcast;
use rr_ui::services::sniffer::SniffingService;

fn benchmark_sniffer_broadcast(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("sniffer_broadcast", |b| {
        b.to_async(&rt).iter(|| async {
            let (tx, _rx) = broadcast::channel(100);
            let _sniffer = SniffingService::new(tx.clone());
            // We can't easily run the infinite loop one tick in bench without refactoring run()
            // But we can benchmark the mock fetch + serialize logic if we extract it.
            // Since SniffingService::run is infinite, we can't bench it directly.
            // We should bench the internal logic or just serialization.
            
            // For now, let's just bench channel send overhead with large payload.
            let payload = r#"{"liveConnections":[{"ip":"1.1.1.1","domain":"example.com","protocol":"tcp","duration":100,"latency":10}]}"#;
            tx.send(payload.to_string().into()).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_sniffer_broadcast);
criterion_main!(benches);
