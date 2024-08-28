use crate::error_handler::Error;
use crate::token::{LiteralType, Token};
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, LiteralType>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    // Store a variable in our hashmap. Note, we allow redefining variables
    pub fn define(&mut self, name: String, value: LiteralType) {
        self.values.insert(name, value);
    }

    // Get a variable from our hashmap
    pub fn get(&self, token: Token) -> Result<&LiteralType, Error> {
        self.values.get(&token.lexeme).ok_or(Error::RuntimeError(
            token.line,
            format!("Undefined variable '{}'", token.lexeme),
        ))
    }

    pub fn assign(&mut self, token: Token, value: LiteralType) -> Result<(), Error> {
        if self.values.contains_key(&token.lexeme) {
            self.values.insert(token.lexeme, value);
            Ok(())
        } else {
            Err(Error::RuntimeError(
                token.line,
                format!("Undefined variable '{}'", token.lexeme),
            ))
        }
    }
}
