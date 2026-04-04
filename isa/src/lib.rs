use std::fmt::Display;

pub static REG_COUNT: u32 = 3;
pub static MEM_SIZE: u32 = 256;

#[derive(Clone, Debug, PartialEq)]
pub enum AddressType {
    Code,
    Data,
}

impl Display for AddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressType::Code => write!(f, "Code"),
            AddressType::Data => write!(f, "Data"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OperandType {
    Register,
    MemoryAddress(AddressType),
    Constant,
}

impl Display for OperandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperandType::Register => write!(f, "Register"),
            OperandType::MemoryAddress(AddressType::Code) => write!(f, "MemoryAddress (Code)"),
            OperandType::MemoryAddress(AddressType::Data) => write!(f, "MemoryAddress (Data)"),
            OperandType::Constant => write!(f, "Constant"),
        }
    }
}

#[derive(Clone)]
pub struct OperandSpec {
    pub operand_type: OperandType,
    pub bit_count: u8,
}

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
        static REG: OperandSpec = OperandSpec {
            operand_type: OperandType::Register,
            bit_count: 2,
        };
        static MEM_C: OperandSpec = OperandSpec {
            operand_type: OperandType::MemoryAddress(AddressType::Code),
            bit_count: 10,
        };
        static MEM_D: OperandSpec = OperandSpec {
            operand_type: OperandType::MemoryAddress(AddressType::Data),
            bit_count: 8,
        };
        static CONST: OperandSpec = OperandSpec {
            operand_type: OperandType::Constant,
            bit_count: 8,
        };

        Self {
            opcode_bit_count: 6,
            opttab: vec![
                Operation::new("HALT", vec![]),
                Operation::new("IN", vec![&REG]),
                Operation::new("OUT", vec![&REG]),
                Operation::new("OUT_16", vec![]),
                Operation::new("OUT_CHAR", vec![&REG]),
                Operation::new("MOVER", vec![&REG, &MEM_D]),
                Operation::new("MOVEM", vec![&REG, &MEM_D]),
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
                Operation::new("JMP", vec![&MEM_C]),
                Operation::new("JZ", vec![&MEM_C]),
                Operation::new("JNZ", vec![&MEM_C]),
                Operation::new("PUSH", vec![&REG]),
                Operation::new("POP", vec![&REG]),
                Operation::new("CALL", vec![&MEM_C]),
                Operation::new("RET", vec![]),
                // Operation::new("AND", vec![&REG, &REG, &REG]),
                // Operation::new("OR", vec![&REG, &REG, &REG]),
                // Operation::new("XOR", vec![&REG, &REG, &REG]),
                // Operation::new("NOT", vec![&REG]),
                // Operation::new("SHL", vec![&REG]),
                // Operation::new("SHR", vec![&REG]),
                // Operation::new("CMP", vec![&REG, &REG]),
                // Operation::new("CMPI", vec![&REG, &CONST]),
                // Operation::new("JG", vec![&MEM_C]),
                // Operation::new("JGE", vec![&MEM_C]),
                // Operation::new("JL", vec![&MEM_C]),
                // Operation::new("JLE", vec![&MEM_C]),
                // Operation::new("JNE", vec![&MEM_C]),
                // Operation::new("JE", vec![&MEM_C]),
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
