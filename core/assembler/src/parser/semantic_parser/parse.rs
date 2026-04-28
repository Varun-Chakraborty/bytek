use super::super::super::render_error::{Diagnostic, render_error};
use super::super::instruction::Statement;
use super::{AddressType, SemanticError, SemanticNode, SemanticParser};

impl SemanticParser {
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
                        if !self.regexes.label.is_match(label.value.as_str()) {
                            return Err(SemanticError::InvalidLabel {
                                message: label.to_string(),
                            });
                        }

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
                                        if entry.address_type == AddressType::Code {
                                            self.location_counter
                                        } else {
                                            self.location_counter / 8
                                        };
                                }
                            }

                            // remove the entry from the tii
                            self.tii.remove(label.value.as_str());
                        };
                    }
                };
            }
            if statement.identifier.is_some() {
                let instr = self.analyze_statement(statement, source_lines)?;
                self.location_counter += instr.size;
                self.statement_counter += 1;
                semantic_nodes.push(SemanticNode::Instruction(instr));
            } else if statement.directive.is_some() {
                let data = self.analyze_directive(statement, source_lines)?;
                for data in data {
                    self.location_counter += data.bit_count;
                    self.statement_counter += 1;
                    semantic_nodes.push(SemanticNode::RawBinary(data));
                }
            }
        }
        if !self.tii.is_empty() {
            let mut message = String::new();
            for (key, values) in &self.tii {
                for value in values {
                    message.push_str(
                        render_error(Diagnostic {
                            headline: format!(
                                "Token '{}' does not look like a memory address",
                                key
                            ),
                            line: value.at.line,
                            column: value.at.column,
                            source_line: &source_lines[value.at.line as usize - 1],
                            help: Some(
                                format!("Memory operand must match the regex: '^[0-9]+$'").as_str(),
                            ),
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
