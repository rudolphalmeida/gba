use crate::ui::{COLOR_MNEMONIC, COLOR_REGISTER};
use eframe::egui;
use eframe::egui::Response;
use gba::cpu::opcodes::{
    Condition, DataProcessingOpcode, DataProcessingOperand, DecodedArmOpcode, Opcode, ror,
};

pub fn opcode_disassembly(ui: &mut egui::Ui, opcode: &Opcode) -> Response {
    ui.horizontal(|ui| {
        match opcode {
            Opcode::Arm(decoded_arm_opcode) => format_decoded_arm_opcode(ui, decoded_arm_opcode),
            Opcode::Thumb => {
                ui.label("Thumb disassembly not implemented".to_string());
            }
        };
    })
    .response
}

pub fn condition_text(condition: Condition) -> egui::RichText {
    egui::RichText::new(match condition {
        Condition::Equal => "EQ",
        Condition::NotEqual => "NE",
        Condition::CarrySet => "CS",
        Condition::CarryCleared => "CC",
        Condition::Minus => "MI",
        Condition::Positive => "PL",
        Condition::Overflow => "VS",
        Condition::NoOverflow => "VC",
        Condition::UnsignedHigher => "HI",
        Condition::UnsignedLowerOrSame => "LS",
        Condition::GreaterOrEqual => "GE",
        Condition::LessThan => "LT",
        Condition::GreaterThan => "GT",
        Condition::LessOrEqual => "LE",
        Condition::Always => "AL",
        Condition::Never => "NV",
    })
}

fn format_decoded_arm_opcode(ui: &mut egui::Ui, opcode: &DecodedArmOpcode) {
    match opcode {
        DecodedArmOpcode::B { offset } => format_opcode_b_bl(ui, *offset, false),
        DecodedArmOpcode::BL { offset } => format_opcode_b_bl(ui, *offset, true),
        DecodedArmOpcode::BX { register_idx } => format_opcode_bx(ui, *register_idx as usize),
        DecodedArmOpcode::DataProcessing {
            operand,
            rd,
            rn,
            sub_opcode,
            set_flags,
        } => format_data_processing(ui, operand, *rd, *rn, sub_opcode, *set_flags),
        _ => {
            ui.label("Opcode not implemented");
        }
    };
}

fn format_opcode_b_bl(ui: &mut egui::Ui, mut offset: u32, is_bl: bool) {
    if !is_bl && offset & 0x800000 != 0x00 {
        // Offset is a 24-bit signed value
        offset |= 0xFF000000; // Sign extend to 32-bits
    }
    // Add 8 because PC is assumed to be leading in the actual opcode
    offset = offset.wrapping_mul(4).wrapping_add(8);
    ui.colored_label(COLOR_MNEMONIC, if is_bl { "BL" } else { "B" });
    ui.label(egui::RichText::new(format!("${:X}", offset as i32)).underline());
}

fn format_opcode_bx(ui: &mut egui::Ui, register_idx: usize) {
    ui.colored_label(COLOR_MNEMONIC, "BX");
    ui.colored_label(COLOR_REGISTER, format_register(register_idx).to_string());
}

fn format_register(idx: usize) -> String {
    match idx {
        0..=13 => format!("R{}", idx),
        14 => "LR".to_string(),
        15 => "PC".to_string(),
        _ => "UNKNOWN".to_string(), // Should not happen
    }
}

fn format_data_processing(
    ui: &mut egui::Ui,
    operand: &DataProcessingOperand,
    rd: usize,
    rn: usize,
    sub_opcode: &DataProcessingOpcode,
    _set_flags: bool,
) {
    ui.colored_label(COLOR_MNEMONIC, format!("{:?}", sub_opcode));

    let register_idx = if *sub_opcode != DataProcessingOpcode::TST
        && *sub_opcode != DataProcessingOpcode::TEQ
        && *sub_opcode != DataProcessingOpcode::CMP
        && *sub_opcode != DataProcessingOpcode::CMN
    {
        rd
    } else {
        rn
    };
    ui.colored_label(COLOR_REGISTER, format_register(register_idx).to_string());
    ui.label(", ".to_string());
    format_data_processing_operand(ui, operand);
}

fn format_data_processing_operand(ui: &mut egui::Ui, operand: &DataProcessingOperand) {
    match operand {
        DataProcessingOperand::Immediate(value) => {
            ui.label(format!("${:#X}", *value));
        }
        DataProcessingOperand::ShiftedImmediate { operand, shift } => {
            ui.label(format!("${:#X}", ror(*operand, *shift)))
                .on_hover_ui(|ui| {
                    ui.label(format!("Operand = {}", operand));
                    ui.label(format!("Shift = {}", shift));
                });
        }
        _ => {
            ui.label("TODO");
        } // DataProcessingOperand::RegisterShiftedRegister {
          //     operand_register,
          //     shift_register,
          //     shift_type,
          // } => "RegisterShiftedRegister".to_string(),
          // DataProcessingOperand::ImmediateShiftedRegister {
          //     operand_register,
          //     shift,
          //     shift_type,
          // } => "ImmediateShiftedRegister".to_string(),
    };
}
