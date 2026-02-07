use eframe::egui;
use eframe::egui::{Color32, Response};
use gba::cpu::opcodes::{
    DataProcessingOpcode, DataProcessingOperand, DecodedArmOpcode, Opcode, ror,
};

pub fn opcode_disassembly(ui: &mut egui::Ui, opcode: &Opcode) -> Response {
    ui.horizontal(|ui| match opcode {
        Opcode::Arm(decoded_arm_opcode) => format_decoded_arm_opcode(ui, decoded_arm_opcode),
        Opcode::Thumb => ui.label("Thumb disassembly not implemented".to_string()),
    })
    .response
}

fn format_decoded_arm_opcode(ui: &mut egui::Ui, opcode: &DecodedArmOpcode) -> Response {
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
        } => format_data_processing(ui, operand, *rd, *rn, sub_opcode),
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

fn format_opcode_b_bl(ui: &mut egui::Ui, mut offset: u32, is_bl: bool) -> Response {
    if !is_bl && offset & 0x800000 != 0x00 {
        // Offset is a 24-bit signed value
        offset |= 0xFF000000; // Sign extend to 32-bits
    }
    ui.label(
        egui::RichText::new(if is_bl { "BL" } else { "B" }).color(Color32::from_rgb(70, 70, 245)),
    );
    ui.label(egui::RichText::new(format!("${}", offset as i32)).underline())
}

fn format_opcode_bx(ui: &mut egui::Ui, register_idx: usize) -> Response {
    ui.label(egui::RichText::new("BX").color(Color32::from_rgb(70, 70, 245)));
    ui.label(
        egui::RichText::new(format!("{}", format_register(register_idx)))
            .color(Color32::from_rgb(120, 240, 80)),
    )
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
) -> Response {
    ui.label(
        egui::RichText::new(format!("{:?}", sub_opcode)).color(Color32::from_rgb(70, 70, 245)),
    );

    let register_idx = if *sub_opcode != DataProcessingOpcode::TST
        && *sub_opcode != DataProcessingOpcode::TEQ
        && *sub_opcode != DataProcessingOpcode::CMP
        && *sub_opcode != DataProcessingOpcode::CMN
    {
        rd
    } else {
        rn
    };
    ui.label(
        egui::RichText::new(format!("{}", format_register(register_idx)))
            .color(Color32::from_rgb(120, 240, 80)),
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
