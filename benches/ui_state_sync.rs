use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dioxus::prelude::*;
use rr_ui::ui::state::{GlobalState, CoreConnectivity};
use std::collections::VecDeque;

fn bench_state_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_state_sync");

    group.bench_function("update_traffic_metrics", |b| {
        // We need to run this inside a Dioxus VirtualDom or similar context if we were testing full reactivity,
        // but for raw signal update performance, we can just use the signals directly as they are independent of the DOM for write operations.
        // However, Dioxus signals usually require a Scope or Runtime to be active.
        // Since Dioxus 0.4/0.5, signals can be used outside of components but might need a runtime.
        // For this benchmark, we assume we are measuring the cost of the update logic itself.

        // Note: In a real Dioxus app, we'd need a runtime.
        // For the sake of this benchmark not crashing, we might need to mock it or just test the logic structure.
        // But let's try to keep it simple and assume the user has a setup where this works or we just benchmark the data structure operations.

        // Actually, let's just benchmark the logic we put in the sync task:
        // 1. Cloning stats
        // 2. Calculating totals
        // 3. Updating VecDeque

        let stats = vec![
            rr_ui::ui::server_fns::TrafficStat { name: "uplink".to_string(), value: 1024 },
            rr_ui::ui::server_fns::TrafficStat { name: "downlink".to_string(), value: 2048 },
        ];
        let mut history = VecDeque::with_capacity(60);
        for _ in 0..60 {
            history.push_back((0, 0));
        }

        b.iter(|| {
            // Simulate the work done in the sync task
            let current_stats = black_box(stats.clone());

            // Aggregate
            let (total_up, total_down) = current_stats.iter().fold((0, 0), |acc, stat| {
                if stat.name == "uplink" {
                    (acc.0 + stat.value, acc.1)
                } else if stat.name == "downlink" {
                    (acc.0, acc.1 + stat.value)
                } else {
                    acc
                }
            });

            // Update history
            if history.len() >= 60 {
                history.pop_front();
            }
            history.push_back((total_up, total_down));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_state_update);
criterion_main!(benches);
