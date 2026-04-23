use isa::AddressingMode;

use super::super::lexer::token::SourceLoc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OperandType {
    Immediate,
    IndirectMemory,
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct StatementField {
    pub value: String,
    pub loc: SourceLoc,
    pub operand_type: Option<OperandType>,
}

impl std::fmt::Display for StatementField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.value, self.loc)
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub label: Option<StatementField>,
    pub identifier: Option<StatementField>,
    pub directive: Option<StatementField>,
    pub operands: Option<Vec<StatementField>>,
}

impl Statement {
    pub fn new() -> Self {
        Self {
            label: None,
            identifier: None,
            directive: None,
            operands: None,
        }
    }

    pub fn set_label(&mut self, value: String, loc: SourceLoc) {
        self.label = Some(StatementField {
            value,
            loc,
            operand_type: None,
        });
    }

    pub fn set_identifier(&mut self, value: String, loc: SourceLoc) {
        self.identifier = Some(StatementField {
            value,
            loc,
            operand_type: None,
        });
    }

    pub fn set_directive(&mut self, value: String, loc: SourceLoc) {
        self.directive = Some(StatementField {
            value,
            loc,
            operand_type: None,
        });
    }

    pub fn add_operand(
        &mut self,
        value: String,
        loc: SourceLoc,
        operand_type: Option<OperandType>,
    ) {
        if let Some(operands) = &mut self.operands {
            operands.push(StatementField {
                value,
                loc,
                operand_type: operand_type,
            });
        } else {
            self.operands = Some(vec![StatementField {
                value,
                loc,
                operand_type: operand_type,
            }]);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InstructionField {
    pub value: u32,
    pub bit_count: u32,
    pub addressing_mode: Option<AddressingMode>,
}

#[derive(Debug, Clone, Default)]
pub struct Instruction {
    pub opcode: InstructionField,
    pub operands: Option<Vec<InstructionField>>,
    pub size: u32,
}

#[derive(Debug, Clone, Default)]
pub struct RawBinary {
    pub value: u32,
    pub bit_count: u32,
}

#[derive(Debug, Clone)]
pub enum SemanticNode {
    Instruction(Instruction),
    RawBinary(RawBinary),
}
