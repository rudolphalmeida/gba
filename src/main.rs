use ui::GbaUi;

mod ui;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("GBA Emulator", native_options, Box::new(|cc| Ok(Box::new(GbaUi::new(cc))))).unwrap()
}

