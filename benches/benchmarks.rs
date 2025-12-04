use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sena1996_ai::intelligence::autonomous::AutonomousAgent;
use sena1996_ai::memory::{MemoryEntry, MemoryStore, MemoryType};
use sena1996_ai::tools::{ToolCall, ToolSystem};
use std::collections::HashMap;

fn benchmark_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory");

    group.bench_function("create_entry", |b| {
        b.iter(|| {
            MemoryEntry::new(
                black_box("Test memory content for benchmarking"),
                black_box(MemoryType::Fact),
            )
        })
    });

    group.bench_function("entry_with_tags", |b| {
        b.iter(|| {
            MemoryEntry::new("Test content", MemoryType::Fact)
                .with_tags(black_box(vec![
                    "tag1".to_string(),
                    "tag2".to_string(),
                    "tag3".to_string(),
                ]))
                .with_importance(black_box(0.8))
        })
    });

    let mut store = MemoryStore::new();
    for i in 0..100 {
        store.add(
            MemoryEntry::new(format!("Memory entry {}", i), MemoryType::Fact)
                .with_tags(vec!["bench".to_string()])
                .with_importance(0.5),
        );
    }

    group.bench_function("search_100_entries", |b| {
        b.iter(|| store.search(black_box("entry")))
    });

    group.bench_function("relevance_score", |b| {
        let entry = MemoryEntry::new("Rust programming language", MemoryType::Preference)
            .with_importance(0.9);
        b.iter(|| entry.relevance_score(black_box("rust")))
    });

    group.finish();
}

fn benchmark_tool_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tools");

    group.bench_function("create_tool_call", |b| {
        b.iter(|| {
            let mut params = HashMap::new();
            params.insert("path".to_string(), serde_json::json!("/tmp/test.txt"));
            ToolCall::new(black_box("file_read"), black_box(params))
        })
    });

    group.bench_function("create_tool_system", |b| b.iter(|| ToolSystem::new()));

    let tool_system = ToolSystem::new();

    group.bench_function("list_tools", |b| b.iter(|| tool_system.list_tools()));

    group.bench_function("get_tool", |b| {
        b.iter(|| tool_system.get_tool(black_box("file_read")))
    });

    group.finish();
}

fn benchmark_memory_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory_Scaling");

    for size in [10, 100, 1000].iter() {
        let mut store = MemoryStore::new();
        for i in 0..*size {
            store.add(
                MemoryEntry::new(format!("Memory entry number {}", i), MemoryType::Fact)
                    .with_tags(vec!["scaling".to_string(), format!("batch{}", i / 10)])
                    .with_importance((i as f64) / (*size as f64)),
            );
        }

        group.bench_with_input(BenchmarkId::new("search", size), size, |b, _| {
            b.iter(|| store.search(black_box("entry")))
        });

        group.bench_with_input(BenchmarkId::new("search_by_type", size), size, |b, _| {
            b.iter(|| store.search_by_type(black_box(&MemoryType::Fact)))
        });

        group.bench_with_input(BenchmarkId::new("search_by_tag", size), size, |b, _| {
            b.iter(|| store.search_by_tag(black_box("scaling")))
        });
    }

    group.finish();
}

fn benchmark_autonomous_agent(c: &mut Criterion) {
    let mut group = c.benchmark_group("Autonomous");

    group.bench_function("create_agent", |b| b.iter(|| AutonomousAgent::new()));

    group.finish();
}

criterion_group!(
    benches,
    benchmark_memory_operations,
    benchmark_tool_system,
    benchmark_memory_scaling,
    benchmark_autonomous_agent,
);

criterion_main!(benches);
