use crate::cpu::Arm7Cpu;
use crate::cpu::registers::{CpuState, RegisterFile, PC_IDX};
use crate::system_bus::{SystemBus, ACCESS_CODE, ACCESS_NONSEQ, ACCESS_SEQ};

pub fn decode_arm_opcode(opcode: u32) -> Option<Opcode> {
    if is_b_bl(opcode) {
        return Some(Opcode::Arm(decode_b_bl(opcode)?));
    }

    if is_bx(opcode) {
        return Some(Opcode::Arm(decode_bx(opcode)?));
    }

    None
}

#[repr(u32)]
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
    let condition = unsafe { std::mem::transmute::<u32, Condition>(opcode >> 28) };

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

pub enum ArmOpcode {
    B { offset: u32 },  // Offset is a signed 24-bit quantity
    BL { offset: u32, } , // Offset is a signed 24-bit quantity
    BX { register_idx: u8 }
}

pub enum Opcode {
    Arm(ArmOpcode),
    Thumb,
}

// B/BL
fn is_b_bl(opcode: u32) -> bool {
    opcode & 0xE000000 == 0xA000000
}

fn decode_b_bl(opcode: u32) -> Option<ArmOpcode> {
    let mask = 1 << 24;
    match opcode & mask {
        0 => Some(ArmOpcode::B { offset: opcode & 0xFFFFFF }),
        mask => Some(ArmOpcode::BL { offset: opcode & 0xFFFFFF }),
    }
}

pub fn execute_b<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType, mut offset: u32) {
    if offset & 0x800000 != 0x00 { // Offset is a 24-bit signed value
        offset |= 0xFF000000;      // Sign extend to 32-bits
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
fn is_bx(opcode: u32) -> bool { opcode & 0x0FFFFF10 == 0x012FFF10 } // Ignoring BLX

fn decode_bx(opcode: u32) -> Option<ArmOpcode> {
    Some(ArmOpcode::BX { register_idx: opcode as u8 & 0xF })
}

pub fn execute_arm_to_thumb_bx<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType, register_idx: usize) {
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
