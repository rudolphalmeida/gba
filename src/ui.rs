use eframe::egui::Context;
use eframe::{egui, CreationContext, Frame};
use gba::gba::Gba;
use std::path::PathBuf;

pub struct GbaApp {
    trace_opcode_viewer: TraceOpcodeViewer,
    gba: Option<Gba>,

    rom_path: Option<PathBuf>,
    bios_path: Option<PathBuf>,

    show_opcode_tracer: bool,
}

impl GbaApp {
    pub fn new(_cc: &CreationContext) -> Self {
        Self {
            trace_opcode_viewer: TraceOpcodeViewer::new(),
            gba: None,
            rom_path: None,
            bios_path: None,

            show_opcode_tracer: true,
        }
    }

    pub fn render_ui(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("main-menu").show(ctx, |ui| {
            self.show_main_menu(ui, ctx, frame);
        });

        if let Some(gba) = self.gba.as_mut() {
            self.trace_opcode_viewer.render_ui(gba);
        } else {
        }
    }

    fn begin_rom_if_possible(&mut self) {
        if let Some(rom) = self.rom_path.as_ref() && let Some(bios) = self.bios_path.as_ref() {
            self.gba = Some(Gba::new(rom, bios).unwrap())
        }
    }

    fn show_main_menu(&mut self, ui: &mut egui::Ui, _ctx: &Context, _frame: &mut Frame) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    if let Some(rom) = rfd::FileDialog::new().pick_file() {
                        self.rom_path = Some(rom.into());
                        self.begin_rom_if_possible();
                    }
                }
                if ui.button("Select BIOS").clicked() {
                    if let Some(bios) = rfd::FileDialog::new().pick_file() {
                        self.bios_path = Some(bios.into());
                        self.begin_rom_if_possible();
                    }
                }
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            })
        });
    }
}

impl eframe::App for GbaApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.render_ui(ctx, frame);
    }
}

struct TraceOpcodeViewer;

impl TraceOpcodeViewer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render_ui(&mut self, gba: &Gba) {
        // egui::Window::new("Opcode trace").vscroll(true).open(true).show(ui, |ui| egui::Label::new("Opcodes"))
    }
}
