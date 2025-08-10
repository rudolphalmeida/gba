use crate::cpu::registers::{CpuState, RegisterFile, PC_IDX};
use crate::cpu::Arm7Cpu;
use crate::system_bus::{SystemBus, ACCESS_CODE, ACCESS_NONSEQ, ACCESS_SEQ};

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
        sub_opcode: DataProcessingOpcode,
        rd: usize,
        rn: usize,
        operand: u32,
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
        let nn = opcode as u8 & 0xFF;
        // Shifted in jumps of 2 so 7 instead of 8 to keep LSB 0
        let shift = ((opcode & 0xF00) >> 8) as u8;

        let operand = nn.rotate_right(shift);

        return Some(ArmOpcode::DataProcessing {
            sub_opcode,
            operand,
            rd,
            rn,
        });
    } else {
        // Register
        todo!()
    }

    None
}

pub fn execute_data_processing<BusType: SystemBus>(
    cpu: &mut Arm7Cpu,
    bus: &mut BusType,
    sub_opcode: DataProcessingOpcode,
    rd: usize,
    rn: usize,
    operand: u32,
) {
    let operation = match sub_opcode {
        DataProcessingOpcode::AND => execute_and,
        DataProcessingOpcode::EOR => execute_and,
        DataProcessingOpcode::SUB => execute_and,
        DataProcessingOpcode::RSB => execute_and,
        DataProcessingOpcode::ADD => execute_and,
        DataProcessingOpcode::ADC => execute_and,
        DataProcessingOpcode::SBC => execute_and,
        DataProcessingOpcode::RSC => execute_and,
        DataProcessingOpcode::TST => execute_and,
        DataProcessingOpcode::TEQ => execute_and,
        DataProcessingOpcode::CMP => execute_and,
        DataProcessingOpcode::CMN => execute_and,
        DataProcessingOpcode::ORR => execute_and,
        DataProcessingOpcode::MOV => execute_and,
        DataProcessingOpcode::BIC => execute_and,
        DataProcessingOpcode::MVN => execute_and,
    };
    operation(cpu, rd, rn, operand);

    if rd == PC_IDX {
        cpu.reload_pipeline(bus);
    }
}

fn execute_and(cpu: &mut Arm7Cpu, rd: usize, rn: usize, operand: u32) {
    cpu.registers[rd] = cpu.registers[rn] & operand;
}
