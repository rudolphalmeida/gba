use ui::GbaUi;

mod ui;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "GBA",
        native_options,
        Box::new(|cc| Ok(Box::new(GbaUi::new(cc)))),
    )
    .unwrap()
}
