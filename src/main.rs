use std::{path::PathBuf, process::exit};

use clap::Parser;
use gba::gba::Gba;

fn main() {
    env_logger::init();
    let args = Args::parse();
    let mut gba = match Gba::new(args.rom, args.bios) {
        Ok(gba) => gba,
        Err(e) => {
            eprintln!("{e}");
            exit(-1);
        }
    };

    loop {
        gba.step();
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    rom: PathBuf,

    #[arg(long)]
    bios: PathBuf,
}
