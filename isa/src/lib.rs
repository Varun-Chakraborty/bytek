use std::fmt::Display;

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone, Debug, PartialEq)]
pub enum OperandType {
    Register,
    MemoryAddress,
    Label,
    Constant,
}

impl Display for OperandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperandType::Register => write!(f, "Register"),
            OperandType::MemoryAddress => write!(f, "MemoryAddress"),
            OperandType::Label => write!(f, "Label"),
            OperandType::Constant => write!(f, "Constant"),
        }
    }
}

pub struct OperandDef {
    pub operand_regex: Lazy<Regex>,
    pub operand_type: OperandType,
    
}

#[derive(Clone)]
pub struct OperandSpec {
    pub operand_def: &'static OperandDef,
    pub bit_count: u8,
}

pub static REGISTER: OperandDef = OperandDef { operand_regex: Lazy::new(|| Regex::new("^R[0-3]$").unwrap()), operand_type: OperandType::Register };
pub static MEMORY_ADDRESS: OperandDef = OperandDef { operand_regex: Lazy::new(|| Regex::new("^[0-9]+$").unwrap()), operand_type: OperandType::MemoryAddress };
pub static LABEL: OperandDef = OperandDef { operand_regex: Lazy::new(|| Regex::new("^[A-Z]+$").unwrap()), operand_type: OperandType::Label };
pub static CONSTANT: OperandDef = OperandDef { operand_regex: Lazy::new(|| Regex::new("^-?[0-9]+$").unwrap()), operand_type: OperandType::Constant };

pub struct Operation {
    pub operation_name: String,
    pub operands: Vec<&'static OperandSpec>,
}

impl Operation {
    fn new(operation_name: &str, operands: Vec<&'static OperandSpec>) -> Self {
        Self {
            operation_name: operation_name.to_string(),
            operands,
        }
    }
}

pub struct OptSpec {
    pub opcode_bit_count: u8,
    opttab: Vec<Operation>,
}

impl OptSpec {
    pub fn clone() -> Self {
        static REG: OperandSpec = OperandSpec { operand_def: &REGISTER, bit_count: 2 };
        static MEM: OperandSpec = OperandSpec { operand_def: &MEMORY_ADDRESS, bit_count: 4 };
        static LABEL_: OperandSpec = OperandSpec { operand_def: &LABEL, bit_count: 8 };
        static CONST: OperandSpec = OperandSpec { operand_def: &CONSTANT, bit_count: 8 };

        Self {
            opcode_bit_count: 6,
            opttab: vec![
                Operation::new("HALT", vec![]),
                Operation::new("IN", vec![&REG]),
                Operation::new("OUT", vec![&REG]),
                Operation::new("OUT_16", vec![]),
                Operation::new("OUT_CHAR", vec![&REG]),
                
                Operation::new("MOVER", vec![&REG, &MEM]),
                Operation::new("MOVEM", vec![&REG, &MEM]),
                Operation::new("MOVEI", vec![&REG, &CONST]),
                
                Operation::new("ADD", vec![&REG, &REG, &REG]),
                Operation::new("SUB", vec![&REG, &REG, &REG]),
                Operation::new("MULT", vec![&REG, &REG, &REG]),
                
                Operation::new("ADDI", vec![&REG, &REG, &CONST]),
                Operation::new("SUBI", vec![&REG, &REG, &CONST]),
                Operation::new("MULTI", vec![&REG, &REG, &CONST]),
                
                Operation::new("ADC", vec![&REG, &REG, &REG]),
                Operation::new("SBC", vec![&REG, &REG, &REG]),
                
                Operation::new("ADCI", vec![&REG, &REG, &CONST]),
                Operation::new("SBCI", vec![&REG, &REG, &CONST]),
                
                Operation::new("MULT_16", vec![&REG]),
                Operation::new("MULTI_16", vec![&CONST]),
                
                Operation::new("JMP", vec![&LABEL_]),
                Operation::new("JZ", vec![&LABEL_]),
                Operation::new("JNZ", vec![&LABEL_]),
                Operation::new("PUSH", vec![&REG]),
                Operation::new("POP", vec![&REG]),
                Operation::new("CALL", vec![&LABEL_]),
                Operation::new("RET", vec![]),

                // Operation::new("AND", vec![&REG, &REG, &REG]),
                // Operation::new("OR", vec![&REG, &REG, &REG]),
                // Operation::new("XOR", vec![&REG, &REG, &REG]),
                // Operation::new("NOT", vec![&REG]),
                // Operation::new("SHL", vec![&REG]),
                // Operation::new("SHR", vec![&REG]),
                // Operation::new("CMP", vec![&REG, &REG]),
                // Operation::new("CMPI", vec![&REG, &CONST]),
                // Operation::new("JG", vec![&LABEL_]),
                // Operation::new("JGE", vec![&LABEL_]),
                // Operation::new("JL", vec![&LABEL_]),
                // Operation::new("JLE", vec![&LABEL_]),
                // Operation::new("JNE", vec![&LABEL_]),
                // Operation::new("JE", vec![&LABEL_]),
            ],
        }
    }

    pub fn get_by_opcode(&self, opcode: &u32) -> Option<&Operation> {
        // opcode represent index in the array
        self.opttab.get(*opcode as usize)
    }

    pub fn get_by_operation_name(&self, operation_name: &str) -> Option<(&Operation, usize)> {
        // also include opcode in the returning struct
        self.opttab
            .iter()
            .enumerate()
            .find(|(_, operation)| operation.operation_name == operation_name)
            .map(|(opcode, operation)| (operation, opcode))
    }
}
