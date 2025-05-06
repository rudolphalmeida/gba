use crate::ui::emulation_thread::{
    emulation_thread, EmulationCommand, EmulationCtx, EmulatorUpdate,
};
use eframe::egui::{menu, Context};
use eframe::{egui, Frame};
use egui_notify::Toasts;
use std::path::PathBuf;
use std::time::Duration;

mod emulation_thread;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct GbaUi {
    bios_path: Option<PathBuf>,

    #[serde(skip)]
    toasts: Toasts,

    #[serde(skip)]
    comm_ctx: Option<EmuCommCtx>,
}

struct EmuCommCtx {
    emulation_thread: std::thread::JoinHandle<()>,
    cmd_send: std::sync::mpsc::SyncSender<EmulationCommand>,
    emu_recv: std::sync::mpsc::Receiver<EmulatorUpdate>,
}

impl EmuCommCtx {
    pub fn new(emulation_ctx: EmulationCtx) -> Self {
        let (cmd_send, cmd_recv) = std::sync::mpsc::sync_channel(0);
        let (emu_send, emu_recv) = std::sync::mpsc::channel();

        let emulation_thread = std::thread::spawn(move || {
            emulation_thread(emulation_ctx, cmd_recv, emu_send);
        });

        EmuCommCtx {
            emulation_thread,
            cmd_send,
            emu_recv,
        }
    }
}

impl GbaUi {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_logger::builder().init().unwrap();

        let emulation_ctx = EmulationCtx::default();
        let comm_ctx = EmuCommCtx::new(emulation_ctx);

        if let Some(storage) = cc.storage {
            return Self {
                comm_ctx: Some(comm_ctx),
                ..eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
            };
        }

        Self {
            comm_ctx: Some(comm_ctx),
            ..Self::default()
        }
    }

    fn send_command(&mut self, command: EmulationCommand) {
        self.comm_ctx
            .as_mut()
            .unwrap()
            .cmd_send
            .send(command)
            .unwrap();
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
                        self.send_command(EmulationCommand::LoadRom {
                            rom: path,
                            bios: self.bios_path.clone().unwrap(),
                        })
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
        while let Ok(update) = self.comm_ctx.as_mut().unwrap().emu_recv.try_recv() {
            match update {
                EmulatorUpdate::LoadError(e) => {
                    self.toasts.error(e).closable(true);
                }
                EmulatorUpdate::LoadSuccess(s) => {
                    self.toasts.success(s).closable(true);
                }
            }
        }

        self.toasts.show(ctx);

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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.send_command(EmulationCommand::Exit);
        self.comm_ctx
            .take()
            .unwrap()
            .emulation_thread
            .join()
            .unwrap();
    }
}
