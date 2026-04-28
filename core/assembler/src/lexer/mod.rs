pub mod token;

use super::render_error::{Diagnostic, render_error};
use self::token::{SourceLoc, Token, TokenStream, TokenType};
use std::mem;

#[derive(Debug, thiserror::Error)]
pub enum LexerError {
    #[error("{message}")]
    InCompleteEscapeSequence { message: String },
    #[error("{message}")]
    InvalidEscapeSequence { message: String },
}

pub struct Lexer {
    tokens: TokenStream,
    token: String,
    line: u32,
    column: u32,
    token_loc: SourceLoc,
    source_lines: Vec<String>,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            tokens: TokenStream::new(),
            token: String::new(),
            line: 1,
            column: 0,
            token_loc: SourceLoc { line: 1, column: 1 },
            source_lines: Vec::new(),
        }
    }

    pub fn push_string(&mut self) {
        self.tokens.push(Token {
            token_type: TokenType::String,
            value: Some(mem::take(&mut self.token)),
            source_loc: mem::take(&mut self.token_loc),
        });
    }

    pub fn push_identifier(&mut self) {
        if self.token.is_empty() {
            return;
        }
        self.tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some(mem::take(&mut self.token)),
            source_loc: mem::take(&mut self.token_loc),
        });
    }

    pub fn push_symbol(&mut self, ch: char) {
        self.tokens.push(Token {
            token_type: TokenType::Symbol,
            value: Some(ch.to_string()),
            source_loc: mem::take(&mut self.token_loc),
        });
    }

    pub fn push_whitespace(&mut self) {
        self.tokens.push(Token {
            token_type: TokenType::Whitespace,
            value: None,
            source_loc: mem::take(&mut self.token_loc),
        });
    }

    pub fn push_newline(&mut self) {
        self.tokens.push(Token {
            token_type: TokenType::Newline,
            value: None,
            source_loc: mem::take(&mut self.token_loc),
        });
    }

    pub fn push_eof(&mut self) {
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            value: None,
            source_loc: mem::take(&mut self.token_loc),
        });
    }

    pub fn lex(&mut self, program: &str) -> Result<(TokenStream, Vec<String>), LexerError> {
        // Split the assembly program into lines for future use
        self.source_lines = program.split('\n').map(|s| s.to_string()).collect();

        // Tokenize
        let mut is_comment = false;
        let mut is_string = false;
        let mut escape_seq = false;

        for char in program.chars() {
            self.column += 1;
            if is_comment {
                if char == '\n' {
                    is_comment = false;
                    self.push_identifier();
                    self.token_loc.line = self.line;
                    self.token_loc.column = self.column;
                    self.push_newline();
                    self.line += 1;
                    self.column = 0;
                }
                continue;
            }
            if is_string {
                if char == '"' {
                    if escape_seq {
                        return Err(LexerError::InCompleteEscapeSequence {
                            message: render_error(Diagnostic {
                                headline: "Incomplete escape sequence".to_string(),
                                line: self.line,
                                column: self.column,
                                source_line: &self.source_lines[self.line as usize - 1],
                                help: None,
                            })
                        });
                    }
                    is_string = false;
                    self.push_string();
                    continue;
                }
                if char == '\\' {
                    escape_seq = true;
                    continue;
                }
                if escape_seq {
                    // push the character as escape sequence
                    match char {
                        'n' => self.token.push('\n'),
                        't' => self.token.push('\t'),
                        '0' => self.token.push('\0'),
                        _ => return Err(LexerError::InvalidEscapeSequence {
                            message: render_error(Diagnostic {
                                headline: "Invalid escape sequence".to_string(),
                                line: self.line,
                                column: self.column,
                                source_line: &self.source_lines[self.line as usize - 1],
                                help: None,
                            })
                        }),
                    }
                    escape_seq = false;
                    continue;
                }
                self.token.push(char);
                continue;
            }
            match char {
                ':' | ',' | '+' | '(' | ')' | '&' | '.' | '#' | '[' | ']' => {
                    self.push_identifier();
                    self.token_loc.line = self.line;
                    self.token_loc.column = self.column;
                    self.push_symbol(char);
                }
                '"' => {
                    is_string = true;
                    self.token_loc.line = self.line;
                    self.token_loc.column = self.column;
                }
                ';' => {
                    is_comment = true;
                }
                ' ' | '\t' => {
                    self.push_identifier();
                    self.token_loc.line = self.line;
                    self.token_loc.column = self.column;
                    self.push_whitespace();
                }
                '\n' => {
                    self.push_identifier();
                    self.token_loc.line = self.line;
                    self.token_loc.column = self.column;
                    self.push_newline();
                    self.line += 1;
                    self.column = 0;
                }
                _ => {
                    if self.token.is_empty() {
                        self.token_loc.line = self.line;
                        self.token_loc.column = self.column;
                    }
                    self.token.push(char);
                }
            }
        }
        self.push_identifier();
        self.token_loc.line = self.line;
        self.token_loc.column = self.column + 1;
        self.push_eof();
        Ok((
            mem::take(&mut self.tokens),
            mem::take(&mut self.source_lines),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex("MOVER R1, 0").unwrap().0.tokens;

        assert_eq!(tokens.len(), 7);

        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].value, Some("MOVER".to_string()));
        assert_eq!(tokens[0].source_loc, SourceLoc { line: 1, column: 1 });

        assert_eq!(tokens[1].token_type, TokenType::Whitespace);
        assert_eq!(tokens[1].value, None);
        assert_eq!(tokens[1].source_loc, SourceLoc { line: 1, column: 6 });

        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].value, Some("R1".to_string()));
        assert_eq!(tokens[2].source_loc, SourceLoc { line: 1, column: 7 });

        assert_eq!(tokens[3].token_type, TokenType::Symbol);
        assert_eq!(tokens[3].value, Some(','.to_string()));
        assert_eq!(tokens[3].source_loc, SourceLoc { line: 1, column: 9 });

        assert_eq!(tokens[4].token_type, TokenType::Whitespace);
        assert_eq!(tokens[4].value, None);
        assert_eq!(
            tokens[4].source_loc,
            SourceLoc {
                line: 1,
                column: 10
            }
        );

        assert_eq!(tokens[5].token_type, TokenType::Identifier);
        assert_eq!(tokens[5].value, Some("0".to_string()));
        assert_eq!(
            tokens[5].source_loc,
            SourceLoc {
                line: 1,
                column: 11
            }
        );

        assert_eq!(tokens[6].token_type, TokenType::Eof);
        assert_eq!(tokens[6].value, None);
        assert_eq!(
            tokens[6].source_loc,
            SourceLoc {
                line: 1,
                column: 12
            }
        );
    }

    #[test]
    fn test() {
        let mut lexer = Lexer::new();
        let tokens = lexer
            .lex("MOVE: MOVER R0, 0\nMOVE1: MOVER R0, 0;comment\n")
            .unwrap()
            .0
            .tokens;
        assert_eq!(tokens.len(), 21);

        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].value, Some("MOVE".to_string()));
        assert_eq!(tokens[0].source_loc, SourceLoc { line: 1, column: 1 });

        assert_eq!(tokens[1].token_type, TokenType::Symbol);
        assert_eq!(tokens[1].value, Some(':'.to_string()));
        assert_eq!(tokens[1].source_loc, SourceLoc { line: 1, column: 5 });

        assert_eq!(tokens[2].token_type, TokenType::Whitespace);
        assert_eq!(tokens[2].value, None);
        assert_eq!(tokens[2].source_loc, SourceLoc { line: 1, column: 6 });

        assert_eq!(tokens[3].token_type, TokenType::Identifier);
        assert_eq!(tokens[3].value, Some("MOVER".to_string()));
        assert_eq!(tokens[3].source_loc, SourceLoc { line: 1, column: 7 });

        assert_eq!(tokens[4].token_type, TokenType::Whitespace);
        assert_eq!(tokens[4].value, None);
        assert_eq!(
            tokens[4].source_loc,
            SourceLoc {
                line: 1,
                column: 12
            }
        );

        assert_eq!(tokens[5].token_type, TokenType::Identifier);
        assert_eq!(tokens[5].value, Some("R0".to_string()));
        assert_eq!(
            tokens[5].source_loc,
            SourceLoc {
                line: 1,
                column: 13
            }
        );

        assert_eq!(tokens[6].token_type, TokenType::Symbol);
        assert_eq!(tokens[6].value, Some(','.to_string()));
        assert_eq!(
            tokens[6].source_loc,
            SourceLoc {
                line: 1,
                column: 15
            }
        );

        assert_eq!(tokens[7].token_type, TokenType::Whitespace);
        assert_eq!(tokens[7].value, None);
        assert_eq!(
            tokens[7].source_loc,
            SourceLoc {
                line: 1,
                column: 16
            }
        );

        assert_eq!(tokens[8].token_type, TokenType::Identifier);
        assert_eq!(tokens[8].value, Some("0".to_string()));
        assert_eq!(
            tokens[8].source_loc,
            SourceLoc {
                line: 1,
                column: 17
            }
        );

        assert_eq!(tokens[9].token_type, TokenType::Newline);
        assert_eq!(tokens[9].value, None);
        assert_eq!(
            tokens[9].source_loc,
            SourceLoc {
                line: 1,
                column: 18
            }
        );

        assert_eq!(tokens[10].token_type, TokenType::Identifier);
        assert_eq!(tokens[10].value, Some("MOVE1".to_string()));
        assert_eq!(tokens[10].source_loc, SourceLoc { line: 2, column: 1 });

        assert_eq!(tokens[11].token_type, TokenType::Symbol);
        assert_eq!(tokens[11].value, Some(':'.to_string()));
        assert_eq!(tokens[11].source_loc, SourceLoc { line: 2, column: 6 });

        assert_eq!(tokens[12].token_type, TokenType::Whitespace);
        assert_eq!(tokens[12].value, None);
        assert_eq!(tokens[12].source_loc, SourceLoc { line: 2, column: 7 });

        assert_eq!(tokens[13].token_type, TokenType::Identifier);
        assert_eq!(tokens[13].value, Some("MOVER".to_string()));
        assert_eq!(tokens[13].source_loc, SourceLoc { line: 2, column: 8 });

        assert_eq!(tokens[14].token_type, TokenType::Whitespace);
        assert_eq!(tokens[14].value, None);
        assert_eq!(
            tokens[14].source_loc,
            SourceLoc {
                line: 2,
                column: 13
            }
        );

        assert_eq!(tokens[15].token_type, TokenType::Identifier);
        assert_eq!(tokens[15].value, Some("R0".to_string()));
        assert_eq!(
            tokens[15].source_loc,
            SourceLoc {
                line: 2,
                column: 14
            }
        );

        assert_eq!(tokens[16].token_type, TokenType::Symbol);
        assert_eq!(tokens[16].value, Some(','.to_string()));
        assert_eq!(
            tokens[16].source_loc,
            SourceLoc {
                line: 2,
                column: 16
            }
        );

        assert_eq!(tokens[17].token_type, TokenType::Whitespace);
        assert_eq!(tokens[17].value, None);
        assert_eq!(
            tokens[17].source_loc,
            SourceLoc {
                line: 2,
                column: 17
            }
        );

        assert_eq!(tokens[18].token_type, TokenType::Identifier);
        assert_eq!(tokens[18].value, Some("0".to_string()));
        assert_eq!(
            tokens[18].source_loc,
            SourceLoc {
                line: 2,
                column: 18
            }
        );

        assert_eq!(tokens[19].token_type, TokenType::Newline);
        assert_eq!(tokens[19].value, None);
        assert_eq!(
            tokens[19].source_loc,
            SourceLoc {
                line: 2,
                column: 27
            }
        );

        assert_eq!(tokens[20].token_type, TokenType::Eof);
        assert_eq!(tokens[20].value, None);
        assert_eq!(tokens[20].source_loc, SourceLoc { line: 3, column: 1 });
    }
}
