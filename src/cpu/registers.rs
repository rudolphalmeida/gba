pub const PC_IDX: usize = 15;

#[derive(Debug, Copy, Clone)]
pub struct RegisterFile {
    /// Registers shared between the system and user states
    /// R0-R7 and R15 are shared across all states
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
        match value & 0x1F {
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
    pub fn get_and_incr_pc(&mut self, by: u32) -> u32 {
        let pc = &mut self.user_bank[PC_IDX];
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

    pub fn spsr_moded(&self) -> u32 {
        match self.mode() {
            CpuMode::User => self.cpsr, // TODO: What goes here?
            CpuMode::Fiq => self.spsr_fiq,
            CpuMode::Irq => self.spsr_irq,
            CpuMode::Supervisor => self.spsr_svc,
            CpuMode::Abort => self.spsr_abt,
            CpuMode::Undefined => self.spsr_und,
            CpuMode::System => self.cpsr, // TODO: Is this right?
        }
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

    pub fn update_flag(&mut self, flag: CondFlag, value: bool) {
        if value {
            self.cpsr = self.cpsr | (flag as u32);
        } else {
            self.cpsr = self.cpsr & !(flag as u32);
        }
    }
}

impl std::ops::Index<usize> for RegisterFile {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        match self.mode() {
            CpuMode::User => &self.user_bank[index],
            CpuMode::Fiq if (8..=14).contains(&index) => &self.fiq_registers[index - 8],
            CpuMode::Fiq => &self.user_bank[index],
            CpuMode::Irq if index == 13 => &self.r13_irq,
            CpuMode::Irq if index == 14 => &self.r14_irq,
            CpuMode::Irq => &self.user_bank[index],
            CpuMode::Supervisor if index == 13 => &self.r13_svc,
            CpuMode::Supervisor if index == 14 => &self.r14_svc,
            CpuMode::Supervisor => &self.user_bank[index],
            CpuMode::Abort if index == 13 => &self.r13_abt,
            CpuMode::Abort if index == 14 => &self.r14_abt,
            CpuMode::Abort => &self.user_bank[index],
            CpuMode::Undefined if index == 13 => &self.r13_und,
            CpuMode::Undefined if index == 14 => &self.r14_und,
            CpuMode::Undefined => &self.user_bank[index],
            CpuMode::System => &self.user_bank[index],
        }
    }
}

impl std::ops::IndexMut<usize> for RegisterFile {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.mode() {
            CpuMode::User => &mut self.user_bank[index],
            CpuMode::Fiq if (8..=14).contains(&index) => &mut self.fiq_registers[index - 8],
            CpuMode::Fiq => &mut self.user_bank[index],
            CpuMode::Irq if index == 13 => &mut self.r13_irq,
            CpuMode::Irq if index == 14 => &mut self.r14_irq,
            CpuMode::Irq => &mut self.user_bank[index],
            CpuMode::Supervisor if index == 13 => &mut self.r13_svc,
            CpuMode::Supervisor if index == 14 => &mut self.r14_svc,
            CpuMode::Supervisor => &mut self.user_bank[index],
            CpuMode::Abort if index == 13 => &mut self.r13_abt,
            CpuMode::Abort if index == 14 => &mut self.r14_abt,
            CpuMode::Abort => &mut self.user_bank[index],
            CpuMode::Undefined if index == 13 => &mut self.r13_und,
            CpuMode::Undefined if index == 14 => &mut self.r14_und,
            CpuMode::Undefined => &mut self.user_bank[index],
            CpuMode::System => &mut self.user_bank[index],
        }
    }
}
