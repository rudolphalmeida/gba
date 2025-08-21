use crate::cpu::registers::{CpuState, RegisterFile, PC_IDX};
use crate::cpu::Arm7Cpu;
use crate::system_bus::{SystemBus, ACCESS_CODE, ACCESS_NONSEQ, ACCESS_SEQ};

use super::registers::CondFlag;

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

    let zero = registers.zero();
    let carry = registers.carry();
    let overflow = registers.overflow();
    let sign = registers.sign();

    match condition {
        Condition::Equal => zero,
        Condition::NotEqual => !zero,
        Condition::CarrySet => carry,
        Condition::CarryCleared => !carry,
        Condition::Minus => sign,
        Condition::Positive => !sign,
        Condition::Overflow => overflow,
        Condition::NoOverflow => !overflow,
        Condition::UnsignedHigher => carry && !zero,
        Condition::UnsignedLowerOrSame => !carry || zero,
        Condition::GreaterOrEqual => sign == overflow,
        Condition::LessThan => sign != overflow,
        Condition::GreaterThan => !zero && (sign == overflow),
        Condition::LessOrEqual => zero || (sign != overflow),
        Condition::Always => true,
        Condition::Never => false,
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd)]
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

pub enum DecodedArmOpcode {
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
        /// Destination register index. Can be `PC_IDX` in which case the behaviour of the opcode changes
        rd: usize,
        /// First operand register index
        rn: usize,
        /// Second operand. Can be register shifted or immediate shifted or immediate
        operand: u32,
        sub_opcode: DataProcessingOpcode,
        /// Last shifted bit if applicable. `None` indicates no shift
        shifter_carry: Option<bool>,
        /// If the operand is an immediate or a register shifted operand
        shifted_operand: bool,
        set_flags: bool,
    },
}

pub enum Opcode {
    Arm(DecodedArmOpcode),
    Thumb,
}

fn try_decode_b_bl(opcode: u32) -> Option<DecodedArmOpcode> {
    if opcode & 0xE000000 != 0xA000000 {
        return None;
    }

    let mask = 1 << 24;
    match opcode & mask {
        0 => Some(DecodedArmOpcode::B {
            offset: opcode & 0xFFFFFF,
        }),
        mask => Some(DecodedArmOpcode::BL {
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
fn try_decode_bx(opcode: u32) -> Option<DecodedArmOpcode> {
    if opcode & 0x0FFFFF10 != 0x012FFF10 {
        // Ignoring BLX
        return None;
    }

    Some(DecodedArmOpcode::BX {
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
fn try_decode_data_processing(opcode: u32) -> Option<DecodedArmOpcode> {
    if opcode & 0x0C000000 != 0 {
        return None;
    }

    let sub_opcode = ((opcode & 0x1E00000) >> 21) as u8;
    // Set condition code flag must be *true* for test and compare opcodes
    if (0x8..=0xB).contains(&sub_opcode) && (opcode & (1 << 20) != (1 << 20)) {
        return None;
    }
    let set_flags = opcode & (1 << 20) == (1 << 20);

    // First operand register must be *0000* for MOV and MVN
    if (sub_opcode == 0xD || sub_opcode == 0xF) && (opcode & (0b1111 << 16) != 0) {
        return None;
    }

    let rn = (opcode as usize & (0b1111 << 16)) >> 16;
    let rd = (opcode as usize & (0b1111 << 12)) >> 12;
    if (0x8..=0xB).contains(&sub_opcode) {
        // Destination register must be *0000* or *1111* for TST/TEQ/CMP/CMN
        if rd != 0b0000 && rd != 0b1111 {
            return None;
        }
    };

    let sub_opcode = unsafe { std::mem::transmute::<u8, DataProcessingOpcode>(sub_opcode) };

    let is_immediate = opcode & (1 << 25) != 0;
    if is_immediate {
        let nn = opcode & 0xFF;
        // Shifted in jumps of 2 so 7 instead of 8 to keep LSB 0
        let shift = (opcode & 0xF00) >> 7;
        let operand = ror(nn, shift);

        let shifter_carry = if shift != 0 {
            Some((nn >> (shift - 1)) & 1 != 0)
        } else {
            None
        };
        return Some(DecodedArmOpcode::DataProcessing {
            sub_opcode,
            operand,
            rd,
            rn,
            shifter_carry,
            shifted_operand: is_immediate, // TODO: Also OR with register shifted
            set_flags,
        });
    } else {
        // Register
        todo!()
    }
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
    shifter_carry: Option<bool>,
    shifted_operand: bool,
    set_flags: bool,
) {
    let operand_1 = cpu.registers[rn]; // rn might be aliased to rd and we need the initial value
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
    let (result, carry, overflow) = operation(cpu, rd, rn, operand);

    if set_flags {
        cpu.registers.update_flag(CondFlag::Zero, result == 0x00);
        cpu.registers
            .update_flag(CondFlag::Sign, (result as i32) < 0);

        match sub_opcode {
            DataProcessingOpcode::AND
            | DataProcessingOpcode::EOR
            | DataProcessingOpcode::TST
            | DataProcessingOpcode::TEQ
            | DataProcessingOpcode::ORR
            | DataProcessingOpcode::MOV
            | DataProcessingOpcode::BIC
            | DataProcessingOpcode::MVN => {
                if let Some(carry) = shifter_carry {
                    cpu.registers.update_flag(CondFlag::Carry, carry);
                }
            }
            _ => {
                cpu.registers.update_flag(CondFlag::Carry, carry);
                cpu.registers.update_flag(CondFlag::Overflow, overflow);
            }
        }
    }

    cpu.next_access = ACCESS_CODE | ACCESS_SEQ;

    if rd == PC_IDX {
        if set_flags {
            // TODO: Should not be used in user mode. (What if it is?)
            cpu.registers.cpsr = cpu.registers.spsr_moded();
        }
        if sub_opcode != DataProcessingOpcode::TST
            && sub_opcode != DataProcessingOpcode::TEQ
            && sub_opcode != DataProcessingOpcode::CMP
            && sub_opcode != DataProcessingOpcode::CMN
        {
            cpu.reload_pipeline(bus);
        } else if shifted_operand {
            cpu.registers.get_and_incr_pc(4);
        }
    } else if shifted_operand {
        cpu.registers.get_and_incr_pc(4);
    }
}

fn do_sub(operand_a: u32, operand_b: u32) -> (u32, bool, bool) {
    let result = operand_a.wrapping_sub(operand_b);
    let overflow = (((operand_a ^ operand_b) & (operand_a ^ result)) >> 31) != 0;
    (result, operand_a >= operand_b, overflow)
}

fn do_add(operand_a: u32, operand_b: u32) -> (u32, bool, bool) {
    let (result, carry) = operand_a.overflowing_add(operand_b);
    let overflow = ((!(operand_a ^ operand_b) & (operand_a ^ result)) >> 31) != 0;
    (result, carry, overflow)
}

fn do_sbc(operand_a: u32, operand_b: u32, carry: bool) -> (u32, bool, bool) {
    let operand_c = (if carry { 1 } else { 0 }) ^ 1;
    let result = operand_a.wrapping_sub(operand_b).wrapping_sub(operand_c);

    let carry = (operand_a as u64) >= ((operand_b as u64) + (operand_c as u64));
    let overflow = (((operand_a ^ operand_b) & (operand_a ^ result)) >> 31) != 0;

    (result, carry, overflow)
}

fn execute_and(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    cpu.registers[rd] = cpu.registers[rn] & operand;
    (cpu.registers[rd], false, false)
}

fn execute_eor(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    cpu.registers[rd] = cpu.registers[rn] ^ operand;
    (cpu.registers[rd], false, false)
}

fn execute_sub(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    let result = do_sub(cpu.registers[rn], operand);
    cpu.registers[rd] = result.0;
    result
}

fn execute_rsb(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    let result = do_sub(operand, cpu.registers[rn]);
    cpu.registers[rd] = result.0;
    result
}

fn execute_add(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    let result = do_add(cpu.registers[rn], operand);
    cpu.registers[rd] = result.0;
    result
}

fn execute_adc(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    let operand_a = cpu.registers[rn] as u64;
    let operand_b = operand as u64;
    let carry = if cpu.registers.carry() { 1 } else { 0 };

    let result = operand_a.wrapping_add(operand_b).wrapping_add(carry);

    let carry = result & (1 << 32) != 0;
    let overflow =
        (!(cpu.registers[rn] ^ operand) & (cpu.registers[rn] ^ (result as u32)) >> 31) != 0;
    cpu.registers[rd] = result as u32;

    (cpu.registers[rd], carry, overflow)
}

fn execute_sbc(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    let result = do_sbc(cpu.registers[rn], operand, cpu.registers.carry());
    cpu.registers[rd] = result.0;
    result
}

fn execute_rsc(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    let result = do_sbc(operand, cpu.registers[rn], cpu.registers.carry());
    cpu.registers[rd] = result.0;
    result
}

fn execute_tst(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    (cpu.registers[rn] & operand, false, false)
}

fn execute_teq(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    (cpu.registers[rn] ^ operand, false, false)
}

fn execute_cmp(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    do_sub(cpu.registers[rn], operand)
}

fn execute_cmn(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    do_add(cpu.registers[rn], operand)
}

fn execute_orr(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    cpu.registers[rd] = cpu.registers[rn] | operand;
    (cpu.registers[rd], false, false)
}

fn execute_mov(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    cpu.registers[rd] = operand;
    (cpu.registers[rd], false, false)
}

fn execute_bic(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    cpu.registers[rd] = cpu.registers[rn] & !operand;
    (cpu.registers[rd], false, false)
}

fn execute_mvn(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) -> (u32, bool, bool) {
    cpu.registers[rd] = !operand;
    (cpu.registers[rd], false, false)
}
