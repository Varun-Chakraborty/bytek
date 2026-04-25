use isa::{AddressingMode, MODE_BIT_COUNT};

use crate::memory::{Memory, MemoryError};

#[derive(Debug, thiserror::Error)]
pub enum InstructionError {
    #[error("{0}")]
    MemoryError(#[from] MemoryError),
    #[error("Invalid opcode: {0} at address: {1}")]
    InvalidOpcode(u32, u32),
    #[error("Invalid addressing mode: {0} at address: {1}")]
    InvalidAddressingMode(AddressingMode, u32),
}

#[derive(Debug)]
pub struct Operand {
    pub mode: AddressingMode,
    pub value: u32,
}

impl Operand {
    pub fn new(mode: AddressingMode) -> Self {
        Self { mode, value: 0 }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pc: u32,
    opcode: u32,
    operation_name: String,
    operands: Vec<Operand>,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PC: {}, Opcode: {}, Operation_name: {}, Operands: {:?}",
            self.pc, self.opcode, self.operation_name, self.operands
        )
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
        let start_pc = *pc;
        let opcode = get_bits(memory, *pc, optspec.opcode_bit_count as u32)?;

        let operation = match optspec.get_by_opcode(&opcode) {
            Some(operation) => operation,
            None => return Err(InstructionError::InvalidOpcode(opcode, *pc)),
        };

        *pc += optspec.opcode_bit_count as u32;

        let operands = operation.operands.iter().fold(
            Ok(Vec::new()),
            |acc: Result<Vec<Operand>, InstructionError>, operand_spec| {
                let mut acc = acc?;
                let mode = get_bits(memory, *pc, MODE_BIT_COUNT)?;
                let mode = AddressingMode::from_bits(mode);
                if !operand_spec.allowed_modes.contains(&mode) {
                    return Err(InstructionError::InvalidAddressingMode(mode, *pc));
                }
                *pc += MODE_BIT_COUNT;
                let bits_count = mode.bit_count();
                let mut operand = Operand::new(mode);
                operand.value = get_bits(memory, *pc, bits_count as u32)?;
                *pc += bits_count as u32;
                acc.push(operand);
                Ok(acc)
            },
        )?;

        Ok(Self {
            pc: start_pc,
            opcode,
            operands,
            operation_name: operation.operation_name.clone(),
        })
    }

    pub fn get_operands(&self) -> &Vec<Operand> {
        return &self.operands;
    }

    pub fn get_operation_name(&self) -> &String {
        return &self.operation_name;
    }
}
