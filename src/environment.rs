use crate::error_handler::Error;
use crate::native_functions::*;
use crate::token::{LiteralType, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvRef = Rc<RefCell<Environment>>;

pub struct Environment {
    enclosing: Option<EnvRef>,
    values: HashMap<String, LiteralType>,
}

impl Environment {
    pub fn new(enclosing_env: Option<EnvRef>) -> Self {
        Self {
            enclosing: enclosing_env,
            values: HashMap::new(),
        }
    }

    // Store a variable in our hashmap. Note, we allow redefining variables
    pub fn define(&mut self, name: String, value: LiteralType) {
        self.values.insert(name, value);
    }

    // Get a variable from our hashmap. If it doesn't exist, check the enclosing environment
    pub fn get(&self, token: Token) -> Result<LiteralType, Error> {
        if let Some(value) = self.values.get(&token.lexeme) {
            Ok(value.clone())
        } else if self.enclosing.is_some() {
            self.enclosing.as_ref().unwrap().borrow().get(token)
        } else {
            Err(Error::RuntimeError(
                token.line,
                format!("Undefined variable '{}'", token.lexeme),
            ))
        }
    }

    // Assign a value to a variable in our hashmap. If it doesn't exist, check the enclosing environment
    pub fn assign(&mut self, token: Token, value: LiteralType) -> Result<(), Error> {
        if self.values.contains_key(&token.lexeme) {
            self.values.insert(token.lexeme, value);
            Ok(())
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.borrow_mut().assign(token, value)
        } else {
            Err(Error::RuntimeError(
                token.line,
                format!("Undefined variable '{}'", token.lexeme),
            ))
        }
    }
}
