use crate::cpu::registers::RegisterFile;
use crate::system_bus::{SystemBus, ACCESS_CODE, ACCESS_SEQ};

pub fn decode_arm_opcode(opcode: u32) -> Option<Opcode> {
    if is_b_bl_blx(opcode) {
        return Some(Opcode::Arm(decode_b_bl_blx(opcode)?));
    }

    None
}

pub fn check_condition(registers: &RegisterFile, opcode: u32) -> bool {
    true
}

pub enum Condition {
    
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

pub fn execute_b<BusType: SystemBus>(registers: &mut RegisterFile, bus: &mut BusType, offset: u32) {
    let destination = (registers.pc() - 4).wrapping_add((offset & 0xFFFFFF) << 2);
    bus.read_word(destination, ACCESS_CODE);
    registers.set_pc(destination);
    bus.read_word(destination + 4, ACCESS_SEQ | ACCESS_CODE);
}

pub fn execute_bl<BusType: SystemBus>(registers: &mut RegisterFile, bus: &mut BusType, offset: u32) {
    let destination = (registers.pc() - 4).wrapping_add((offset & 0xFFFFFF) << 2);
    bus.read_word(destination, ACCESS_CODE);
    registers.set_pc(destination);
    bus.read_word(destination + 4, ACCESS_SEQ | ACCESS_CODE);
}

pub fn execute_blx<BusType: SystemBus>(registers: &mut RegisterFile, bus: &mut BusType, offset: u32) {

}


