use std::collections::HashMap;

use super::{
    lexer::token::{Token, TokenStream, TokenType},
    render_error::{Diagnostic, render_error},
};

#[derive(PartialEq, Debug)]
pub enum DefinitionDFA {
    AfterMacroKeyword,
    MacroHeader,
    ExpectSpaceOrNewline,
    ExpectAmpersandOrNewline,
    AfterParameter,
    ExpectAmpersand,
    ExpectParameter,
    ModelStatements,
    MENDOrModelStatements,
    ExpectNewlineOrEof,
}

#[derive(Debug, thiserror::Error)]
pub enum PreProcessorError {
    #[error("{message}")]
    InvalidToken { message: String },
}

pub struct PreProcessor {
    macros: HashMap<String, Vec<Token>>,
    macro_name: String,
}

impl PreProcessor {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            macro_name: String::new(),
        }
    }

    pub fn definition(
        &mut self,
        tokens: &mut TokenStream,
        source_lines: &Vec<String>,
    ) -> Result<(), PreProcessorError> {
        loop {
            while let Some(token) = tokens.seek(0) {
                if let Some(token) = &token.value
                    && *token == "MACRO".to_string()
                {
                    break;
                }
                if token.token_type == TokenType::Eof {
                    tokens.reset();
                    return Ok(());
                }
                tokens.next();
            }
            tokens.remove();
            let mut state = DefinitionDFA::AfterMacroKeyword;
            loop {
                let current_token = tokens.seek(0).unwrap().clone();

                if current_token.token_type != TokenType::Eof {
                    tokens.remove();
                }

                match state {
                    DefinitionDFA::AfterMacroKeyword => match current_token.token_type {
                        TokenType::Newline => {
                            state = DefinitionDFA::MacroHeader;
                        }
                        TokenType::Whitespace => {}
                        _ => {
                            return Err(PreProcessorError::InvalidToken {
                                message: render_error(Diagnostic {
                                    headline: format!("Invalid token or EOF encountered"),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: Some("A newline is expected after macro keyword"),
                                }),
                            });
                        }
                    },
                    DefinitionDFA::MacroHeader => match current_token.token_type {
                        TokenType::Identifier => {
                            self.macro_name = current_token.value.unwrap();
                            self.macros.insert(self.macro_name.clone(), Vec::new());
                            state = DefinitionDFA::ExpectSpaceOrNewline;
                        }
                        TokenType::Whitespace | TokenType::Newline => {}
                        TokenType::Symbol | TokenType::Eof => {
                            return Err(PreProcessorError::InvalidToken {
                                message: "A macro name is expected".to_string(),
                            });
                        }
                    },
                    DefinitionDFA::ExpectSpaceOrNewline => match current_token.token_type {
                        TokenType::Whitespace => state = DefinitionDFA::ExpectAmpersandOrNewline,
                        TokenType::Newline => state = DefinitionDFA::ModelStatements,
                        _ => {
                            return Err(PreProcessorError::InvalidToken {
                                message: render_error(Diagnostic {
                                    headline: format!("Invalid token or EOF encountered"),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: Some("A space or newline is expected"),
                                }),
                            });
                        }
                    },
                    DefinitionDFA::ExpectAmpersandOrNewline => match current_token.token_type {
                        TokenType::Symbol => {
                            if current_token.value.unwrap() != "&" {
                                return Err(PreProcessorError::InvalidToken {
                                    message: render_error(Diagnostic {
                                        headline: format!("Invalid token or EOF encountered"),
                                        line: current_token.source_loc.line,
                                        source_line: &source_lines
                                            [current_token.source_loc.line as usize - 1],
                                        column: current_token.source_loc.column,
                                        help: Some("A parameter or newline is expected"),
                                    }),
                                });
                            }
                            state = DefinitionDFA::ExpectParameter;
                        }
                        TokenType::Newline => state = DefinitionDFA::ModelStatements,
                        _ => {
                            return Err(PreProcessorError::InvalidToken {
                                message: render_error(Diagnostic {
                                    headline: format!("Invalid token or EOF encountered"),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: Some("A parameter or newline is expected"),
                                }),
                            });
                        }
                    },
                    DefinitionDFA::ExpectParameter => match current_token.token_type {
                        TokenType::Identifier => {
                            //TODO introduce parameters
                            state = DefinitionDFA::AfterParameter;
                        }
                        _ => {
                            return Err(PreProcessorError::InvalidToken {
                                message: render_error(Diagnostic {
                                    headline: format!("Invalid token or EOF encountered"),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: Some("A parameter is expected"),
                                }),
                            });
                        }
                    },
                    DefinitionDFA::AfterParameter => match current_token.token_type {
                        TokenType::Symbol => {
                            if current_token.value.unwrap() != "," {
                                return Err(PreProcessorError::InvalidToken {
                                    message: render_error(Diagnostic {
                                        headline: format!("Invalid token or EOF encountered"),
                                        line: current_token.source_loc.line,
                                        source_line: &source_lines
                                            [current_token.source_loc.line as usize - 1],
                                        column: current_token.source_loc.column,
                                        help: Some(
                                            "A comma followed by another parameter or newline is expected",
                                        ),
                                    }),
                                });
                            }
                            state = DefinitionDFA::ExpectAmpersand;
                        }
                        TokenType::Whitespace => {}
                        TokenType::Newline => state = DefinitionDFA::ModelStatements,
                        _ => {
                            return Err(PreProcessorError::InvalidToken {
                                message: render_error(Diagnostic {
                                    headline: format!("Invalid token or EOF encountered"),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: Some(
                                        "A comma followed by another parameter or newline is expected",
                                    ),
                                }),
                            });
                        }
                    },
                    DefinitionDFA::ExpectAmpersand => match current_token.token_type {
                        TokenType::Symbol => {
                            if current_token.value.unwrap() != "&" {
                                return Err(PreProcessorError::InvalidToken {
                                    message: render_error(Diagnostic {
                                        headline: format!("Invalid token or EOF encountered"),
                                        line: current_token.source_loc.line,
                                        source_line: &source_lines
                                            [current_token.source_loc.line as usize - 1],
                                        column: current_token.source_loc.column,
                                        help: Some("A parameter or newline is expected"),
                                    }),
                                });
                            }
                            state = DefinitionDFA::ExpectParameter;
                        }
                        TokenType::Whitespace => {}
                        _ => {}
                    },
                    DefinitionDFA::ModelStatements => {
                        if current_token.token_type == TokenType::Eof {
                            return Err(PreProcessorError::InvalidToken {
                                message: render_error(Diagnostic {
                                    headline: format!(
                                        "EOF encountered before the end of the macro definition"
                                    ),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: Some(
                                        "MEND keyword should follow newline after model statements to end the macro definition",
                                    ),
                                }),
                            });
                        } else {
                            if current_token.token_type == TokenType::Newline {
                                state = DefinitionDFA::MENDOrModelStatements;
                            }
                            self.macros
                                .get_mut(&self.macro_name)
                                .unwrap()
                                .push(current_token);
                        }
                    }
                    DefinitionDFA::MENDOrModelStatements => match current_token.token_type {
                        TokenType::Identifier => {
                            if current_token.value == Some("MEND".to_string()) {
                                state = DefinitionDFA::ExpectNewlineOrEof;
                            } else {
                                self.macros
                                    .get_mut(&self.macro_name)
                                    .unwrap()
                                    .push(current_token);
                                state = DefinitionDFA::ModelStatements;
                            }
                        }
                        TokenType::Eof => {
                            return Err(PreProcessorError::InvalidToken {
                                message: render_error(Diagnostic {
                                    headline: format!(
                                        "EOF encountered before the end of the macro definition"
                                    ),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: Some(
                                        "MEND keyword is required to denote the end of the macro definition",
                                    ),
                                }),
                            });
                        }
                        TokenType::Newline => {}
                        _ => {
                            self.macros
                                .get_mut(&self.macro_name)
                                .unwrap()
                                .push(current_token);
                            state = DefinitionDFA::ModelStatements;
                        }
                    },
                    DefinitionDFA::ExpectNewlineOrEof => match current_token.token_type {
                        TokenType::Newline | TokenType::Eof => break,
                        _ => {
                            return Err(PreProcessorError::InvalidToken {
                                message: render_error(Diagnostic {
                                    headline: format!("Invalid token encountered"),
                                    line: current_token.source_loc.line,
                                    source_line: &source_lines
                                        [current_token.source_loc.line as usize - 1],
                                    column: current_token.source_loc.column,
                                    help: Some("A newline is expected"),
                                }),
                            });
                        }
                    },
                }
            }
        }
    }

    pub fn invocation(&mut self, tokens: &mut TokenStream) -> Result<(), PreProcessorError> {
        tokens.reset();
        Ok(())
    }

    pub fn preprocess(
        &mut self,
        tokens: &mut TokenStream,
        source_lines: &Vec<String>,
    ) -> Result<(), PreProcessorError> {
        self.definition(tokens, source_lines)?;
        self.invocation(tokens)?;
        Ok(())
    }
}
