use crate::cpu::opcodes::{
    check_condition, decode_arm_opcode, execute_arm_to_thumb_bx, execute_b, execute_bl,
    execute_data_processing, ArmOpcode, Opcode,
};
use crate::cpu::registers::{CondFlag, CpuMode, CpuState, PC_IDX};
use crate::system_bus::{SystemBus, ACCESS_CODE, ACCESS_SEQ};
use registers::RegisterFile;

pub mod opcodes;
pub mod registers;

#[derive(Debug, Copy, Clone, Default)]
pub struct Arm7Cpu {
    registers: RegisterFile,
    /// The ARM7TDMI has a 3 stage pipeline. The opcode to execute is taken from
    /// index `0` and then the fetched opcode is pushed at the back and decoded
    /// is shifted to next execute
    /// - Fetch - 1
    /// - Decode - 0
    /// - Execute - 0 pre-fetch
    pipeline: [u32; 2],
    next_access: u8,
}

impl Arm7Cpu {
    pub fn new() -> Self {
        Self {
            registers: RegisterFile::default(),
            pipeline: [0; 2],
            next_access: ACCESS_CODE,
        }
    }

    fn toggle_cpu_state(&mut self) {
        self.registers.cpsr ^= CondFlag::State as u32;
    }

    fn switch_cpu_mode(&mut self, cpu_mode: CpuMode) {
        self.registers.cpsr =
            (self.registers.cpsr & !(CondFlag::ModeMask as u32)) | (cpu_mode as u32);
    }

    fn fetch_word<BusType: SystemBus>(&mut self, bus: &mut BusType) -> u32 {
        bus.read_word(self.registers.get_and_incr_pc(4), ACCESS_CODE)
    }

    fn reload_pipeline<BusType: SystemBus>(&mut self, bus: &mut BusType) {
        match self.registers.state() {
            CpuState::Arm => self.reload_pipeline32(bus),
            CpuState::Thumb => self.reload_pipeline16(bus),
        }
    }

    fn reload_pipeline16<BusType: SystemBus>(&mut self, bus: &mut BusType) {
        self.pipeline[0] =
            bus.read_half_word(self.registers.get_and_incr_pc(2), self.next_access) as u32;
        self.pipeline[1] =
            bus.read_half_word(self.registers.get_and_incr_pc(2), ACCESS_CODE | ACCESS_SEQ) as u32;
        self.next_access = ACCESS_CODE | ACCESS_SEQ;
    }

    fn reload_pipeline32<BusType: SystemBus>(&mut self, bus: &mut BusType) {
        self.pipeline[0] = bus.read_word(self.registers.get_and_incr_pc(4), self.next_access);
        self.pipeline[1] =
            bus.read_word(self.registers.get_and_incr_pc(4), ACCESS_CODE | ACCESS_SEQ);
        self.next_access = ACCESS_CODE | ACCESS_SEQ;

        // TODO: IRQ disable
    }

    pub fn step<BusType: SystemBus>(&mut self, bus: &mut BusType) {
        match self.registers.state() {
            CpuState::Arm => self.execute_next_arm(bus),
            CpuState::Thumb => todo!(),
        }
    }

    fn execute_next_arm<BusType: SystemBus>(&mut self, bus: &mut BusType) {
        let execute_opcode = self.pipeline[0];

        self.registers[PC_IDX] &= !1;

        self.pipeline[0] = self.pipeline[1];
        self.pipeline[1] = bus.read_word(self.registers[PC_IDX], self.next_access);

        if let Some(Opcode::Arm(opcode)) = decode_arm_opcode(execute_opcode) {
            if check_condition(&self.registers, execute_opcode) {
                self.execute_arm_opcode(opcode, bus);
            } else {
                bus.read_word(self.registers.get_and_incr_pc(4), ACCESS_CODE);
                self.next_access = ACCESS_CODE | ACCESS_SEQ;
            }
        } else {
            eprintln!("Failed to decode opcode {execute_opcode:#08X}");
        }
    }

    fn execute_arm_opcode<BusType: SystemBus>(&mut self, opcode: ArmOpcode, bus: &mut BusType) {
        match opcode {
            ArmOpcode::B { offset } => execute_b(self, bus, offset),
            ArmOpcode::BL { offset } => execute_bl(self, bus, offset),
            ArmOpcode::BX { register_idx } => {
                execute_arm_to_thumb_bx(self, bus, register_idx as usize)
            }
            ArmOpcode::DataProcessing {
                sub_opcode,
                rd,
                rn,
                operand,
                shifter_carry,
                set_flags,
            } => execute_data_processing(
                self,
                bus,
                sub_opcode,
                rd,
                rn,
                operand,
                shifter_carry,
                set_flags,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::registers::{CpuMode, CpuState, RegisterFile, PC_IDX};
    use crate::cpu::Arm7Cpu;
    use crate::system_bus::{SystemBus, ACCESS_CODE};
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::fs::File;
    use test_case::test_case;

    #[test]
    fn test_cpu_startup() {
        let cpu = Arm7Cpu::new();

        assert_eq!(cpu.registers[PC_IDX], 0x00000000);
        assert_eq!(cpu.registers.mode(), CpuMode::System);
        assert_eq!(cpu.registers.state(), CpuState::Arm);
        assert_eq!(cpu.registers.cpsr, 0x000000DF); // IRQ and FIQ disabled
    }

    #[test]
    fn test_cpu_mode_change() {
        let mut cpu = Arm7Cpu::new();
        assert_eq!(cpu.registers.mode(), CpuMode::System);

        cpu.switch_cpu_mode(CpuMode::Irq);
        assert_eq!(cpu.registers.mode(), CpuMode::Irq);

        cpu.switch_cpu_mode(CpuMode::Abort);
        assert_eq!(cpu.registers.mode(), CpuMode::Abort);

        cpu.switch_cpu_mode(CpuMode::Supervisor);
        assert_eq!(cpu.registers.mode(), CpuMode::Supervisor);

        cpu.switch_cpu_mode(CpuMode::Fiq);
        assert_eq!(cpu.registers.mode(), CpuMode::Fiq);

        cpu.switch_cpu_mode(CpuMode::Undefined);
        assert_eq!(cpu.registers.mode(), CpuMode::Undefined);

        cpu.switch_cpu_mode(CpuMode::User);
        assert_eq!(cpu.registers.mode(), CpuMode::User);
    }

    // Opcode tests
    #[derive(Serialize, Deserialize)]
    struct TestCpuState {
        #[serde(alias = "R")]
        r: [u32; 16],
        #[serde(alias = "R_fiq")]
        r_fiq: [u32; 7],
        #[serde(alias = "R_svc")]
        r_svc: [u32; 2],
        #[serde(alias = "R_abt")]
        r_abt: [u32; 2],
        #[serde(alias = "R_irq")]
        r_irq: [u32; 2],
        #[serde(alias = "R_und")]
        r_und: [u32; 2],
        #[serde(alias = "CPSR")]
        cpsr: u32,
        #[serde(alias = "SPSR")]
        spsr: [u32; 5],
        pipeline: [u32; 2],
        access: u8,
    }

    #[derive(Serialize, Deserialize)]
    struct Transaction {
        kind: u32,
        size: usize,
        addr: u32,
        data: u32,
        cycle: usize,
        access: usize,
    }

    struct TransactionSystemBus<'a> {
        test_state: &'a TestState,
        opcode: u32,
        next_index: usize,
    }

    impl<'a> TransactionSystemBus<'a> {
        fn find_transaction_for_addr(&mut self, address: u32) -> Option<&Transaction> {
            if self.next_index >= self.test_state.transactions.len() {
                None
            } else {
                let transaction = &self.test_state.transactions[self.next_index];
                if transaction.addr == address {
                    self.next_index += 1;
                    Some(transaction)
                } else {
                    None
                }
            }
        }
    }

    impl<'a> SystemBus for TransactionSystemBus<'a> {
        fn read_word(&mut self, mut address: u32, access: u8) -> u32 {
            address &= !3;
            if access & ACCESS_CODE != ACCESS_CODE {
                return if let Some(transaction) = self.find_transaction_for_addr(address) {
                    transaction.data
                } else {
                    address
                };
            }
            if address == self.test_state.base_addr {
                self.test_state.opcode
            } else {
                address
            }
        }

        fn write_word(&mut self, address: u32, data: u32, _access: u8) {
            todo!()
        }

        fn read_half_word(&mut self, mut address: u32, access: u8) -> u16 {
            address &= !1;
            if access & ACCESS_CODE != ACCESS_CODE {
                return if let Some(transaction) = self.find_transaction_for_addr(address) {
                    transaction.data as u16
                } else {
                    address as u16
                };
            }
            if address == self.test_state.base_addr {
                self.test_state.opcode as u16
            } else {
                address as u16
            }
        }

        fn write_half_word(&mut self, address: u32, data: u16, access: u8) {
            todo!()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct TestState {
        initial: TestCpuState,
        r#final: TestCpuState,
        transactions: Vec<Transaction>,
        opcode: u32,
        base_addr: u32,
    }

    fn read_test_data(test_name: &'static str) -> Vec<TestState> {
        serde_json::from_reader(File::open(format!("./ARM7TDMI/v1/{test_name}.json")).unwrap())
            .unwrap()
    }

    fn cpu_with_state(state: &TestCpuState) -> Arm7Cpu {
        let registers = RegisterFile {
            user_bank: state.r,
            fiq_registers: state.r_fiq,
            spsr_fiq: state.spsr[0],
            r13_svc: state.r_svc[0],
            r14_svc: state.r_svc[1],
            spsr_svc: state.spsr[1],
            r13_abt: state.r_abt[0],
            r14_abt: state.r_abt[1],
            spsr_abt: state.spsr[2],
            r13_irq: state.r_irq[0],
            r14_irq: state.r_irq[1],
            spsr_irq: state.spsr[3],
            r13_und: state.r_und[0],
            r14_und: state.r_und[1],
            spsr_und: state.spsr[4],
            cpsr: state.cpsr,
        };

        Arm7Cpu {
            registers,
            pipeline: state.pipeline,
            next_access: state.access,
        }
    }

    #[derive(Debug)]
    enum OpcodeExecFailure {
        RegisterMismatch {
            expected: u32,
            actual: u32,
            register: String,
        },
        PipelineMismatch {
            index: usize,
            expected: u32,
            actual: u32,
        },
        FinalAccessMismatch {
            expected: u8,
            actual: u8,
        },
        // Unreliable for now
        IncorrectCycles {
            actual: usize,
        },
        // Unreliable for now
        MemoryTransaction {
            field: &'static str,
            expected: usize,
            actual: usize,
            index: usize,
        },
    }

    fn compare_cpu_with_state(
        opcode: u32,
        cpu: &Arm7Cpu,
        state: &TestCpuState,
        failures: &mut Vec<(u32, OpcodeExecFailure)>,
    ) {
        for (i, (expected, actual)) in state
            .r
            .iter()
            .zip(cpu.registers.user_bank.iter())
            .enumerate()
        {
            if expected != actual {
                failures.push((
                    opcode,
                    OpcodeExecFailure::RegisterMismatch {
                        expected: *expected,
                        actual: *actual,
                        register: if i == 15 {
                            "PC".to_string()
                        } else {
                            format!("R{i}")
                        },
                    },
                ));
            }
        }

        for (i, (expected, actual)) in state
            .r_fiq
            .iter()
            .zip(cpu.registers.fiq_registers.iter())
            .enumerate()
        {
            if expected != actual {
                failures.push((
                    opcode,
                    OpcodeExecFailure::RegisterMismatch {
                        expected: *expected,
                        actual: *actual,
                        register: format!("R_fiq {}", i + 8),
                    },
                ));
            }
        }

        if cpu.registers.spsr_fiq != state.spsr[0] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.spsr[0],
                    actual: cpu.registers.spsr_fiq,
                    register: "SPSR_fiq".to_string(),
                },
            ));
        }

        // SVC
        if cpu.registers.r13_svc != state.r_svc[0] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.r_svc[0],
                    actual: cpu.registers.r13_svc,
                    register: "R_svc 13".to_string(),
                },
            ));
        }

        if cpu.registers.r14_svc != state.r_svc[1] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.r_svc[1],
                    actual: cpu.registers.r14_svc,
                    register: "R_svc 14".to_string(),
                },
            ));
        }

        if cpu.registers.spsr_svc != state.spsr[1] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.spsr[1],
                    actual: cpu.registers.spsr_svc,
                    register: "SPSR_svc".to_string(),
                },
            ));
        }

        // ABT
        if cpu.registers.r13_abt != state.r_abt[0] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.r_abt[0],
                    actual: cpu.registers.r13_abt,
                    register: "R_abt 13".to_string(),
                },
            ));
        }

        if cpu.registers.r14_abt != state.r_abt[1] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.r_abt[1],
                    actual: cpu.registers.r14_abt,
                    register: "R_abt 14".to_string(),
                },
            ));
        }

        if cpu.registers.spsr_abt != state.spsr[2] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.spsr[2],
                    actual: cpu.registers.spsr_abt,
                    register: "SPSR_abt".to_string(),
                },
            ));
        }

        // IRQ
        if cpu.registers.r13_irq != state.r_irq[0] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.r_irq[0],
                    actual: cpu.registers.r13_irq,
                    register: "R_irq 13".to_string(),
                },
            ));
        }

        if cpu.registers.r14_irq != state.r_irq[1] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.r_irq[1],
                    actual: cpu.registers.r14_irq,
                    register: "R_irq 14".to_string(),
                },
            ));
        }

        if cpu.registers.spsr_irq != state.spsr[3] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.spsr[3],
                    actual: cpu.registers.spsr_irq,
                    register: "SPSR_irq".to_string(),
                },
            ));
        }

        // UND
        if cpu.registers.r13_und != state.r_und[0] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.r_und[0],
                    actual: cpu.registers.r13_und,
                    register: "R_und 13".to_string(),
                },
            ));
        }

        if cpu.registers.r14_und != state.r_und[1] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.r_und[1],
                    actual: cpu.registers.r14_und,
                    register: "R_und 14".to_string(),
                },
            ));
        }

        if cpu.registers.spsr_und != state.spsr[4] {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.spsr[4],
                    actual: cpu.registers.spsr_und,
                    register: "SPSR_und".to_string(),
                },
            ));
        }

        // CPSR
        if cpu.registers.cpsr != state.cpsr {
            failures.push((
                opcode,
                OpcodeExecFailure::RegisterMismatch {
                    expected: state.cpsr,
                    actual: cpu.registers.cpsr,
                    register: "CPSR".to_string(),
                },
            ));
        }

        // Pipeline
        if cpu.pipeline[0] != state.pipeline[0] {
            failures.push((
                opcode,
                OpcodeExecFailure::PipelineMismatch {
                    index: 0,
                    expected: state.pipeline[0],
                    actual: cpu.pipeline[0],
                },
            ));
        }

        if cpu.pipeline[1] != state.pipeline[1] {
            failures.push((
                opcode,
                OpcodeExecFailure::PipelineMismatch {
                    index: 1,
                    expected: state.pipeline[1],
                    actual: cpu.pipeline[1],
                },
            ));
        }

        if cpu.next_access != state.access {
            failures.push((
                opcode,
                OpcodeExecFailure::FinalAccessMismatch {
                    expected: state.access,
                    actual: cpu.next_access,
                },
            ));
        }
    }

    #[test_case("arm_b_bl")]
    #[test_case("arm_bx")]
    #[test_case("arm_data_proc_immediate")]
    fn test_arm_opcode(name: &'static str) {
        let test_state = read_test_data(name);

        let mut opcode_failures: Vec<(u32, OpcodeExecFailure)> = vec![];

        for test_case in test_state.iter() {
            let mut bus = TransactionSystemBus {
                test_state: test_case,
                opcode: test_case.opcode,
                next_index: 0,
            };
            let mut cpu = cpu_with_state(&test_case.initial);

            cpu.execute_next_arm(&mut bus);
            compare_cpu_with_state(
                test_case.opcode,
                &cpu,
                &test_case.r#final,
                &mut opcode_failures,
            );
        }

        if opcode_failures.len() > 1 {
            for (opcode, failure) in opcode_failures.iter() {
                eprintln!("Opcode {opcode} ({opcode:#010X}) failed with {failure:?}");
            }
        }

        assert_eq!(opcode_failures.len(), 0);
    }
}
