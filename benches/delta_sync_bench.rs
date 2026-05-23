use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;

#[path = "../src/services/delta_sync.rs"]
mod delta_sync;

use delta_sync::DeltaCalculator;

fn bench_delta_calculation(c: &mut Criterion) {
    let mut old_users = Vec::new();
    let mut new_users = Vec::new();

    for i in 0..1000 {
        old_users.push(json!({"id": format!("user{}", i), "email": "test@example.com", "uuid": "uuid1", "quota_gb": 10, "expires_at": 1000}));
        if i % 2 == 0 {
            new_users.push(json!({"id": format!("user{}", i), "email": "test@example.com", "uuid": "uuid1", "quota_gb": 10, "expires_at": 1000}));
        } else {
            new_users.push(json!({"id": format!("user{}", i), "email": "new@example.com", "uuid": "uuid2", "quota_gb": 20, "expires_at": 2000}));
        }
    }

    for i in 1000..1500 {
        new_users.push(json!({"id": format!("user{}", i), "email": "test@example.com", "uuid": "uuid1", "quota_gb": 10, "expires_at": 1000}));
    }

    let old = json!({ "users": old_users });
    let new = json!({ "users": new_users });

    c.bench_function("delta_calculation", |b| {
        b.iter(|| DeltaCalculator::calculate(black_box(&old), black_box(&new)))
    });
}

criterion_group!(benches, bench_delta_calculation);
criterion_main!(benches);
