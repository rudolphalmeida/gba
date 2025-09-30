use crate::cpu::opcodes::{DecodedArmOpcode, Opcode};

pub fn disassemble_opcode(opcode: &Opcode) -> String {
    match opcode {
        Opcode::Arm(decoded_arm_opcode) => format_decoded_arm_opcode(decoded_arm_opcode),
        Opcode::Thumb => todo!(),
    }
}

fn format_decoded_arm_opcode(opcode: &DecodedArmOpcode) -> String {
    match opcode {
        DecodedArmOpcode::B { offset } => format!("B ${:#X}", *offset),
        DecodedArmOpcode::BL { offset } => todo!(),
        DecodedArmOpcode::BX { register_idx } => todo!(),
        DecodedArmOpcode::DataProcessing {
            operand,
            rd,
            rn,
            sub_opcode,
            set_flags,
        } => todo!(),
    }
}
