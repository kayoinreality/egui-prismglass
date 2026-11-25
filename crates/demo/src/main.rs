//! Native launcher for the `egui-prismglass` showcase demo (`cargo run -p demo`).

use demo::DemoApp;
use eframe::egui;

struct EframeDemo {
    app: DemoApp,
}

impl eframe::App for EframeDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.app.show(ui);
            });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-prismglass — Interactive Showcase")
            .with_inner_size([1280.0, 810.0])
            .with_min_inner_size([920.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-prismglass",
        options,
        Box::new(|_cc| {
            Ok(Box::new(EframeDemo {
                app: DemoApp::default(),
            }))
        }),
    )
}
