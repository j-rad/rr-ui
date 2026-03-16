// benches/routing_engine_latency.rs
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rr_ui::xray_config::RoutingTemplate;

/// Benchmark default routing rule generation
fn bench_default_routing(c: &mut Criterion) {
    c.bench_function("routing_default", |b| {
        b.iter(|| {
            let template = RoutingTemplate::Default;
            black_box(template.generate_rules())
        });
    });
}

/// Benchmark China-optimized routing rule generation
fn bench_china_routing(c: &mut Criterion) {
    c.bench_function("routing_china", |b| {
        b.iter(|| {
            let template = RoutingTemplate::ChinaOptimized;
            black_box(template.generate_rules())
        });
    });
}

/// Benchmark Iran-optimized routing rule generation
fn bench_iran_routing(c: &mut Criterion) {
    c.bench_function("routing_iran", |b| {
        b.iter(|| {
            let template = RoutingTemplate::IranOptimized;
            black_box(template.generate_rules())
        });
    });
}

/// Benchmark Russia-optimized routing rule generation
fn bench_russia_routing(c: &mut Criterion) {
    c.bench_function("routing_russia", |b| {
        b.iter(|| {
            let template = RoutingTemplate::RussiaOptimized;
            black_box(template.generate_rules())
        });
    });
}

/// Benchmark all routing templates together
fn bench_all_templates(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_all_templates");

    for template in &[
        RoutingTemplate::Default,
        RoutingTemplate::ChinaOptimized,
        RoutingTemplate::IranOptimized,
        RoutingTemplate::RussiaOptimized,
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", template)),
            template,
            |b, &template| {
                b.iter(|| black_box(template.generate_rules()));
            },
        );
    }

    group.finish();
}

/// Benchmark routing rule generation at scale (simulating 500-rule template)
fn bench_large_scale_routing(c: &mut Criterion) {
    c.bench_function("routing_500_rules", |b| {
        b.iter(|| {
            // Simulate generating a large routing configuration
            let mut all_rules = Vec::new();

            // Generate rules from all templates multiple times to simulate complexity
            for _ in 0..125 {
                all_rules.extend(RoutingTemplate::Default.generate_rules());
                all_rules.extend(RoutingTemplate::ChinaOptimized.generate_rules());
            }

            black_box(all_rules)
        });
    });
}

/// Benchmark serialization of routing rules to JSON
fn bench_routing_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_serialization");

    for template in &[
        RoutingTemplate::Default,
        RoutingTemplate::ChinaOptimized,
        RoutingTemplate::IranOptimized,
        RoutingTemplate::RussiaOptimized,
    ] {
        let rules = template.generate_rules();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", template)),
            &rules,
            |b, rules| {
                b.iter(|| black_box(serde_json::to_string(rules).unwrap()));
            },
        );
    }

    group.finish();
}

/// Benchmark routing rule lookup/matching simulation
fn bench_routing_rule_lookup(c: &mut Criterion) {
    let china_rules = RoutingTemplate::ChinaOptimized.generate_rules();

    c.bench_function("routing_rule_lookup", |b| {
        b.iter(|| {
            // Simulate looking up rules by checking each rule's type
            for rule in &china_rules {
                if let Some(rule_type) = rule.get("type") {
                    black_box(rule_type);
                }
                if let Some(outbound) = rule.get("outboundTag") {
                    black_box(outbound);
                }
            }
        });
    });
}

/// Benchmark memory usage of different routing templates
fn bench_routing_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_memory");

    group.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let templates = vec![
                RoutingTemplate::Default.generate_rules(),
                RoutingTemplate::ChinaOptimized.generate_rules(),
                RoutingTemplate::IranOptimized.generate_rules(),
                RoutingTemplate::RussiaOptimized.generate_rules(),
            ];
            black_box(templates)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_default_routing,
    bench_china_routing,
    bench_iran_routing,
    bench_russia_routing,
    bench_all_templates,
    bench_large_scale_routing,
    bench_routing_serialization,
    bench_routing_rule_lookup,
    bench_routing_memory
);

criterion_main!(benches);
