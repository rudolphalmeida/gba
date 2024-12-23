use eframe::egui::{menu, Context};
use eframe::{egui, Frame};
use egui_notify::Toasts;
use gba::gba::Gba;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct GbaUi {
    #[serde(skip)]
    gba: Option<Gba>,
    bios_path: Option<PathBuf>,

    #[serde(skip)]
    toasts: Toasts,
}

impl GbaUi {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_logger::builder().init().unwrap();

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self::default()
    }

    fn show_main_menu(&mut self, ui: &mut egui::Ui) {
        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    if self.bios_path.is_none() {
                        self.toasts
                            .info("BIOS path needs to be selected")
                            .duration(Some(Duration::from_secs(5)));
                        return;
                    }

                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("GBA Roms", &["gba"])
                        .pick_file()
                    {
                        self.gba = match Gba::new(path, self.bios_path.as_ref().unwrap()) {
                            Ok(gba) => Some(gba),
                            Err(e) => {
                                self.toasts.error(e).closable(true);
                                None
                            }
                        };
                    }
                }
            });

            ui.menu_button("BIOS", |ui| {
                if let Some(path) = self.bios_path.as_ref() {
                    ui.label(path.to_str().unwrap().to_string());
                }

                if ui.button("Select").clicked() {
                    self.bios_path = rfd::FileDialog::new()
                        .add_filter("GBA BIOS", &["bin"])
                        .pick_file();
                }
            });
        });
    }
}

impl eframe::App for GbaUi {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        self.toasts.show(ctx);

        if let Some(gba) = self.gba.as_mut() {
            gba.step();
        }

        egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
            self.show_main_menu(ui);
        });

        egui::Window::new("Log").show(ctx, |ui| {
            egui_logger::logger_ui().show(ui);
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
