//! Forensics Component Benchmarks
//!
//! Benchmarks for Phase 3 forensics data structures:
//! - TraceHistory push throughput (1000+ entries)
//! - TraceHistory height computation
//! - Routing canvas layout engine (compute_positions)
//! - FragmentationViz fork path computation

use criterion::{Criterion, black_box, criterion_group, criterion_main};

// ─── Import data models directly ───────────────────────────────────────────────
// We re-declare minimal versions here to avoid pulling in the full Dioxus
// render tree (which requires wasm/web features). This follows the same
// pattern used by ui_render_bench.rs.

use std::collections::VecDeque;

// ── TraceHistory mock ─────────────────────────────────────────────────────────

struct BenchTraceEntry {
    latency_ms: f32,
    timestamp_ms: u64,
}

struct BenchTraceHistory {
    entries: VecDeque<BenchTraceEntry>,
    capacity: usize,
}

impl BenchTraceHistory {
    fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, entry: BenchTraceEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn total_height(&self, padding_top: f32, padding_bottom: f32) -> f32 {
        let node_spacing = 80.0_f32;
        if self.entries.is_empty() {
            return padding_top + padding_bottom;
        }
        padding_top + ((self.entries.len() - 1) as f32 * node_spacing) + padding_bottom
    }
}

// ── Routing layout mock ───────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum MockNodeType {
    AppSource,
    LogicalFilter,
    OutboundNode,
}

impl MockNodeType {
    fn column(&self) -> u8 {
        match self {
            MockNodeType::AppSource => 0,
            MockNodeType::LogicalFilter => 1,
            MockNodeType::OutboundNode => 2,
        }
    }
}

struct MockRoutingNode {
    node_type: MockNodeType,
    x: f32,
    y: f32,
}

fn compute_positions_bench(nodes: &mut [MockRoutingNode], canvas_width: f32, canvas_height: f32) {
    if nodes.is_empty() || canvas_width <= 0.0 || canvas_height <= 0.0 {
        return;
    }

    let col_x = [
        canvas_width * 0.15,
        canvas_width * 0.50,
        canvas_width * 0.85,
    ];

    let mut col_counts = [0u32; 3];
    for node in nodes.iter() {
        let c = node.node_type.column() as usize;
        if c < 3 {
            col_counts[c] += 1;
        }
    }

    let mut col_indices = [0u32; 3];
    for node in nodes.iter_mut() {
        let c = node.node_type.column() as usize;
        if c >= 3 {
            continue;
        }
        node.x = col_x[c];
        let count = col_counts[c];
        let index = col_indices[c];
        if count == 1 {
            node.y = canvas_height / 2.0;
        } else {
            let spacing = canvas_height / (count + 1) as f32;
            node.y = spacing * (index + 1) as f32;
        }
        col_indices[c] += 1;
    }
}

// ── Fork paths mock ───────────────────────────────────────────────────────────

fn compute_fork_paths_bench(
    origin_y: f32,
    stream_count: usize,
    spread_height: f32,
) -> Vec<(f32, usize)> {
    if stream_count == 0 {
        return Vec::new();
    }
    if stream_count == 1 {
        return vec![(origin_y, 0)];
    }
    let half = spread_height / 2.0;
    let step = spread_height / (stream_count - 1) as f32;
    (0..stream_count)
        .map(|i| {
            let y = origin_y - half + (i as f32 * step);
            (y, i)
        })
        .collect()
}

// ─── Benchmarks ────────────────────────────────────────────────────────────────

fn bench_trace_history_push_1000(c: &mut Criterion) {
    c.bench_function("trace_history_push_1000", |b| {
        b.iter(|| {
            let mut hist = BenchTraceHistory::new(256);
            for i in 0..1000u64 {
                hist.push(BenchTraceEntry {
                    latency_ms: (i % 200) as f32,
                    timestamp_ms: i * 10,
                });
            }
            black_box(hist.len())
        })
    });
}

fn bench_trace_history_height_computation(c: &mut Criterion) {
    let mut hist = BenchTraceHistory::new(1024);
    for i in 0..1000u64 {
        hist.push(BenchTraceEntry {
            latency_ms: (i % 200) as f32,
            timestamp_ms: i * 10,
        });
    }

    c.bench_function("trace_history_height_1000_entries", |b| {
        b.iter(|| black_box(hist.total_height(30.0, 30.0)))
    });
}

fn bench_routing_layout_20_nodes(c: &mut Criterion) {
    c.bench_function("routing_layout_20_nodes", |b| {
        b.iter(|| {
            let mut nodes: Vec<MockRoutingNode> = Vec::with_capacity(20);
            // 2 app sources, 6 filters, 12 outbounds
            for _ in 0..2 {
                nodes.push(MockRoutingNode {
                    node_type: MockNodeType::AppSource,
                    x: 0.0,
                    y: 0.0,
                });
            }
            for _ in 0..6 {
                nodes.push(MockRoutingNode {
                    node_type: MockNodeType::LogicalFilter,
                    x: 0.0,
                    y: 0.0,
                });
            }
            for _ in 0..12 {
                nodes.push(MockRoutingNode {
                    node_type: MockNodeType::OutboundNode,
                    x: 0.0,
                    y: 0.0,
                });
            }
            compute_positions_bench(&mut nodes, 800.0, 400.0);
            black_box(nodes.len())
        })
    });
}

fn bench_fork_paths_8_streams(c: &mut Criterion) {
    c.bench_function("fork_paths_8_streams", |b| {
        b.iter(|| {
            let paths = compute_fork_paths_bench(200.0, 8, 200.0);
            black_box(paths.len())
        })
    });
}

criterion_group!(
    forensics_benches,
    bench_trace_history_push_1000,
    bench_trace_history_height_computation,
    bench_routing_layout_20_nodes,
    bench_fork_paths_8_streams,
);
criterion_main!(forensics_benches);
