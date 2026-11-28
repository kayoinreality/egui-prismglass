//! Launcher for the `egui-prismglass` Prism Studio demo.
//!
//! - Native: `cargo run -p demo`
//! - Web: `trunk serve` (or `trunk build --release`) inside `crates/demo`, see `index.html`.

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

/// Installs fonts and the GPU blur provider (when compiled in and available), then builds the app.
fn create_app(cc: &eframe::CreationContext<'_>) -> EframeDemo {
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
    EframeDemo {
        app: StudioApp::default(),
        #[cfg(feature = "blur-glow")]
        glow_blur,
    }
}

#[cfg(not(target_arch = "wasm32"))]
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
        Box::new(|cc| Ok(Box::new(create_app(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("no document");
        let canvas = document
            .get_element_by_id("prism_canvas")
            .expect("missing <canvas id=\"prism_canvas\">")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#prism_canvas is not a canvas");
        let start = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(create_app(cc)))),
            )
            .await;
        if let Some(loading) = document.get_element_by_id("loading") {
            match start {
                Ok(()) => loading.remove(),
                Err(e) => loading.set_inner_html(&format!(
                    "<p>Prism Studio could not start: {e:?}. Your browser may lack WebGL 2.</p>"
                )),
            }
        }
    });
}
