use registers::RegisterFile;
use crate::cpu::registers::{CpuMode, CpuState};
use crate::system_bus::SystemBus;

pub mod opcodes;
pub mod registers;

#[derive(Debug, Copy, Clone, Default)]
pub struct Arm7Cpu {
    registers: RegisterFile,
}

impl Arm7Cpu {
    pub fn new() -> Self {
        Self {
            registers: RegisterFile::default(),
        }
    }

    fn toggle_cpu_state(&mut self) {
        todo!()
    }

    fn switch_cpu_mode(&mut self, cpu_mode: CpuMode) {
        todo!()
    }

    fn fetch_word(&mut self, bus: &mut SystemBus) -> u32 {
        bus.read_word(self.registers.fetch_add_pc(4))
    }

    pub fn step(&mut self, bus: &mut SystemBus) {
        match self.registers.state() {
            CpuState::Arm => self.execute_next_arm(bus),
            CpuState::Thumb => todo!(),
        }
    }

    fn execute_next_arm(&mut self, bus: &mut SystemBus) {
        let opcode = self.fetch_word(bus);
        todo!("{:#010X} not implemented", opcode)
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::Arm7Cpu;
    use crate::cpu::registers::{CpuMode, CpuState};

    #[test]
    fn test_cpu_startup() {
        let cpu = Arm7Cpu::new();

        assert_eq!(cpu.registers.pc(), 0x00000000);
        assert_eq!(cpu.registers.mode(), CpuMode::Supervisor);
        assert_eq!(cpu.registers.state(), CpuState::Arm);
    }
}
