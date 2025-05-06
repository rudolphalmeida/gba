use gba::gba::Gba;
use std::path::PathBuf;

#[derive(Default)]
pub struct EmulationCtx {
    gba: Option<Gba>,
}

pub enum EmulationCommand {
    LoadRom { rom: PathBuf, bios: PathBuf },

    Exit,
}

pub enum EmulatorUpdate {
    LoadSuccess(String),
    LoadError(String),
}

pub fn emulation_thread(
    mut ctx: EmulationCtx,
    cmd_recv: std::sync::mpsc::Receiver<EmulationCommand>,
    mut emu_send: std::sync::mpsc::Sender<EmulatorUpdate>,
) {
    'cmd_loop: loop {
        match cmd_recv.recv().unwrap() {
            EmulationCommand::LoadRom { rom, bios } => {
                ctx.gba = match Gba::new(&rom, bios) {
                    Ok(gba) => {
                        send_emulator_update(
                            &mut emu_send,
                            EmulatorUpdate::LoadSuccess(format!(
                                "Successfully loaded ROM at {:?}",
                                rom.file_name().unwrap()
                            )),
                        );
                        Some(gba)
                    }
                    Err(e) => {
                        send_emulator_update(&mut emu_send, EmulatorUpdate::LoadError(e));
                        None
                    }
                };
            }
            EmulationCommand::Exit => {
                if let Some(_gba) = ctx.gba.take() {
                    // TODO: Stop and save ROM
                }
                break 'cmd_loop;
            }
        }
    }
}

fn send_emulator_update(
    emu_send: &mut std::sync::mpsc::Sender<EmulatorUpdate>,
    data: EmulatorUpdate,
) {
    emu_send.send(data).unwrap();
}
