use crate::system_bus::SystemBus;

pub struct Arm7Cpu {}

impl Arm7Cpu {
    pub fn new() -> Self {
        Self {}
    }

    // Extract out a trait for SystemBus impl's and make this method generic
    // over it
    pub fn tick(&mut self, bus: &mut SystemBus) {
        todo!()
    }
}
