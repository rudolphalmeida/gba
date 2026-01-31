use eframe::egui;

use crate::ui::GbaApp;

mod ui;

const APP_ID: &'static str = "rgba";

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_app_id(APP_ID).with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rGBA",
        options,
        Box::new(|cc| {
            Ok(Box::new(GbaApp::new(cc)))
        }),
    )
}
