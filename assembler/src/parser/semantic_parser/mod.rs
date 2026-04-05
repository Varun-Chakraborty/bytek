mod analyze_directive;
mod analyze_statement;
mod parse;
mod parse_operand;

use super::super::lexer::token::SourceLoc;
use super::instruction::{SemanticNode, Statement};
use isa::{OptSpec, REG_COUNT};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    #[error("{message}")]
    ShapeDoesNotMatch { message: String },
    #[error("{message}")]
    OperandCountDoesNotMatch { message: String },
    #[error("{message}")]
    UnknownOperation { message: String },
    #[error("Unable to parse the token as an integer: {0}")]
    ParseInt(String),
    #[error("Unable to parse the token as a signed 8 bit integer: {0}")]
    NotI8(String),
    #[error("Label {0} already in use")]
    LabelAlreadyInUse(String),
    #[error("{message}")]
    UndefinedLabel { message: String },
    #[error("{message}")]
    InvalidLabel { message: String },
    #[error("{message}")]
    ValueTooLarge { message: String },
    #[error("{message}")]
    UnknownDirective { message: String },
}

pub struct Regexes {
    register: Lazy<Regex>,
    label: Lazy<Regex>,
}

pub struct TiiEntry {
    at: SourceLoc,
    statement_number: usize,
    operand_number: usize,
}

pub struct SemanticParser {
    optspec: OptSpec,
    symtab: HashMap<String, u32>,
    tii: HashMap<String, Vec<TiiEntry>>,
    location_counter: u32,
    statement_counter: usize,
    regexes: Regexes,
}

impl SemanticParser {
    pub fn new() -> Self {
        Self {
            optspec: OptSpec::clone(),
            symtab: HashMap::new(),
            tii: HashMap::new(),
            location_counter: 0,
            statement_counter: 0,
            regexes: Regexes {
                register: Lazy::new(|| Regex::new(&format!(r"^R[0-{}]$", REG_COUNT - 1)).unwrap()),
                label: Lazy::new(|| Regex::new(r"^[A-Za-z][A-Za-z0-9_]*$").unwrap()),
            },
        }
    }

    pub fn normalize(&self, statements: Vec<Statement>) -> Result<Vec<Statement>, SemanticError> {
        statements
            .iter()
            .map(|statement| {
                let mut new_statement = statement.clone();
                new_statement.label = statement.label.clone();
                new_statement.identifier = statement.identifier.clone();
                new_statement.operands = if let Some(operands) = new_statement.operands {
                    match &statement.identifier {
                        Some(identifier) => match identifier.value.as_str() {
                            "ADD" | "ADDI" | "ADC" | "ADCI" | "SUB" | "SUBI" | "SBC" | "SBCI"
                            | "MULT" | "MULTI" | "AND" | "OR" | "XOR" => {
                                if operands.len() == 2 {
                                    Some(vec![
                                        operands[0].clone(),
                                        operands[0].clone(),
                                        operands[1].clone(),
                                    ])
                                } else {
                                    Some(operands.clone())
                                }
                            }
                            "NOT" => {
                                if operands.len() == 1 {
                                    Some(vec![operands[0].clone(), operands[0].clone()])
                                } else {
                                    Some(operands.clone())
                                }
                            }
                            _ => Some(operands.clone()),
                        },
                        None => Some(operands.clone()),
                    }
                } else {
                    None
                };
                Ok(new_statement)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::lexer::token::SourceLoc;
    use super::super::{
        instruction::{InstructionField, Statement, StatementField},
        semantic_parser::SemanticParser,
    };
    use super::SemanticNode;

    #[test]
    fn test_semantic_parser() {
        let statements = vec![
            Statement {
                label: Some(StatementField {
                    value: "MOVE".to_string(),
                    loc: SourceLoc { line: 1, column: 1 },
                }),
                directive: None,
                identifier: None,
                operands: None,
            },
            Statement {
                label: None,
                identifier: Some(StatementField {
                    value: "MOVER".to_string(),
                    loc: SourceLoc { line: 2, column: 1 },
                }),
                directive: None,
                operands: Some(vec![
                    StatementField {
                        value: "R0".to_string(),
                        loc: SourceLoc { line: 2, column: 7 },
                    },
                    StatementField {
                        value: "0".to_string(),
                        loc: SourceLoc {
                            line: 2,
                            column: 11,
                        },
                    },
                ]),
            },
        ];

        let mut semantic_parser = SemanticParser::new();
        let source_lines = ["", ""].map(|s| s.to_string()).to_vec();
        let semantic_nodes = semantic_parser.parse(statements, &source_lines).unwrap();

        assert_eq!(semantic_nodes.len(), 1);
        let semantic_node = &semantic_nodes[0];
        match semantic_node {
            SemanticNode::Instruction(instr) => {
                assert_eq!(
                    instr.opcode,
                    InstructionField {
                        value: 1,
                        bit_count: 6
                    }
                );
                let operands = instr.operands.as_ref().unwrap();
                assert_eq!(operands.len(), 2);
                assert_eq!(
                    operands[0],
                    InstructionField {
                        value: 0,
                        bit_count: 2
                    }
                );
                assert_eq!(
                    operands[1],
                    InstructionField {
                        value: 0,
                        bit_count: 4
                    }
                );
            }
            SemanticNode::RawBinary { .. } => panic!("Expected Instruction but got Raw Binary"),
        }
    }
}
