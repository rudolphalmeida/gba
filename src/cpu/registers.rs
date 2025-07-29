#[derive(Debug, Copy, Clone)]
pub struct RegisterFile {
    /// Registers shared between the system and user states
    /// R0-R7 are shared across all states
    pub user_bank: [u32; 16],

    /// FIQ state R8-R14
    pub fiq_registers: [u32; 7],
    pub spsr_fiq: u32,

    pub r13_svc: u32,
    pub r14_svc: u32,
    pub spsr_svc: u32,

    pub r13_abt: u32,
    pub r14_abt: u32,
    pub spsr_abt: u32,

    pub r13_irq: u32,
    pub r14_irq: u32,
    pub spsr_irq: u32,

    pub r13_und: u32,
    pub r14_und: u32,
    pub spsr_und: u32,

    pub cpsr: u32,
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum CpuState {
    #[default]
    Arm = 0,
    Thumb = 1 << 5,
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
    Fiq = 0b10001,
    Irq = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    #[default]
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

#[allow(clippy::enum_clike_unportable_variant)]
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
            user_bank: Default::default(),
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
            cpsr: (0b11 << 6) | (CpuMode::default() as u32) | (CpuState::default() as u32),
        }
    }
}

impl RegisterFile {
    pub fn pc(&self) -> u32 {
        self.user_bank[15]
    }

    pub fn set_pc(&mut self, pc: u32) { self.user_bank[15] = pc; }

    pub fn r13(&self) -> u32 {
        match self.mode() {
            CpuMode::User => self.user_bank[13],
            CpuMode::Fiq => self.fiq_registers[5],
            CpuMode::Irq => self.r13_irq,
            CpuMode::Supervisor => self.r13_svc,
            CpuMode::Abort => self.r13_abt,
            CpuMode::Undefined => self.r13_und,
            CpuMode::System => self.user_bank[13],
        }
    }

    pub fn set_r13(&mut self, value: u32) {
        match self.mode() {
            CpuMode::User => self.user_bank[13] = value,
            CpuMode::Fiq => self.fiq_registers[5] = value,
            CpuMode::Irq => self.r13_irq = value,
            CpuMode::Supervisor => self.r13_svc = value,
            CpuMode::Abort => self.r13_abt = value,
            CpuMode::Undefined => self.r13_und = value,
            CpuMode::System => self.user_bank[13] = value,
        }
    }

    pub fn r14(&self) -> u32 {
        match self.mode() {
            CpuMode::User => self.user_bank[14],
            CpuMode::Fiq => self.fiq_registers[6],
            CpuMode::Irq => self.r14_irq,
            CpuMode::Supervisor => self.r14_svc,
            CpuMode::Abort => self.r14_abt,
            CpuMode::Undefined => self.r14_und,
            CpuMode::System => self.user_bank[14],
        }
    }

    pub fn set_r14(&mut self, value: u32) {
        match self.mode() {
            CpuMode::User => self.user_bank[14] = value,
            CpuMode::Fiq => self.fiq_registers[6] = value,
            CpuMode::Irq => self.r14_irq = value,
            CpuMode::Supervisor => self.r14_svc = value,
            CpuMode::Abort => self.r14_abt = value,
            CpuMode::Undefined => self.r14_und = value,
            CpuMode::System => self.user_bank[14] = value,
        }
    }

    pub fn fetch_add_pc(&mut self, by: u32) -> u32 {
        let pc = &mut self.user_bank[15];
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

    pub fn sign(&self) -> bool {
        self.cpsr & (CondFlag::Sign as u32) != 0
    }

    pub fn zero(&self) -> bool {
        self.cpsr & (CondFlag::Zero as u32) != 0
    }

    pub fn carry(&self) -> bool {
        self.cpsr & (CondFlag::Carry as u32) != 0
    }

    pub fn overflow(&self) -> bool {
        self.cpsr & (CondFlag::Overflow as u32) != 0
    }
}
