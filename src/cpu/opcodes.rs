use super::registers::CondFlag;
use crate::cpu::Arm7Cpu;
use crate::cpu::registers::{CpuMode, CpuState, LINK_IDX, PC_IDX, RegisterFile};
use crate::system_bus::{ACCESS_CODE, ACCESS_LOCK, ACCESS_NONSEQ, ACCESS_SEQ, SystemBus};
use crate::{extract_mask, test_bit};
use std::cmp::PartialEq;

pub fn decode_arm_opcode(opcode: u32) -> Option<Opcode> {
    // TODO: This is possibly a slow decoding scheme. Try a LUT?

    let decoders = [
        try_decode_b_bl,
        try_decode_bx,
        try_decode_data_processing,
        try_decode_ldm_stm,
        try_decode_swp,
        try_decode_swi,
        try_decode_data_transfer,
    ];

    for decoder in decoders {
        if let Some(decoded_opcode) = decoder(opcode) {
            return Some(Opcode::Arm(decoded_opcode));
        }
    }

    None
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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

pub fn condition_from_opcode(opcode: u32) -> Condition {
    unsafe { std::mem::transmute::<u8, Condition>((opcode >> 28) as u8) }
}

pub fn check_condition(registers: &RegisterFile, opcode: u32) -> bool {
    let condition = condition_from_opcode(opcode);

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
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ShiftType {
    /// Logical Shift Left
    Lsl = 0b00,
    /// Logical Shift Right
    Lsr = 0b01,
    /// Arithmetic Shift Right
    Asr = 0b10,
    /// Rotate Right
    Ror = 0b11,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataProcessingOperand {
    Immediate(u32),
    ShiftedImmediate {
        operand: u32,
        shift: u32, // Always ROR
    },
    RegisterShiftedRegister {
        operand_register: u8,
        shift_register: u8,
        shift_type: ShiftType,
    },
    ImmediateShiftedRegister {
        operand_register: u8,
        shift: u32,
        shift_type: ShiftType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegisterTransferType {
    Store,
    Load,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OffsetArgument {
    ImmdiateOffset(u8),
    RegisterOffset(u8),
}

impl OffsetArgument {
    pub fn value(&self, registers: &RegisterFile) -> u8 {
        match self {
            OffsetArgument::ImmdiateOffset(value) => *value,
            OffsetArgument::RegisterOffset(register_idx) => registers[*register_idx as usize] as u8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataTransferSize {
    /// Only signed byte supported via LDRSB i.e. sign extend
    Byte,
    /// true: Signed half word via LDRSH i.e. sign extend
    /// false: Store half word via STRH, Load unsigned halfword via LDRH i.e. zero extend
    HalfWord(bool),
    /// Only unsigned load/store supported
    DoubleWord,
}

#[derive(Debug, Clone, Copy)]
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
    DataProcessing {
        operand: DataProcessingOperand,
        /// Destination register index. Can be `PC_IDX` in which case the behavior of the opcode changes
        rd: u8,
        /// First operand register index
        rn: u8,
        sub_opcode: DataProcessingOpcode,
        set_flags: bool,
    },

    // LDM & STM
    BlockDataTransfer {
        base_register: u8,
        transfer_type: RegisterTransferType,
        pre_increment: bool,    // Add offset before transfer. False implies post
        increment: bool,        // Offset adds. False implies down i.e. subtract
        psr_n_force_user: bool, // Load PSR or force user mode
        write_address_into_base: bool,
        rlist: u16,
    },

    DataTransfer {
        transfer_type: RegisterTransferType,
        transfer_size: DataTransferSize,
        pre_increment: bool, // or pre_decrement
        increment: bool,
        offset: OffsetArgument,
        write_back: bool,
        base_register: u8,
        target_register: u8,
    },

    Swap {
        base_register: u8,
        src_register: u8,
        dest_register: u8,
        word: bool, // 32 bits. False implies swap 8 bits
    },
    Swi {
        comment: u32, // Only low 24 bits are valid
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Arm(DecodedArmOpcode),
    Thumb,
}

// B, BL
fn try_decode_b_bl(opcode: u32) -> Option<DecodedArmOpcode> {
    if opcode & 0xE000000 != 0xA000000 {
        return None;
    }

    if test_bit!(opcode, 24) {
        Some(DecodedArmOpcode::BL {
            offset: opcode & 0xFFFFFF,
        })
    } else {
        Some(DecodedArmOpcode::B {
            offset: opcode & 0xFFFFFF,
        })
    }
}

pub fn execute_b<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType, mut offset: u32) {
    if test_bit!(offset, 23) {
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
    cpu.registers[LINK_IDX] = link;
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
    if test_bit!(destination, 0) {
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

    let sub_opcode = extract_mask!(opcode, 0x1E00000u32) as u8;
    let set_flags = test_bit!(opcode, 20);
    // Set condition code flag must be *true* for test and compare opcodes
    if (0x8..=0xB).contains(&sub_opcode) && !set_flags {
        return None;
    }

    // First operand register must be *0000* for MOV and MVN
    if (sub_opcode == 0xD || sub_opcode == 0xF) && (opcode & (0b1111 << 16) != 0) {
        return None;
    }

    let rn = extract_mask!(opcode, 0xF0000u32) as u8;
    let rd = extract_mask!(opcode, 0xF000u32) as u8;
    if (0x8..=0xB).contains(&sub_opcode) {
        // Destination register must be *0000* or *1111* for TST/TEQ/CMP/CMN
        if rd != 0b0000 && rd != 0b1111 {
            return None;
        }
    };

    let sub_opcode = unsafe { std::mem::transmute::<u8, DataProcessingOpcode>(sub_opcode) };

    let is_immediate = test_bit!(opcode, 25);
    let operand = if is_immediate {
        let nn = extract_mask!(opcode, 0xFFu32);
        // Shifted in jumps of 2 so 7 instead of 8 to keep LSB 0
        let shift = extract_mask!(opcode, 0xF00u32) << 1;

        if shift != 0 {
            DataProcessingOperand::ShiftedImmediate { operand: nn, shift }
        } else {
            DataProcessingOperand::Immediate(nn)
        }
    } else {
        // Register
        let shift_by_register = test_bit!(opcode, 4);
        if shift_by_register && test_bit!(opcode, 7) {
            // Bit 7 must be 0 when shifting by register
            return None;
        }

        let operand_register = extract_mask!(opcode, 0xFu32) as u8;
        let shift_type =
            unsafe { std::mem::transmute::<u8, ShiftType>(extract_mask!(opcode, 0x60u32) as u8) };

        if shift_by_register {
            let shift_register = extract_mask!(opcode, 0xF00u32) as u8;
            DataProcessingOperand::RegisterShiftedRegister {
                operand_register,
                shift_register,
                shift_type,
            }
        } else {
            DataProcessingOperand::ImmediateShiftedRegister {
                operand_register,
                shift: (opcode & 0xF80) >> 7,
                shift_type,
            }
        }
    };

    Some(DecodedArmOpcode::DataProcessing {
        operand,
        rd,
        rn,
        sub_opcode,
        set_flags,
    })
}

fn lsl(value: u32, amount: u32) -> u32 {
    value.unbounded_shl(amount)
}

fn lsr(value: u32, amount: u32) -> u32 {
    value.unbounded_shr(amount)
}

fn asr(value: u32, amount: u32) -> u32 {
    (value as i32).unbounded_shr(amount) as u32
}

pub fn ror(value: u32, amount: u32) -> u32 {
    value.rotate_right(amount)
}

/// Calls the proper shift function and returns the shifted (rotated) value and shifted out carry
fn shift(shift_type: ShiftType, value: u32, amount: u32) -> (u32, bool) {
    if amount == 0 {
        return (value, false);
    }

    let expanded_value = value as u64;
    let expanded_amount = 33.min(amount);

    match shift_type {
        ShiftType::Lsl => (
            lsl(value, amount),
            ((expanded_value.unbounded_shl(expanded_amount - 1)) >> 31 & 0b1) != 0,
        ),
        ShiftType::Lsr => (
            lsr(value, amount),
            (expanded_value.unbounded_shr(expanded_amount - 1)) & 0b1 != 0,
        ),
        ShiftType::Asr => (
            asr(value, amount),
            ((expanded_value as i32 as i64).unbounded_shr(expanded_amount - 1)) & 0b1 != 0,
        ),
        ShiftType::Ror => {
            let res = ror(value, amount);
            let carry = (res >> 31) & 0b1 != 0;
            (res, carry)
        }
    }
}

pub fn execute_data_processing<BusType: SystemBus>(
    cpu: &mut Arm7Cpu,
    bus: &mut BusType,
    sub_opcode: DataProcessingOpcode,
    rd: u8,
    rn: u8,
    operand: DataProcessingOperand,
    set_flags: bool,
) {
    cpu.next_access = ACCESS_CODE | ACCESS_SEQ;

    let operand_a = cpu.registers[rn as usize]; // rn might be aliased to rd and we need the initial value
    let (operand_b, shifted_carry) = match operand {
        DataProcessingOperand::Immediate(value) => (value, None),
        DataProcessingOperand::ShiftedImmediate { operand, shift } => {
            let shifted_operand = ror(operand, shift);
            let shifted_carry = (operand >> (shift - 1)) & 1 != 0;
            (shifted_operand, Some(shifted_carry))
        }
        DataProcessingOperand::RegisterShiftedRegister {
            operand_register,
            shift_register,
            shift_type,
        } => {
            // Only lower 8 bits of shift amount are used
            let shift_amount = extract_mask!(cpu.registers[shift_register as usize], 0xFFu32);

            bus.idle();
            cpu.registers[PC_IDX] += 4;
            cpu.next_access = ACCESS_CODE | ACCESS_NONSEQ; // nOPC is 1 when shift(Rs)

            let value = cpu.registers[operand_register as usize];

            let (value, carry) = shift(shift_type, value, shift_amount);

            (value, if shift_amount != 0 { Some(carry) } else { None })
        }
        DataProcessingOperand::ImmediateShiftedRegister {
            operand_register,
            shift: shift_amount,
            shift_type,
        } if shift_amount == 0 => {
            let value = cpu.registers[operand_register as usize];
            match shift_type {
                ShiftType::Lsl => (value, None), // No shift, C flag not affected
                ShiftType::Lsr => (0, Some(value & (1 << 31) != 0)), // operand is 0, C flag is bit 31 of register
                ShiftType::Asr => (((value as i32) >> 31) as u32, Some(value & (1 << 31) != 0)), // all operand bit and C are copies of bit 31 of register value
                ShiftType::Ror => {
                    // Same as ror(value, 1) but bit 31 set to current C
                    let carry = cpu.registers.carry();
                    let result = ror(value, 1);
                    let mask = 1 << 31;
                    let result = if carry { result | mask } else { result & !mask };
                    (result, Some(value & 1 != 0))
                }
            }
        }
        DataProcessingOperand::ImmediateShiftedRegister {
            operand_register,
            shift: shift_amount,
            shift_type,
        } => {
            let value = cpu.registers[operand_register as usize];
            let (value, carry) = shift(
                shift_type,
                value,
                // ASR#32 is encoded as ASR#0
                if shift_type == ShiftType::Asr && shift_amount == 0 {
                    32
                } else {
                    shift_amount
                },
            );
            (value, Some(carry))
        }
    };

    let (result, carry, overflow) = match sub_opcode {
        DataProcessingOpcode::AND => execute_and(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::EOR => execute_eor(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::SUB => execute_sub(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::RSB => execute_rsb(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::ADD => execute_add(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::ADC => execute_adc(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::SBC => execute_sbc(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::RSC => execute_rsc(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::TST => execute_tst(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::TEQ => execute_teq(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::CMP => execute_cmp(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::CMN => execute_cmn(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::ORR => execute_orr(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::MOV => execute_mov(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::BIC => execute_bic(cpu, rd as usize, rn as usize, operand_b),
        DataProcessingOpcode::MVN => execute_mvn(cpu, rd as usize, rn as usize, operand_b),
    };

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
                if let Some(carry) = shifted_carry {
                    cpu.registers.update_flag(CondFlag::Carry, carry);
                }
            }
            _ => {
                cpu.registers.update_flag(CondFlag::Carry, carry);
                cpu.registers.update_flag(CondFlag::Overflow, overflow);
            }
        }
    }

    if rd as usize == PC_IDX {
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
        } else if !matches!(
            operand,
            DataProcessingOperand::RegisterShiftedRegister { .. }
        ) {
            cpu.registers.get_and_incr_pc(4);
        }
    } else if !matches!(
        operand,
        DataProcessingOperand::RegisterShiftedRegister { .. }
    ) {
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
        (!(cpu.registers[rn] ^ operand) & (cpu.registers[rn] ^ (result as u32))) >> 31 != 0;
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

// Block Data Transfer (LDM, STM)
fn try_decode_ldm_stm(opcode: u32) -> Option<DecodedArmOpcode> {
    if opcode & 0x0E000000 != 0x08000000 {
        return None;
    }

    let pre_increment = opcode & 0x1000000 == 0x1000000;
    let increment = opcode & 0x800000 == 0x800000;
    let psr_n_force_user = opcode & 0x400000 == 0x400000;
    let write_address_into_base = opcode & 0x200000 == 0x200000;
    let transfer_type = if opcode & 0x100000 == 0x100000 {
        RegisterTransferType::Load
    } else {
        RegisterTransferType::Store
    };
    let base_register = ((opcode as usize & 0xF0000) >> 16) as u8;
    let rlist = opcode as u16;

    Some(DecodedArmOpcode::BlockDataTransfer {
        transfer_type,
        pre_increment,
        increment,
        psr_n_force_user,
        write_address_into_base,
        base_register,
        rlist,
    })
}

pub fn execute_block_data_transfer<BusType: SystemBus>(
    cpu: &mut Arm7Cpu,
    bus: &mut BusType,
    base_register: u8,
    transfer_type: RegisterTransferType,
    pre_increment: bool,
    increment: bool,
    psr_n_force_user: bool,
    write_address_into_base: bool,
    rlist: u16,
) {
    let mut words_to_transfer = rlist.count_ones();
    // Empty Rlist does R15 on ARMv4
    let rlist = if words_to_transfer == 0 {
        words_to_transfer = 16; // When updating *only* PC it is as if all 16 registers are transferred
        1 << 15
    } else {
        rlist
    };

    let first = rlist.trailing_zeros() as usize;
    let rb_in_rlist = rlist & (1 << base_register) != 0;
    let pc_in_rlist = rlist & (1 << PC_IDX) != 0;
    let rb_first_in_rlist = rb_in_rlist && (base_register as usize == first);

    let old_base_address = cpu.registers[base_register as usize];

    let (mut address, new_base_address) = if increment {
        (
            old_base_address,
            old_base_address + (words_to_transfer << 2),
        )
    } else {
        let new_base_address = old_base_address - (words_to_transfer << 2);
        (new_base_address + 4, new_base_address)
    };

    if pre_increment {
        address = if increment { address + 4 } else { address - 4 };
    }

    cpu.registers.get_and_incr_pc(4);

    let old_mode = cpu.registers.mode();
    let switch_mode = psr_n_force_user
        && ((transfer_type == RegisterTransferType::Store) || !pc_in_rlist)
        && old_mode != CpuMode::User
        && old_mode != CpuMode::System;

    if switch_mode {
        cpu.switch_cpu_mode(CpuMode::User);
    }

    // The first write is non-sequential
    cpu.next_access = ACCESS_NONSEQ;

    let mut reload_pipeline = false;

    {
        for i in (first..16).filter(|i| rlist & (1 << i) != 0) {
            if RegisterTransferType::Store == transfer_type {
                bus.write_word(address, cpu.registers[i], cpu.next_access);
                if write_address_into_base && i == first {
                    cpu.registers[base_register as usize] = new_base_address;
                    if base_register as usize == PC_IDX {
                        reload_pipeline = true;
                    }
                }
            } else {
                let value = bus.read_word(address, cpu.next_access);
                if i == PC_IDX {
                    reload_pipeline = true;
                }
                if i == first {
                    if write_address_into_base {
                        cpu.registers[base_register as usize] = new_base_address;
                        if base_register as usize == PC_IDX {
                            reload_pipeline = true;
                        }
                    }
                }
                // Need to do it here because if write back and Rb is first in Rlist `value` needs
                // to override `new_base_address` i.e. no writeback
                cpu.registers[i] = value;
            }

            address += 4;
            // Every write after the first is sequential
            cpu.next_access = ACCESS_SEQ;
        }
    }

    if transfer_type == RegisterTransferType::Load {
        if switch_mode {
            // TODO: User mode conflict goes here
        }

        if pc_in_rlist && psr_n_force_user {
            cpu.registers.cpsr = cpu.registers.spsr_moded();
        }
    }

    // TODO: This needs to be scheduled for one instruction after the current LDM/STM
    if switch_mode {
        cpu.switch_cpu_mode(old_mode);
    }

    cpu.next_access = ACCESS_CODE | ACCESS_NONSEQ;
    if reload_pipeline {
        cpu.reload_pipeline(bus);
    }
}

fn try_decode_data_transfer(opcode: u32) -> Option<DecodedArmOpcode> {
    if opcode & 0x0E000090 != 0x00000090 {
        return None;
    }

    let pre_increment = test_bit!(opcode, 24); // or decrement
    let increment = test_bit!(opcode, 23);
    let immediate_offset = test_bit!(opcode, 22);

    let write_back = if pre_increment {
        test_bit!(opcode, 21)
    } else {
        // If post indexing then writeback bit must always be `0` however
        // writeback is always enabled
        if test_bit!(opcode, 21) {
            return None;
        }
        true
    };
    let mut transfer_type = if test_bit!(opcode, 20) {
        RegisterTransferType::Load
    } else {
        RegisterTransferType::Store
    };

    let base_register = extract_mask!(opcode, 0xF0000u32) as u8;
    let target_register = extract_mask!(opcode, 0xF000u32) as u8;

    let offset = if immediate_offset {
        let offset_high = (extract_mask!(opcode, 0xF00u32) << 4) as u8;
        let offset_low = extract_mask!(opcode, 0xFu32) as u8;
        OffsetArgument::ImmdiateOffset(offset_high | offset_low)
    } else {
        // If register offset bits 11-8 are unused and must be 0000
        if opcode & 0xF00 != 0x000 {
            return None;
        }
        let register = extract_mask!(opcode, 0xFu32) as u8;
        OffsetArgument::RegisterOffset(register)
    };

    let transfer_size = match extract_mask!(opcode, 0x60u32) {
        0b00 if transfer_type == RegisterTransferType::Store => return None, // Reserved for SWP
        0b01 if transfer_type == RegisterTransferType::Store => DataTransferSize::HalfWord(false),
        0b10 if transfer_type == RegisterTransferType::Store => {
            transfer_type = RegisterTransferType::Load; // Actually a LDRD
            DataTransferSize::DoubleWord
        }
        0b11 if transfer_type == RegisterTransferType::Store => DataTransferSize::DoubleWord,

        0b00 if transfer_type == RegisterTransferType::Load => return None, // Reserved
        0b01 if transfer_type == RegisterTransferType::Load => DataTransferSize::HalfWord(false),
        0b10 if transfer_type == RegisterTransferType::Load => DataTransferSize::Byte,
        0b11 if transfer_type == RegisterTransferType::Load => DataTransferSize::HalfWord(true),

        _ => panic!("Impossible match arm"),
    };

    Some(DecodedArmOpcode::DataTransfer {
        transfer_type,
        transfer_size,
        pre_increment,
        increment,
        write_back,
        base_register,
        target_register,
        offset,
    })
}

pub fn execute_data_transfer<BusType: SystemBus>(
    cpu: &mut Arm7Cpu,
    bus: &mut BusType,
    transfer_type: RegisterTransferType,
    transfer_size: DataTransferSize,
    pre_increment: bool,
    increment: bool,
    offset: OffsetArgument,
    write_back: bool,
    base_register: u8,
    target_register: u8,
) {
    cpu.registers.get_and_incr_pc(4);

    // Base address is always read as PC + 8
    let base_address = cpu.registers[base_register as usize];

    let mut address = base_address;
    if pre_increment {
        if increment {
            address += offset.value(&cpu.registers) as u32;
        } else {
            address -= offset.value(&cpu.registers) as u32;
        }
    }

    match transfer_type {
        RegisterTransferType::Store => match transfer_size {
            DataTransferSize::Byte => panic!("STRB does not exist"),
            DataTransferSize::HalfWord(_) => {
                bus.write_half_word(
                    address,
                    cpu.registers[target_register as usize] as u16,
                    ACCESS_NONSEQ,
                );
            }
            DataTransferSize::DoubleWord => todo!("STRD not supported on ARMv4"),
        },
        RegisterTransferType::Load => match transfer_size {
            DataTransferSize::Byte => {
                let mut value = bus.read_byte(address, ACCESS_NONSEQ) as u32;
                if test_bit!(value, 7) {
                    value |= 0xFFFFFF00; // Sign extend
                }
                cpu.registers[target_register as usize] = value;
            }
            DataTransferSize::HalfWord(sign_extend) => {
                let mut value = bus.read_half_word(address, ACCESS_NONSEQ) as u32;
                if test_bit!(value, 15) && sign_extend {
                    value |= 0xFFFF0000;
                }
                cpu.registers[target_register as usize] = value;
            }
            DataTransferSize::DoubleWord => todo!("LDRD not supported on ARMv4"),
        },
    }

    if target_register as usize == PC_IDX && transfer_type == RegisterTransferType::Load {
        cpu.reload_pipeline(bus);
    }

    if write_back {
        cpu.registers[base_register as usize] = address;
    }

    if !pre_increment && let OffsetArgument::RegisterOffset(_) = offset {
        //TODO: Post increment register offset
    }

    cpu.next_access = ACCESS_CODE | ACCESS_NONSEQ;
}

fn try_decode_swp(opcode: u32) -> Option<DecodedArmOpcode> {
    if opcode & 0x0FB00FF0 != 0x01000090 {
        return None;
    }

    let word = !test_bit!(opcode, 22);
    let src_register = opcode as u8 & 0xF;
    let dest_register = ((opcode >> 12) & 0xF) as u8;
    let base_register = ((opcode >> 16) & 0xF) as u8;

    Some(DecodedArmOpcode::Swap {
        base_register,
        src_register,
        dest_register,
        word,
    })
}

pub fn execute_swp<BusType: SystemBus>(
    cpu: &mut Arm7Cpu,
    bus: &mut BusType,
    base_register: u8,
    src_register: u8,
    dest_register: u8,
    word: bool,
) {
    cpu.registers.get_and_incr_pc(4);

    let base_address = cpu.registers[base_register as usize];
    let src_value = cpu.registers[src_register as usize];

    if word {
        cpu.registers[dest_register as usize] = bus.read_word(base_address, ACCESS_NONSEQ);
        if base_address != (base_address & !3) {
            cpu.registers[dest_register as usize] = ror(
                cpu.registers[dest_register as usize],
                (base_address & 3) * 8,
            );
        }
        bus.write_word(base_address, src_value, ACCESS_NONSEQ | ACCESS_LOCK);
    } else {
        cpu.registers[dest_register as usize] = bus.read_byte(base_address, ACCESS_NONSEQ) as u32;
        bus.write_byte(base_address, src_value as u8, ACCESS_NONSEQ | ACCESS_LOCK);
    }

    bus.idle();
    cpu.next_access = ACCESS_CODE;

    if dest_register as usize == PC_IDX {
        cpu.reload_pipeline(bus);
    }
}

fn try_decode_swi(opcode: u32) -> Option<DecodedArmOpcode> {
    let mask = 0xF << 24;
    if opcode & mask != mask {
        return None;
    }

    let comment = opcode & 0x00FFFFFF;

    Some(DecodedArmOpcode::Swi { comment })
}

pub fn execute_swi<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType) {
    cpu.registers.r14_svc = cpu.registers.user_bank[PC_IDX] - 4;
    cpu.registers.spsr_svc = cpu.registers.cpsr;
    cpu.switch_cpu_mode(CpuMode::Supervisor);
    cpu.registers.cpsr |= CondFlag::IrqDisable as u32;
    cpu.registers.user_bank[PC_IDX] = 0x00000008;
    cpu.reload_pipeline(bus);
}
