//! Automated visual rendering & shader regression test matrix for `egui-prismglass`.
//!
//! Covers six visual quality domains:
//! 1. Golden screenshot comparison across releases (`PRISMGLASS_UPDATE_SNAPSHOTS=1` to refresh)
//!    and bit-exact rendering determinism.
//! 2. Scissor clipping (`ScrollArea` / `clip_rect`) and analytic SDF corner rounding
//!    (`0 pt`, standard `20 pt`, clamped pill `999 pt`).
//! 3. DPI scaling matrix (`1.0x`, `1.5x`, `2.0x`, `2.5x` `pixels_per_point`).
//! 4. Resolution matrix (`320×240` .. `1280×720`) and dynamic runtime resizing.
//! 5. Viewport boundary / edge artifact verification (no UV wrap-around, NaNs, or alpha leaks).
//! 6. Shader optical invariants (blur monotonicity, Snell bevel refraction, RGB chromatic
//!    dispersion, Beer-Lambert tint transmission, GPU/fallback toggle) and GPU resource reuse
//!    (in-place 6-texture pyramid reuse + static backdrop cache hits).

#![cfg(feature = "blur-wgpu")]
#![allow(missing_docs)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use egui::{Area, Color32, Id, Margin, Rect, ScrollArea, UiBuilder, pos2, vec2};
use egui_kittest::Harness;
use egui_prismglass::blur_wgpu::wgpu::TextureFormat;
use egui_prismglass::blur_wgpu::{WgpuBlurMetrics, WgpuBlurProvider};
use egui_prismglass::{
    AccentPalette, BackdropScene, BlurCapability, GlassCard, GlassElevation, GlassSurfaceTokens,
    PrismContextExt, PrismTheme, paint_prism_backdrop_ui,
};
use image::RgbaImage;

/// Serializes headless `wgpu` device creation/teardown across parallel `cargo test` threads.
/// Mesa's software Vulkan driver (`lavapipe` / LLVM JIT) can segfault if multiple `VkDevice`
/// instances initialize and compile shaders concurrently inside the same process.
static GPU_TEST_LOCK: Mutex<()> = Mutex::new(());

fn lock_gpu() -> MutexGuard<'static, ()> {
    GPU_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

// ---------------------------------------------------------------------------
// Synthetic Backdrop & Image Analysis Helpers
// ---------------------------------------------------------------------------

fn build_checkerboard(width: u32, height: u32, cell: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        for col in 0..width {
            let light = ((col / cell) + (row / cell)).is_multiple_of(2);
            if light {
                bytes.extend_from_slice(&[242, 246, 255, 255]);
            } else {
                bytes.extend_from_slice(&[14, 18, 28, 255]);
            }
        }
    }
    bytes
}

fn build_vertical_stripes(width: u32, height: u32, stripe: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((width * height * 4) as usize);
    for _row in 0..height {
        for col in 0..width {
            if (col / stripe).is_multiple_of(2) {
                bytes.extend_from_slice(&[246, 248, 252, 255]);
            } else {
                bytes.extend_from_slice(&[12, 16, 24, 255]);
            }
        }
    }
    bytes
}

fn build_solid_rgba8(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..(width * height) {
        bytes.extend_from_slice(&rgba);
    }
    bytes
}

fn pixel_luma(rgba: &[u8]) -> f64 {
    0.2126 * f64::from(rgba[0]) + 0.7152 * f64::from(rgba[1]) + 0.0722 * f64::from(rgba[2])
}

fn luma_variance(img: &RgbaImage, x0: u32, y0: u32, w: u32, h: u32) -> f64 {
    let raw = img.as_raw();
    let img_w = img.width();
    let mut sum = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    let count = f64::from(w * h);
    for py in y0..y0 + h {
        for px in x0..x0 + w {
            let idx = ((py * img_w + px) * 4) as usize;
            let luma = pixel_luma(&raw[idx..idx + 4]);
            sum += luma;
            sum_sq += luma * luma;
        }
    }
    let mean = sum / count;
    (sum_sq / count) - (mean * mean)
}

fn mean_rgb(img: &RgbaImage, x0: u32, y0: u32, w: u32, h: u32) -> [f64; 3] {
    let raw = img.as_raw();
    let img_w = img.width();
    let mut sums = [0.0_f64; 3];
    let count = f64::from(w * h);
    for py in y0..y0 + h {
        for px in x0..x0 + w {
            let idx = ((py * img_w + px) * 4) as usize;
            sums[0] += f64::from(raw[idx]);
            sums[1] += f64::from(raw[idx + 1]);
            sums[2] += f64::from(raw[idx + 2]);
        }
    }
    [sums[0] / count, sums[1] / count, sums[2] / count]
}

fn region_mae(a: &RgbaImage, b: &RgbaImage, x0: u32, y0: u32, w: u32, h: u32) -> f64 {
    assert_eq!(a.dimensions(), b.dimensions());
    let raw_a = a.as_raw();
    let raw_b = b.as_raw();
    let img_w = a.width();
    let mut diff_sum = 0.0_f64;
    let samples = f64::from(w * h) * 3.0;
    for py in y0..y0 + h {
        for px in x0..x0 + w {
            let idx = ((py * img_w + px) * 4) as usize;
            for ch in 0..3 {
                diff_sum += (f64::from(raw_a[idx + ch]) - f64::from(raw_b[idx + ch])).abs();
            }
        }
    }
    diff_sum / samples
}

fn expected_6tex_vram_bytes(width: u32, height: u32) -> u64 {
    let w1 = (width / 2).max(1);
    let h1 = (height / 2).max(1);
    let w2 = (width / 4).max(1);
    let h2 = (height / 4).max(1);
    let w3 = (width / 8).max(1);
    let h3 = (height / 8).max(1);
    let w4 = (width / 16).max(1);
    let h4 = (height / 16).max(1);
    let px = |w: u32, h: u32| u64::from(w) * u64::from(h) * 8;
    px(width, height)
        + px(w1, h1) // down1_or_deep
        + px(w2, h2) // down2
        + px(w3, h3) // down3
        + px(w4, h4) // down4
        + px(w1, h1) // blur_med
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
}

fn assert_matches_golden(name: &str, actual: &RgbaImage) {
    let dir = snapshot_dir();
    let path = dir.join(format!("{name}.png"));
    let update = std::env::var("PRISMGLASS_UPDATE_SNAPSHOTS")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    if update {
        std::fs::create_dir_all(&dir).expect("create snapshots directory");
        actual
            .save(&path)
            .unwrap_or_else(|e| panic!("failed to write golden snapshot {}: {e}", path.display()));
        return;
    }

    let expected = image::open(&path)
        .unwrap_or_else(|e| {
            panic!(
                "missing golden snapshot {} ({e}). Run with PRISMGLASS_UPDATE_SNAPSHOTS=1 to generate.",
                path.display()
            )
        })
        .to_rgba8();

    assert_eq!(
        actual.dimensions(),
        expected.dimensions(),
        "snapshot `{name}` dimension mismatch"
    );

    let raw_a = actual.as_raw();
    let raw_e = expected.as_raw();
    let total_pixels = (actual.width() * actual.height()) as usize;
    let mut abs_sum = 0_u64;
    let mut max_delta = 0_u8;
    let mut pixels_over_tolerance = 0_usize;

    for px in 0..total_pixels {
        let base = px * 4;
        let mut px_max = 0_u8;
        for ch in 0..4 {
            let d = raw_a[base + ch].abs_diff(raw_e[base + ch]);
            abs_sum += u64::from(d);
            px_max = px_max.max(d);
            max_delta = max_delta.max(d);
        }
        if px_max > 4 {
            pixels_over_tolerance += 1;
        }
    }

    let mae = abs_sum as f64 / (total_pixels as f64 * 4.0);
    let pct_over = (pixels_over_tolerance as f64 / total_pixels as f64) * 100.0;

    assert!(
        mae <= 1.5 && pct_over <= 0.5,
        "golden snapshot `{name}` diverged: MAE={mae:.3}, max_delta={max_delta}, pixels_over_4={pct_over:.3}%"
    );
}

fn render_fixture(
    width: u32,
    height: u32,
    ppp: f32,
    theme: PrismTheme,
    draw_cards: impl Fn(&mut egui::Ui, &PrismTheme) + 'static,
) -> RgbaImage {
    let _gpu = lock_gpu();
    let mut installed = false;
    let mut harness = Harness::builder()
        .with_size(vec2(width as f32, height as f32))
        .with_pixels_per_point(ppp)
        .build_ui(move |ui| {
            ui.ctx().set_prism_theme(theme.clone());
            if !installed {
                egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    TextureFormat::Rgba8Unorm,
                );
                installed = true;
            }
            paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);
            draw_cards(ui, &theme);
        });
    harness.step();
    harness.step();
    harness
        .render()
        .expect("headless wgpu fixture should render")
}

// ---------------------------------------------------------------------------
// 1. Cross-Version Golden Screenshot Comparison & Bit-Exact Determinism
// ---------------------------------------------------------------------------

#[test]
fn frame_rendering_is_bit_exact_deterministic() {
    let draw = |ui: &mut egui::Ui, _theme: &PrismTheme| {
        Area::new(Id::NULL.with("det_card"))
            .fixed_pos(pos2(48.0, 36.0))
            .show(ui.ctx(), |ui| {
                GlassCard::new()
                    .elevation(GlassElevation::Floating)
                    .padding(Margin::same(0))
                    .show(ui, |ui| {
                        ui.allocate_exact_size(vec2(160.0, 110.0), egui::Sense::hover());
                    });
            });
    };

    let first = render_fixture(256, 180, 1.0, PrismTheme::glacial(), draw);
    let second = render_fixture(256, 180, 1.0, PrismTheme::glacial(), draw);

    assert_eq!(
        first.as_raw(),
        second.as_raw(),
        "identical scenes must render bit-for-bit identically"
    );
}

#[test]
fn golden_snapshots_match_baseline() {
    // 1) Glacial Liquid Glass — 3 elevation tiers on SculptedWaves
    let glacial = render_fixture(384, 240, 1.0, PrismTheme::glacial(), |ui, _theme| {
        for (idx, (elev, x)) in [
            (GlassElevation::Subtle, 20.0),
            (GlassElevation::Elevated, 140.0),
            (GlassElevation::Floating, 260.0),
        ]
        .into_iter()
        .enumerate()
        {
            Area::new(Id::NULL.with(("glacial_tier", idx)))
                .fixed_pos(pos2(x, 44.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(elev)
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(102.0, 152.0), egui::Sense::hover());
                        });
                });
        }
    });
    assert_matches_golden("glacial_liquid_glass", &glacial);

    // 2) Onyx Dark on LiquidPrism with Cobalt & Violet accent cards
    let mut onyx_theme = PrismTheme::onyx().with_accent_palette(AccentPalette::Violet);
    onyx_theme.backdrop_scene = BackdropScene::LiquidPrism;
    let onyx = render_fixture(384, 240, 1.0, onyx_theme, |ui, _theme| {
        Area::new(Id::NULL.with("onyx_left"))
            .fixed_pos(pos2(24.0, 32.0))
            .show(ui.ctx(), |ui| {
                GlassCard::new()
                    .elevation(GlassElevation::Elevated)
                    .padding(Margin::same(0))
                    .show(ui, |ui| {
                        ui.allocate_exact_size(vec2(160.0, 176.0), egui::Sense::hover());
                    });
            });
        Area::new(Id::NULL.with("onyx_right"))
            .fixed_pos(pos2(204.0, 52.0))
            .show(ui.ctx(), |ui| {
                GlassCard::new()
                    .elevation(GlassElevation::Floating)
                    .corner_radius(999.0)
                    .padding(Margin::same(0))
                    .show(ui, |ui| {
                        ui.allocate_exact_size(vec2(156.0, 64.0), egui::Sense::hover());
                    });
            });
    });
    assert_matches_golden("onyx_dark_glass", &onyx);

    // 3) Frost Light on ArchitecturalGrid with Emerald accent
    let mut frost_theme = PrismTheme::frost().with_accent_palette(AccentPalette::Emerald);
    frost_theme.backdrop_scene = BackdropScene::ArchitecturalGrid;
    let frost = render_fixture(384, 240, 1.0, frost_theme, |ui, _theme| {
        Area::new(Id::NULL.with("frost_card"))
            .fixed_pos(pos2(44.0, 36.0))
            .show(ui.ctx(), |ui| {
                GlassCard::new()
                    .elevation(GlassElevation::Floating)
                    .padding(Margin::same(0))
                    .show(ui, |ui| {
                        ui.allocate_exact_size(vec2(296.0, 168.0), egui::Sense::hover());
                    });
            });
    });
    assert_matches_golden("frost_light_glass", &frost);

    // 4) Tinted pills & corner radii (0pt, 18pt, 999pt pill)
    let tinted = render_fixture(384, 240, 1.0, PrismTheme::glacial(), |ui, _theme| {
        let specs = [
            (
                0.0_f32,
                Color32::from_rgba_unmultiplied(235, 36, 68, 92),
                20.0,
            ),
            (
                18.0_f32,
                Color32::from_rgba_unmultiplied(16, 185, 129, 92),
                140.0,
            ),
            (
                999.0_f32,
                Color32::from_rgba_unmultiplied(36, 110, 245, 96),
                260.0,
            ),
        ];
        for (idx, (radius, tint, x)) in specs.into_iter().enumerate() {
            Area::new(Id::NULL.with(("tint_pill", idx)))
                .fixed_pos(pos2(x, 60.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(GlassElevation::Floating)
                        .corner_radius(radius)
                        .tint(tint)
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(104.0, 104.0), egui::Sense::hover());
                        });
                });
        }
    });
    assert_matches_golden("tinted_pills_and_corners", &tinted);

    // 5) Neumorphic Minimal Light (Ruby accent)
    let min_light_theme = PrismTheme::minimal_light().with_accent_palette(AccentPalette::Ruby);
    let min_light = render_fixture(384, 240, 1.0, min_light_theme, |ui, _theme| {
        for (idx, (elev, x)) in [
            (GlassElevation::Subtle, 24.0),
            (GlassElevation::Elevated, 144.0),
            (GlassElevation::Floating, 264.0),
        ]
        .into_iter()
        .enumerate()
        {
            Area::new(Id::NULL.with(("min_light", idx)))
                .fixed_pos(pos2(x, 48.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(elev)
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(96.0, 144.0), egui::Sense::hover());
                        });
                });
        }
    });
    assert_matches_golden("neumorphic_minimal_light", &min_light);

    // 6) Neumorphic Minimal Dark (Cyan accent)
    let min_dark_theme = PrismTheme::minimal_dark().with_accent_palette(AccentPalette::Cyan);
    let min_dark = render_fixture(384, 240, 1.0, min_dark_theme, |ui, _theme| {
        for (idx, (elev, x)) in [
            (GlassElevation::Subtle, 24.0),
            (GlassElevation::Elevated, 144.0),
            (GlassElevation::Floating, 264.0),
        ]
        .into_iter()
        .enumerate()
        {
            Area::new(Id::NULL.with(("min_dark", idx)))
                .fixed_pos(pos2(x, 48.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(elev)
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(96.0, 144.0), egui::Sense::hover());
                        });
                });
        }
    });
    assert_matches_golden("neumorphic_minimal_dark", &min_dark);
}

// ---------------------------------------------------------------------------
// 2. Clipping (`clip_rect` / `ScrollArea`) & Analytic SDF Corner Rounding
// ---------------------------------------------------------------------------

#[test]
fn scroll_and_clip_rect_have_zero_pixel_bleed() {
    let width = 320u32;
    let height = 240u32;
    let clip = Rect::from_min_max(pos2(40.0, 30.0), pos2(200.0, 150.0));

    let backdrop_only = render_fixture(width, height, 1.0, PrismTheme::glacial(), |_, _| {});

    let clipped = render_fixture(width, height, 1.0, PrismTheme::glacial(), move |ui, _| {
        ui.scope_builder(UiBuilder::new().max_rect(clip), |ui| {
            ui.set_clip_rect(clip);
            ScrollArea::vertical()
                .id_salt("clip_scroll_test")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    // Card intentionally overflows clip rect on both right & bottom!
                    GlassCard::new()
                        .elevation(GlassElevation::Floating)
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(240.0, 200.0), egui::Sense::hover());
                        });
                });
        });
    });

    // Verify glass actually rendered inside the clip rect.
    let inside_diff = region_mae(&backdrop_only, &clipped, 60, 50, 100, 70);
    assert!(
        inside_diff > 10.0,
        "expected visible clipped glass inside clip_rect (inside_diff={inside_diff:.2})"
    );

    // Verify ZERO pixels leaked outside clip_rect (1px tolerance for integer scissor boundary).
    let raw_ref = backdrop_only.as_raw();
    let raw_clip = clipped.as_raw();
    let mut leaked_pixels = 0_usize;
    for py in 0..height {
        for px in 0..width {
            let outside_x = !(39..=201).contains(&px);
            let outside_y = !(29..=151).contains(&py);
            if outside_x || outside_y {
                let idx = ((py * width + px) * 4) as usize;
                let max_ch_diff = (0..4)
                    .map(|c| raw_ref[idx + c].abs_diff(raw_clip[idx + c]))
                    .max()
                    .unwrap_or(0);
                if max_ch_diff > 1 {
                    leaked_pixels += 1;
                }
            }
        }
    }
    assert_eq!(
        leaked_pixels, 0,
        "GPU glass surface or shadow bled {leaked_pixels} pixels outside clip_rect"
    );
}

#[test]
fn corner_rounding_zero_standard_and_pill_clamped() {
    let width = 320u32;
    let height = 200u32;
    let checker = build_checkerboard(width, height, 8);

    let render_with_radius = |radius: Option<f32>| -> RgbaImage {
        let _gpu = lock_gpu();
        let checker_clone = checker.clone();
        let mut installed = false;
        let mut harness = Harness::builder()
            .with_size(vec2(width as f32, height as f32))
            .with_pixels_per_point(1.0)
            .build_ui(move |ui| {
                let theme = PrismTheme::onyx();
                ui.ctx().set_prism_theme(theme.clone());
                if !installed {
                    let provider = egui_prismglass::blur_wgpu::install_with_format(
                        ui.ctx(),
                        TextureFormat::Rgba8Unorm,
                    );
                    provider.set_external_backdrop_rgba8(width, height, checker_clone.clone());
                    installed = true;
                }
                paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);
                if let Some(r) = radius {
                    let mut tokens = theme
                        .surface(GlassElevation::Floating)
                        .with_corner_radius(r);
                    // Disable outer drop shadow to isolate exact analytic SDF corner mask.
                    tokens.shadow = egui::Shadow::NONE;
                    tokens.contact_shadow = egui::Shadow::NONE;
                    Area::new(Id::NULL.with("radius_card"))
                        .fixed_pos(pos2(60.0, 40.0))
                        .show(ui.ctx(), |ui| {
                            GlassCard::new()
                                .tokens(tokens)
                                .padding(Margin::same(0))
                                .show(ui, |ui| {
                                    ui.allocate_exact_size(
                                        vec2(200.0, 120.0),
                                        egui::Sense::hover(),
                                    );
                                });
                        });
                }
            });
        harness.step();
        harness.step();
        harness.render().expect("corner rounding render")
    };

    let ref_bg = render_with_radius(None);
    let sharp = render_with_radius(Some(0.0));
    let rounded = render_with_radius(Some(24.0));
    let pill = render_with_radius(Some(999.0));

    // Top-left corner patch of card at (60..64, 40..44):
    // - Sharp (r=0) covers this corner completely -> diverges strongly from backdrop.
    // - Rounded (r=24) leaves (60..64, 40..44) strictly outside the quarter-circle arc -> matches backdrop!
    // - Pill (r=999 clamped to 60) leaves (60..72, 40..52) outside the semicircle arc -> matches backdrop!
    let sharp_corner_diff = region_mae(&ref_bg, &sharp, 61, 41, 4, 4);
    let rounded_corner_diff = region_mae(&ref_bg, &rounded, 61, 41, 4, 4);
    let pill_outer_diff = region_mae(&ref_bg, &pill, 62, 42, 10, 10);
    let pill_center_var = luma_variance(&pill, 120, 75, 80, 50);
    let bg_var = luma_variance(&ref_bg, 120, 75, 80, 50);

    assert!(
        sharp_corner_diff > 15.0,
        "sharp corner (r=0) should render glass into corner pixel (diff={sharp_corner_diff:.2})"
    );
    assert!(
        rounded_corner_diff < 1.5,
        "rounded corner (r=24) must not paint outside quarter-circle arc (diff={rounded_corner_diff:.2})"
    );
    assert!(
        pill_outer_diff < 1.5,
        "pill corner (r=999 clamped to h/2) must not paint outside semicircle arc (diff={pill_outer_diff:.2})"
    );
    assert!(
        pill_center_var < bg_var * 0.10,
        "pill center must still receive full Dual-Kawase blur (center_var={pill_center_var:.1}, bg_var={bg_var:.1})"
    );
}

// ---------------------------------------------------------------------------
// 3. DPI Scaling Matrix (`1.0x`, `1.5x`, `2.0x`, `2.5x`)
// ---------------------------------------------------------------------------

#[test]
fn dpi_scaling_matrix_1x_to_2_5x() {
    let logical_w = 240u32;
    let logical_h = 160u32;
    let checker = build_checkerboard(640, 480, 8);

    for ppp in [1.0_f32, 1.5_f32, 2.0_f32, 2.5_f32] {
        let expected_w = (logical_w as f32 * ppp).round() as u32;
        let expected_h = (logical_h as f32 * ppp).round() as u32;
        let checker_clone = checker.clone();
        let metrics_slot: Arc<Mutex<Option<Arc<WgpuBlurProvider>>>> = Arc::new(Mutex::new(None));
        let metrics_slot_ui = metrics_slot.clone();

        let _gpu = lock_gpu();
        let mut harness = Harness::builder()
            .with_size(vec2(logical_w as f32, logical_h as f32))
            .with_pixels_per_point(ppp)
            .build_ui(move |ui| {
                let theme = PrismTheme::glacial();
                ui.ctx().set_prism_theme(theme.clone());
                let mut slot = metrics_slot_ui.lock().expect("lock");
                if slot.is_none() {
                    let provider = egui_prismglass::blur_wgpu::install_with_format(
                        ui.ctx(),
                        TextureFormat::Rgba8Unorm,
                    );
                    provider.set_external_backdrop_rgba8(640, 480, checker_clone.clone());
                    *slot = Some(provider);
                }
                drop(slot);

                paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);
                Area::new(Id::NULL.with("dpi_card"))
                    .fixed_pos(pos2(50.0, 30.0))
                    .show(ui.ctx(), |ui| {
                        GlassCard::new()
                            .elevation(GlassElevation::Floating)
                            .padding(Margin::same(0))
                            .show(ui, |ui| {
                                ui.allocate_exact_size(vec2(140.0, 100.0), egui::Sense::hover());
                            });
                    });
            });

        harness.step();
        harness.step();
        let img = harness.render().expect("DPI render should succeed");
        assert_eq!(
            img.dimensions(),
            (expected_w, expected_h),
            "DPI {ppp}x output dimensions mismatch"
        );

        let provider = metrics_slot
            .lock()
            .expect("lock")
            .clone()
            .expect("provider installed");
        let metrics = provider.metrics();
        assert_eq!(
            metrics.pyramid_size,
            [expected_w, expected_h],
            "DPI {ppp}x pyramid must allocate in physical pixels"
        );
        assert_eq!(
            metrics.estimated_vram_bytes,
            expected_6tex_vram_bytes(expected_w, expected_h)
        );

        // Verify blur works at every DPI scale.
        let out_var = luma_variance(
            &img,
            (6.0 * ppp) as u32,
            (6.0 * ppp) as u32,
            (36.0 * ppp) as u32,
            (36.0 * ppp) as u32,
        );
        let in_var = luma_variance(
            &img,
            (80.0 * ppp) as u32,
            (55.0 * ppp) as u32,
            (80.0 * ppp) as u32,
            (50.0 * ppp) as u32,
        );
        assert!(
            in_var < out_var * 0.12,
            "DPI {ppp}x blur did not smooth checkerboard (in={in_var:.1}, out={out_var:.1})"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Resolution Matrix & Dynamic Resize Reallocation Discipline
// ---------------------------------------------------------------------------

#[test]
fn resolution_matrix_and_dynamic_resize() {
    let provider_slot: Arc<Mutex<Option<Arc<WgpuBlurProvider>>>> = Arc::new(Mutex::new(None));
    let provider_ui = provider_slot.clone();

    let _gpu = lock_gpu();
    let mut harness = Harness::builder()
        .with_size(vec2(320.0, 240.0))
        .with_pixels_per_point(1.0)
        .build_ui(move |ui| {
            let theme = PrismTheme::glacial();
            ui.ctx().set_prism_theme(theme.clone());
            let mut slot = provider_ui.lock().expect("lock");
            if slot.is_none() {
                *slot = Some(egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    TextureFormat::Rgba8Unorm,
                ));
            }
            drop(slot);

            paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);
            Area::new(Id::NULL.with("resize_card"))
                .fixed_pos(pos2(32.0, 32.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(GlassElevation::Elevated)
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(180.0, 110.0), egui::Sense::hover());
                        });
                });
        });

    let resolutions = [(320u32, 240u32), (640, 360), (800, 600), (1280, 720)];
    for (step_idx, (w, h)) in resolutions.into_iter().enumerate() {
        harness.set_size(vec2(w as f32, h as f32));
        harness.step();
        let img1 = harness.render().expect("resize frame 1");
        assert_eq!(img1.dimensions(), (w, h));

        // Second stationary frame at same resolution must NOT reallocate!
        harness.step();
        let img2 = harness.render().expect("resize frame 2");
        assert_eq!(img2.dimensions(), (w, h));

        let metrics = provider_slot
            .lock()
            .expect("lock")
            .as_ref()
            .expect("provider")
            .metrics();

        assert_eq!(metrics.pyramid_size, [w, h]);
        assert_eq!(metrics.pyramid_texture_count, 6);
        assert_eq!(
            metrics.estimated_vram_bytes,
            expected_6tex_vram_bytes(w, h),
            "VRAM accounting mismatch at {w}x{h}"
        );
        assert_eq!(
            metrics.pyramid_allocations,
            (step_idx + 1) as u64,
            "expected exactly 1 allocation per resolution change at {w}x{h}"
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Screen Boundary & Edge Artifacts Verification
// ---------------------------------------------------------------------------

#[test]
fn screen_boundary_cards_have_no_edge_artifacts() {
    let width = 320u32;
    let height = 240u32;
    // Bright uniform pastel sky backdrop so any UV wrap-around / black edge / NaN stands out immediately.
    let sky = build_solid_rgba8(width, height, [210, 224, 244, 255]);

    let _gpu = lock_gpu();
    let mut installed = false;
    let mut harness = Harness::builder()
        .with_size(vec2(width as f32, height as f32))
        .with_pixels_per_point(1.0)
        .build_ui(move |ui| {
            let theme = PrismTheme::frost();
            ui.ctx().set_prism_theme(theme.clone());
            if !installed {
                let provider = egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    TextureFormat::Rgba8Unorm,
                );
                provider.set_external_backdrop_rgba8(width, height, sky.clone());
                installed = true;
            }
            paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);

            let mut tokens = theme
                .surface(GlassElevation::Floating)
                .with_refraction(1.0)
                .with_dispersion(1.0);
            tokens.shadow = egui::Shadow::NONE;
            tokens.contact_shadow = egui::Shadow::NONE;

            // Four cards flush / slightly overhanging all four viewport edges!
            let positions = [
                pos2(-8.0, -8.0),
                pos2(216.0, -8.0),
                pos2(-8.0, 164.0),
                pos2(216.0, 164.0),
            ];
            for (idx, pos) in positions.into_iter().enumerate() {
                Area::new(Id::NULL.with(("edge_card", idx)))
                    .fixed_pos(pos)
                    .show(ui.ctx(), |ui| {
                        GlassCard::new()
                            .tokens(tokens)
                            .padding(Margin::same(0))
                            .show(ui, |ui| {
                                ui.allocate_exact_size(vec2(112.0, 84.0), egui::Sense::hover());
                            });
                    });
            }
        });

    harness.step();
    harness.step();
    let img = harness.render().expect("edge artifact render");
    let raw = img.as_raw();

    for py in 0..height {
        for px in 0..width {
            let idx = ((py * width + px) * 4) as usize;
            let pixel = &raw[idx..idx + 4];
            assert_eq!(
                pixel[3], 255,
                "opaque viewport pixel ({px},{py}) had non-255 alpha={}",
                pixel[3]
            );
            let luma = pixel_luma(pixel);
            assert!(
                luma > 100.0,
                "dark edge/wrap/NaN artifact at ({px},{py}): rgba={pixel:?} luma={luma:.1}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Shader Optical Regressions & Intermediate Texture / Cache Reuse
// ---------------------------------------------------------------------------

#[test]
fn blur_radius_monotonicity() {
    let width = 320u32;
    let height = 200u32;
    let checker = build_checkerboard(width, height, 6);

    let measure_blur_var = |radius: f32| -> f64 {
        let _gpu = lock_gpu();
        let checker_clone = checker.clone();
        let mut installed = false;
        let mut harness = Harness::builder()
            .with_size(vec2(width as f32, height as f32))
            .with_pixels_per_point(1.0)
            .build_ui(move |ui| {
                let theme = PrismTheme::onyx();
                ui.ctx().set_prism_theme(theme.clone());
                if !installed {
                    let provider = egui_prismglass::blur_wgpu::install_with_format(
                        ui.ctx(),
                        TextureFormat::Rgba8Unorm,
                    );
                    provider.set_external_backdrop_rgba8(width, height, checker_clone.clone());
                    installed = true;
                }
                paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);
                let tokens = theme
                    .surface(GlassElevation::Elevated)
                    .with_blur_radius(radius)
                    .with_refraction(0.0)
                    .with_dispersion(0.0);
                Area::new(Id::NULL.with("mono_card"))
                    .fixed_pos(pos2(70.0, 40.0))
                    .show(ui.ctx(), |ui| {
                        GlassCard::new()
                            .tokens(tokens)
                            .padding(Margin::same(0))
                            .show(ui, |ui| {
                                ui.allocate_exact_size(vec2(180.0, 120.0), egui::Sense::hover());
                            });
                    });
            });
        harness.step();
        harness.step();
        let img = harness.render().expect("monotonicity render");
        luma_variance(&img, 100, 65, 120, 70)
    };

    let var_sharp = measure_blur_var(0.5);
    let var_med = measure_blur_var(10.0);
    let var_deep = measure_blur_var(32.0);

    assert!(
        var_sharp > var_med * 3.0,
        "blur=10pt should significantly smooth blur=0.5pt (var0.5={var_sharp:.1}, var10={var_med:.1})"
    );
    assert!(
        var_med > var_deep * 1.4,
        "blur=32pt should be smoother than blur=10pt (var10={var_med:.1}, var32={var_deep:.1})"
    );
}

#[test]
fn snell_refraction_and_chromatic_dispersion_invariants() {
    let width = 320u32;
    let height = 200u32;
    let stripes = build_vertical_stripes(width, height, 10);

    let render_optics = |refraction: f32, dispersion: f32| -> RgbaImage {
        let _gpu = lock_gpu();
        let stripes_clone = stripes.clone();
        let mut installed = false;
        let mut harness = Harness::builder()
            .with_size(vec2(width as f32, height as f32))
            .with_pixels_per_point(1.0)
            .build_ui(move |ui| {
                let theme = PrismTheme::glacial();
                ui.ctx().set_prism_theme(theme.clone());
                if !installed {
                    let provider = egui_prismglass::blur_wgpu::install_with_format(
                        ui.ctx(),
                        TextureFormat::Rgba8Unorm,
                    );
                    provider.set_external_backdrop_rgba8(width, height, stripes_clone.clone());
                    installed = true;
                }
                paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);
                let tokens = GlassSurfaceTokens {
                    blur_radius: 2.0,
                    refraction_strength: refraction,
                    chromatic_dispersion: dispersion,
                    prism_intensity: 0.0,
                    grain_intensity: 0.0,
                    ..theme.surface(GlassElevation::Floating)
                };
                Area::new(Id::NULL.with("optics_card"))
                    .fixed_pos(pos2(60.0, 40.0))
                    .show(ui.ctx(), |ui| {
                        GlassCard::new()
                            .tokens(tokens)
                            .padding(Margin::same(0))
                            .show(ui, |ui| {
                                ui.allocate_exact_size(vec2(200.0, 120.0), egui::Sense::hover());
                            });
                    });
            });
        harness.step();
        harness.step();
        harness.render().expect("optics render")
    };

    let flat = render_optics(0.0, 0.0);
    let refracted = render_optics(1.0, 0.0);
    let dispersed = render_optics(1.0, 1.0);

    // Convex bezel strip on the left edge of the card (x = 63..77, y = 60..140):
    let bezel_refraction_diff = region_mae(&flat, &refracted, 63, 60, 14, 80);
    assert!(
        bezel_refraction_diff > 6.0,
        "Snell refraction should visibly bend stripes along the bezel (diff={bezel_refraction_diff:.2})"
    );

    // Chromatic dispersion splits R & B channels across the bezel transition.
    let mean_rb_delta = |img: &RgbaImage| -> f64 {
        let raw = img.as_raw();
        let mut sum = 0.0_f64;
        let mut n = 0.0_f64;
        for py in 60..140 {
            for px in 63..77 {
                let idx = ((py * width + px) * 4) as usize;
                sum += (f64::from(raw[idx]) - f64::from(raw[idx + 2])).abs();
                n += 1.0;
            }
        }
        sum / n
    };

    let rb_no_disp = mean_rb_delta(&refracted);
    let rb_with_disp = mean_rb_delta(&dispersed);
    assert!(
        rb_with_disp > rb_no_disp + 1.5,
        "RGB chromatic dispersion must widen |R-B| fringe at the bezel (without={rb_no_disp:.2}, with={rb_with_disp:.2})"
    );
}

#[test]
fn beer_lambert_tinted_glass_transmission() {
    let width = 320u32;
    let height = 180u32;
    let white = build_solid_rgba8(width, height, [238, 240, 244, 255]);

    let _gpu = lock_gpu();
    let mut installed = false;
    let mut harness = Harness::builder()
        .with_size(vec2(width as f32, height as f32))
        .with_pixels_per_point(1.0)
        .build_ui(move |ui| {
            let theme = PrismTheme::glacial();
            ui.ctx().set_prism_theme(theme.clone());
            if !installed {
                let provider = egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    TextureFormat::Rgba8Unorm,
                );
                provider.set_external_backdrop_rgba8(width, height, white.clone());
                installed = true;
            }
            paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);

            Area::new(Id::NULL.with("ruby_card"))
                .fixed_pos(pos2(24.0, 30.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(GlassElevation::Floating)
                        .tint(Color32::from_rgba_unmultiplied(235, 36, 68, 128))
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(120.0, 120.0), egui::Sense::hover());
                        });
                });

            Area::new(Id::NULL.with("cobalt_card"))
                .fixed_pos(pos2(176.0, 30.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(GlassElevation::Floating)
                        .tint(Color32::from_rgba_unmultiplied(36, 110, 245, 128))
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(120.0, 120.0), egui::Sense::hover());
                        });
                });
        });

    harness.step();
    harness.step();
    let img = harness.render().expect("beer-lambert render");

    let [rr, rg, rb] = mean_rgb(&img, 54, 60, 60, 60);
    let [cr, cg, cb] = mean_rgb(&img, 206, 60, 60, 60);

    assert!(
        rr > rg * 1.30 && rr > rb * 1.25,
        "Ruby Beer-Lambert glass should strongly transmit red (rgb=[{rr:.1}, {rg:.1}, {rb:.1}])"
    );
    assert!(
        cb > cr * 1.30 && cb > cg * 1.10,
        "Cobalt Beer-Lambert glass should strongly transmit blue (rgb=[{cr:.1}, {cg:.1}, {cb:.1}])"
    );
}

#[test]
fn fallback_toggle_switches_cleanly_between_gpu_and_cpu() {
    let real_blur_flag = Arc::new(Mutex::new(true));
    let real_blur_ui = real_blur_flag.clone();
    let observed_cap = Arc::new(Mutex::new(BlurCapability::FallbackOnly));
    let observed_cap_ui = observed_cap.clone();
    let mut installed = false;

    let _gpu = lock_gpu();
    let mut harness = Harness::builder()
        .with_size(vec2(256.0, 180.0))
        .with_pixels_per_point(1.0)
        .build_ui(move |ui| {
            let mut theme = PrismTheme::glacial();
            theme.real_blur_enabled = *real_blur_ui.lock().expect("lock");
            ui.ctx().set_prism_theme(theme.clone());
            if !installed {
                egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    TextureFormat::Rgba8Unorm,
                );
                installed = true;
            }
            *observed_cap_ui.lock().expect("lock") = ui.ctx().prism_blur_capability();

            paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);
            Area::new(Id::NULL.with("toggle_card"))
                .fixed_pos(pos2(48.0, 36.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(GlassElevation::Floating)
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(160.0, 108.0), egui::Sense::hover());
                        });
                });
        });

    harness.step();
    let gpu_img = harness.render().expect("gpu frame");
    assert_eq!(
        *observed_cap.lock().expect("lock"),
        BlurCapability::RealBackdrop {
            backend: "wgpu-dual-kawase"
        }
    );

    *real_blur_flag.lock().expect("lock") = false;
    harness.step();
    let cpu_img = harness.render().expect("fallback frame");
    assert_eq!(
        *observed_cap.lock().expect("lock"),
        BlurCapability::FallbackOnly
    );

    *real_blur_flag.lock().expect("lock") = true;
    harness.step();
    let gpu_img_again = harness.render().expect("gpu re-enabled frame");

    assert!(
        region_mae(&gpu_img, &cpu_img, 60, 50, 120, 80) > 5.0,
        "GPU blur and CPU fallback should produce visibly distinct glass interiors"
    );
    assert_eq!(
        gpu_img.as_raw(),
        gpu_img_again.as_raw(),
        "re-enabling real GPU blur must restore bit-exact output"
    );
}

#[test]
fn intermediate_texture_reuse_and_static_pyramid_caching() {
    #[derive(Clone, Copy)]
    struct FrameConfig {
        time_secs: f32,
        card_count: usize,
    }

    let config = Arc::new(Mutex::new(FrameConfig {
        time_secs: 0.0,
        card_count: 20,
    }));
    let config_ui = config.clone();
    let provider_slot: Arc<Mutex<Option<Arc<WgpuBlurProvider>>>> = Arc::new(Mutex::new(None));
    let provider_ui = provider_slot.clone();

    let _gpu = lock_gpu();
    let mut harness = Harness::builder()
        .with_size(vec2(640.0, 360.0))
        .with_pixels_per_point(1.0)
        .build_ui(move |ui| {
            let theme = PrismTheme::glacial();
            ui.ctx().set_prism_theme(theme.clone());
            let mut slot = provider_ui.lock().expect("lock");
            if slot.is_none() {
                *slot = Some(egui_prismglass::blur_wgpu::install_with_format(
                    ui.ctx(),
                    TextureFormat::Rgba8Unorm,
                ));
            }
            drop(slot);

            let cfg = *config_ui.lock().expect("lock");
            paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, cfg.time_secs, 1.0);
            for idx in 0..cfg.card_count {
                let col = (idx % 8) as f32;
                let row = (idx / 8) as f32;
                Area::new(Id::NULL.with(("reuse_card", idx)))
                    .fixed_pos(pos2(16.0 + col * 76.0, 16.0 + row * 64.0))
                    .show(ui.ctx(), |ui| {
                        GlassCard::new()
                            .elevation(GlassElevation::Elevated)
                            .padding(Margin::same(0))
                            .show(ui, |ui| {
                                ui.allocate_exact_size(vec2(68.0, 52.0), egui::Sense::hover());
                            });
                    });
            }
        });

    let metrics = || -> WgpuBlurMetrics {
        provider_slot
            .lock()
            .expect("lock")
            .as_ref()
            .expect("provider")
            .metrics()
    };

    // Phase 1: 10 stationary frames with 20 cards -> 1 allocation, 1 pyramid execution, 9 cache hits!
    for _ in 0..10 {
        harness.step();
        let _ = harness.render().expect("static cached frame");
    }
    let m1 = metrics();
    assert_eq!(m1.pyramid_texture_count, 6);
    assert_eq!(m1.pyramid_allocations, 1);
    assert_eq!(m1.pyramid_passes_executed, 1);
    assert_eq!(m1.pyramid_cache_hits, 9);
    assert_eq!(m1.card_buffer_allocations, 1);
    assert_eq!(m1.last_frame_card_count, 20);

    // Phase 2: 5 animated frames -> pyramid textures reused (0 reallocs), 5 new pyramid passes.
    for step in 1..=5 {
        config.lock().expect("lock").time_secs = step as f32 * 0.016;
        harness.step();
        let _ = harness.render().expect("animated frame");
    }
    let m2 = metrics();
    assert_eq!(m2.pyramid_allocations, 1);
    assert_eq!(m2.pyramid_passes_executed, 6);
    assert_eq!(m2.pyramid_cache_hits, 9);
    assert_eq!(m2.card_buffer_allocations, 1);

    // Phase 3: Grow to 40 simultaneous cards (> initial capacity 32) -> grows buffer once, then reuses!
    config.lock().expect("lock").card_count = 40;
    for _ in 0..3 {
        harness.step();
        let _ = harness.render().expect("40-card frame");
    }
    let m3 = metrics();
    assert_eq!(m3.pyramid_allocations, 1);
    assert_eq!(m3.card_buffer_allocations, 2);
    assert_eq!(m3.last_frame_card_count, 40);
    assert_eq!(m3.pyramid_cache_hits, 12);
}
