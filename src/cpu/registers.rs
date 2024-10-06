use std::ops::{Index, IndexMut};

pub const PC: usize = 15;

#[derive(Default, Debug, Copy, Clone)]
pub struct RegisterFile {
    registers: [u32; 16],
    cpsr: u32,
    spsr: u32,
}

impl RegisterFile {
    pub fn get_and_increment_pc(&mut self, by: u32) -> u32 {
        let pc = self[PC];
        self[PC] = self[PC].wrapping_add(by);
        pc
    }
}

impl Index<usize> for RegisterFile {
    type Output = u32;
    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}

impl IndexMut<usize> for RegisterFile {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.registers[index]
    }
}

/// Saved registers when switching modes
#[derive(Debug, Copy, Clone)]
pub enum BankedRegisters {
    User { r13: u32, r14: u32, r15: u32 },
    Fiq { registers: [u32; 7], spsr: u32 },
    Supervisor { r13: u32, r14: u32, spsr: u32 },
    Abort { r13: u32, r14: u32, spsr: u32 },
    Irq { r13: u32, r14: u32, spsr: u32 },
    Undefined { r13: u32, r14: u32, spsr: u32 },
}
