use crate::cpu::Arm7Cpu;
use crate::gamepak::Gamepak;
use crate::system_bus::Bus;
use std::path::Path;

pub struct Gba {
    system_bus: Bus,
    cpu: Arm7Cpu,
}

impl Gba {
    pub fn new(
        rom_path: impl AsRef<Path>,
        bios_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self, String> {
        let gamepak = Gamepak::new(rom_path.as_ref())?;

        log::info!(
            "Loaded GamePak from {}",
            rom_path.as_ref().to_str().unwrap()
        );
        log::info!("Title: {}", gamepak.header.title);
        log::info!("Game Code: {}", gamepak.header.game_code);
        log::info!("Maker Code: {}", gamepak.header.maker_code);
        log::info!("ROM size: {} bytes", gamepak.rom.len());

        let bios = std::fs::read(bios_path).map_err(|e| e.to_string())?;
        let system_bus = Bus::new(gamepak, bios);
        let cpu = Arm7Cpu::new();
        log::debug!("Initialized CPU");

        Ok(Self { system_bus, cpu })
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.system_bus);
    }
}
