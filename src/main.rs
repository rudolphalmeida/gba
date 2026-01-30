use eframe::egui;

use crate::ui::GbaApp;

mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rGBA",
        options,
        Box::new(|_cc| {
            Ok(Box::new(GbaApp::new()))
        }),
    )
}
