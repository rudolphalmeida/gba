use crate::cpu::registers::{CpuMode, CpuState};
use crate::system_bus::SystemBus;
use registers::RegisterFile;

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

    fn fetch_word<BusType: SystemBus>(&mut self, bus: &mut BusType) -> u32 {
        bus.read_word(self.registers.fetch_add_pc(4))
    }

    pub fn step<BusType: SystemBus>(&mut self, bus: &mut BusType) {
        match self.registers.state() {
            CpuState::Arm => self.execute_next_arm(bus),
            CpuState::Thumb => todo!(),
        }
    }

    fn execute_next_arm<BusType: SystemBus>(&mut self, bus: &mut BusType) {
        let opcode = self.fetch_word(bus);
        todo!("{:#010X} not implemented", opcode)
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::registers::{CpuMode, CpuState, RegisterFile};
    use crate::cpu::Arm7Cpu;
    use crate::system_bus::SystemBus;
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::fs::File;

    #[test]
    fn test_cpu_startup() {
        let cpu = Arm7Cpu::new();

        assert_eq!(cpu.registers.pc(), 0x00000000);
        assert_eq!(cpu.registers.mode(), CpuMode::Supervisor);
        assert_eq!(cpu.registers.state(), CpuState::Arm);
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
        access: usize,
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
        transactions: &'a Vec<Transaction>,
    }

    impl<'a> SystemBus for TransactionSystemBus<'a> {
        fn read_word(&mut self, address: u32) -> u32 {
            todo!()
        }

        fn write_word(&mut self, address: u32, data: u32) {
            todo!()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct TestState {
        initial: TestCpuState,
        r#final: TestCpuState,
        transactions: Vec<Transaction>,
        opcode: u32,
        base_addr: Vec<u32>,
    }

    fn read_test_data(test_name: &'static str) -> Vec<TestState> {
        serde_json::from_reader(File::open(format!("./ARM7TDMI/v1/{test_name}.json")).unwrap())
            .unwrap()
    }

    fn cpu_with_state(state: &TestCpuState) -> Arm7Cpu {
        let registers = RegisterFile {
            registers: state.r,
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

        Arm7Cpu { registers }
    }

    #[test]
    fn test_arm_b_bl() {
        let test_state = read_test_data("arm_b_bl");

        for test_case in test_state.iter() {
            let bus = TransactionSystemBus {
                transactions: &test_case.transactions,
            };
            let cpu = cpu_with_state(&test_case.initial);
        }
    }
}
