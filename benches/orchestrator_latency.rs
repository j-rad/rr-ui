use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rr_ui::domain::models::{MeshNode, MeshNodeStatus, NodeCapacity, NodeHealth};
use rr_ui::services::mesh_orchestrator::{MeshOrchestrator, RoutingStrategy};
use rr_ui::services::route_optimizer::RouteOptimizer;
use tokio::runtime::Runtime;

fn bench_route_optimizer(c: &mut Criterion) {
    let optimizer = RouteOptimizer::new(24);

    // Create 100 fake nodes
    let mut nodes = Vec::new();
    for i in 0..100 {
        nodes.push(MeshNode {
            node_id: format!("node-{}", i),
            name: format!("Node {}", i),
            address: format!("10.0.0.{}", i),
            status: MeshNodeStatus::Online,
            is_local: false,
            health: NodeHealth {
                cpu_percent: (i % 100) as f32, // Varying CPU
                memory_percent: 50.0,
                disk_percent: 20.0,
                latency_ms: 10.0,
                packet_loss_percent: 0.0,
            },
            ..Default::default()
        });
    }

    c.bench_function("route_optimizer_anycast_100_nodes", |b| {
        b.iter(|| {
            optimizer.generate_optimal_subscription_link(black_box(&nodes), black_box("user-123"))
        })
    });
}

fn bench_mesh_orchestrator_sync(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut orchestrator = MeshOrchestrator::new(RoutingStrategy::RoundRobin);

    for i in 0..100 {
        let node = MeshNode {
            node_id: format!("node-{}", i),
            name: format!("Node {}", i),
            address: format!("10.0.0.{}", i),
            status: MeshNodeStatus::Online,
            is_local: false,
            ..Default::default()
        };
        orchestrator.register_node(node);
    }

    c.bench_function("mesh_orchestrator_sync_100_nodes", |b| {
        b.to_async(&rt)
            .iter(|| async { orchestrator.run_node_sync_job(black_box(vec![])).await })
    });
}

criterion_group!(benches, bench_route_optimizer, bench_mesh_orchestrator_sync);
criterion_main!(benches);
