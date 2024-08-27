#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: LiteralType,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    String(String),
    Number(f64),
    Bool(bool),
    Empty,
}

impl Into<String> for LiteralType {
    fn into(self) -> String {
        match self {
            Self::Empty => String::default(),
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
            line
        }
    }

    pub fn to_string(self) -> String {
        let lit: String = self.literal.into();
        format!("{:?} {:?} {:?}",self.token_type, self.lexeme, lit)
    }
}

#[derive(Clone, Debug,PartialEq)]
pub enum TokenType {
    // Single Character Tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or Two Character Tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals
    Identifier, String, Number,

    // Keywords
    And, Class, Else, False, Funk, For, If, Null, Or,
    Print, Return, Super, This, True, Var, While,

    // End of File
    Eof
}