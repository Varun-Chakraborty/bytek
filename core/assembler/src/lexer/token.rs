#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Identifier,
    Symbol,
    Whitespace,
    Newline,
    Eof,
}

#[derive(Debug, Clone, Default, PartialEq, Copy, Eq, Hash)]
pub struct SourceLoc {
    pub line: u32,
    pub column: u32,
}

impl std::fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: Option<String>,
    pub token_type: TokenType,
    pub source_loc: SourceLoc,
}

#[derive(Default, Debug)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
    pub index: usize,
}

impl TokenStream {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            index: 0,
        }
    }

    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub fn seek(&self, at: u32) -> Option<&Token> {
        if at + self.index as u32 >= self.tokens.len() as u32 {
            return None;
        }
        let token = Some(&self.tokens[self.index + at as usize]);
        token
    }

    pub fn seek_as_symbol(&self, at: u32) -> Option<char> {
        let token = self.seek(at);
        match token {
            Some(token) => match token.token_type {
                TokenType::Symbol => match &token.value {
                    Some(value) => Some(value.chars().next().unwrap()),
                    None => None,
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn next(&mut self) {
        self.index += 1;
    }

    pub fn remove(&mut self) -> Token {
        self.tokens.remove(self.index)
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }
}
