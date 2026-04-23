use isa::MODE_BIT_COUNT;

use super::super::super::render_error::{Diagnostic, render_error};
use super::super::instruction::{Instruction, InstructionField, Statement};
use super::{SemanticError, SemanticParser};

impl SemanticParser {
    pub fn analyze_statement(
        &mut self,
        statement: Statement,
        source_lines: &Vec<String>,
    ) -> Result<Instruction, SemanticError> {
        let identifier = statement.identifier.unwrap();

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
                        source_line: &source_lines[identifier.loc.line as usize - 1],
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
                        source_line: &source_lines[identifier.loc.line as usize - 1],
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
                        source_line: &source_lines[identifier.loc.line as usize - 1],
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
                        source_line: &source_lines[identifier.loc.line as usize - 1],
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
            .map(|(i, (spec, token))| self.parse_operand(token.clone(), spec, i, source_lines))
            .collect();

        let operands = operands?;

        let size = self.optspec.opcode_bit_count
            + operands
                .iter()
                .fold(0, |acc, operand| acc + operand.bit_count + MODE_BIT_COUNT);

        self.statement_counter += 1;

        let instr = Instruction {
            opcode: InstructionField {
                value: opcode as u32,
                bit_count: self.optspec.opcode_bit_count,
                addressing_mode: None,
            },
            operands: Some(operands),
            size,
        };

        Ok(instr)
    }
}
