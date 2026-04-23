pub mod instruction;
mod semantic_parser;
mod syntactic_parser;

use crate::{
    lexer::token::TokenStream,
    parser::{
        instruction::SemanticNode,
        semantic_parser::{SemanticError, SemanticParser},
        syntactic_parser::{SyntacticError, SyntacticParser},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Syntactic parsing error:\n{0}")]
    SyntacticParsing(#[from] SyntacticError),
    #[error("Semantic parsing error:\n{0}")]
    SemanticParsing(#[from] SemanticError),
}

pub struct Parser {
    syntactic_parser: SyntacticParser,
    semantic_parser: SemanticParser,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            syntactic_parser: SyntacticParser::new(),
            semantic_parser: SemanticParser::new(),
        }
    }

    pub fn parse(
        &mut self,
        tokens: TokenStream,
        source_lines: &Vec<String>,
    ) -> Result<Vec<SemanticNode>, ParserError> {
        let statements = self.syntactic_parser.parse(tokens, source_lines)?;
        let semantic_nodes = self.semantic_parser.parse(statements, source_lines)?;

        Ok(semantic_nodes)
    }
}

#[cfg(test)]
mod tests {
    use isa::AddressingMode;

    use crate::parser::instruction::SemanticNode;

    use super::{
        super::lexer::token::{SourceLoc, Token, TokenStream, TokenType},
        Parser,
        instruction::InstructionField,
    };

    #[test]
    fn test_parser() {
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
            token_type: TokenType::Newline,
            value: None,
            source_loc: SourceLoc { line: 1, column: 7 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("MOVER".to_string()),
            source_loc: SourceLoc { line: 2, column: 1 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("R0".to_string()),
            source_loc: SourceLoc { line: 2, column: 7 },
        });
        tokens.push(Token {
            token_type: TokenType::Symbol,
            value: Some(",".to_string()),
            source_loc: SourceLoc { line: 2, column: 9 },
        });
        tokens.push(Token {
            token_type: TokenType::Identifier,
            value: Some("0".to_string()),
            source_loc: SourceLoc {
                line: 2,
                column: 11,
            },
        });
        tokens.push(Token {
            token_type: TokenType::Eof,
            value: None,
            source_loc: SourceLoc {
                line: 2,
                column: 12,
            },
        });
        let mut parser = Parser::new();
        let source_lines = ["", ""].map(|s| s.to_string()).to_vec();
        let semantic_nodes = parser.parse(tokens, &source_lines).unwrap();
        assert_eq!(semantic_nodes.len(), 1);
        let semantic_node = &semantic_nodes[0];
        match semantic_node {
            SemanticNode::Instruction(instruction) => {
                assert_eq!(
                    instruction.opcode,
                    InstructionField {
                        value: 5,
                        bit_count: 6,
                        addressing_mode: None
                    }
                );
                let operands = instruction.operands.as_ref().unwrap();
                assert_eq!(operands.len(), 2);
                assert_eq!(
                    operands[0],
                    InstructionField {
                        value: 0,
                        bit_count: 2,
                        addressing_mode: Some(AddressingMode::Register)
                    }
                );
                assert_eq!(
                    operands[1],
                    InstructionField {
                        value: 0,
                        bit_count: 8,
                        addressing_mode: Some(AddressingMode::DirectData)
                    }
                );
            }
            _ => panic!("Unexpected semantic node"),
        }
    }
}
