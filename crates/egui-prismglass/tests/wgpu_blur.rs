//! Headless GPU integration test verifying real WGSL Dual-Kawase blur & Liquid Glass rendering.

#![cfg(feature = "blur-wgpu")]
#![allow(missing_docs, clippy::cast_precision_loss)]

use egui::{Area, Id, Margin, pos2, vec2};
use egui_kittest::Harness;
use egui_prismglass::blur_wgpu::wgpu::TextureFormat;
use egui_prismglass::{
    BackdropScene, BlurCapability, GlassCard, GlassElevation, PrismContextExt, PrismTheme,
    paint_prism_backdrop_ui,
};

fn build_high_contrast_checkerboard(width: u32, height: u32, cell: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        for col in 0..width {
            let light = ((col / cell) + (row / cell)).is_multiple_of(2);
            if light {
                bytes.extend_from_slice(&[240, 245, 255, 255]);
            } else {
                bytes.extend_from_slice(&[14, 18, 28, 255]);
            }
        }
    }
    bytes
}

fn luma_variance(raw: &[u8], img_width: u32, x0: u32, y0: u32, w: u32, h: u32) -> f64 {
    let mut sum = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    let count = f64::from(w * h);
    for py in y0..y0 + h {
        for px in x0..x0 + w {
            let idx = ((py * img_width + px) * 4) as usize;
            let luma = 0.2126 * f64::from(raw[idx])
                + 0.7152 * f64::from(raw[idx + 1])
                + 0.0722 * f64::from(raw[idx + 2]);
            sum += luma;
            sum_sq += luma * luma;
        }
    }
    let mean = sum / count;
    (sum_sq / count) - (mean * mean)
}

#[test]
fn wgpu_dual_kawase_actually_blurs_high_contrast_backdrop() {
    let width = 320u32;
    let height = 240u32;
    let checker = build_high_contrast_checkerboard(width, height, 8);

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
                provider.set_external_backdrop_rgba8(width, height, checker.clone());
                installed = true;
            }

            assert_eq!(
                ui.ctx().prism_blur_capability(),
                BlurCapability::RealBackdrop {
                    backend: "wgpu-dual-kawase"
                }
            );

            paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.0, 1.0);

            Area::new(Id::NULL.with("test_blur_card"))
                .fixed_pos(pos2(90.0, 60.0))
                .show(ui.ctx(), |ui| {
                    GlassCard::new()
                        .elevation(GlassElevation::Floating)
                        .padding(Margin::same(0))
                        .show(ui, |ui| {
                            ui.allocate_exact_size(vec2(140.0, 120.0), egui::Sense::hover());
                        });
                });
        });

    harness.step();
    harness.step();

    let image = harness
        .render()
        .expect("wgpu headless render should succeed with blur-wgpu shaders");
    let raw = image.as_raw();
    let img_w = image.width();

    // Outside region (sharp checkerboard): very high luminance variance.
    let outside_var = luma_variance(raw, img_w, 10, 10, 64, 64);
    // Inside GlassCard center (Dual-Kawase blurred): dramatically smoothed variance!
    let inside_var = luma_variance(raw, img_w, 120, 88, 80, 64);

    assert!(
        outside_var > 4000.0,
        "expected sharp checkerboard outside card (var={outside_var:.1})"
    );
    assert!(
        inside_var < outside_var * 0.08,
        "expected real GPU Dual-Kawase blur inside card to crush variance (inside={inside_var:.1}, outside={outside_var:.1})"
    );
}

#[test]
fn wgpu_procedural_scenes_render_without_validation_errors() {
    for scene in [
        BackdropScene::SculptedWaves,
        BackdropScene::LiquidPrism,
        BackdropScene::ArchitecturalGrid,
    ] {
        let mut installed = false;
        let mut harness = Harness::builder()
            .with_size(vec2(256.0, 192.0))
            .with_pixels_per_point(1.0)
            .build_ui(move |ui| {
                let mut theme = PrismTheme::glacial();
                theme.backdrop_scene = scene;
                ui.ctx().set_prism_theme(theme.clone());

                if !installed {
                    egui_prismglass::blur_wgpu::install_with_format(
                        ui.ctx(),
                        TextureFormat::Rgba8Unorm,
                    );
                    installed = true;
                }

                paint_prism_backdrop_ui(ui, ui.max_rect(), &theme, 0.25, 1.0);
                GlassCard::new().show(ui, |ui| {
                    ui.allocate_exact_size(vec2(120.0, 80.0), egui::Sense::hover());
                });
            });

        harness.step();
        let image = harness
            .render()
            .expect("procedural WGSL studio scene should render cleanly");
        assert_eq!(image.width(), 256);
        assert_eq!(image.height(), 192);
    }
}
