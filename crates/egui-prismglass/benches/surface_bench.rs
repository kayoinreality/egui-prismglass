//! Reproducible CPU micro-benchmarks for `egui-prismglass` token resolution,
//! fallback & neumorphic shape generation, backdrop request construction, and spring physics.
//!
//! Run with:
//! ```bash
//! cargo bench -p egui-prismglass
//! ```

#![allow(missing_docs)]

use std::hint::black_box;
use std::time::Instant;

use egui::{Rect, pos2, vec2};
use egui_prismglass::{
    BackdropRequest, GlassElevation, PrismTheme, SpringConfig, SpringState, ThemeMode,
    glass_surface_shapes,
};

struct BenchResult {
    name: &'static str,
    mean_ns: f64,
    p50_ns: f64,
    p95_ns: f64,
    ops_per_sec: f64,
}

fn run_bench<F>(name: &'static str, batch_ops: usize, samples: usize, mut func: F) -> BenchResult
where
    F: FnMut(),
{
    // Warmup CPU caches and branch predictors.
    for _ in 0..5 {
        for _ in 0..batch_ops {
            func();
        }
    }

    let mut per_op_ns = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        for _ in 0..batch_ops {
            func();
        }
        let elapsed_ns = start.elapsed().as_nanos() as f64;
        per_op_ns.push(elapsed_ns / batch_ops as f64);
    }

    per_op_ns.sort_by(f64::total_cmp);
    let mean_ns = per_op_ns.iter().sum::<f64>() / per_op_ns.len() as f64;
    let p50_ns = per_op_ns[per_op_ns.len() / 2];
    let p95_idx = ((per_op_ns.len() as f64) * 0.95).ceil() as usize;
    let p95_ns = per_op_ns[p95_idx.min(per_op_ns.len() - 1)];
    let ops_per_sec = 1_000_000_000.0 / mean_ns.max(1e-6);

    BenchResult {
        name,
        mean_ns,
        p50_ns,
        p95_ns,
        ops_per_sec,
    }
}

fn format_time(ns: f64) -> String {
    if ns >= 1_000.0 {
        format!("{:.2} µs", ns / 1_000.0)
    } else {
        format!("{ns:.1} ns")
    }
}

fn main() {
    let rect = Rect::from_min_size(pos2(32.0, 48.0), vec2(320.0, 180.0));
    let glacial = PrismTheme::glacial();
    let min_light = PrismTheme::minimal_light();
    let min_dark = PrismTheme::minimal_dark();

    let glass_elevated = glacial.surface(GlassElevation::Elevated);
    let glass_floating = glacial.surface(GlassElevation::Floating);
    let neu_light = min_light.surface(GlassElevation::Elevated);
    let neu_dark = min_dark.surface(GlassElevation::Elevated);

    let mut results = Vec::new();

    results.push(run_bench(
        "PrismTheme::surface(Elevated)",
        50_000,
        60,
        || {
            black_box(glacial.surface(black_box(GlassElevation::Elevated)));
        },
    ));

    results.push(run_bench(
        "glass_surface_shapes (Glass Elevated)",
        5_000,
        60,
        || {
            black_box(glass_surface_shapes(
                black_box(rect),
                black_box(&glass_elevated),
            ));
        },
    ));

    results.push(run_bench(
        "glass_surface_shapes (Glass Floating)",
        5_000,
        60,
        || {
            black_box(glass_surface_shapes(
                black_box(rect),
                black_box(&glass_floating),
            ));
        },
    ));

    results.push(run_bench(
        "glass_surface_shapes (Neumorphic Light)",
        5_000,
        60,
        || {
            black_box(glass_surface_shapes(black_box(rect), black_box(&neu_light)));
        },
    ));

    results.push(run_bench(
        "glass_surface_shapes (Neumorphic Dark)",
        5_000,
        60,
        || {
            black_box(glass_surface_shapes(black_box(rect), black_box(&neu_dark)));
        },
    ));

    results.push(run_bench(
        "BackdropRequest::from_tokens_with_mode",
        50_000,
        60,
        || {
            black_box(BackdropRequest::from_tokens_with_mode(
                black_box(rect),
                black_box(&glass_floating),
                ThemeMode::Dark,
            ));
        },
    ));

    let spring_cfg = SpringConfig::snappy();
    results.push(run_bench(
        "SpringState::step (1,000 springs/frame)",
        500,
        60,
        || {
            let mut state = SpringState::new(0.0);
            for _ in 0..1_000 {
                state.step(black_box(1.0), black_box(1.0 / 60.0), spring_cfg);
            }
            black_box(state);
        },
    ));

    println!("# egui-prismglass CPU Surface Micro-Benchmarks\n");
    println!("| Benchmark | Mean | p50 | p95 | Throughput |");
    println!("|---|---:|---:|---:|---:|");
    for r in &results {
        println!(
            "| `{}` | {} | {} | {} | {:.1}k ops/s |",
            r.name,
            format_time(r.mean_ns),
            format_time(r.p50_ns),
            format_time(r.p95_ns),
            r.ops_per_sec / 1_000.0,
        );
    }
}
