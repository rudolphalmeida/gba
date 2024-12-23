#[derive(Debug, Copy, Clone)]
pub struct RegisterFile {
    /// Registers shared between the system and user states
    /// R0-R7 are shared across all states
    registers: [u32; 16],

    /// FIQ state R8-R14
    fiq_registers: [u32; 7],
    spsr_fiq: u32,

    r13_svc: u32,
    r14_svc: u32,
    spsr_svc: u32,

    r13_abt: u32,
    r14_abt: u32,
    spsr_abt: u32,

    r13_irq: u32,
    r14_irq: u32,
    spsr_irq: u32,

    r13_und: u32,
    r14_und: u32,
    spsr_und: u32,

    cpsr: u32,
    spsr: u32,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self {
            registers: Default::default(),
            fiq_registers: Default::default(),
            spsr_fiq: Default::default(),
            r13_svc: Default::default(),
            r14_svc: Default::default(),
            spsr_svc: Default::default(),
            r13_abt: Default::default(),
            r14_abt: Default::default(),
            spsr_abt: Default::default(),
            r13_irq: Default::default(),
            r14_irq: Default::default(),
            spsr_irq: Default::default(),
            r13_und: Default::default(),
            r14_und: Default::default(),
            spsr_und: Default::default(),
            cpsr: Default::default(),
            spsr: Default::default(),
        }
    }
}

impl RegisterFile {
    pub fn pc(&self) -> u32 {
        self.registers[15]
    }

    pub fn get_and_increment_pc(&mut self, by: u32) -> u32 {
        let pc = &mut self.registers[15];
        *pc = pc.wrapping_add(by);
        *pc
    }
}
