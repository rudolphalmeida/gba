use std::io;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;

use gba::cpu::Arm7Cpu;
use gba::gamepak::Gamepak;
use gba::system_bus::SystemBus;

fn main() -> io::Result<()> {
    let options = Options::parse();
    let gamepak = match Gamepak::new(&options.rom) {
        Ok(gamepak) => gamepak,
        Err(e) => {
            eprintln!("Error when parsing ROM: {}", e);
            exit(-1);
        }
    };
    let bios = std::fs::read(&options.bios)?;

    let mut bus = SystemBus::new(gamepak, bios);
    let mut cpu = Arm7Cpu::new();

    loop {
        cpu.execute_next(&mut bus);
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Options {
    #[arg(short, long)]
    rom: PathBuf,
    #[arg(short, long)]
    bios: PathBuf,
}
