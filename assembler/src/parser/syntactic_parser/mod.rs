use std::mem;

use super::{
    super::{
        lexer::token::{TokenStream, TokenType},
        render_error::{Diagnostic, render_error},
    },
    instruction::Statement,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SyntacticError {
    #[error("{message}")]
    UnexpectedToken { message: String },
}

#[derive(PartialEq, Debug)]
enum DFAState {
    Start,
    AfterLabel,
    AfterOpcode,
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
            match current_token.token_type {
                TokenType::Identifier => {
                    match state {
                        DFAState::Start => {
                            // label or opcode
                            if let Some(':') = tokens.seek_as_symbol(1) {
                                statement.set_label(
                                    current_token.value.clone().unwrap(),
                                    current_token.source_loc,
                                );
                                tokens.next();
                                tokens.next();
                                state = DFAState::AfterLabel;
                            } else {
                                statement.set_identifier(
                                    current_token.value.clone().unwrap(),
                                    current_token.source_loc,
                                );
                                tokens.next();
                                state = DFAState::AfterOpcode;
                            }
                        }
                        DFAState::AfterLabel => {
                            statement.set_identifier(
                                current_token.value.clone().unwrap(),
                                current_token.source_loc,
                            );
                            tokens.next();
                            state = DFAState::AfterOpcode;
                        }
                        DFAState::ExpectOperand | DFAState::AfterOpcode => {
                            statement.add_operand(
                                current_token.value.clone().unwrap(),
                                current_token.source_loc,
                            );
                            tokens.next();
                            state = DFAState::AfterOperand;
                        }
                        _ => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: format!(
                                        "Unexpected identifier '{}'",
                                        current_token.value.clone().unwrap()
                                    ),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
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
                }
                TokenType::Symbol => {
                    if state == DFAState::AfterOperand
                        && current_token.value.clone().unwrap().as_str() == ","
                    {
                        state = DFAState::ExpectOperand;
                        tokens.next();
                    } else {
                        return Err(SyntacticError::UnexpectedToken {
                            message: render_error(Diagnostic {
                                headline: format!(
                                    "Unexpected symbol '{}'",
                                    current_token.value.clone().unwrap()
                                ),
                                line: current_token.source_loc.line,
                                source_line: &source_lines
                                    [current_token.source_loc.line as usize - 1],
                                column: current_token.source_loc.column,
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
                                    _ => "",
                                }),
                            }),
                        });
                    }
                }
                TokenType::Newline => {
                    match state {
                        DFAState::ExpectOperand => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "An identifier is expected after comma".to_string(),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        DFAState::Start => {}
                        _ => self.statements.push(statement),
                    }
                    statement = Statement::new();
                    tokens.next();
                    state = DFAState::Start;
                }
                TokenType::Eof => {
                    match state {
                        DFAState::ExpectOperand => {
                            return Err(SyntacticError::UnexpectedToken {
                                message: render_error(Diagnostic {
                                    headline: "An identifier is expected after comma".to_string(),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: None,
                                }),
                            });
                        }
                        DFAState::Start => {}
                        _ => self.statements.push(statement),
                    }
                    return Ok(mem::take(&mut self.statements));
                }
                _ => tokens.next(),
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
                loc: SourceLoc { line: 1, column: 1 }
            })
        );
        assert_eq!(
            statements[0].identifier,
            Some(StatementField {
                value: "MOVER".to_string(),
                loc: SourceLoc { line: 1, column: 7 }
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
                    }
                },
                StatementField {
                    value: "0".to_string(),
                    loc: SourceLoc {
                        line: 1,
                        column: 17
                    }
                }
            ])
        );
        assert_eq!(
            statements[1].label,
            Some(StatementField {
                value: "MOVE1".to_string(),
                loc: SourceLoc { line: 2, column: 1 }
            })
        );
        assert_eq!(
            statements[1].identifier,
            Some(StatementField {
                value: "MOVER".to_string(),
                loc: SourceLoc { line: 2, column: 8 }
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
                    }
                },
                StatementField {
                    value: "0".to_string(),
                    loc: SourceLoc {
                        line: 2,
                        column: 18
                    }
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
                loc: SourceLoc { line: 1, column: 1 }
            })
        );
        assert_eq!(
            statements[0].operands,
            Some(vec![StatementField {
                value: "R0".to_string(),
                loc: SourceLoc { line: 1, column: 6 }
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
                loc: SourceLoc { line: 1, column: 1 }
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
