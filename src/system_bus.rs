use crate::gamepak::Gamepak;

/// The `SystemBus` is responsible for routing all the read/write signals to the proper
/// mapped component for a particular address
#[allow(dead_code)]
pub struct SystemBus {
    gamepak: Gamepak,
    bios: Vec<u8>,
}

impl SystemBus {
    pub fn new(gamepak: Gamepak, bios: Vec<u8>) -> Self {
        Self { gamepak, bios }
    }
}
