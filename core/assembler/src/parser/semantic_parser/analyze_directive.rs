use isa::WORD_SIZE;

use super::super::super::render_error::{Diagnostic, render_error};
use super::super::instruction::{RawBinary, Statement, OperandType};
use super::{SemanticError, SemanticParser};

impl SemanticParser {
    pub fn analyze_directive(
        &mut self,
        statement: Statement,
        source_lines: &Vec<String>,
    ) -> Result<Vec<RawBinary>, SemanticError> {
        let directive = statement.directive.unwrap();
        let data = match directive.value.as_str() {
            "byte" => {
                let data = match statement.operands {
                    Some(operands) => {
                        if operands.len() != 1 {
                            return Err(SemanticError::ShapeDoesNotMatch {
                                message: render_error(Diagnostic {
                                    headline: "Directive .byte must have exactly one operand"
                                        .to_string(),
                                    line: directive.loc.line,
                                    column: directive.loc.column,
                                    source_line: &source_lines[directive.loc.line as usize - 1],
                                    help: None,
                                }),
                            });
                        }
                        &operands[0].clone()
                    }
                    None => {
                        return Err(SemanticError::ShapeDoesNotMatch {
                            message: render_error(Diagnostic {
                                headline: "Directive .byte must have exactly one operand"
                                    .to_string(),
                                line: directive.loc.line,
                                column: directive.loc.column,
                                source_line: &source_lines[directive.loc.line as usize - 1],
                                help: None,
                            }),
                        });
                    }
                };

                let value = data
                    .value
                    .parse()
                    .map_err(|_| SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!(
                                "Token '{}' does not look like a constant",
                                data.value
                            ),
                            line: data.loc.line,
                            column: data.loc.column,
                            source_line: &source_lines[data.loc.line as usize - 1],
                            help: Some(
                                format!("Constant operand must be parseable as an integer")
                                    .as_str(),
                            ),
                        }),
                    })?;

                if value >= 1 << WORD_SIZE {
                    return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!("Token '{}' is too large", data.value),
                            line: data.loc.line,
                            column: data.loc.column,
                            source_line: &source_lines[data.loc.line as usize - 1],
                            help: Some(
                                format!("Constant operand must be less than {}", 1 << WORD_SIZE)
                                    .as_str(),
                            ),
                        }),
                    });
                };

                vec![RawBinary {
                    value: value,
                    bit_count: 8,
                }]
            }
            "ascii" => {
                let data = match statement.operands {
                    Some(operands) => {
                        if operands.len() != 1 {
                            return Err(SemanticError::ShapeDoesNotMatch {
                                message: render_error(Diagnostic {
                                    headline: "Directive .ascii must have exactly one operand"
                                        .to_string(),
                                    line: directive.loc.line,
                                    column: directive.loc.column,
                                    source_line: &source_lines[directive.loc.line as usize - 1],
                                    help: None,
                                }),
                            });
                        }
                        &operands[0].clone()
                    }
                    None => {
                        return Err(SemanticError::ShapeDoesNotMatch {
                            message: render_error(Diagnostic {
                                headline: "Directive .ascii must have exactly one operand"
                                    .to_string(),
                                line: directive.loc.line,
                                column: directive.loc.column,
                                source_line: &source_lines[directive.loc.line as usize - 1],
                                help: None,
                            }),
                        });
                    }
                };

                if data.operand_type != Some(OperandType::String) {
                    return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: "Directive .ascii must have a string operand".to_string(),
                            line: directive.loc.line,
                            column: directive.loc.column,
                            source_line: &source_lines[directive.loc.line as usize - 1],
                            help: None,
                        }),
                    });
                }

                let data: Vec<RawBinary> = data.value.chars().map(|char| {
                    RawBinary {
                        value: char as u32,
                        bit_count: 8,
                    }
                }).collect();

                data
            }
            "align" => {
                if let Some(operands) = statement.operands {
                    if operands.len() != 0 {
                        return Err(SemanticError::ShapeDoesNotMatch {
                            message: render_error(Diagnostic {
                                headline: "Directive .align must have no operands".to_string(),
                                line: directive.loc.line,
                                column: directive.loc.column,
                                source_line: &source_lines[directive.loc.line as usize - 1],
                                help: None,
                            }),
                        });
                    }
                }
                vec![RawBinary {
                    value: 0,
                    bit_count: (8 - self.location_counter % 8) % 8,
                }]
            }
            _ => {
                return Err(SemanticError::UnknownDirective {
                    message: render_error(Diagnostic {
                        headline: format!("Unknown directive '{}'", directive.value),
                        line: directive.loc.line,
                        column: directive.loc.column,
                        source_line: &source_lines[directive.loc.line as usize - 1],
                        help: None,
                    }),
                });
            }
        };
        Ok(data)
    }
}
