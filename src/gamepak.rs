use std::path::Path;

use thiserror::Error;

/// The GBA GamePak is extracted from 192 bytes region at the start of a ROM
/// file (Mapped to `0x80000000`-`0x800000BF` in the memory space
/// In addition to the included fields, the byte at offset `0xB2` must be
/// `0x96` and at `0xB3` must be `0x00` for current GBA models
#[derive(Debug, Clone)]
pub struct GamePakHeader {
    /// The `title` is a 12 byte uppercase ASCII string located at offset `0xA0`
    pub title: String,
    /// The `game_code` is a 4 byte uppercase ASCII code at offset `0xAC`
    /// It is built from 3 components (`UTTD`):
    /// 1. U  Unique Code (Usually `"A"` or `"B"` or some special meaning)
    /// 2. TT Short Title (or something random and unique if clash)
    /// 3. D  Destination (e.g. `"J" for Japan, `"E"` for USA/English, etc.)
    pub game_code: String,
    /// The `maker_code` is a 2 byte ASCII uppercase value representing the
    /// developer of the game. E.g. "01" = Nintendo at offset `0xB0`
    pub maker_code: String,
}

/// The `Gamepak` struct contains the header and ROM bytes to be mapped to
/// memory beginning from `0x80000000`
#[derive(Debug, Clone)]
pub struct Gamepak {
    pub header: GamePakHeader,
    pub rom: Vec<u8>,
}

impl Gamepak {
    /// Extract out the header and init a `Gamepak` from the given ROM bytes
    pub fn new(path: &Path) -> anyhow::Result<Gamepak, String> {
        let rom = std::fs::read(path).map_err(|e| e.to_string())?;
        Gamepak::build_rom(rom).map_err(|e| e.to_string())
    }

    fn build_rom(rom: Vec<u8>) -> anyhow::Result<Gamepak, GamePakError> {
        let header = Gamepak::parse_header(&rom[..0xC0])?;
        let mut rom_data = rom[0xC0..].to_vec();

        if !rom_data.len().is_power_of_two() {
            rom_data.resize(rom_data.len().next_power_of_two(), 0x00);
        }

        Ok(Gamepak {
            header,
            rom: rom_data,
        })
    }

    /// Extract out fields from the header and also check the expected bytes
    /// and checksum
    fn parse_header(header: &[u8]) -> anyhow::Result<GamePakHeader, GamePakError> {
        if header.len() != 0xC0 {
            return Err(GamePakError::Size {
                expected: 0xC0,
                got: header.len(),
            });
        }

        // Extract out fields
        let title = match std::str::from_utf8(&header[0xA0..0xAC]) {
            Ok(value) => value.to_string(),
            Err(e) => {
                return Err(GamePakError::Header {
                    expected: "Expected ASCII title at offset 0xA0-0xAB".to_string(),
                    got: e.to_string(),
                });
            }
        };

        let game_code = match std::str::from_utf8(&header[0xAC..0xB0]) {
            Ok(value) => value.to_string(),
            Err(e) => {
                return Err(GamePakError::Header {
                    expected: "Expected ASCII Game Code at offset 0xAC-0xAF".to_string(),
                    got: e.to_string(),
                });
            }
        };

        let maker_code = match std::str::from_utf8(&header[0xB0..0xB2]) {
            Ok(value) => value.to_string(),
            Err(e) => {
                return Err(GamePakError::Header {
                    expected: "Expected ASCII Maker Code at offset 0xB0-0xB1".to_string(),
                    got: e.to_string(),
                });
            }
        };

        if header[0xB2] != 0x96 {
            return Err(GamePakError::Header {
                expected: "0x96 at offset 0xB2".to_string(),
                got: format!("{:#04X}", header[0xB2]),
            });
        }

        if header[0xB3] != 0x00 {
            return Err(GamePakError::Header {
                expected: "0x00 at offset 0xB3".to_string(),
                got: format!("{:#04X}", header[0xB3]),
            });
        }

        Ok(GamePakHeader {
            title,
            game_code,
            maker_code,
        })
    }
}

#[derive(Error, Debug)]
pub enum GamePakError {
    #[error("Invalid header (expected '{expected}'; got {got})")]
    Header { expected: String, got: String },
    #[error("Invalid size (expected '{expected}'; got '{got}')")]
    Size { expected: usize, got: usize },
}

#[cfg(test)]
mod tests {
    use crate::gamepak::{Gamepak, GamePakError, GamePakHeader};

    fn gen_header() -> Vec<u8> {
        let mut header_bytes = vec![0x00; 0xC0];

        header_bytes[0xA0..0xAC].copy_from_slice("ZEROMISSIONE".as_bytes());
        header_bytes[0xAC..0xB0].copy_from_slice("BMXE".as_bytes());
        header_bytes[0xB0..0xB2].copy_from_slice("01".as_bytes());
        header_bytes[0xB2] = 0x96;

        header_bytes
    }

    #[test]
    fn test_valid_header() -> anyhow::Result<()> {
        let header_bytes = gen_header();
        let header = Gamepak::parse_header(&header_bytes)?;

        assert_eq!(header.title, "ZEROMISSIONE");
        assert_eq!(header.game_code, "BMXE");
        assert_eq!(header.maker_code, "01");

        Ok(())
    }

    #[test]
    fn test_invalid_header() {
        let mut header_bytes = gen_header();

        // Invalid UTF-8 in title
        let temp = header_bytes[0xA1];
        header_bytes[0xA1] = 0xFF;
        let header = Gamepak::parse_header(&header_bytes);
        assert!(matches!(
            header,
            Err(GamePakError::Header {
                expected: _,
                got: _
            })
        ));
        header_bytes[0xA1] = temp;

        // Unexpected value at offset `0xB2`
        header_bytes[0xB2] = 0xFF;
        let header = Gamepak::parse_header(&header_bytes);
        assert!(matches!(
            header,
            Err(GamePakError::Header {
                expected: _,
                got: _,
            })
        ));
        header_bytes[0xB2] = 0x96;

        // Unexpected value at offset `0xB3`
        header_bytes[0xB3] = 0x1;
        let header = Gamepak::parse_header(&header_bytes);
        assert!(matches!(
            header,
            Err(GamePakError::Header {
                expected: _,
                got: _,
            })
        ));
        header_bytes[0xB3] = 0x00;

        // Correct header
        let header = Gamepak::parse_header(&header_bytes);
        assert!(matches!(
            header,
            Ok(GamePakHeader {
                title: _,
                game_code: _,
                maker_code: _
            })
        ));
    }

    #[test]
    fn test_invalid_size() {
        let mut header_bytes = gen_header();
        header_bytes.resize(0xC1, 0x00);

        // Invalid size
        let header = Gamepak::parse_header(&header_bytes);
        assert!(matches!(
            header,
            Err(GamePakError::Size {
                expected: 0xC0,
                got: 0xC1
            })
        ));

        // Valid size
        let header = Gamepak::parse_header(&header_bytes[..0xC0]);
        assert!(matches!(
            header,
            Ok(GamePakHeader {
                title: _,
                game_code: _,
                maker_code: _
            })
        ));
    }

    #[test]
    fn test_rom_size() {
        let mut rom = gen_header(); // Len 0xC0
        rom.resize(0x3FFA, 0x00);
        let gamepak = Gamepak::build_rom(rom);
        assert!(gamepak.is_ok());
        let gamepak = gamepak.unwrap();
        let rom_len = gamepak.rom.len();
        assert_eq!(rom_len, 0x4000);
        assert_eq!(rom_len & (rom_len - 1), 0); // ROM size should be power of 2
    }
}
