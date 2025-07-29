use crate::cpu::Arm7Cpu;
use crate::cpu::registers::RegisterFile;
use crate::system_bus::{SystemBus, ACCESS_CODE, ACCESS_SEQ};

pub fn decode_arm_opcode(opcode: u32) -> Option<Opcode> {
    if is_b_bl_blx(opcode) {
        return Some(Opcode::Arm(decode_b_bl_blx(opcode)?));
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
    BLX { offset: u32, }, // Offset is a signed 25-bit quantity
}

pub enum Opcode {
    Arm(ArmOpcode),
    Thumb,
}

// B/BL/BLX
fn is_b_bl_blx(opcode: u32) -> bool {
    opcode & 0xE000000 == 0xA000000
}

fn decode_b_bl_blx(opcode: u32) -> Option<ArmOpcode> {
    let is_blx = (opcode & 0xF0000000) == 0xF0000000;
    let mask = 1 << 24;
    match opcode & mask {
        0 if !is_blx => Some(ArmOpcode::B { offset: opcode & 0xFFFFFF }),
        mask if !is_blx => Some(ArmOpcode::BL { offset: opcode & 0xFFFFFF }),
        _ if is_blx => Some(ArmOpcode::BLX { offset: opcode & 0x1FFFFFF }),
        _ => None,
    }
}

pub fn execute_b<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType, mut offset: u32) {
    if offset & 0x800000 != 0x00 { // Offset is a 24-bit signed value
        offset |= 0xFF000000;      // Sign extend to 32-bits
    }
    let destination = cpu.registers.pc().wrapping_add(offset.wrapping_mul(4));
    bus.read_word(destination, ACCESS_CODE);
    cpu.registers.set_pc(destination);
    bus.read_word(destination + 4, ACCESS_SEQ | ACCESS_CODE);
    cpu.next_access = ACCESS_CODE | ACCESS_SEQ;

    cpu.reload_pipeline32(bus);
}

pub fn execute_bl<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType, offset: u32) {
    let link = cpu.registers.pc().wrapping_sub(4);
    execute_b(cpu, bus, offset);
    cpu.registers.set_r14(link);
}

pub fn execute_blx<BusType: SystemBus>(cpu: &mut Arm7Cpu, bus: &mut BusType, offset: u32) {

}


