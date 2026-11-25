//! Renders real high-DPI screenshots of the `egui-prismglass` showcase into `assets/`.
//!
//! Run with:
//! ```bash
//! cargo run -p demo --example render_assets
//! ```

#![allow(missing_docs)]

use std::fs;
use std::path::Path;

use demo::{DemoApp, DemoTab};
use egui::vec2;
use egui_kittest::Harness;
use egui_prismglass::PrismPreset;

fn capture(preset: PrismPreset, tab: DemoTab, path: &Path) {
    let mut app = DemoApp::with_preset(preset);
    app.active_tab = tab;

    let mut harness = Harness::builder()
        .with_size(vec2(1280.0, 810.0))
        .with_pixels_per_point(1.5)
        .build_ui(move |ui| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    app.show(ui);
                });
        });

    harness.step();
    harness.step();

    let image = harness
        .render()
        .expect("headless wgpu renderer should produce RGBA image");
    image
        .save(path)
        .expect("failed to write rendered showcase screenshot");
    println!(
        "Saved {} ({}x{})",
        path.display(),
        image.width(),
        image.height()
    );
}

fn main() {
    let assets_dir = Path::new("assets");
    fs::create_dir_all(assets_dir).expect("failed to create assets directory");

    capture(
        PrismPreset::Frost,
        DemoTab::Overview,
        &assets_dir.join("showcase-frost.png"),
    );
    capture(
        PrismPreset::Onyx,
        DemoTab::Overview,
        &assets_dir.join("showcase-onyx.png"),
    );
    capture(
        PrismPreset::MinimalLight,
        DemoTab::Overview,
        &assets_dir.join("showcase-minimal-light.png"),
    );
    capture(
        PrismPreset::MinimalDark,
        DemoTab::Overview,
        &assets_dir.join("showcase-minimal-dark.png"),
    );
    capture(
        PrismPreset::Glacial,
        DemoTab::Components,
        &assets_dir.join("showcase-glacial.png"),
    );
    capture(
        PrismPreset::Onyx,
        DemoTab::Motion,
        &assets_dir.join("showcase-motion.png"),
    );
    capture(
        PrismPreset::Onyx,
        DemoTab::Tokens,
        &assets_dir.join("showcase-tokens.png"),
    );
}
