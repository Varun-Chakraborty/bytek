use isa::{CONSTANT, OperandSpec, OperandType, OptSpec};
use std::collections::HashMap;

use super::{
    super::{
        lexer::token::SourceLoc,
        render_error::{Diagnostic, render_error},
    },
    instruction::{Instruction, InstructionField, SemanticNode, Statement, StatementField},
};

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
}

struct TiiEntry {
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
}

impl SemanticParser {
    pub fn new() -> Self {
        Self {
            optspec: OptSpec::clone(),
            symtab: HashMap::new(),
            tii: HashMap::new(),
            location_counter: 0,
            statement_counter: 0,
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

    pub fn parse_operand(
        &mut self,
        token: StatementField,
        spec: &OperandSpec,
        operand_number: usize,
        source_lines: &Vec<String>,
    ) -> Result<InstructionField, SemanticError> {
        match spec.operand_def.operand_type {
            OperandType::Register => {
                if !spec.operand_def.operand_regex.is_match(&token.value) {
                    return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!(
                                "Token '{}' does not look like a register",
                                token.value
                            ),
                            line: token.loc.line,
                            column: token.loc.column,
                            source_line: &source_lines[token.loc.line as usize as usize - 1],
                            help: Some(
                                format!(
                                    "Register operand must match the regex: {}",
                                    spec.operand_def.operand_regex.as_str()
                                )
                                .as_str(),
                            ),
                        }),
                    });
                }
                let value = token.value[1..]
                    .parse()
                    .map_err(|_| SemanticError::ParseInt(token.to_string()))?;
                let bit_count = spec.bit_count;
                Ok(InstructionField { value, bit_count })
            }
            OperandType::Constant => {
                if !spec.operand_def.operand_regex.is_match(&token.value) {
                    return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!(
                                "Token '{}' does not look like a constant",
                                token.value
                            ),
                            line: token.loc.line,
                            column: token.loc.column,
                            source_line: &source_lines[token.loc.line as usize - 1],
                            help: Some(
                                format!(
                                    "Constant operand must match the regex: {}",
                                    spec.operand_def.operand_regex.as_str()
                                )
                                .as_str(),
                            ),
                        }),
                    });
                }
                let value = token
                    .value
                    .parse::<i8>()
                    .map_err(|_| SemanticError::NotI8(token.value))?
                    as u8 as u32;
                let bit_count = spec.bit_count;
                Ok(InstructionField { value, bit_count })
            }
            OperandType::MemoryAddress => {
                if !spec.operand_def.operand_regex.is_match(&token.value) {
                    return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!(
                                "Token '{}' does not look like a memory address",
                                token.value
                            ),
                            line: token.loc.line,
                            column: token.loc.column,
                            source_line: &source_lines[token.loc.line as usize - 1],
                            help: Some(
                                format!(
                                    "Memory operand must match the regex: {}",
                                    spec.operand_def.operand_regex.as_str()
                                )
                                .as_str(),
                            ),
                        }),
                    });
                }
                let value = token
                    .value
                    .parse::<u32>()
                    .map_err(|_| SemanticError::ParseInt(token.value))?;
                let bit_count = spec.bit_count;
                Ok(InstructionField { value, bit_count })
            }
            OperandType::Label => {
                if !spec.operand_def.operand_regex.is_match(&token.value) {
                    return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!("Token '{}' does not look like a label", token.value),
                            line: token.loc.line,
                            column: token.loc.column,
                            source_line: &source_lines[token.loc.line as usize - 1].clone(),
                            help: Some(
                                format!("Label must match the regex: {}", spec.operand_def.operand_regex.as_str())
                                    .as_str(),
                            ),
                        }),
                    });
                }
                if let Some(location) = self.symtab.get(&token.value) {
                    Ok(InstructionField {
                        value: *location,
                        bit_count: spec.bit_count,
                    })
                } else {
                    self.tii.entry(token.value).or_default().push(TiiEntry {
                        at: token.loc,
                        statement_number: self.statement_counter,
                        operand_number,
                    });
                    Ok(InstructionField {
                        value: 0,
                        bit_count: spec.bit_count,
                    })
                }
            }
        }
    }

    pub fn analyze_statement(
        &mut self,
        statement: Statement,
        source_lines: &Vec<String>,
    ) -> Result<SemanticNode, SemanticError> {
        let identifier = statement.identifier.unwrap();
        let node = match identifier.value.as_str() {
            "DB" => {
                let data = match statement.operands {
                    Some(operands) => {
                        if operands.len() != 1 {
                            return Err(SemanticError::ShapeDoesNotMatch {
                                message: render_error(Diagnostic {
                                    headline: "DB statement must have one operand".to_string(),
                                    line: identifier.loc.line,
                                    column: identifier.loc.column,
                                    source_line: &source_lines[identifier.loc.line as usize - 1].clone(),
                                    help: None,
                                }),
                            });
                        }
                        &operands[0].clone()
                    },
                    None => return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: "DB statement must have one operand".to_string(),
                            line: identifier.loc.line,
                            column: identifier.loc.column,
                            source_line: &source_lines[identifier.loc.line as usize - 1].clone(),
                            help: None,
                        }),
                    }),
                };

                if !CONSTANT.operand_regex.is_match(&data.value) {
                    return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!(
                                "Token '{}' does not look like a constant",
                                data.value
                            ),
                            line: data.loc.line,
                            column: data.loc.column,
                            source_line: &source_lines[data.loc.line as usize - 1],
                            help: Some(
                                format!(
                                    "Constant operand must match the regex: {}",
                                    CONSTANT.operand_regex.as_str()
                                )
                                .as_str(),
                            ),
                        }),
                    });
                }
                
                SemanticNode::Data(data.value.parse().map_err(|_| SemanticError::ParseInt(data.value.clone()))?)
            },
            _ => {
                let (operation, opcode) = match self
                    .optspec
                    .get_by_operation_name(&identifier.value.as_str())
                {
                    Some(operation) => operation,
                    None => {
                        return Err(SemanticError::UnknownOperation {
                            message: render_error(Diagnostic {
                                headline: format!("Unknown opcode '{}'", identifier.value),
                                line: identifier.loc.line,
                                column: identifier.loc.column,
                                source_line: &source_lines[identifier.loc.line as usize - 1].clone(),
                                help: None,
                            }),
                        });
                    }
                };
        
                let expected_operands = operation.operands.clone();
        
                let operands = if let Some(operands) = statement.operands {
                    if operands.len() < expected_operands.len() {
                        return Err(SemanticError::ShapeDoesNotMatch {
                            message: render_error(Diagnostic {
                                headline: "Too few operands".to_string(),
                                line: identifier.loc.line,
                                source_line: &source_lines[identifier.loc.line as usize - 1].clone(),
                                column: identifier.loc.column,
                                help: Some(
                                    format!(
                                        "Operation {} requires {} operands",
                                        identifier.value,
                                        expected_operands.len()
                                    )
                                    .as_str(),
                                ),
                            }),
                        });
                    } else if operands.len() > expected_operands.len() {
                        return Err(SemanticError::ShapeDoesNotMatch {
                            message: render_error(Diagnostic {
                                headline: "Too many operands".to_string(),
                                line: identifier.loc.line,
                                source_line: &source_lines[identifier.loc.line as usize - 1].clone(),
                                column: identifier.loc.column,
                                help: Some(
                                    format!(
                                        "Operation {} requires {} operands",
                                        identifier.value,
                                        expected_operands.len()
                                    )
                                    .as_str(),
                                ),
                            }),
                        });
                    } else {
                        operands
                    }
                } else {
                    if expected_operands.len() != 0 {
                        return Err(SemanticError::ShapeDoesNotMatch {
                            message: render_error(Diagnostic {
                                headline: "Missing operands".to_string(),
                                line: identifier.loc.line,
                                source_line: &source_lines[identifier.loc.line as usize - 1].clone(),
                                column: identifier.loc.column,
                                help: Some(
                                    format!(
                                        "Operation {} requires {} operands",
                                        identifier.value,
                                        expected_operands.len()
                                    )
                                    .as_str(),
                                ),
                            }),
                        });
                    } else {
                        vec![]
                    }
                };
        
                let operands: Result<Vec<InstructionField>, SemanticError> = expected_operands
                    .iter()
                    .zip(operands.iter())
                    .enumerate()
                    .map(|(i, (spec, token))| {
                        self.parse_operand(token.clone(), spec, i, source_lines)
                    })
                    .collect();
        
                let operands = operands?;
        
                let size = (self.optspec.opcode_bit_count
                    + operands
                        .iter()
                        .fold(0, |acc, operand| acc + operand.bit_count)) as u32;
        
                self.statement_counter += 1;

                SemanticNode::Instruction(Instruction {
                    opcode: InstructionField {
                        value: opcode as u32,
                        bit_count: self.optspec.opcode_bit_count,
                    },
                    operands: Some(operands),
                    size,
                })
            }
        };

        Ok(node)

    }

    pub fn parse(
        &mut self,
        statements: Vec<Statement>,
        source_lines: &Vec<String>,
    ) -> Result<Vec<SemanticNode>, SemanticError> {
        let statements = self.normalize(statements)?;
        let mut semantic_nodes = Vec::<SemanticNode>::new();
        for statement in statements {
            if let Some(label) = &statement.label {
                match self.symtab.contains_key(label.value.as_str()) {
                    true => {
                        return Err(SemanticError::LabelAlreadyInUse(label.to_string()));
                    }
                    false => {
                        self.symtab
                            .insert(label.value.clone(), self.location_counter);

                        // patch
                        if let Some(tii_entries) = self.tii.get(label.value.as_str()) {
                            for entry in tii_entries {
                                // irrespective of statement type we need to patch
                                if let SemanticNode::Instruction(instr) =
                                    &mut semantic_nodes[entry.statement_number]
                                {
                                    instr.operands.as_mut().unwrap()[entry.operand_number].value =
                                        self.location_counter;
                                }
                            }

                            // remove the entry from the tii
                            self.tii.remove(label.value.as_str());
                        };
                    }
                };
            }
            if statement.identifier.is_some() {
                let semantic_node = self.analyze_statement(statement, source_lines)?;
                self.location_counter += match &semantic_node {
                    SemanticNode::Instruction(instruction) => instruction.size,
                    SemanticNode::Data(_) => 8,
                };
                semantic_nodes.push(semantic_node);
            }
        }
        if !self.tii.is_empty() {
            let mut message = String::new();
            for (key, values) in &self.tii {
                for value in values {
                    message.push_str(
                        render_error(Diagnostic {
                            headline: format!("Undefined label '{}'", key),
                            line: value.at.line,
                            source_line: &source_lines[value.at.line as usize - 1].clone(),
                            column: value.at.column,
                            help: None,
                        })
                        .as_str(),
                    );
                }
            }
            return Err(SemanticError::UndefinedLabel { message: message });
        }
        Ok(semantic_nodes)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::semantic_parser::SemanticNode;

    use super::super::{
        super::lexer::token::SourceLoc,
        instruction::{InstructionField, Statement, StatementField},
        semantic_parser::SemanticParser,
    };

    #[test]
    fn test_semantic_parser() {
        let statements = vec![
            Statement {
                label: Some(StatementField {
                    value: "MOVE".to_string(),
                    loc: SourceLoc { line: 1, column: 1 },
                }),
                identifier: None,
                operands: None,
            },
            Statement {
                label: None,
                identifier: Some(StatementField {
                    value: "MOVER".to_string(),
                    loc: SourceLoc { line: 2, column: 1 },
                }),
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
            SemanticNode::Data(_) => panic!("Expected Instruction but got Data"),
        }
    }
}
