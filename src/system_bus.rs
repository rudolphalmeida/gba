#[allow(dead_code)]
use crate::gamepak::Gamepak;

pub trait SystemBus {
    fn read_word(&mut self, address: u32) -> u32;
    fn write_word(&mut self, address: u32, data: u32);
}

pub struct Bus {
    gamepak: Gamepak,
    bios: Vec<u8>,
    bios_active: bool,
}

impl Bus {
    pub fn new(gamepak: Gamepak, bios: Vec<u8>) -> Self {
        Self {
            gamepak,
            bios,
            bios_active: true,
        }
    }

    pub fn toggle_bios(&mut self) {
        self.bios_active = !self.bios_active;
        if self.bios_active {
            log::info!("Enabled BIOS");
        } else {
            log::info!("Disabled BIOS");
        }
    }
}

impl SystemBus for Bus {
    fn read_word(&mut self, address: u32) -> u32 {
        let address = address as usize;
        match address {
            0x00000000..0x00004000 if self.bios_active => {
                u32::from_le_bytes(self.bios[address..address + 4].try_into().unwrap())
            }
            _ => todo!(
                "Unimplemented memory map region for read_word: {:#010X}",
                address
            ),
        }
    }

    fn write_word(&mut self, address: u32, data: u32) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::gamepak::{GamePakHeader, Gamepak};
    use crate::system_bus::Bus;

    fn test_gamepak() -> Gamepak {
        let header = GamePakHeader {
            title: "TEST ROM".to_string(),
            game_code: "TEST".to_string(),
            maker_code: "RA".to_string(),
        };
        let rom = vec![0x00; 0x4000];
        Gamepak { header, rom }
    }

    const BIOS: &[u8] = include_bytes!("../roms/gba_bios.bin");

    #[test]
    fn test_bus_startup() {
        let bus = Bus::new(test_gamepak(), BIOS.to_vec());

        assert!(bus.bios_active);
    }
}
