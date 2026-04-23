mod lexer;

use thiserror::Error;

use self::lexer::{Lexer, LexerError};

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Lexer error:\n{0}")]
    Lexer(#[from] LexerError),
}

pub struct MyCompiler {}

impl MyCompiler {
    pub fn new() -> Result<Self, CompilerError> {
        Ok(Self {})
    }

    pub fn compile(&mut self, program: &str) -> Result<(), CompilerError> {
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.lex(program)?;
        println!("Tokens: {tokens:#?}");
        Ok(())
    }
}
