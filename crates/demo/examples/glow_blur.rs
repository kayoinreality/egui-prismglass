//! Demonstrates real live-framebuffer GPU backdrop blur on `eframe` + `glow` (OpenGL).
//!
//! Run with:
//! ```bash
//! cargo run -p demo --example glow_blur --no-default-features --features blur-glow
//! ```

#![allow(missing_docs)]

use std::sync::Arc;

use demo::DemoApp;
use eframe::egui;
use egui_prismglass::blur_glow::GlowBlurProvider;

struct GlowDemoApp {
    app: DemoApp,
    blur_provider: Option<Arc<GlowBlurProvider>>,
}

impl eframe::App for GlowDemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.app.show(ui);
            });
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        if let (Some(provider), Some(gl)) = (&self.blur_provider, gl) {
            provider.destroy(gl);
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-prismglass v0.2 — Real OpenGL Grab-Pass Blur (glow)")
            .with_inner_size([1280.0, 810.0])
            .with_min_inner_size([920.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-prismglass-glow",
        options,
        Box::new(|cc| {
            let blur_provider = cc
                .gl
                .as_ref()
                .and_then(|gl| egui_prismglass::blur_glow::install(&cc.egui_ctx, gl).ok());
            Ok(Box::new(GlowDemoApp {
                app: DemoApp::default(),
                blur_provider,
            }))
        }),
    )
}
