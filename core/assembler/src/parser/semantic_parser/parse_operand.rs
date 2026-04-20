use super::super::super::render_error::{Diagnostic, render_error};
use super::super::instruction::{InstructionField, StatementField};
use super::{SemanticError, SemanticParser, TiiEntry};
use isa::{AddressType, OperandSpec, OperandType};

impl SemanticParser {
    pub fn parse_operand(
        &mut self,
        token: StatementField,
        spec: &OperandSpec,
        operand_number: usize,
        source_lines: &Vec<String>,
    ) -> Result<InstructionField, SemanticError> {
        match &spec.operand_type {
            OperandType::Register => {
                if !self.regexes.register.is_match(&token.value) {
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
                let bit_count = spec.bit_count;
                Ok(InstructionField { value, bit_count })
            }
            OperandType::Constant => {
                let value = token
                    .value
                    .parse()
                    .map_err(|_| SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!(
                                "Token '{}' does not look like a constant",
                                token.value
                            ),
                            line: token.loc.line,
                            column: token.loc.column,
                            source_line: &source_lines[token.loc.line as usize - 1],
                            help: Some(
                                format!("Constant operand must be parseable as an integer")
                                    .as_str(),
                            ),
                        }),
                    })?;

                if value >= 1 << spec.bit_count {
                    return Err(SemanticError::ShapeDoesNotMatch {
                        message: render_error(Diagnostic {
                            headline: format!(
                                "Value '{}' does not fit into {} bits",
                                token.value, spec.bit_count
                            ),
                            line: token.loc.line,
                            column: token.loc.column,
                            source_line: &source_lines[token.loc.line as usize - 1],
                            help: None,
                        }),
                    });
                }
                let bit_count = spec.bit_count;
                Ok(InstructionField { value, bit_count })
            }
            OperandType::MemoryAddress(a) => {
                if let Some(location) = self.symtab.get(&token.value) {
                    // handling labels
                    if a == &AddressType::Data {
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
                            bit_count: spec.bit_count,
                        });
                    }
                    Ok(InstructionField {
                        value: *location,
                        bit_count: spec.bit_count,
                    })
                } else if self.regexes.label.is_match(&token.value) {
                    self.tii.entry(token.value).or_default().push(TiiEntry {
                        at: token.loc,
                        statement_number: self.statement_counter,
                        operand_number,
                    });
                    Ok(InstructionField {
                        value: 0,
                        bit_count: spec.bit_count,
                    })
                } else {
                    let value = token
                        .value
                        .parse::<u32>()
                        .map_err(|_| SemanticError::ParseInt(token.value))?;
                    if value >= 1 << spec.bit_count {
                        return Err(SemanticError::ValueTooLarge {
                            message: render_error(Diagnostic {
                                headline: format!("Value '{}' is too large", value),
                                line: token.loc.line,
                                column: token.loc.column,
                                source_line: &source_lines[token.loc.line as usize - 1],
                                help: Some(
                                    format!("Value must be less than 2^{}", spec.bit_count)
                                        .as_str(),
                                ),
                            }),
                        });
                    }
                    let bit_count = spec.bit_count;
                    Ok(InstructionField { value, bit_count })
                }
            }
        }
    }
}
