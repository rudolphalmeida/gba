use registers::RegisterFile;

use crate::system_bus::SystemBus;

pub mod opcodes;
pub mod registers;

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
enum ExecutionMode {
    #[default]
    Arm,
    Thumb,
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
enum SystemMode {
    User,
    Fiq,
    #[default]
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
            system_mode: SystemMode::Supervisor,
            execution_mode: ExecutionMode::Arm,
        }
    }

    fn toggle_execution_mode(&mut self) {
        todo!()
    }

    fn switch_system_mode(&mut self, system_mode: SystemMode) {
        todo!()
    }

    fn fetch_word(&mut self, bus: &mut SystemBus) -> u32 {
        bus.read_word(self.registers.get_and_increment_pc(4))
    }

    pub fn step(&mut self, bus: &mut SystemBus) {
        match self.execution_mode {
            ExecutionMode::Arm => self.execute_next_arm(bus),
            ExecutionMode::Thumb => todo!(),
        }
    }

    fn execute_next_arm(&mut self, bus: &mut SystemBus) {
        let opcode = self.fetch_word(bus);
        todo!("{:#010X} not implemented", opcode)
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::{Arm7Cpu, ExecutionMode, SystemMode};

    #[test]
    fn test_cpu_startup() {
        let cpu = Arm7Cpu::new();

        assert_eq!(cpu.registers.pc(), 0x00000000);
        assert_eq!(cpu.system_mode, SystemMode::Supervisor);
        assert_eq!(cpu.execution_mode, ExecutionMode::Arm);
    }
}
