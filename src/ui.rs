use circular_buffer::CircularBuffer;
use gba::cpu::disasm::disassemble_opcode;
use gba::cpu::opcodes::Opcode;
use gba::cpu::EXECUTED_OPCODE_EVENT_ID;
use gba::events::Event;
use gba::gba::Gba;
use iced::widget::{button, Column};
use iced::Alignment::Center;
use iced::Length::Fill;
use iced::{
    widget::{column, text},
    Element, Theme,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub trait Page {
    fn update(&mut self, message: AppMessage) -> Option<Box<dyn Page>>;
    fn view(&self) -> iced::Element<'_, AppMessage>;
}

pub fn boot_page() -> Box<dyn Page> {
    Box::new(SelectFilePage::default())
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum AppMessage {
    SelectFileMessage(SelectFileMessage),
    PlayRomMessage(PlayRomMessage),
}

pub fn title(_: &Box<dyn Page>) -> String {
    "GBA emulator".to_string()
}

pub fn theme(_: &Box<dyn Page>) -> Theme {
    Theme::Dark
}

pub fn update(page: &mut Box<dyn Page>, message: AppMessage) {
    match message {
        message => {
            if let Some(next_page) = page.update(message) {
                *page = next_page;
            }
        }
    }
}

pub fn view<'a>(page: &'a Box<dyn Page>) -> Element<'a, AppMessage> {
    page.view()
}

#[derive(Debug, Clone, Default)]
struct SelectFilePage {
    bios_path: Option<PathBuf>,
    rom_path: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum SelectFileMessage {
    ShowBiosPicker,
    ShowRomPicker,
    PlaySelectedRom,
}

impl SelectFilePage {
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

impl Page for SelectFilePage {
    fn update(&mut self, message: AppMessage) -> Option<Box<dyn Page>> {
        match message {
            AppMessage::SelectFileMessage(message) => {
                match message {
                    SelectFileMessage::ShowBiosPicker => self.choose_bios_file(),
                    SelectFileMessage::ShowRomPicker => self.choose_rom_file(),
                    SelectFileMessage::PlaySelectedRom => {
                        match PlayRomPage::new(
                            &self.bios_path.as_ref().unwrap(),
                            &self.rom_path.as_ref().unwrap(),
                        ) {
                            Ok(page) => return Some(Box::new(page)),
                            Err(err) => eprintln!("Failed to load ROM: {err}"),
                        }
                    }
                };
            }
            _ => (),
        }

        None
    }

    fn view(&self) -> iced::Element<'_, AppMessage> {
        let load_bios_button = button("Load BIOS").on_press(AppMessage::SelectFileMessage(
            SelectFileMessage::ShowBiosPicker,
        ));
        let load_rom_button = button("Load ROM").on_press(AppMessage::SelectFileMessage(
            SelectFileMessage::ShowRomPicker,
        ));

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

        let mut play_rom_button = button("Play");
        if self.rom_path.is_some() && self.bios_path.is_some() {
            play_rom_button = play_rom_button.on_press(AppMessage::SelectFileMessage(
                SelectFileMessage::PlaySelectedRom,
            ));
        }

        column![
            text("GBA emulator").size(36),
            bios_path_display,
            load_bios_button,
            rom_path_display,
            load_rom_button,
            play_rom_button
        ]
        .spacing(20)
        .width(Fill)
        .align_x(Center)
        .into()
    }
}

struct PlayRomPage {
    gba: Gba,

    // Information for the debug UIs
    executed_opcodes: Arc<Mutex<CircularBuffer<10, Opcode>>>,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum PlayRomMessage {
    StepSingleInstruction,
}

impl PlayRomPage {
    pub fn new(bios_path: &dyn AsRef<Path>, rom_path: &dyn AsRef<Path>) -> Result<Self, String> {
        let mut gba = Gba::new(rom_path, bios_path)?;

        let executed_opcodes = Arc::new(Mutex::new(CircularBuffer::new()));

        let executed_opcodes_buffer = executed_opcodes.clone();
        gba.event_bus.register_handler(
            EXECUTED_OPCODE_EVENT_ID,
            Arc::new(move |event: &dyn Event| {
                let opcode = *event.payload().unwrap().get_ref::<Opcode>().unwrap();
                executed_opcodes_buffer.lock().unwrap().push_back(opcode);
            }),
        );

        Ok(Self {
            gba,
            executed_opcodes,
        })
    }

    fn executed_opcodes_view(&self) -> iced::Element<'_, AppMessage> {
        let ops = self
            .executed_opcodes
            .lock()
            .unwrap()
            .iter()
            .map(|opcode| text(disassemble_opcode(opcode)).into())
            .collect();

        Column::from_vec(ops).into()
    }
}

impl Page for PlayRomPage {
    fn update(&mut self, message: AppMessage) -> Option<Box<dyn Page>> {
        if let AppMessage::PlayRomMessage(message) = message {
            match message {
                PlayRomMessage::StepSingleInstruction => self.gba.step(),
            }
        }

        None
    }

    fn view(&self) -> iced::Element<'_, AppMessage> {
        column![
            text(format!("{:?}", self.gba.header)),
            button("Step").on_press(AppMessage::PlayRomMessage(
                PlayRomMessage::StepSingleInstruction
            )),
            self.executed_opcodes_view(),
        ]
        .into()
    }
}
