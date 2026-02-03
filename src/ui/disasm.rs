use eframe::egui;
use eframe::egui::{Color32, Response};
use gba::cpu::opcodes::{
    DataProcessingOpcode, DataProcessingOperand, DecodedArmOpcode, Opcode, ror,
};

pub fn opcode_disassembly(ui: &mut egui::Ui, opcode: &Opcode) -> Response {
    ui.horizontal(|ui| match opcode {
        Opcode::Arm(decoded_arm_opcode) => format_decoded_arm_opcode(ui, decoded_arm_opcode),
        Opcode::Thumb => ui.label("Thumb disassembly not implemented".to_string()),
    }).response
}

fn format_decoded_arm_opcode(ui: &mut egui::Ui, opcode: &DecodedArmOpcode) -> Response {

    match opcode {
        DecodedArmOpcode::B { offset } => format_opcode_b_bl(ui, *offset, false),
        DecodedArmOpcode::BL { offset} => format_opcode_b_bl(ui, *offset, true),
        _ => ui.label("Opcode not implemented"),
    }

    // match opcode {
    //     DecodedArmOpcode::B { offset } => format!("B ${:#X}", *offset),
    //     DecodedArmOpcode::BL { offset } => format!("BL ${:#X}", *offset),
    //     DecodedArmOpcode::BX { register_idx } => {
    //         format!("BX {}", format_register(*register_idx as usize))
    //     }
    //     DecodedArmOpcode::DataProcessing {
    //         operand,
    //         rd,
    //         rn,
    //         sub_opcode,
    //         set_flags,
    //     } => format_data_processing(operand, *rd, *rn, sub_opcode),
    //     DecodedArmOpcode::BlockDataTransfer {
    //         base_register,
    //         transfer_type,
    //         pre_increment,
    //         increment,
    //         psr_n_force_user,
    //         write_address_into_base,
    //         rlist,
    //     } => "LDM/STM".to_string(),
    // }
}

fn format_opcode_b_bl(ui: &mut egui::Ui, offset: u32, is_bl: bool) -> Response {
    ui.label(egui::RichText::new(if is_bl { "BL".to_string() } else { "B".to_string() }).color(Color32::from_rgb(70, 70, 245)));
    ui.label(egui::RichText::new(format!("${:#X}", offset)).underline())
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
    operand: &DataProcessingOperand,
    rd: usize,
    rn: usize,
    sub_opcode: &DataProcessingOpcode,
) -> String {
    format!(
        "{:?} {}, {}",
        sub_opcode,
        format_register(
            if *sub_opcode != DataProcessingOpcode::TST
                && *sub_opcode != DataProcessingOpcode::TEQ
                && *sub_opcode != DataProcessingOpcode::CMP
                && *sub_opcode != DataProcessingOpcode::CMN
            {
                rd
            } else {
                rn
            }
        ),
        format_data_processing_operand(operand)
    )
}

fn format_data_processing_operand(operand: &DataProcessingOperand) -> String {
    match operand {
        DataProcessingOperand::Immediate(value) => format!("${:#X}", *value),
        DataProcessingOperand::ShiftedImmediate { operand, shift } => {
            format!("${:#X}", ror(*operand, *shift))
        }
        DataProcessingOperand::RegisterShiftedRegister {
            operand_register,
            shift_register,
            shift_type,
        } => "RegisterShiftedRegister".to_string(),
        DataProcessingOperand::ImmediateShiftedRegister {
            operand_register,
            shift,
            shift_type,
        } => "ImmediateShiftedRegister".to_string(),
    }
}
