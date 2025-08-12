use crate::cpu::registers::{CpuState, RegisterFile, PC_IDX};
use crate::cpu::Arm7Cpu;
use crate::system_bus::{SystemBus, ACCESS_CODE, ACCESS_NONSEQ, ACCESS_SEQ};

use super::registers::{CondFlag, CpuMode};

pub fn decode_arm_opcode(opcode: u32) -> Option<Opcode> {
    if let Some(decoded_opcode) = try_decode_b_bl(opcode) {
        return Some(Opcode::Arm(decoded_opcode));
    }

    if let Some(decoded_opcode) = try_decode_bx(opcode) {
        return Some(Opcode::Arm(decoded_opcode));
    }

    if let Some(decoded_opcode) = try_decode_data_processing(opcode) {
        return Some(Opcode::Arm(decoded_opcode));
    }

    None
}

#[repr(u8)]
pub enum Condition {
    Equal = 0x0,
    NotEqual = 0x1,
    CarrySet = 0x2,
    CarryCleared = 0x3,
    Minus = 0x4,
    Positive = 0x5,
    Overflow = 0x6,
    NoOverflow = 0x7,
    UnsignedHigher = 0x8,
    UnsignedLowerOrSame = 0x9,
    GreaterOrEqual = 0xA,
    LessThan = 0xB,
    GreaterThan = 0xC,
    LessOrEqual = 0xD,
    Always = 0xE,
    Never = 0xF,
}

pub fn check_condition(registers: &RegisterFile, opcode: u32) -> bool {
    let condition = unsafe { std::mem::transmute::<u8, Condition>((opcode >> 28) as u8) };

    match condition {
        Condition::Equal => registers.zero(),
        Condition::NotEqual => !registers.zero(),
        Condition::CarrySet => registers.carry(),
        Condition::CarryCleared => !registers.carry(),
        Condition::Minus => registers.sign(),
        Condition::Positive => !registers.sign(),
        Condition::Overflow => registers.overflow(),
        Condition::NoOverflow => !registers.overflow(),
        Condition::UnsignedHigher => registers.carry() && !registers.zero(),
        Condition::UnsignedLowerOrSame => !registers.carry() || registers.zero(),
        Condition::GreaterOrEqual => registers.sign() == registers.overflow(),
        Condition::LessThan => registers.sign() != registers.overflow(),
        Condition::GreaterThan => !registers.zero() && (registers.sign() == registers.overflow()),
        Condition::LessOrEqual => registers.zero() || (registers.sign() != registers.overflow()),
        Condition::Always => true,
        Condition::Never => false,
    }
}

#[repr(u8)]
pub enum DataProcessingOpcode {
    AND = 0x0,
    EOR = 0x1,
    SUB = 0x2,
    RSB = 0x3,
    ADD = 0x4,
    ADC = 0x5,
    SBC = 0x6,
    RSC = 0x7,
    TST = 0x8,
    TEQ = 0x9,
    CMP = 0xA,
    CMN = 0xB,
    ORR = 0xC,
    MOV = 0xD,
    BIC = 0xE,
    MVN = 0xF,
}

pub enum ArmOpcode {
    B {
        offset: u32,
    }, // Offset is a signed 24-bit quantity
    BL {
        offset: u32,
    }, // Offset is a signed 24-bit quantity
    BX {
        register_idx: u8,
    },

    // Data processing group
    DataProcessing {
        rd: usize,
        rn: usize,
        operand: u32,
        sub_opcode: DataProcessingOpcode,
        set_flags: bool,
    },
}

pub enum Opcode {
    Arm(ArmOpcode),
    Thumb,
}

fn try_decode_b_bl(opcode: u32) -> Option<ArmOpcode> {
    if opcode & 0xE000000 != 0xA000000 {
        return None;
    }

    let mask = 1 << 24;
    match opcode & mask {
        0 => Some(ArmOpcode::B {
            offset: opcode & 0xFFFFFF,
        }),
        mask => Some(ArmOpcode::BL {
            offset: opcode & 0xFFFFFF,
        }),
    }
}

pub fn execute_b<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType, mut offset: u32) {
    if offset & 0x800000 != 0x00 {
        // Offset is a 24-bit signed value
        offset |= 0xFF000000; // Sign extend to 32-bits
    }
    let destination = cpu.registers[PC_IDX].wrapping_add(offset.wrapping_mul(4));
    cpu.registers[PC_IDX] = destination;
    cpu.next_access = ACCESS_CODE | ACCESS_SEQ;

    cpu.reload_pipeline(bus);
}

pub fn execute_bl<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType, offset: u32) {
    let link = cpu.registers[PC_IDX].wrapping_sub(4);
    execute_b(cpu, bus, offset);
    cpu.registers[14] = link;
}

// BX
fn try_decode_bx(opcode: u32) -> Option<ArmOpcode> {
    if opcode & 0x0FFFFF10 != 0x012FFF10 {
        // Ignoring BLX
        return None;
    }

    Some(ArmOpcode::BX {
        register_idx: opcode as u8 & 0xF,
    })
}

pub fn execute_arm_to_thumb_bx<BusType: SystemBus>(
    cpu: &mut Arm7Cpu,
    bus: &mut BusType,
    register_idx: usize,
) {
    assert_eq!(cpu.registers.state(), CpuState::Arm);
    let mut destination = cpu.registers[register_idx];
    if destination & 0b1 == 0b1 {
        destination &= !1;
        cpu.toggle_cpu_state();
    }
    cpu.registers[PC_IDX] = destination;
    cpu.next_access = ACCESS_CODE | ACCESS_NONSEQ;

    cpu.reload_pipeline(bus);
}

// Data processing
fn try_decode_data_processing(opcode: u32) -> Option<ArmOpcode> {
    if opcode & 0xC000000 != 0 {
        return None;
    }

    let sub_opcode = ((opcode & 0x1E00000) >> 21) as u8;
    // Set condition code flag must be *true* for test and compare opcodes
    if (0x8..=0xF).contains(&sub_opcode) && (opcode & (1 << 20) != (1 << 20)) {
        return None;
    }
    let set_flags = opcode & (1 << 20) == (1 << 20);

    // First operand register must be *0000* for MOV and MVN
    if (sub_opcode == 0xD || sub_opcode == 0xF) && (opcode & (0b1111 << 16) != 0) {
        return None;
    }
    let rn = ((opcode & (0b1111 << 16)) >> 16) as usize;

    let mut rd = 0b0000;
    if (0x8..=0xA).contains(&sub_opcode) {
        let dest_reg_mask = 0b1111 << 12;
        let dest_reg = (opcode & dest_reg_mask) >> 12;
        // Destination register must be *0000* or *1111* for TST/TEQ/CMP/CMN
        if dest_reg != 0b0000 || dest_reg != 0b1111 {
            return None;
        }
        rd = dest_reg as usize;
    };

    let sub_opcode = unsafe { std::mem::transmute::<u8, DataProcessingOpcode>(sub_opcode) };

    let is_immediate = opcode & (1 << 25) != 0;
    if is_immediate {
        let nn = opcode & 0xFF;
        // Shifted in jumps of 2 so 7 instead of 8 to keep LSB 0
        let shift = (opcode & 0xF00) >> 7;
        let operand = ror(nn, shift);

        return Some(ArmOpcode::DataProcessing {
            sub_opcode,
            operand,
            rd,
            rn,
            set_flags,
        });
    } else {
        // Register
        todo!()
    }

    None
}

fn ror(value: u32, amount: u32) -> u32 {
    //! Rotate Right

    // FIXME
    value.rotate_right(amount)
}

fn rrx(value: u32, amount: u32, extension: bool) -> u32 {
    //! Rotate right extended. The extension is used as the 33rd bit when shifting
    //! in from the right. Some of the possible values for `extension` are:
    //! - `RegisterFile::carry()`

    // FIXME
    value
}

pub fn execute_data_processing<BusType: SystemBus>(
    cpu: &mut Arm7Cpu,
    bus: &mut BusType,
    sub_opcode: DataProcessingOpcode,
    rd: usize,
    rn: usize,
    operand: u32,
    set_flags: bool,
) {
    let operation = match sub_opcode {
        DataProcessingOpcode::AND => execute_and,
        DataProcessingOpcode::EOR => execute_eor,
        DataProcessingOpcode::SUB => execute_sub,
        DataProcessingOpcode::RSB => execute_rsb,
        DataProcessingOpcode::ADD => execute_add,
        DataProcessingOpcode::ADC => execute_adc,
        DataProcessingOpcode::SBC => execute_sbc,
        DataProcessingOpcode::RSC => execute_rsc,
        DataProcessingOpcode::TST => execute_tst,
        DataProcessingOpcode::TEQ => execute_teq,
        DataProcessingOpcode::CMP => execute_cmp,
        DataProcessingOpcode::CMN => execute_cmn,
        DataProcessingOpcode::ORR => execute_orr,
        DataProcessingOpcode::MOV => execute_mov,
        DataProcessingOpcode::BIC => execute_bic,
        DataProcessingOpcode::MVN => execute_mvn,
    };
    let (result, carry) = operation(cpu, rd, rn, operand);

    if rd == PC_IDX {
        if set_flags {
            // TODO: Should not be used in user mode. (What if it is?)
            cpu.registers.cpsr = cpu.registers.spsr_moded();
            cpu.switch_cpu_mode(CpuMode::User);
            cpu.registers[PC_IDX] = result;
        }
        cpu.reload_pipeline(bus);
        return;
    }

    if !set_flags {
        return;
    }

    match sub_opcode {
        DataProcessingOpcode::AND
        | DataProcessingOpcode::EOR
        | DataProcessingOpcode::TST
        | DataProcessingOpcode::TEQ
        | DataProcessingOpcode::ORR
        | DataProcessingOpcode::MOV
        | DataProcessingOpcode::BIC
        | DataProcessingOpcode::MVN => {
            cpu.registers.update_flag(CondFlag::Zero, result == 0x00);
            cpu.registers
                .update_flag(CondFlag::Sign, result & (1 << 31) != (1 << 31));
        }
        _ => {
            cpu.registers.update_flag(CondFlag::Zero, result == 0x00);
            cpu.registers
                .update_flag(CondFlag::Sign, result & (1 << 31) != (1 << 31));
            cpu.registers.update_flag(CondFlag::Carry, carry);
        }
    }
}

fn execute_and(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    cpu.registers[rd] = cpu.registers[rn] & operand;
    (cpu.registers[rd], false)
}

fn execute_eor(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    cpu.registers[rd] = cpu.registers[rn] ^ operand;
    (cpu.registers[rd], false)
}

fn execute_sub(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    let (result, borrow) = cpu.registers[rn].overflowing_sub(operand);
    cpu.registers[rd] = result;
    (result, borrow)
}

fn execute_rsb(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    let (result, borrow) = operand.overflowing_sub(cpu.registers[rn]);
    cpu.registers[rd] = result;
    (result, borrow)
}

fn execute_add(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    let (result, carry) = cpu.registers[rn].overflowing_add(operand);
    cpu.registers[rd] = result;
    (result, carry)
}

fn execute_adc(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    let (result, carry1) = cpu.registers[rn].overflowing_add(operand);
    let (result, carry2) = result.overflowing_add(if cpu.registers.carry() { 1 } else { 0 });
    cpu.registers[rd] = result;
    (result, carry1 || carry2)
}

fn execute_sbc(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    let (result, carry1) = cpu.registers[rn].overflowing_sub(operand);
    let (result, carry2) = result.overflowing_add(if cpu.registers.carry() { 1 } else { 0 });
    cpu.registers[rd] = result;
    (result, carry1 || carry2)
}

fn execute_rsc(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    let (result, carry1) = operand.overflowing_add(cpu.registers[rn]);
    let (result, carry2) = result.overflowing_add(if cpu.registers.carry() { 1 } else { 0 });
    cpu.registers[rd] = result;
    (result, carry1 || carry2)
}

fn execute_tst(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    (cpu.registers[rn] & operand, false)
}

fn execute_teq(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    (cpu.registers[rn] ^ operand, false)
}

fn execute_cmp(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    cpu.registers[rn].overflowing_sub(operand)
}

fn execute_cmn(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    cpu.registers[rn].overflowing_add(operand)
}

fn execute_orr(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    cpu.registers[rd] = cpu.registers[rn] | operand;
    (cpu.registers[rd], false)
}

fn execute_mov(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    cpu.registers[rd] = operand;
    (cpu.registers[rd], false)
}

fn execute_bic(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    cpu.registers[rd] = cpu.registers[rn] & !operand;
    (cpu.registers[rd], false)
}

fn execute_mvn(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool) {
    cpu.registers[rd] = !operand;
    (cpu.registers[rd], false)
}
