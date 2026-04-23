use crate::parser::instruction::OperandType;
use crate::parser::semantic_parser::AddressType;

use super::super::super::render_error::{Diagnostic, render_error};
use super::super::instruction::{InstructionField, StatementField};
use super::{SemanticError, SemanticParser, TiiEntry};
use isa::{AddressingMode, OperandSpec};

impl SemanticParser {
    pub fn parse_operand(
        &mut self,
        token: StatementField,
        spec: &OperandSpec,
        operand_number: usize,
        source_lines: &Vec<String>,
    ) -> Result<InstructionField, SemanticError> {
        match &token.operand_type {
            Some(operand_type) => {
                match operand_type {
                    OperandType::Unknown => {
                        // AddressingMode::Register
                        // AddressingMode::DirectCode
                        // AddressingMode::DirectData

                        if spec.allowed_modes.contains(&AddressingMode::Register) {
                            if !self.regexes.register.is_match(&token.value) {
                                return Err(SemanticError::ShapeDoesNotMatch {
                                    message: render_error(Diagnostic {
                                        headline: format!(
                                            "Token '{}' does not look like a register",
                                            token.value
                                        ),
                                        line: token.loc.line,
                                        column: token.loc.column,
                                        source_line: &source_lines
                                            [token.loc.line as usize as usize - 1],
                                        help: Some(
                                            format!(
                                                "Register operand must match the regex: {}",
                                                self.regexes.register.as_str()
                                            )
                                            .as_str(),
                                        ),
                                    }),
                                });
                            }
                            let value = token.value[1..]
                                .parse()
                                .map_err(|_| SemanticError::ParseInt(token.to_string()))?;
                            let bit_count = AddressingMode::Register.bit_count();
                            Ok(InstructionField {
                                value,
                                bit_count,
                                addressing_mode: Some(AddressingMode::Register),
                            })
                        } else if spec.allowed_modes.contains(&AddressingMode::DirectData) {
                            let bit_count = AddressingMode::DirectData.bit_count();
                            if let Some(location) = self.symtab.get(&token.value) {
                                // handling labels
                                if self.location_counter % 8 != 0 {
                                    return Err(SemanticError::InvalidLabel {
                                    message: render_error(Diagnostic {
                                        headline: format!(
                                            "'{}' does not resolve to a byte aligned address",
                                            token.value
                                        ),
                                        line: token.loc.line,
                                        column: token.loc.column,
                                        source_line: &source_lines[token.loc.line as usize - 1],
                                        help: Some(
                                            format!("Data addresses must be byte-aligned (multiples of 8 bits), but this resolves to bit offset {}", location)
                                            .as_str(),
                                        ),
                                    }),
                                });
                                }
                                return Ok(InstructionField {
                                    value: location / 8,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::DirectData),
                                });
                            } else if self.regexes.label.is_match(&token.value) {
                                self.tii.entry(token.value).or_default().push(TiiEntry {
                                    at: token.loc,
                                    statement_number: self.statement_counter,
                                    operand_number,
                                    address_type: AddressType::Data,
                                });
                                Ok(InstructionField {
                                    value: 0,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::DirectData),
                                })
                            } else {
                                let value = token
                                    .value
                                    .parse::<u32>()
                                    .map_err(|_| SemanticError::ParseInt(token.value))?;
                                if value >= 1 << bit_count {
                                    return Err(SemanticError::ValueTooLarge {
                                        message: render_error(Diagnostic {
                                            headline: format!("Value '{}' is too large", value),
                                            line: token.loc.line,
                                            column: token.loc.column,
                                            source_line: &source_lines[token.loc.line as usize - 1],
                                            help: Some(
                                                format!("Value must be less than 2^{}", bit_count)
                                                    .as_str(),
                                            ),
                                        }),
                                    });
                                }
                                Ok(InstructionField {
                                    value,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::DirectData),
                                })
                            }
                        } else if spec.allowed_modes.contains(&AddressingMode::DirectCode) {
                            let bit_count = AddressingMode::DirectCode.bit_count();
                            if let Some(location) = self.symtab.get(&token.value) {
                                // handling labels
                                Ok(InstructionField {
                                    value: *location,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::DirectCode),
                                })
                            } else if self.regexes.label.is_match(&token.value) {
                                self.tii.entry(token.value).or_default().push(TiiEntry {
                                    at: token.loc,
                                    statement_number: self.statement_counter,
                                    operand_number,
                                    address_type: AddressType::Code,
                                });
                                Ok(InstructionField {
                                    value: 0,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::DirectCode),
                                })
                            } else {
                                let value = token
                                    .value
                                    .parse::<u32>()
                                    .map_err(|_| SemanticError::ParseInt(token.value))?;
                                if value >= 1 << bit_count {
                                    return Err(SemanticError::ValueTooLarge {
                                        message: render_error(Diagnostic {
                                            headline: format!("Value '{}' is too large", value),
                                            line: token.loc.line,
                                            column: token.loc.column,
                                            source_line: &source_lines[token.loc.line as usize - 1],
                                            help: Some(
                                                format!("Value must be less than 2^{}", bit_count)
                                                    .as_str(),
                                            ),
                                        }),
                                    });
                                }
                                Ok(InstructionField {
                                    value,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::DirectCode),
                                })
                            }
                        } else {
                            return Err(SemanticError::ShapeDoesNotMatch {
                                message: render_error(Diagnostic {
                                    headline: format!(
                                        "Token '{}' does not look like a valid operand",
                                        token.value
                                    ),
                                    line: token.loc.line,
                                    column: token.loc.column,
                                    source_line: &source_lines[token.loc.line as usize - 1],
                                    help: Some(
                                        format!(
                                            "Refer to the instruction documentation for more info"
                                        )
                                        .as_str(),
                                    ),
                                }),
                            });
                        }
                    }
                    OperandType::Immediate => {
                        if !spec.allowed_modes.contains(&AddressingMode::Immediate) {
                            return Err(SemanticError::ShapeDoesNotMatch {
                                message: render_error(Diagnostic {
                                    headline: format!(
                                        "This operand is not supposed to be an immediate value"
                                    ),
                                    line: token.loc.line,
                                    column: token.loc.column,
                                    source_line: &source_lines[token.loc.line as usize - 1],
                                    help: Some(
                                        format!(
                                            "Refer to the instruction documentation for more info"
                                        )
                                        .as_str(),
                                    ),
                                }),
                            });
                        }
                        if self.regexes.label.is_match(&token.value) {
                            if let Some(location) = self.symtab.get(&token.value) {
                                if location % 8 != 0 {
                                    return Err(SemanticError::ShapeDoesNotMatch {
                                        message: render_error(Diagnostic {
                                            headline: format!(
                                                "Label '{}' is not aligned to a byte boundary",
                                                token.value
                                            ),
                                            line: token.loc.line,
                                            column: token.loc.column,
                                            source_line: &source_lines[token.loc.line as usize - 1],
                                            help: Some(
                                                format!(
                                                    "Refer to the instruction documentation for more info"
                                                )
                                                .as_str(),
                                            ),
                                        }),
                                    });
                                };
                                return Ok(InstructionField {
                                    value: *location / 8,
                                    bit_count: AddressingMode::Immediate.bit_count(),
                                    addressing_mode: Some(AddressingMode::Immediate),
                                });
                            } else {
                                self.tii.entry(token.value).or_default().push(TiiEntry {
                                    at: token.loc,
                                    statement_number: self.statement_counter,
                                    operand_number,
                                    address_type: AddressType::Data,
                                });
                                Ok(InstructionField {
                                    value: 0,
                                    bit_count: AddressingMode::Immediate.bit_count(),
                                    addressing_mode: Some(AddressingMode::Immediate),
                                })
                            }
                        } else {
                            let value = token.value.parse().map_err(|_| {
                                SemanticError::ShapeDoesNotMatch {
                                    message: render_error(Diagnostic {
                                        headline: format!(
                                            "Token '{}' does not look like a valid immediate value",
                                            token.value
                                        ),
                                        line: token.loc.line,
                                        column: token.loc.column,
                                        source_line: &source_lines[token.loc.line as usize - 1],
                                        help: Some(
                                            format!(
                                                "Constant operand must be parseable as an integer"
                                            )
                                            .as_str(),
                                        ),
                                    }),
                                }
                            })?;

                            let bit_count = AddressingMode::Immediate.bit_count();
                            if value >= 1 << bit_count {
                                return Err(SemanticError::ShapeDoesNotMatch {
                                    message: render_error(Diagnostic {
                                        headline: format!(
                                            "Value '{}' does not fit into {} bits",
                                            token.value, bit_count
                                        ),
                                        line: token.loc.line,
                                        column: token.loc.column,
                                        source_line: &source_lines[token.loc.line as usize - 1],
                                        help: None,
                                    }),
                                });
                            }
                            Ok(InstructionField {
                                value,
                                bit_count,
                                addressing_mode: Some(AddressingMode::Immediate),
                            })
                        }
                    }
                    OperandType::IndirectMemory => {
                        // AddressingMode::Indirect
                        // AddressingMode::IndirectRegister
                        if spec
                            .allowed_modes
                            .contains(&AddressingMode::IndirectRegister)
                            && self.regexes.register.is_match(&token.value)
                        {
                            let bit_count = AddressingMode::IndirectRegister.bit_count();

                            let value = token.value[1..]
                                .parse()
                                .map_err(|_| SemanticError::ParseInt(token.to_string()))?;

                            Ok(InstructionField {
                                value,
                                bit_count,
                                addressing_mode: Some(AddressingMode::IndirectRegister),
                            })
                        } else if spec.allowed_modes.contains(&AddressingMode::Indirect) {
                            let bit_count = AddressingMode::Indirect.bit_count();
                            if let Some(location) = self.symtab.get(&token.value) {
                                // handling labels
                                if self.location_counter % 8 != 0 {
                                    return Err(SemanticError::InvalidLabel {
                                            message: render_error(Diagnostic {
                                                headline: format!(
                                                    "'{}' does not resolve to a byte aligned address",
                                                    token.value
                                                ),
                                                line: token.loc.line,
                                                column: token.loc.column,
                                                source_line: &source_lines[token.loc.line as usize - 1],
                                                help: Some(
                                                    format!("Data addresses must be byte-aligned (multiples of 8 bits), but this resolves to bit offset {}", location)
                                                    .as_str(),
                                                ),
                                            })
                                        });
                                }
                                return Ok(InstructionField {
                                    value: location / 8,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::Indirect),
                                });
                            } else if self.regexes.label.is_match(&token.value) {
                                self.tii.entry(token.value).or_default().push(TiiEntry {
                                    at: token.loc,
                                    statement_number: self.statement_counter,
                                    operand_number,
                                    address_type: AddressType::Data,
                                });
                                Ok(InstructionField {
                                    value: 0,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::Indirect),
                                })
                            } else {
                                let value = token
                                    .value
                                    .parse::<u32>()
                                    .map_err(|_| SemanticError::ParseInt(token.value))?;
                                if value >= 1 << bit_count {
                                    return Err(SemanticError::ValueTooLarge {
                                        message: render_error(Diagnostic {
                                            headline: format!("Value '{}' is too large", value),
                                            line: token.loc.line,
                                            column: token.loc.column,
                                            source_line: &source_lines[token.loc.line as usize - 1],
                                            help: Some(
                                                format!("Value must be less than 2^{}", bit_count)
                                                    .as_str(),
                                            ),
                                        }),
                                    });
                                }
                                let bit_count = bit_count;
                                Ok(InstructionField {
                                    value,
                                    bit_count,
                                    addressing_mode: Some(AddressingMode::Indirect),
                                })
                            }
                        } else {
                            return Err(SemanticError::ShapeDoesNotMatch {
                                message: render_error(Diagnostic {
                                    headline: format!(
                                        "This operand is not supposed to be an indirect value"
                                    ),
                                    line: token.loc.line,
                                    column: token.loc.column,
                                    source_line: &source_lines[token.loc.line as usize - 1],
                                    help: Some(
                                        format!(
                                            "Refer to the instruction documentation for more info"
                                        )
                                        .as_str(),
                                    ),
                                }),
                            });
                        }
                    }
                }
            }
            None => panic!("Operand does not have a address mode"),
        }
    }
}
