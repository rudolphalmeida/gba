use crate::ui::disasm::{condition_text, opcode_disassembly};
use eframe::egui::{Color32, Context, Ui};
use eframe::{egui, CreationContext, Frame};
use gba::cpu::{ExecutedOpcode, OpcodeTraceLog};
use gba::gba::Gba;
use std::path::PathBuf;

mod disasm;

const COLOR_ERROR: Color32 = Color32::LIGHT_RED;
const COLOR_DECODED_INSTR_ADDR: Color32 = Color32::LIGHT_GREEN;
const COLOR_NOT_DECODED_INSTR_ADDR: Color32 = COLOR_ERROR;
const COLOR_CONDITION_SUCCESS: Color32 = Color32::LIGHT_GRAY;
const COLOR_CONDITION_FAIL: Color32 = Color32::DARK_GRAY;
const COLOR_MNEMONIC: Color32 = Color32::WHITE;
const COLOR_REGISTER: Color32 = Color32::LIGHT_BLUE;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct GbaApp {
    trace_opcode_viewer: TraceOpcodeViewer,
    #[serde(skip)]
    gba: Option<Gba>,

    #[serde(skip)]
    rom_path: Option<PathBuf>,
    bios_path: Option<PathBuf>,
}

impl GbaApp {
    pub fn new(cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self {
            trace_opcode_viewer: TraceOpcodeViewer::new(),
            gba: None,
            rom_path: None,
            bios_path: None,
        }
    }

    pub fn render_ui(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("main-menu").show(ctx, |ui| {
            self.show_main_menu(ui, ctx, frame);
        });

        if let Some(gba) = self.gba.as_mut() {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.trace_opcode_viewer.render_ui(ui, ctx, gba);
            });
        } else {
        }
    }

    fn begin_rom_if_possible(&mut self) {
        if let Some(rom) = self.rom_path.as_ref()
            && let Some(bios) = self.bios_path.as_ref()
        {
            self.gba = Some(Gba::new(rom, bios).unwrap());
            self.gba.as_mut().unwrap().start()
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
                ui.separator();
                if ui.button("Select BIOS").clicked() {
                    if let Some(bios) = rfd::FileDialog::new().pick_file() {
                        self.bios_path = Some(bios.into());
                        self.begin_rom_if_possible();
                    }
                }
                if let Some(bios) = self.bios_path.as_ref() {
                    ui.label(format!("{}", bios.to_str().unwrap()));
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Debug", |ui| {
                if ui.button("Trace").clicked() {
                    self.trace_opcode_viewer.toggle_is_open();
                }
            })
        });
    }
}

impl eframe::App for GbaApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.render_ui(ctx, frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct TraceOpcodeViewer {
    is_open: bool,
}

impl TraceOpcodeViewer {
    pub fn new() -> Self {
        Self { is_open: true }
    }

    pub fn toggle_is_open(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn render_ui(&mut self, _ui: &mut Ui, ctx: &Context, gba: &mut Gba) {
        egui::Window::new("Opcode trace")
            .vscroll(true)
            .open(&mut self.is_open)
            .show(ctx, |ui| {
                if ui.button("Step").clicked() {
                    gba.step();
                }
                ui.separator();

                let opcodes = &gba.cpu.opcode_traces;

                egui::Grid::new("opcodes_trace")
                    .num_columns(3)
                    .spacing([10.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for opcode in opcodes {
                            match opcode {
                                OpcodeTraceLog::Decoded(opcode) => {
                                    Self::decoded_opcode_row(ui, &opcode);
                                }
                                OpcodeTraceLog::NotDecoded(execute_address, execute_opcode) => {
                                    Self::not_decoded_opcode_row(
                                        ui,
                                        *execute_address,
                                        *execute_opcode,
                                    );
                                }
                            }
                            ui.end_row();
                        }
                    })
                    .response
            });
    }

    fn not_decoded_opcode_row(ui: &mut Ui, execute_address: u32, execute_opcode: u32) {
        ui.add(|ui: &mut Ui| {
            ui.colored_label(
                COLOR_NOT_DECODED_INSTR_ADDR,
                format!("{:#08X}", execute_address),
            )
        });
        ui.add(|ui: &mut Ui| {
            ui.label("")
        });
        ui.add_sized(ui.available_size(), |ui: &mut Ui| {
            ui.label(format!("Failed to decode opcode {:#08X}", execute_opcode))
        });
    }

    fn decoded_opcode_row(ui: &mut Ui, opcode: &ExecutedOpcode) {
        ui.add(|ui: &mut Ui| {
            ui.colored_label(
                COLOR_DECODED_INSTR_ADDR,
                format!("{:#08X}", opcode.address),
            )
        });
        ui.add(|ui: &mut Ui| {
            ui.colored_label(if opcode.did_execute {
                COLOR_CONDITION_SUCCESS
            } else {
                COLOR_CONDITION_FAIL
            }, condition_text(opcode.condition))
        });
        ui.add_sized(ui.available_size(), |ui: &mut Ui| {
            opcode_disassembly(ui, &opcode.opcode)
        });
    }
}
