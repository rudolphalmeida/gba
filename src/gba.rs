use crate::cpu::Arm7Cpu;
use crate::gamepak::Gamepak;
use crate::system_bus::SystemBus;
use std::path::Path;

pub struct Gba {
    system_bus: SystemBus,
    cpu: Arm7Cpu,
}

impl Gba {
    pub fn new(rom_path: impl AsRef<Path>, bios_path: impl AsRef<Path>) -> anyhow::Result<Self, String> {
        let gamepak = Gamepak::new(rom_path.as_ref())?;
        let bios = std::fs::read(bios_path).map_err(|e| e.to_string())?;
        let system_bus = SystemBus::new(gamepak, bios);
        let cpu = Arm7Cpu::new();

        Ok(Self { system_bus, cpu })
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.system_bus);
    }
}
