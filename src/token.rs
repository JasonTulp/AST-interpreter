use crate::error_handler::Error;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: LiteralType,
    pub line: u32,
}

impl Token {
    pub fn get_line(&self) -> u32 {
        if self.token_type == TokenType::Eof {
            self.line.saturating_sub(1)
        } else {
            self.line
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    String(String),
    Number(f64),
    Bool(bool),
    Empty,
}

impl LiteralType {
    // Returns the bool value if it is a bool, false if it's null and true if anything else
    // This follows Ruby's rule where false and null are falsey and everything else truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralType::Bool(b) => *b,
            LiteralType::Empty => false,
            _ => true,
        }
    }
}

impl TryInto<f64> for LiteralType {
    type Error = String;

    fn try_into(self) -> Result<f64, String> {
        match self {
            Self::Number(n) => Ok(n),
            _ => Err("Cannot convert to decimal".to_string()),
        }
    }
}

impl TryInto<bool> for LiteralType {
    type Error = String;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Self::Bool(b) => Ok(b),
            _ => Err("Cannot convert to bool".to_string()),
        }
    }
}

impl Into<String> for LiteralType {
    fn into(self) -> String {
        match self {
            Self::Empty => "null".to_string(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::String(s) => s,
        }
    }
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: LiteralType, line: u32) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }

    pub fn to_string(self) -> String {
        let lit: String = self.literal.into();
        format!("{:?} {:?} {:?}", self.token_type, self.lexeme, lit)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    // Single Character Tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or Two Character Tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Funk,
    For,
    If,
    Null,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // End of File
    Eof,
}
