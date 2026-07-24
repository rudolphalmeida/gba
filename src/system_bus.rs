#[allow(dead_code)]
use crate::gamepak::Gamepak;

pub const ACCESS_NONSEQ: u8 = 0;
pub const ACCESS_SEQ: u8 = 1;
pub const ACCESS_CODE: u8 = 2;
pub const ACCESS_DMA: u8 = 4;
pub const ACCESS_LOCK: u8 = 8;

pub trait SystemBus {
    fn idle(&mut self);

    fn read_word(&mut self, address: u32, access: u8) -> u32;
    fn write_word(&mut self, address: u32, data: u32, access: u8);

    fn read_half_word(&mut self, address: u32, access: u8) -> u16;
    fn write_half_word(&mut self, address: u32, data: u16, access: u8);

    fn read_byte(&mut self, address: u32, access: u8) -> u8;
    fn write_byte(&mut self, address: u32, data: u8, access: u8);
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

    fn read_at<const N: usize>(&mut self, address: u32, _access: u8) -> [u8; N] {
        let mut bytes = [0x00; N];

        match address {
            0x00000000..0x00004000 if self.bios_active => {
                let address = address as usize;
                // self.bios[address..address + 4].iter()
                bytes[..=3].copy_from_slice(&self.bios[address..address + N]);
            }
            _ => todo!(
                "Unimplemented memory map region for read_word: {:#010X}",
                address
            ),
        }

        bytes
    }
}

impl SystemBus for Bus {
    fn idle(&mut self) {}

    fn read_word(&mut self, address: u32, access: u8) -> u32 {
        u32::from_le_bytes(self.read_at::<4>(address & !3, access))
    }

    fn write_word(&mut self, address: u32, data: u32, _access: u8) {
        todo!()
    }

    fn read_half_word(&mut self, address: u32, access: u8) -> u16 {
        u16::from_le_bytes(self.read_at::<2>(address & !1, access))
    }

    fn write_half_word(&mut self, address: u32, data: u16, access: u8) {
        todo!()
    }

    fn read_byte(&mut self, address: u32, access: u8) -> u8 {
        self.read_at::<1>(address, access)[0]
    }

    fn write_byte(&mut self, address: u32, data: u8, access: u8) {
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
