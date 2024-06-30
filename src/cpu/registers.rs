/// Registers for the current executing mode
#[derive(Default, Debug, Copy, Clone)]
pub struct RegisterFile {
    /// System/User more registers
    /// `R0`-`R7`, `R15` are shared across all modes. `R15` is the *program counter*
    /// `R8`-`R12` are shared for all except for FIQ which uses `RX_fiq`
    /// Each mode gets its own `R13` and `R14`
    /// `R13` is also known as the *stack pointer*
    /// `R14` is also known as the **
    /// `R15` is shared for all modes except System/User which gets its own
    registers: [u32; 16],
    /// Current Program Status Register
    cpsr: u32,
    /// Saved Program Status Register
    /// Not used in System/User mode
    spsr: u32,
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
