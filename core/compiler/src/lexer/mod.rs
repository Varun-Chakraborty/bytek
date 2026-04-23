use std::mem;

#[derive(Debug, Default)]
pub struct TokenStream {}

#[derive(Debug, thiserror::Error)]
pub enum LexerError {}

pub struct Lexer {
    tokens: TokenStream,
    source_lines: Vec<String>,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            tokens: TokenStream {},
            source_lines: Vec::new(),
        }
    }

    pub fn lex(&mut self, program: &str) -> Result<(TokenStream, Vec<String>), LexerError> {
        // Split the program into lines for future use
        self.source_lines = program.split('\n').map(|s| s.to_string()).collect();

        // Tokenize

        Ok((
            mem::take(&mut self.tokens),
            mem::take(&mut self.source_lines),
        ))
    }
}
