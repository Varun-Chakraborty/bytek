use std::mem;

use super::super::{
    lexer::token::{TokenStream, TokenType},
    render_error::{Diagnostic, render_error},
};
use super::instruction::{OperandType, Statement};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SyntacticError {
    #[error("{message}")]
    UnexpectedToken { message: String },
}

#[derive(PartialEq, Debug)]
enum DFAState {
    Start,
    AfterLabel,
    ExpectDirective,
    AfterDirective,
    AfterOpcode,
    ExpectImmediate,
    ExpectIndirect,
    ExpectClosingSquareBracket,
    AfterOperand,
    ExpectOperand, // after comma
}

pub struct SyntacticParser {
    statements: Vec<Statement>,
}

impl SyntacticParser {
    pub fn new() -> Self {
        return Self { statements: vec![] };
    }

    pub fn parse(
        &mut self,
        mut tokens: TokenStream,
        source_lines: &Vec<String>,
    ) -> Result<Vec<Statement>, SyntacticError> {
        let mut statement = Statement::new();
        let mut state = DFAState::Start;
        loop {
            let current_token = tokens.seek(0).unwrap();
            let source_loc = current_token.source_loc;
            match current_token.token_type {
                TokenType::Identifier => {
                    let value = current_token.value.clone().unwrap();
                    match state {
                        DFAState::Start => {
                            // label or opcode
                            if let Some(':') = tokens.seek_as_symbol(1) {
                                statement.set_label(value, source_loc);
                                state = DFAState::AfterLabel;
                                tokens.next();
                            } else {
                                statement.set_identifier(value, source_loc);
                                state = DFAState::AfterOpcode;
                            }
                        }
                        DFAState::AfterLabel => {
                            statement.set_identifier(value, source_loc);
                            state = DFAState::AfterOpcode;
                        }
                        DFAState::ExpectOperand
                        | DFAState::AfterOpcode
                        | DFAState::AfterDirective => {
                            statement.add_operand(value, source_loc, Some(OperandType::Unknown));
                            state = DFAState::AfterOperand;
                        }
                        DFAState::ExpectDirective => {
                            statement.set_directive(value, source_loc);
                            state = DFAState::AfterDirective;
                        }
                        DFAState::ExpectImmediate => {
                            statement.add_operand(value, source_loc, Some(OperandType::Immediate));
                            state = DFAState::AfterOperand;
                        }
                        DFAState::ExpectIndirect => {
                            statement.add_operand(
                                value,
                                source_loc,
                                Some(OperandType::IndirectMemory),
                            );
                            state = DFAState::ExpectClosingSquareBracket;
                        }
                        _ => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: format!("Unexpected identifier '{}'", value),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: Some(match state {
                                        DFAState::AfterOperand => {
                                            "Perhaps you meant to use comma(,) instead?"
                                        }
                                        _ => "",
                                    }),
                                }),
                            });
                        }
                    }
                    tokens.next();
                }
                TokenType::Symbol => {
                    let value = current_token.value.clone().unwrap();
                    if state == DFAState::AfterOperand && value.as_str() == "," {
                        state = DFAState::ExpectOperand;
                    } else if (state == DFAState::Start || state == DFAState::AfterLabel)
                        && value.as_str() == "."
                    {
                        state = DFAState::ExpectDirective;
                    } else if state == DFAState::AfterOpcode || state == DFAState::ExpectOperand || state == DFAState::AfterDirective {
                        if value == "#" {
                            state = DFAState::ExpectImmediate;
                        }
                        if value == "[" {
                            state = DFAState::ExpectIndirect;
                        }
                    } else if state == DFAState::ExpectClosingSquareBracket && value == "]" {
                        state = DFAState::AfterOperand;
                    } else {
                        return Err(SyntacticError::UnexpectedToken {
                            message: render_error(Diagnostic {
                                headline: format!("Unexpected symbol '{}'", value),
                                line: source_loc.line,
                                source_line: &source_lines[source_loc.line as usize - 1],
                                column: source_loc.column,
                                help: Some(match state {
                                    DFAState::AfterLabel => {
                                        "Labels must be followed by single colon(:) and then an identifier (opcode) should follow"
                                    }
                                    DFAState::AfterOpcode => {
                                        "An identifier (operand) is expected after opcode"
                                    }
                                    DFAState::ExpectOperand => {
                                        "An identifier is expected after comma"
                                    }
                                    DFAState::AfterOperand => {
                                        "Perhaps you meant to use comma(,) instead?"
                                    }
                                    DFAState::AfterDirective => {
                                        "Directives must be followed by one or more operands"
                                    }
                                    _ => "",
                                }),
                            }),
                        });
                    }
                    tokens.next();
                }
                TokenType::Newline => {
                    match state {
                        DFAState::ExpectOperand => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "An identifier is expected after comma".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        DFAState::ExpectDirective => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "A directive name is expected after dot".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        DFAState::Start => {}
                        _ => self.statements.push(statement),
                    }
                    statement = Statement::new();
                    state = DFAState::Start;
                    tokens.next();
                }
                TokenType::Eof => {
                    match state {
                        DFAState::ExpectOperand => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "An identifier is expected after comma".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        DFAState::Start => {}
                        DFAState::ExpectDirective => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "A directive name is expected after dot".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        DFAState::ExpectClosingSquareBracket => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "A closing square bracket is expected".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        DFAState::ExpectImmediate => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "An immediate value is expected".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        DFAState::ExpectIndirect => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "An indirect value is expected".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        _ => self.statements.push(statement),
                    }
                    return Ok(mem::take(&mut self.statements));
                }
                TokenType::Whitespace => {
                    match state {
                        DFAState::ExpectDirective => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "Unexpected token".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: Some("A directive name is expected after '.'"),
                                }),
                            });
                        }
                        DFAState::ExpectImmediate => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "Unexpected token".to_string(),
                                    line: source_loc.line,
                                    source_line: &source_lines[source_loc.line as usize - 1],
                                    column: source_loc.column,
                                    help: Some("An immediate value is expected"),
                                }),
                            });
                        }
                        _ => {}
                    };
                    tokens.next();
                }
                TokenType::String => {
                    let value = current_token.value.clone().unwrap();
                    match state {
                        DFAState::AfterOpcode | DFAState::ExpectOperand | DFAState::AfterDirective => {
                            statement.add_operand(value, source_loc, Some(OperandType::String));
                        }
                        _ => return Err(SyntacticError::UnexpectedToken {
                            message: render_error(Diagnostic {
                                headline: "Unexpected token".to_string(),
                                line: source_loc.line,
                                source_line: &source_lines[source_loc.line as usize - 1],
                                column: source_loc.column,
                                help: None,
                            }),
                        })
                    };
                    tokens.next();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        super::lexer::token::{SourceLoc, Token, TokenStream, TokenType},
        instruction::StatementField,
    };
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let mut tokens = TokenStream::new();
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("MOVE".to_string()),
            source_loc: SourceLoc { line: 1, column: 1 },
        });
        tokens.push(Token {
            token_type: TokenType::Symbol,
            value: Some(":".to_string()),
            source_loc: SourceLoc { line: 1, column: 6 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("MOVER".to_string()),
            source_loc: SourceLoc { line: 1, column: 7 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("R0".to_string()),
            source_loc: SourceLoc {
                line: 1,
                column: 13,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Symbol,
            value: Some(",".to_string()),
            source_loc: SourceLoc {
                line: 1,
                column: 15,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("0".to_string()),
            source_loc: SourceLoc {
                line: 1,
                column: 17,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Newline,
            value: None,
            source_loc: SourceLoc {
                line: 1,
                column: 18,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("MOVE1".to_string()),
            source_loc: SourceLoc { line: 2, column: 1 },
        });
        tokens.push(Token {
            token_type: TokenType::Symbol,
            value: Some(":".to_string()),
            source_loc: SourceLoc { line: 2, column: 7 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("MOVER".to_string()),
            source_loc: SourceLoc { line: 2, column: 8 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("R0".to_string()),
            source_loc: SourceLoc {
                line: 2,
                column: 14,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Symbol,
            value: Some(",".to_string()),
            source_loc: SourceLoc {
                line: 2,
                column: 16,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("0".to_string()),
            source_loc: SourceLoc {
                line: 2,
                column: 18,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Newline,
            value: None,
            source_loc: SourceLoc {
                line: 2,
                column: 19,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Eof,
            value: None,
            source_loc: SourceLoc { line: 3, column: 1 },
        });
        let mut parser = SyntacticParser::new();
        let source_lines = ["", ""].map(|s| s.to_string()).to_vec();
        let statements = parser.parse(tokens, &source_lines).unwrap();
        assert_eq!(statements.len(), 2);
        assert_eq!(
            statements[0].label,
            Some(StatementField {
                value: "MOVE".to_string(),
                loc: SourceLoc { line: 1, column: 1 },
                operand_type: None
            })
        );
        assert_eq!(
            statements[0].identifier,
            Some(StatementField {
                value: "MOVER".to_string(),
                loc: SourceLoc { line: 1, column: 7 },
                operand_type: None
            })
        );
        assert_eq!(
            statements[0].operands,
            Some(vec![
                StatementField {
                    value: "R0".to_string(),
                    loc: SourceLoc {
                        line: 1,
                        column: 13
                    },
                    operand_type: Some(OperandType::Unknown)
                },
                StatementField {
                    value: "0".to_string(),
                    loc: SourceLoc {
                        line: 1,
                        column: 17
                    },
                    operand_type: Some(OperandType::Unknown)
                }
            ])
        );
        assert_eq!(
            statements[1].label,
            Some(StatementField {
                value: "MOVE1".to_string(),
                loc: SourceLoc { line: 2, column: 1 },
                operand_type: None
            })
        );
        assert_eq!(
            statements[1].identifier,
            Some(StatementField {
                value: "MOVER".to_string(),
                loc: SourceLoc { line: 2, column: 8 },
                operand_type: None
            })
        );
        assert_eq!(
            statements[1].operands,
            Some(vec![
                StatementField {
                    value: "R0".to_string(),
                    loc: SourceLoc {
                        line: 2,
                        column: 14
                    },
                    operand_type: Some(OperandType::Unknown)
                },
                StatementField {
                    value: "0".to_string(),
                    loc: SourceLoc {
                        line: 2,
                        column: 18
                    },
                    operand_type: Some(OperandType::Unknown)
                }
            ])
        );
    }

    #[test]
    fn test_single_operand() {
        let mut tokens = TokenStream::new();
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("CALL".to_string()),
            source_loc: SourceLoc { line: 1, column: 1 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("R0".to_string()),
            source_loc: SourceLoc { line: 1, column: 6 },
        });
        tokens.push(Token {
            token_type: TokenType::Eof,
            value: None,
            source_loc: SourceLoc { line: 1, column: 8 },
        });
        let mut parser = SyntacticParser::new();
        let source_lines = ["", ""].map(|s| s.to_string()).to_vec();
        let statements = parser.parse(tokens, &source_lines).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].label, None);
        assert_eq!(
            statements[0].identifier,
            Some(StatementField {
                value: "CALL".to_string(),
                loc: SourceLoc { line: 1, column: 1 },
                operand_type: None
            })
        );
        assert_eq!(
            statements[0].operands,
            Some(vec![StatementField {
                value: "R0".to_string(),
                loc: SourceLoc { line: 1, column: 6 },
                operand_type: Some(OperandType::Unknown)
            }])
        );
    }

    #[test]
    fn test_no_operand() {
        let mut tokens = TokenStream::new();
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("RET".to_string()),
            source_loc: SourceLoc { line: 1, column: 1 },
        });
        tokens.push(Token {
            token_type: TokenType::Eof,
            value: None,
            source_loc: SourceLoc { line: 1, column: 4 },
        });
        let mut parser = SyntacticParser::new();
        let source_lines = ["", ""].map(|s| s.to_string()).to_vec();
        let statements = parser.parse(tokens, &source_lines).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].label, None);
        assert_eq!(
            statements[0].identifier,
            Some(StatementField {
                value: "RET".to_string(),
                loc: SourceLoc { line: 1, column: 1 },
                operand_type: None
            })
        );
        assert_eq!(statements[0].operands, None);
    }

    #[test]
    fn test_unusual_statement1() {
        let mut tokens = TokenStream::new();
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("MOVE".to_string()),
            source_loc: SourceLoc { line: 1, column: 1 },
        });
        tokens.push(Token {
            token_type: TokenType::Symbol,
            value: Some(":".to_string()),
            source_loc: SourceLoc { line: 1, column: 5 },
        });
        tokens.push(Token {
            token_type: TokenType::Symbol,
            value: Some(":".to_string()),
            source_loc: SourceLoc { line: 1, column: 6 },
        });
        tokens.push(Token {
            token_type: TokenType::Eof,
            value: None,
            source_loc: SourceLoc { line: 1, column: 8 },
        });
        let mut parser = SyntacticParser::new();
        let source_lines = ["", ""].map(|s| s.to_string()).to_vec();
        // should fail
        let statements = parser.parse(tokens, &source_lines);
        assert!(statements.is_err());
    }

    #[test]
    fn test_unusual_statement2() {
        let mut tokens = TokenStream::new();
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("MOVER".to_string()),
            source_loc: SourceLoc { line: 1, column: 1 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("R0".to_string()),
            source_loc: SourceLoc { line: 1, column: 7 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("0".to_string()),
            source_loc: SourceLoc {
                line: 1,
                column: 10,
            },
        });

        let mut parser = SyntacticParser::new();
        let source_lines = ["MOVER R0 0"].map(|s| s.to_string()).to_vec();
        // should fail
        let statements = parser.parse(tokens, &source_lines);
        assert!(statements.is_err());
    }
}
