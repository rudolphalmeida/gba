use iced::widget::{button, center_x};
use iced::{
    widget::{column, text},
    Element, Theme,
};
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    bios_path: Option<PathBuf>,
    rom_path: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone)]
pub enum AppMessage {
    ShowBiosPicker,
    ShowRomPicker,
}

impl AppState {
    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::ShowBiosPicker => self.choose_bios_file(),
            AppMessage::ShowRomPicker => self.choose_rom_file(),
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        let load_bios_button = button("Load BIOS").on_press(AppMessage::ShowBiosPicker);
        let load_rom_button = button("Load ROM").on_press(AppMessage::ShowRomPicker);

        let bios_path_display = if let Some(bios_path) = self.bios_path.as_ref() {
            text(format!("Selected BIOS: {bios_path:?}"))
        } else {
            text("Select BIOS to get started")
        };

        let rom_path_display = if let Some(rom_path) = self.rom_path.as_ref() {
            text(format!("Selected ROM: {rom_path:?}"))
        } else {
            text("Select ROM to get started")
        };

        center_x(
            column![
                text("GBA emulator"),
                bios_path_display,
                load_bios_button,
                rom_path_display,
                load_rom_button
            ]
            .spacing(20),
        )
        .into()
    }

    fn choose_bios_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("GBA BIOS", &["bin"])
            .pick_file()
        {
            self.bios_path = Some(path);
        }
    }

    fn choose_rom_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("GBA ROM", &["gba"])
            .pick_file()
        {
            self.rom_path = Some(path);
        }
    }
}
