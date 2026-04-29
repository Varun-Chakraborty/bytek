pub static REG_COUNT: u32 = 5;
pub static MEM_BYTES: u32 = 256;
pub static MEM_BITS: u32 = MEM_BYTES * 8;
pub static WORD_SIZE: u32 = 8; // bits
pub static MODE_BIT_COUNT: u32 = 3;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy, Default)]
pub enum AddressingMode {
    #[default]
    Register, // R0
    DirectCode,       // 0
    DirectData,       // 0
    Indirect,         // [0]
    IndirectRegister, // [R0]
    Immediate,        // #0
}

impl std::fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::Register => write!(f, "Register"),
            AddressingMode::DirectCode => write!(f, "DirectCode"),
            AddressingMode::DirectData => write!(f, "DirectData"),
            AddressingMode::Indirect => write!(f, "Indirect"),
            AddressingMode::IndirectRegister => write!(f, "IndirectRegister"),
            AddressingMode::Immediate => write!(f, "Immediate"),
        }
    }
}

impl AddressingMode {
    pub fn bit_count(&self) -> u32 {
        match self {
            AddressingMode::Register => 32 - (REG_COUNT - 1).leading_zeros(),
            AddressingMode::DirectCode => 32 - (MEM_BITS - 1).leading_zeros(),
            AddressingMode::DirectData => 32 - (MEM_BYTES - 1).leading_zeros(),
            AddressingMode::Indirect => 32 - (MEM_BYTES - 1).leading_zeros(),
            AddressingMode::IndirectRegister => 32 - (REG_COUNT - 1).leading_zeros(),
            AddressingMode::Immediate => WORD_SIZE,
        }
    }

    pub fn from_bits(bits: u32) -> Self {
        match bits {
            0 => AddressingMode::Register,
            1 => AddressingMode::DirectCode,
            2 => AddressingMode::DirectData,
            3 => AddressingMode::Indirect,
            4 => AddressingMode::IndirectRegister,
            5 => AddressingMode::Immediate,
            _ => panic!("Invalid addressing mode"),
        }
    }
}

#[derive(Clone)]
pub struct OperandSpec {
    pub allowed_modes: &'static [AddressingMode],
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
    pub opcode_bit_count: u32,
    opttab: Vec<Operation>,
}

impl OptSpec {
    pub fn clone() -> Self {
        static REG: OperandSpec = OperandSpec {
            allowed_modes: &[AddressingMode::Register],
        };
        static MEM_C: OperandSpec = OperandSpec {
            allowed_modes: &[AddressingMode::DirectCode],
        };
        static NON_REGISTER_VALUE: OperandSpec = OperandSpec {
            allowed_modes: &[
                AddressingMode::DirectData,
                AddressingMode::Indirect,
                AddressingMode::IndirectRegister,
                AddressingMode::Immediate,
            ],
        };

        static IMMEDIATE_OR_REGISTER: OperandSpec = OperandSpec {
            allowed_modes: &[
                AddressingMode::Register,
                AddressingMode::Immediate,
            ],
        };

        Self {
            opcode_bit_count: 6,
            opttab: vec![
                Operation::new("HALT", vec![]),
                Operation::new("IN", vec![&REG]),
                Operation::new("OUT", vec![&REG]),
                Operation::new("OUT_16", vec![]),
                Operation::new("OUT_CHAR", vec![&REG]),
                Operation::new("MOVER", vec![&REG, &NON_REGISTER_VALUE]),
                Operation::new("MOVEM", vec![&REG, &NON_REGISTER_VALUE]),
                Operation::new("ADD", vec![&REG, &REG, &IMMEDIATE_OR_REGISTER]),
                Operation::new("SUB", vec![&REG, &REG, &IMMEDIATE_OR_REGISTER]),
                Operation::new("MULT", vec![&REG, &REG, &IMMEDIATE_OR_REGISTER]),
                Operation::new("ADC", vec![&REG, &REG, &IMMEDIATE_OR_REGISTER]),
                Operation::new("SBC", vec![&REG, &REG, &IMMEDIATE_OR_REGISTER]),
                Operation::new("MULT_16", vec![&IMMEDIATE_OR_REGISTER]),
                Operation::new("JMP", vec![&MEM_C]),
                Operation::new("JZ", vec![&MEM_C]),
                Operation::new("JNZ", vec![&MEM_C]),
                Operation::new("PUSH", vec![&REG]),
                Operation::new("POP", vec![&REG]),
                Operation::new("CALL", vec![&MEM_C]),
                Operation::new("RET", vec![]),
                Operation::new("CMP", vec![&REG, &IMMEDIATE_OR_REGISTER]),
                // Operation::new("AND", vec![&REG, &REG, &REG]),
                // Operation::new("OR", vec![&REG, &REG, &REG]),
                // Operation::new("XOR", vec![&REG, &REG, &REG]),
                // Operation::new("NOT", vec![&REG]),
                // Operation::new("SHL", vec![&REG]),
                // Operation::new("SHR", vec![&REG]),
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
