use super::super::super::render_error::{Diagnostic, render_error};
use super::super::instruction::{RawBinary, Statement};
use super::{SemanticError, SemanticParser};

impl SemanticParser {
    pub fn analyze_directive(
        &self,
        statement: Statement,
        source_lines: &Vec<String>,
    ) -> Result<RawBinary, SemanticError> {
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
                                    source_line: &source_lines[directive.loc.line as usize - 1]
                                        .clone(),
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
                                source_line: &source_lines[directive.loc.line as usize - 1].clone(),
                                help: None,
                            }),
                        });
                    }
                };

                if !self.regexes.constant.is_match(&data.value) {
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
                                    self.regexes.constant.as_str()
                                )
                                .as_str(),
                            ),
                        }),
                    });
                }

                RawBinary {
                    value: data
                        .value
                        .parse()
                        .map_err(|_| SemanticError::ParseInt(data.value.clone()))?,
                    bit_count: 8,
                }
            }
            "align" => {
                if let Some(operands) = statement.operands {
                    if operands.len() != 0 {
                        return Err(SemanticError::ShapeDoesNotMatch {
                            message: render_error(Diagnostic {
                                headline: "Directive .align must have no operands".to_string(),
                                line: directive.loc.line,
                                column: directive.loc.column,
                                source_line: &source_lines[directive.loc.line as usize - 1].clone(),
                                help: None,
                            }),
                        });
                    }
                }
                RawBinary {
                    value: 0,
                    bit_count: (8 - self.location_counter % 8) % 8,
                }
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
