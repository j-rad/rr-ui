use criterion::{Criterion, criterion_group, criterion_main};
use std::collections::{HashMap, VecDeque};

// Mock the struct since we can't easily import from crate due to potential lib type issues in benches
#[derive(Clone)]
struct ActiveConnection {
    pub id: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
}

fn generate_mock_stats(count: usize) -> Vec<ActiveConnection> {
    (0..count)
        .map(|i| ActiveConnection {
            id: format!("conn-{}", i),
            upload_bytes: 1024 * i as u64,
            download_bytes: 2048 * i as u64,
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 1: Dashboard state update (existing)
// ═══════════════════════════════════════════════════════════════════

fn bench_state_update(c: &mut Criterion) {
    let stats = generate_mock_stats(100);

    c.bench_function("dashboard_update_100_rows", |b| {
        let mut history_map: HashMap<String, VecDeque<(i64, i64)>> = HashMap::new();

        b.iter(|| {
            let mut seen_ids = Vec::with_capacity(100);
            for conn in &stats {
                seen_ids.push(conn.id.clone());
                let deque = history_map
                    .entry(conn.id.clone())
                    .or_insert_with(|| VecDeque::with_capacity(60));
                if deque.len() >= 30 {
                    deque.pop_front();
                }
                deque.push_back((conn.upload_bytes as i64, conn.download_bytes as i64));
            }
            history_map.retain(|k, _| seen_ids.contains(k));
        })
    });
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 2: Signal-like container update latency
// ═══════════════════════════════════════════════════════════════════

fn bench_signal_update_latency(c: &mut Criterion) {
    c.bench_function("signal_container_write_1000", |b| {
        // Simulates the pattern of Signal::with_mut for a HashMap of VecDeques,
        // which is how Dioxus state updates work for our dashboard.
        let mut container: HashMap<String, VecDeque<(i64, i64)>> = HashMap::new();
        // Pre-populate 200 entries
        for i in 0..200 {
            let key = format!("entry-{}", i);
            let mut deque = VecDeque::with_capacity(60);
            for j in 0..30 {
                deque.push_back((j * 100, j * 200));
            }
            container.insert(key, deque);
        }

        b.iter(|| {
            // Simulate 1000 signal writes: read → mutate → write cycle
            for tick in 0..1000_i64 {
                let key = format!("entry-{}", tick % 200);
                if let Some(deque) = container.get_mut(&key) {
                    if deque.len() >= 60 {
                        deque.pop_front();
                    }
                    deque.push_back((tick * 1024, tick * 2048));
                }
            }
        })
    });
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 3: Fuzzy search throughput
// ═══════════════════════════════════════════════════════════════════

/// Case-insensitive substring match — mirrors CommandPalette's fuzzy_match.
fn fuzzy_match(query: &str, haystack: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    let h = haystack.to_lowercase();
    h.contains(&q)
}

#[derive(Clone)]
struct SearchEntry {
    label: String,
    sublabel: String,
}

fn bench_fuzzy_search(c: &mut Criterion) {
    // Build a corpus of 1000 mock user/inbound entries
    let entries: Vec<SearchEntry> = (0..1000)
        .map(|i| SearchEntry {
            label: format!("user-{}@example.com", i),
            sublabel: format!(":443 · vless · inbound-tag-{}", i % 50),
        })
        .collect();

    c.bench_function("fuzzy_search_1000_entries", |b| {
        let queries = ["user-42", "vless", "example", "tag-7", "nonexistent"];

        b.iter(|| {
            for q in &queries {
                let _results: Vec<&SearchEntry> = entries
                    .iter()
                    .filter(|e| fuzzy_match(q, &e.label) || fuzzy_match(q, &e.sublabel))
                    .take(20)
                    .collect();
            }
        })
    });
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 4: Bento layout aggregate computation
// ═══════════════════════════════════════════════════════════════════

fn bench_bento_aggregation(c: &mut Criterion) {
    let stats = generate_mock_stats(500);

    c.bench_function("bento_aggregate_500_connections", |b| {
        b.iter(|| {
            // Aggregate total traffic
            let (_total_up, _total_down) = stats.iter().fold((0u64, 0u64), |(u, d), c| {
                (u + c.upload_bytes, d + c.download_bytes)
            });

            // Top-5 sort
            let mut sorted: Vec<&ActiveConnection> = stats.iter().collect();
            sorted.sort_by(|a, b| {
                (b.upload_bytes + b.download_bytes).cmp(&(a.upload_bytes + a.download_bytes))
            });
            sorted.truncate(5);
        })
    });
}

criterion_group!(
    benches,
    bench_state_update,
    bench_signal_update_latency,
    bench_fuzzy_search,
    bench_bento_aggregation,
);
criterion_main!(benches);
