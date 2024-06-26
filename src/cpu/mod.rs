use registers::RegisterFile;

use crate::system_bus::SystemBus;

pub mod registers;

/// The execution mode the CPU is in. The mode is changed by executing the `BX`
/// opcode.
///
/// ## Automatic State Switching
///
/// - Switch to ARM mode when an exception occurs
/// - (User)[SystemMode::User] mode switches back to previous state when leaving
///   an exception
#[derive(Debug, Copy, Clone, Default)]
enum ExecutionMode {
    /// Uses the full 32-bit instruction set i.e. each instruction is 32-bits
    /// long. Allows access to all (registers)[RegisterFile].
    #[default]
    Arm,
    /// Uses a 16-bit instruction size for the most commonly used instructions
    /// and registers. Most opcodes only allow access to (`R0`-`R7`)[RegisterFile]
    Thumb,
}

#[derive(Debug, Copy, Clone, Default)]
enum SystemMode {
    #[default]
    User,
    Fiq,
    Supervisor,
    Abort,
    Irq,
    Undefined,
}

/// # ARM7TDMI
/// The ARM7TDMI is a 32-bit RISC CPU
/// - `T`: Thumb instruction set. See [`ExecutionMode`]
/// - `D`: Debug extensions
/// - `M`: Fast multiplexer
/// - `I`: Enhanced ICE
///
/// ## Pipelining
///
/// The ARM7TDMI uses a 3 stage pipeline (*fetch*, *decode*, & *execute*).
/// When one instruction is executing, the next one is being decoded, and the
/// one after is being fetched.
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
            system_mode: SystemMode::User,
            execution_mode: ExecutionMode::Arm,
        }
    }

    fn toggle_execution_mode(&mut self) {
        todo!()
    }

    fn switch_system_mode(&mut self, system_mode: SystemMode) {
        todo!()
    }

    // TODO: Extract out a trait for SystemBus implementations and make this
    //       method generic over it
    pub fn tick(&mut self, bus: &mut SystemBus) {
        todo!()
    }
}
