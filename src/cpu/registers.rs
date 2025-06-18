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

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum CpuState {
    #[default]
    Arm = 1 << 5,
    Thumb = 0,
}

impl TryFrom<u32> for CpuState {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == CpuState::Arm as u32 => Ok(CpuState::Arm),
            x if x == CpuState::Thumb as u32 => Ok(CpuState::Thumb),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum CpuMode {
    User = 0b10000,
    Fiq =  0b10001,
    Irq = 0b10010,
    #[default]
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

impl TryFrom<u32> for CpuMode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == CpuMode::User as u32 => Ok(CpuMode::User),
            x if x == CpuMode::Fiq as u32 => Ok(CpuMode::Fiq),
            x if x == CpuMode::Irq as u32 => Ok(CpuMode::Irq),
            x if x == CpuMode::Supervisor as u32 => Ok(CpuMode::Supervisor),
            x if x == CpuMode::Abort as u32 => Ok(CpuMode::Abort),
            x if x == CpuMode::Undefined as u32 => Ok(CpuMode::Undefined),
            x if x == CpuMode::System as u32 => Ok(CpuMode::System),
            _ => Err(()),
        }
    }
}

pub enum CondFlag {
    Sign = 1 << 31,
    Zero = 1 << 30,
    Carry = 1 << 29,
    Overflow = 1 << 28,

    IrqDisable = 1 << 7,
    FiqDisable = 1 << 6,
    State = 1 << 5,
    ModeMask = 0b11111,
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
            cpsr: (CpuMode::default() as u32) | (CpuState::default() as u32),
            spsr: Default::default(),
        }
    }
}

impl RegisterFile {
    pub fn pc(&self) -> u32 {
        self.registers[15]
    }

    pub fn fetch_add_pc(&mut self, by: u32) -> u32 {
        let pc = &mut self.registers[15];
        let res = *pc;
        *pc = pc.wrapping_add(by);
        res
    }

    pub fn state(&self) -> CpuState {
        CpuState::try_from(self.cpsr & (CondFlag::State as u32)).unwrap()
    }

    pub fn mode(&self) -> CpuMode {
        CpuMode::try_from(self.cpsr & (CondFlag::ModeMask as u32)).unwrap()
    }
}
