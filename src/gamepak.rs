use thiserror::Error;

/// The GBA GamePak is extracted from 192 bytes region at the start of a ROM
/// file (Mapped to `0x80000000`-`0x800000BF` in the memory space
/// In addition to the included fields, the byte at offset `0xB2` must be
/// `0x96` and at `0xB3` must be `0x00` for current GBA models
#[derive(Debug, Clone)]
pub struct GamePakHeader {
    /// The `title` is a 12 byte uppercase ASCII string located at offset `0xA0`
    title: String,
    /// The `game_code` is a 4 byte uppercase ASCII code at offset `0xAC`
    /// It is built from 3 components (`UTTD`):
    /// 1. U  Unique Code (Usually `"A"` or `"B"` or some special meaning)
    /// 2. TT Short Title (or something random and unique if clash)
    /// 3. D  Destination (e.g. `"J" for Japan, `"E"` for USA/English, etc.)
    game_code: String,
    /// The `maker_code` is a 2 byte ASCII uppercase value representing the
    /// developer of the game. E.g. "01" = Nintendo at offset `0xB0`
    maker_code: String,
}

#[derive(Debug, Clone)]
pub struct Gamepak {
    header: GamePakHeader,
    rom: Vec<u8>,
}

impl Gamepak {
    pub fn new(rom: Vec<u8>) -> anyhow::Result<Gamepak, GamePakError> {
        let header = Gamepak::parse_header(&rom[..0xC0])?;
        Ok(Gamepak {
            header,
            rom: vec![],
        })
    }

    fn parse_header(header: &[u8]) -> anyhow::Result<GamePakHeader, GamePakError> {
        // Extract out fields
        let title = match std::str::from_utf8(&header[0xA0..0xAC]) {
            Ok(value) => value.to_string(),
            Err(e) => {
                return Err(GamePakError::Header {
                    expected: "Expected ASCII title at offset 0xA0-0xAB".to_string(),
                    got: e.to_string(),
                })
            }
        };

        let game_code = match std::str::from_utf8(&header[0xAC..0xB0]) {
            Ok(value) => value.to_string(),
            Err(e) => {
                return Err(GamePakError::Header {
                    expected: "Expected ASCII Game Code at offset 0xAC-0xAF".to_string(),
                    got: e.to_string(),
                })
            }
        };

        let maker_code = match std::str::from_utf8(&header[0xB0..0xB2]) {
            Ok(value) => value.to_string(),
            Err(e) => {
                return Err(GamePakError::Header {
                    expected: "Expected ASCII Maker Code at offset 0xB0-0xB1".to_string(),
                    got: e.to_string(),
                })
            }
        };

        // TODO: Perform expected byte checks

        Ok(GamePakHeader {
            title,
            game_code,
            maker_code,
        })
    }
}

#[derive(Error, Debug)]
pub enum GamePakError {
    #[error("Invalid header (expected '{expected:?}'; got {got:?})")]
    Header { expected: String, got: String },
    #[error("Invalid size (expected '{expected}'; got '{got}')")]
    Size { expected: usize, got: usize },
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_reading_header() {}

    #[test]
    fn test_invalid_header() {}

    #[test]
    fn test_invalid_size() {}
}
