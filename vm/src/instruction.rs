use crate::memory::{Memory, MemoryError};

#[derive(Debug, thiserror::Error)]
pub enum InstructionError {
    #[error("{0}")]
    MemoryError(#[from] MemoryError),
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u32),
}

#[derive(Debug)]
pub struct Instruction {
    opcode: u32,
    operation_name: String,
    operands: Vec<u32>,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Opcode: {}, Operation_name: {}, Operands: {:?}", self.opcode, self.operation_name, self.operands)
    }
}

fn get_bits(memory: &Memory<u8>, mut start: u32, bits_count: u32) -> Result<u32, InstructionError> {
    let mut value: u32 = 0;
    for _ in 0..bits_count {
        let byte = memory.get(start / 8)?;
        let bit = (byte >> (7 - start % 8)) & 1;
        value = (value << 1) | bit as u32;
        start += 1;
    }
    Ok(value)
}

impl Instruction {
    pub fn new(
        memory: &Memory<u8>,
        pc: &mut u32,
        optspec: &isa::OptSpec,
    ) -> Result<Self, InstructionError> {
        let opcode = get_bits(memory, *pc, optspec.opcode_bit_count as u32)?;
        *pc += optspec.opcode_bit_count as u32;

        let operation = match optspec.get_by_opcode(&opcode) {
            Some(operation) => operation,
            None => return Err(InstructionError::InvalidOpcode(opcode)),
        };

        let operands = operation.operands.iter().fold(
            Ok(Vec::new()),
            |acc: Result<Vec<u32>, InstructionError>, operand_spec| {
                let mut acc = acc?;
                let operand = get_bits(memory, *pc, operand_spec.bit_count as u32)?;
                *pc += operand_spec.bit_count as u32;
                acc.push(operand);
                Ok(acc)
            },
        )?;

        Ok(Self {
            opcode,
            operands,
            operation_name: operation.operation_name.clone(),
        })
    }

    pub fn get_operands(&self) -> &Vec<u32> {
        return &self.operands;
    }

    pub fn get_operation_name(&self) -> &String {
        return &self.operation_name;
    }
}
