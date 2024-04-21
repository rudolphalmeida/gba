use std::io;
use std::path::PathBuf;

use clap::Parser;

use gba::gamepak::Gamepak;

fn main() -> io::Result<()> {
    let options = Options::parse();
    let rom = std::fs::read(options.rom)?;
    let game_pak = Gamepak::new(rom).unwrap();
    println!("{game_pak:?}");

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Options {
    #[arg(short, long)]
    rom: PathBuf,
}
