//! Reproducible end-to-end GPU & multi-surface benchmark harness for `egui-prismglass`.
//!
//! Measures:
//! 1. Dual-Kawase blur pyramid cost across resolutions (`720p`, `900p`, `1080p`, `1440p`)
//!    for both animated backdrops (9 GPU passes/frame) and static cached backdrops (0 blur passes).
//! 2. Scaling with `1`, `4`, `16`, `32`, and `64` simultaneous glass surfaces (GPU Animated,
//!    GPU Cached, Pure-`egui` Fallback, and Neumorphic Minimal) plus marginal `ms/surface`.
//! 3. Intermediate `Rgba16Float` texture reuse, VRAM footprint, and graphics resource cache hits.
//!
//! # Usage
//! ```bash
//! cargo run --release -p egui-prismglass --features blur-wgpu --example gpu_benchmark
//! cargo run --release -p egui-prismglass --features blur-wgpu --example gpu_benchmark -- --frames 20
//! cargo run --release -p egui-prismglass --features blur-wgpu --example gpu_benchmark -- --json
//! ```

#![allow(missing_docs)]

use std::sync::{Arc, Mutex};
use std::time::Instant;

use egui::{Area, Id, Margin, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup};
use egui_prismglass::blur_wgpu::wgpu::{AdapterInfo, DeviceType, TextureFormat};
use egui_prismglass::blur_wgpu::{WgpuBlurMetrics, WgpuBlurProvider};
use egui_prismglass::{
    GlassCard, GlassElevation, PrismContextExt, PrismTheme, paint_prism_backdrop_ui,
};

#[derive(Clone, Copy)]
struct TimingStats {
    mean: f64,
    p50: f64,
    p95: f64,
}

impl TimingStats {
    fn from_samples(mut samples_ms: Vec<f64>) -> Self {
        samples_ms.sort_by(f64::total_cmp);
        let len = samples_ms.len().max(1);
        let mean = samples_ms.iter().sum::<f64>() / len as f64;
        let p50 = samples_ms[len / 2];
        let p95_idx = ((len as f64) * 0.95).ceil() as usize;
        let p95 = samples_ms[p95_idx.min(len - 1)];
        Self { mean, p50, p95 }
    }
}

#[derive(Clone)]
struct SceneConfig {
    theme: PrismTheme,
    card_count: usize,
    time_secs: f32,
}

fn build_harness(
    width: u32,
    height: u32,
    config: Arc<Mutex<SceneConfig>>,
    provider_slot: Arc<Mutex<Option<Arc<WgpuBlurProvider>>>>,
) -> (Harness<'static>, AdapterInfo) {
    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let adapter_info = render_state.adapter.get_info();
    let renderer = WgpuTestRenderer::from_render_state(render_state);

    let harness = Harness::builder()
        .with_size(vec2(width as f32, height as f32))
        .with_pixels_per_point(1.0)
        .renderer(renderer)
        .build_ui(move |ui| {
            let cfg = config.lock().expect("config lock").clone();
            ui.ctx().set_prism_theme(cfg.theme.clone());

            let mut slot = provider_slot.lock().expect("provider lock");
            if slot.is_none() {
                *slot = Some(egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    TextureFormat::Rgba8Unorm,
                ));
            }
            drop(slot);

            paint_prism_backdrop_ui(ui, ui.max_rect(), &cfg.theme, cfg.time_secs, 1.0);

            if cfg.card_count > 0 {
                let cols = 8usize;
                let max_rect = ui.max_rect();
                let cell_w = ((max_rect.width() - 32.0) / cols as f32).max(48.0);
                let rows = cfg.card_count.div_ceil(cols).max(1);
                let cell_h = ((max_rect.height() - 32.0) / rows as f32).clamp(36.0, 120.0);
                let card_w = (cell_w - 10.0).max(32.0);
                let card_h = (cell_h - 10.0).max(28.0);

                for idx in 0..cfg.card_count {
                    let col = (idx % cols) as f32;
                    let row = (idx / cols) as f32;
                    let elev = match idx % 3 {
                        0 => GlassElevation::Subtle,
                        1 => GlassElevation::Elevated,
                        _ => GlassElevation::Floating,
                    };
                    Area::new(Id::NULL.with(("bench_card", idx)))
                        .fixed_pos(pos2(16.0 + col * cell_w, 16.0 + row * cell_h))
                        .show(ui.ctx(), |ui| {
                            GlassCard::new()
                                .elevation(elev)
                                .padding(Margin::same(0))
                                .show(ui, |ui| {
                                    ui.allocate_exact_size(
                                        vec2(card_w, card_h),
                                        egui::Sense::hover(),
                                    );
                                });
                        });
                }
            }
        });
    (harness, adapter_info)
}

fn measure_frames(
    harness: &mut Harness<'_>,
    config: &Arc<Mutex<SceneConfig>>,
    warmup: usize,
    frames: usize,
    animate_backdrop: bool,
) -> TimingStats {
    for step in 0..warmup {
        if animate_backdrop {
            config.lock().expect("lock").time_secs = (step as f32 + 1.0) * 0.016;
        }
        harness.step();
        let _ = harness.render().expect("warmup render");
    }

    let mut samples = Vec::with_capacity(frames);
    for step in 0..frames {
        if animate_backdrop {
            config.lock().expect("lock").time_secs = ((warmup + step + 1) as f32) * 0.016;
        }
        let start = Instant::now();
        harness.step();
        let _ = harness.render().expect("benchmark render");
        samples.push(start.elapsed().as_secs_f64() * 1_000.0);
    }
    TimingStats::from_samples(samples)
}

fn read_metrics(slot: &Arc<Mutex<Option<Arc<WgpuBlurProvider>>>>) -> WgpuBlurMetrics {
    slot.lock()
        .expect("lock")
        .as_ref()
        .expect("provider")
        .metrics()
}

struct ResolutionRow {
    label: &'static str,
    width: u32,
    height: u32,
    animated: TimingStats,
    cached: TimingStats,
    blur_delta_ms: f64,
    vram_mib: f64,
    textures: usize,
    allocations: u64,
}

struct SurfaceRow {
    cards: usize,
    gpu_animated_ms: f64,
    gpu_cached_ms: f64,
    fallback_ms: f64,
    neumorphic_ms: f64,
}

struct ReuseAudit {
    stationary_frames: usize,
    pyramid_allocations: u64,
    pyramid_passes_executed: u64,
    pyramid_cache_hits: u64,
    card_buffer_allocations_24: u64,
    card_buffer_allocations_64: u64,
    resize_steps: usize,
    resize_allocations_delta: u64,
    resize_avg_ms: f64,
}

fn parse_args() -> (Option<usize>, bool) {
    let mut frames = None;
    let mut json = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--frames" => {
                if let Some(val) = args.next()
                    && let Ok(parsed) = val.parse::<usize>()
                {
                    frames = Some(parsed.max(3));
                }
            }
            _ => {}
        }
    }
    (frames, json)
}

fn main() {
    let (frames_arg, json_mode) = parse_args();
    let warmup = 2usize;

    let cfg = Arc::new(Mutex::new(SceneConfig {
        theme: PrismTheme::glacial(),
        card_count: 8,
        time_secs: 0.0,
    }));
    let provider_slot = Arc::new(Mutex::new(None));
    let (mut harness, adapter_info) = build_harness(1280, 720, cfg.clone(), provider_slot.clone());

    let frames = frames_arg.unwrap_or(if adapter_info.device_type == DeviceType::Cpu {
        5
    } else {
        20
    });

    eprintln!(
        "[gpu_benchmark] Adapter: {} ({:?}, {:?}), frames={frames}",
        adapter_info.name, adapter_info.device_type, adapter_info.backend
    );

    // -----------------------------------------------------------------------
    // Suite 1: Blur Cost Across Resolutions (720p, 900p, 1080p, 1440p)
    // -----------------------------------------------------------------------
    let resolutions = [
        ("720p (1280×720)", 1280u32, 720u32),
        ("900p (1440×900)", 1440, 900),
        ("1080p (1920×1080)", 1920, 1080),
        ("1440p (2560×1440)", 2560, 1440),
    ];

    let mut resolution_rows = Vec::with_capacity(resolutions.len());
    let mut prev_allocs = 0u64;
    for (label, width, height) in resolutions {
        eprintln!("[gpu_benchmark] Suite 1 — measuring {label}...");
        harness.set_size(vec2(width as f32, height as f32));
        {
            let mut lock = cfg.lock().expect("lock");
            lock.theme = PrismTheme::glacial();
            lock.card_count = 8;
            lock.time_secs = 0.0;
        }

        // Static cached backdrop timing (0 blur passes per frame after first frame).
        let cached = measure_frames(&mut harness, &cfg, warmup, frames, false);
        // Animated backdrop timing (full 9-pass backdrop + Dual-Kawase pyramid every frame).
        let animated = measure_frames(&mut harness, &cfg, warmup, frames, true);

        let metrics = read_metrics(&provider_slot);
        let delta_allocs = metrics.pyramid_allocations.saturating_sub(prev_allocs);
        prev_allocs = metrics.pyramid_allocations;

        let vram_mib = metrics.estimated_vram_bytes as f64 / (1024.0 * 1024.0);
        let blur_delta_ms = (animated.mean - cached.mean).max(0.0);

        resolution_rows.push(ResolutionRow {
            label,
            width,
            height,
            animated,
            cached,
            blur_delta_ms,
            vram_mib,
            textures: metrics.pyramid_texture_count,
            allocations: delta_allocs,
        });
    }

    // -----------------------------------------------------------------------
    // Suite 2: Multi-Surface Scaling (1, 4, 16, 32, 64 cards at 1280×720)
    // -----------------------------------------------------------------------
    let surface_counts = [1usize, 4, 16, 32, 64];
    let mut surface_rows = Vec::with_capacity(surface_counts.len());
    harness.set_size(vec2(1280.0, 720.0));

    let mut fallback_theme = PrismTheme::glacial();
    fallback_theme.real_blur_enabled = false;
    let neumorphic_theme = PrismTheme::minimal_dark();

    for &cards in &surface_counts {
        eprintln!("[gpu_benchmark] Suite 2 — measuring {cards} simultaneous surfaces @ 720p...");
        // GPU Liquid Glass (Static Cached)
        {
            let mut lock = cfg.lock().expect("lock");
            lock.theme = PrismTheme::glacial();
            lock.card_count = cards;
            lock.time_secs = 0.0;
        }
        let gpu_cached = measure_frames(&mut harness, &cfg, warmup, frames, false);

        // GPU Liquid Glass (Animated Backdrop)
        let gpu_animated = measure_frames(&mut harness, &cfg, warmup, frames, true);

        // CPU Fallback Pure egui
        {
            let mut lock = cfg.lock().expect("lock");
            lock.theme = fallback_theme.clone();
            lock.card_count = cards;
            lock.time_secs = 0.0;
        }
        let fallback = measure_frames(&mut harness, &cfg, warmup, frames, false);

        // Neumorphic Minimal Dark
        {
            let mut lock = cfg.lock().expect("lock");
            lock.theme = neumorphic_theme.clone();
            lock.card_count = cards;
            lock.time_secs = 0.0;
        }
        let neumorphic = measure_frames(&mut harness, &cfg, warmup, frames, false);

        surface_rows.push(SurfaceRow {
            cards,
            gpu_animated_ms: gpu_animated.mean,
            gpu_cached_ms: gpu_cached.mean,
            fallback_ms: fallback.mean,
            neumorphic_ms: neumorphic.mean,
        });
    }

    let first_s = &surface_rows[0];
    let last_s = surface_rows.last().expect("non-empty");
    let delta_cards = (last_s.cards - first_s.cards) as f64;
    let marginal_gpu_anim = (last_s.gpu_animated_ms - first_s.gpu_animated_ms) / delta_cards;
    let marginal_gpu_cached = (last_s.gpu_cached_ms - first_s.gpu_cached_ms) / delta_cards;
    let marginal_fallback = (last_s.fallback_ms - first_s.fallback_ms) / delta_cards;
    let marginal_neumorphic = (last_s.neumorphic_ms - first_s.neumorphic_ms) / delta_cards;

    // -----------------------------------------------------------------------
    // Suite 3: Intermediate Texture Reuse & Resource Cache Audit (fresh counters)
    // -----------------------------------------------------------------------
    eprintln!("[gpu_benchmark] Suite 3 — auditing intermediate texture reuse & resource cache...");
    drop(harness);
    let audit_cfg = Arc::new(Mutex::new(SceneConfig {
        theme: PrismTheme::glacial(),
        card_count: 24,
        time_secs: 0.0,
    }));
    let audit_provider = Arc::new(Mutex::new(None));
    let (mut audit_harness, _) =
        build_harness(1280, 720, audit_cfg.clone(), audit_provider.clone());

    let stationary_frames = 15usize;
    for _ in 0..stationary_frames {
        audit_harness.step();
        let _ = audit_harness.render().expect("stationary frame");
    }
    let m_stationary = read_metrics(&audit_provider);

    // Grow simultaneous cards beyond initial 32-card capacity -> buffer grows once.
    audit_cfg.lock().expect("lock").card_count = 64;
    for _ in 0..3 {
        audit_harness.step();
        let _ = audit_harness.render().expect("64-card frame");
    }
    let m_grown = read_metrics(&audit_provider);

    // Resize stress test across 4 resolution transitions.
    let resize_targets = [(1440u32, 900u32), (1920, 1080), (1440, 900), (1280, 720)];
    let allocs_before_resize = m_grown.pyramid_allocations;
    let resize_start = Instant::now();
    for (rw, rh) in resize_targets {
        audit_harness.set_size(vec2(rw as f32, rh as f32));
        audit_harness.step();
        let _ = audit_harness.render().expect("resize frame");
    }
    let resize_avg_ms =
        (resize_start.elapsed().as_secs_f64() * 1_000.0) / resize_targets.len() as f64;
    let m_after_resize = read_metrics(&audit_provider);

    let audit = ReuseAudit {
        stationary_frames,
        pyramid_allocations: m_stationary.pyramid_allocations,
        pyramid_passes_executed: m_stationary.pyramid_passes_executed,
        pyramid_cache_hits: m_stationary.pyramid_cache_hits,
        card_buffer_allocations_24: m_stationary.card_buffer_allocations,
        card_buffer_allocations_64: m_grown.card_buffer_allocations,
        resize_steps: resize_targets.len(),
        resize_allocations_delta: m_after_resize.pyramid_allocations - allocs_before_resize,
        resize_avg_ms,
    };

    if json_mode {
        print_json(
            &adapter_info,
            frames,
            &resolution_rows,
            &surface_rows,
            (
                marginal_gpu_anim,
                marginal_gpu_cached,
                marginal_fallback,
                marginal_neumorphic,
            ),
            &audit,
        );
    } else {
        print_markdown(
            &adapter_info,
            frames,
            &resolution_rows,
            &surface_rows,
            (
                marginal_gpu_anim,
                marginal_gpu_cached,
                marginal_fallback,
                marginal_neumorphic,
            ),
            &audit,
        );
    }
}

fn print_markdown(
    adapter: &AdapterInfo,
    frames: usize,
    resolutions: &[ResolutionRow],
    surfaces: &[SurfaceRow],
    marginals: (f64, f64, f64, f64),
    audit: &ReuseAudit,
) {
    println!("# egui-prismglass GPU & Multi-Surface Benchmark Report\n");
    println!("- **Adapter**: `{}`", adapter.name);
    println!("- **Device Type**: `{:?}`", adapter.device_type);
    println!("- **Backend**: `{:?}`", adapter.backend);
    println!(
        "- **Driver**: `{} ({})`",
        adapter.driver, adapter.driver_info
    );
    println!(
        "- **Methodology**: `egui_kittest` headless `wgpu` full-frame submit + sync readback ({frames} measured frames after 2 warmup frames; build `--release`).\n"
    );

    println!("## 1. Dual-Kawase Blur Cost & VRAM Across Resolutions (8 GlassCards)\n");
    println!(
        "| Resolution | Animated (mean / p50 / p95) | Static Cached (mean / p50 / p95) | Pure Blur Pyramid Delta | Pyramid VRAM (6 tex) | Allocs |"
    );
    println!("|---|---:|---:|---:|---:|---:|");
    for row in resolutions {
        println!(
            "| {} | {:.2} / {:.2} / {:.2} ms | {:.2} / {:.2} / {:.2} ms | **{:.2} ms** | {:.2} MiB ({} tex) | {} |",
            row.label,
            row.animated.mean,
            row.animated.p50,
            row.animated.p95,
            row.cached.mean,
            row.cached.p50,
            row.cached.p95,
            row.blur_delta_ms,
            row.vram_mib,
            row.textures,
            row.allocations,
        );
    }

    println!("\n## 2. Multi-Surface Scaling at 1280×720 (`1` .. `64` Simultaneous Cards)\n");
    println!(
        "| Surfaces | GPU Liquid Glass (Animated) | GPU Liquid Glass (Cached) | CPU Fallback (Pure egui) | Neumorphic Minimal |"
    );
    println!("|---:|---:|---:|---:|---:|");
    for row in surfaces {
        println!(
            "| {} | {:.2} ms | {:.2} ms | {:.2} ms | {:.2} ms |",
            row.cards, row.gpu_animated_ms, row.gpu_cached_ms, row.fallback_ms, row.neumorphic_ms,
        );
    }
    println!(
        "| **Marginal ms / surface** (`64 vs 1`) | **{:.3} ms/surf** | **{:.3} ms/surf** | **{:.3} ms/surf** | **{:.3} ms/surf** |",
        marginals.0, marginals.1, marginals.2, marginals.3,
    );

    println!("\n## 3. Intermediate Texture Reuse & Resource Cache Audit (`1280×720`)\n");
    println!("| Metric | Observed | Expected |");
    println!("|---|---:|---:|");
    println!(
        "| Stationary frames (`24` cards) | {} frames | — |",
        audit.stationary_frames
    );
    println!(
        "| Pyramid texture allocations | {} | `1` (zero per-frame realloc) |",
        audit.pyramid_allocations
    );
    println!(
        "| Pyramid GPU passes executed | {} | `1` (first frame only) |",
        audit.pyramid_passes_executed
    );
    println!(
        "| Static pyramid cache hits | {} | `{}` (`0.00 ms` blur cost) |",
        audit.pyramid_cache_hits,
        audit.stationary_frames - 1
    );
    println!(
        "| Card uniform buffer allocs (`24` -> `64` cards) | {} -> {} | `1 -> 2` (power-of-two reuse) |",
        audit.card_buffer_allocations_24, audit.card_buffer_allocations_64
    );
    println!(
        "| Dynamic resize allocs ({} transitions) | {} (avg {:.2} ms/resize) | `{}` (1 per resize) |",
        audit.resize_steps, audit.resize_allocations_delta, audit.resize_avg_ms, audit.resize_steps
    );
}

fn print_json(
    adapter: &AdapterInfo,
    frames: usize,
    resolutions: &[ResolutionRow],
    surfaces: &[SurfaceRow],
    marginals: (f64, f64, f64, f64),
    audit: &ReuseAudit,
) {
    println!("{{");
    println!("  \"adapter\": {{");
    println!("    \"name\": {:?},", adapter.name);
    println!("    \"device_type\": \"{:?}\",", adapter.device_type);
    println!("    \"backend\": \"{:?}\",", adapter.backend);
    println!("    \"driver\": {:?}", adapter.driver);
    println!("  }},");
    println!("  \"frames\": {frames},");
    println!("  \"resolutions\": [");
    for (idx, r) in resolutions.iter().enumerate() {
        let comma = if idx + 1 < resolutions.len() { "," } else { "" };
        println!(
            "    {{\"width\": {}, \"height\": {}, \"animated_mean_ms\": {:.3}, \"cached_mean_ms\": {:.3}, \"blur_delta_ms\": {:.3}, \"vram_mib\": {:.3}, \"textures\": {}, \"allocations\": {}}}{comma}",
            r.width,
            r.height,
            r.animated.mean,
            r.cached.mean,
            r.blur_delta_ms,
            r.vram_mib,
            r.textures,
            r.allocations
        );
    }
    println!("  ],");
    println!("  \"surfaces_720p\": [");
    for (idx, s) in surfaces.iter().enumerate() {
        let comma = if idx + 1 < surfaces.len() { "," } else { "" };
        println!(
            "    {{\"cards\": {}, \"gpu_animated_ms\": {:.3}, \"gpu_cached_ms\": {:.3}, \"fallback_ms\": {:.3}, \"neumorphic_ms\": {:.3}}}{comma}",
            s.cards, s.gpu_animated_ms, s.gpu_cached_ms, s.fallback_ms, s.neumorphic_ms
        );
    }
    println!("  ],");
    println!(
        "  \"marginal_ms_per_surface\": {{\"gpu_animated\": {:.4}, \"gpu_cached\": {:.4}, \"fallback\": {:.4}, \"neumorphic\": {:.4}}},",
        marginals.0, marginals.1, marginals.2, marginals.3
    );
    println!(
        "  \"resource_reuse\": {{\"stationary_frames\": {}, \"pyramid_allocations\": {}, \"pyramid_passes_executed\": {}, \"pyramid_cache_hits\": {}, \"card_buffer_allocations_24\": {}, \"card_buffer_allocations_64\": {}, \"resize_allocations_delta\": {}, \"resize_avg_ms\": {:.3}}}",
        audit.stationary_frames,
        audit.pyramid_allocations,
        audit.pyramid_passes_executed,
        audit.pyramid_cache_hits,
        audit.card_buffer_allocations_24,
        audit.card_buffer_allocations_64,
        audit.resize_allocations_delta,
        audit.resize_avg_ms
    );
    println!("}}");
}
