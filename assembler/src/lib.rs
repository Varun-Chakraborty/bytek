mod encoder;
mod lexer;
mod parser;
mod preprocessor;
mod render_error;
pub mod writer;

use thiserror::Error;

use self::{
    encoder::{Encoder, EncoderError, delimiter::DelimiterTable},
    lexer::{Lexer, LexerError},
    parser::{Parser, ParserError},
    preprocessor::{PreProcessor, PreProcessorError},
};

#[derive(Debug, Error)]
pub enum AssemblerError {
    #[error("I/O error:\n{0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown error:\n{msg}")]
    Unknown { msg: String },
    #[error("Lexer error:\n{0}")]
    Lexer(#[from] LexerError),
    #[error("{0}")]
    Parser(#[from] ParserError),
    #[error("Encoder error:\n{0}")]
    Encoder(#[from] EncoderError),
    #[error("Preprocessor error:\n{0}")]
    PreProcessor(#[from] PreProcessorError),
}

pub struct MyAssembler {}

impl MyAssembler {
    pub fn new() -> Result<Self, AssemblerError> {
        Ok(Self {})
    }

    pub fn assemble(
        &mut self,
        assembly_program: &str,
    ) -> Result<(Vec<u8>, DelimiterTable), AssemblerError> {
        let mut lexer = Lexer::new();
        let mut preprocessor = PreProcessor::new();
        let mut parser = Parser::new();
        let mut encoder = Encoder::new();

        println!("Assembling...");
        let (mut tokens, source_lines) = lexer.lex(assembly_program)?;
        preprocessor.preprocess(&mut tokens, &source_lines)?;
        let semantic_nodes = parser.parse(tokens, &source_lines)?;
        let (binary, delimiter_table) = encoder.encode(semantic_nodes)?;

        Ok((binary, delimiter_table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble() {
        let mut assembler = MyAssembler::new().unwrap();
        let (binary, _) = assembler.assemble("MOVE:\nMOVER R0, 0").unwrap();
        assert_eq!(binary, vec![4, 0, 0, 0, 0, 12]);
    }
}
