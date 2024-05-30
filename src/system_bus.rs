use crate::gamepak::Gamepak;

/// The `SystemBus` is responsible for routing all the read/write signals to the proper
/// mapped component for a particular address
pub struct SystemBus {
    pub gamepak: Gamepak,
}

impl SystemBus {
    pub fn new(gamepak: Gamepak) -> Self {
        Self {
            gamepak
        }
    }
}
