//! Native launcher for the `egui-prismglass` Prism Studio demo (`cargo run -p demo`).

use demo::StudioApp;
use eframe::egui;

struct EframeDemo {
    app: StudioApp,
    #[cfg(feature = "blur-glow")]
    glow_blur: Option<std::sync::Arc<egui_prismglass::blur_glow::GlowBlurProvider>>,
}

impl eframe::App for EframeDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.app.show(ui);
            });
    }

    #[cfg(feature = "blur-glow")]
    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        if let (Some(provider), Some(gl)) = (&self.glow_blur, gl) {
            provider.destroy(gl);
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Prism Studio — egui-prismglass")
            .with_inner_size([1280.0, 810.0])
            .with_min_inner_size([920.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-prismglass",
        options,
        Box::new(|cc| {
            #[cfg(feature = "blur-wgpu")]
            if let Some(render_state) = cc.wgpu_render_state.as_ref() {
                egui_prismglass::blur_wgpu::install(&cc.egui_ctx, render_state);
            }

            #[cfg(feature = "blur-glow")]
            let glow_blur = cc
                .gl
                .as_ref()
                .and_then(|gl| egui_prismglass::blur_glow::install(&cc.egui_ctx, gl).ok());

            egui_prismglass::install_fonts(&cc.egui_ctx);
            Ok(Box::new(EframeDemo {
                app: StudioApp::default(),
                #[cfg(feature = "blur-glow")]
                glow_blur,
            }))
        }),
    )
}
