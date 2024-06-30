use registers::RegisterFile;

use crate::system_bus::SystemBus;

pub mod registers;

#[derive(Debug, Copy, Clone, Default)]
enum ExecutionMode {
    #[default]
    Arm,
    Thumb,
}

#[derive(Debug, Copy, Clone, Default)]
enum SystemMode {
    #[default]
    System,
    Fiq,
    Supervisor,
    Abort,
    Irq,
    Undefined,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Arm7Cpu {
    registers: RegisterFile,
    system_mode: SystemMode,
    execution_mode: ExecutionMode,
}

impl Arm7Cpu {
    pub fn new() -> Self {
        Self {
            registers: RegisterFile::default(),
            system_mode: SystemMode::System,
            execution_mode: ExecutionMode::Arm,
        }
    }

    fn toggle_execution_mode(&mut self) {
        todo!()
    }

    fn switch_system_mode(&mut self, system_mode: SystemMode) {
        todo!()
    }

    // Extract out a trait for SystemBus implementations and make this method generic
    // over it
    pub fn tick(&mut self, bus: &mut SystemBus) {
        todo!()
    }
}
