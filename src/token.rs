use crate::callable::Callable;
use core::hash::Hash;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
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
	Array(Vec<LiteralType>),
	Callable(Callable),
	Null,
}

impl Hash for LiteralType {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::String(s) => s.hash(state),
			Self::Number(n) => n.to_bits().hash(state),
			Self::Bool(b) => b.hash(state),
			Self::Array(a) => a.hash(state),
			Self::Callable(c) => c.to_string().hash(state),
			Self::Null => "null".hash(state),
		}
	}
}

impl Eq for LiteralType {}

impl LiteralType {
	// Returns the bool value if it is a bool, false if it's null and true if anything else
	// This follows Ruby's rule where false and null are falsey and everything else truthy
	pub fn is_truthy(&self) -> bool {
		match self {
			Self::Bool(b) => *b,
			Self::Null => false,
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

impl ToString for LiteralType {
	fn to_string(&self) -> String {
		match self {
			Self::Null => "null".to_string(),
			Self::Number(n) => n.to_string(),
			Self::Bool(b) =>
				if *b {
					"yeah".to_string()
				} else {
					"nah".to_string()
				},
			Self::String(s) => s.clone(),
			Self::Callable(c) => c.to_string(),
			// Self::Array(_) => "array".to_string(),
			Self::Array(val) => {
				let mut array = String::from("[");
				for (i, v) in val.iter().enumerate() {
					let s: String = (*v).clone().to_string();
					array.push_str(&s);
					if i != val.len() - 1 {
						array.push_str(", ");
					}
				}
				array.push_str("]");
				array
			},
		}
	}
}

impl Token {
	pub fn new(token_type: TokenType, lexeme: String, literal: LiteralType, line: u32) -> Self {
		Self { token_type, lexeme, literal, line }
	}
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum TokenType {
	// Single Character Tokens
	LeftParen,
	RightParen,
	LeftBrace,
	RightBrace,
	LeftSquare,
	RightSquare,
	Comma,
	Dot,
	Semicolon,
	Modulo,

	// One or Two Character Tokens
	Bang,
	BangEqual,
	Equal,
	EqualEqual,
	Greater,
	GreaterEqual,
	Less,
	LessEqual,
	Plus,
	PlusPlus,
	Minus,
	MinusMinus,
	PlusEqual,
	MinusEqual,
	Slash,
	SlashEqual,
	Star,
	StarEqual,

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
