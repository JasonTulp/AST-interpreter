use crate::{
	error_handler::Error,
	native_functions::*,
	token::{LiteralType, Token},
};
use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, PartialEq, Eq)]
pub struct Environment {
	enclosing: Option<EnvRef>,
	values: HashMap<String, LiteralType>,
}

impl Environment {
	pub fn new(enclosing_env: Option<EnvRef>) -> EnvRef {
		Rc::new(RefCell::new(Self { enclosing: enclosing_env, values: HashMap::new() }))
	}

	// Store a variable in our hashmap. Note, we allow redefining variables
	pub fn define(&mut self, name: String, value: LiteralType) {
		self.values.insert(name, value);
	}

	// Get a variable from our hashmap. If it doesn't exist, check the enclosing environment
	pub fn get(&self, token: &Token) -> Result<LiteralType, Error> {
		if let Some(value) = self.values.get(&token.lexeme) {
			Ok(value.clone())
		} else if self.enclosing.is_some() {
			self.enclosing.as_ref().unwrap().borrow().get(token)
		} else {
			Err(Error::RuntimeError(token.line, format!("Undefined variable '{}'", token.lexeme)))
		}
	}

	/// Gets the value a fixed distance away from the current environment
	/// No need to check if the value exists because the resolver should have already verified
	/// it's existence
	pub fn get_at(&self, distance: u64, name: &str) -> Result<LiteralType, Error> {
		if distance == 0 {
			return Ok(self.values.get(name).unwrap().clone());
		}

		if let Some(enclosing) = &self.enclosing {
			return enclosing.borrow().get_at(distance - 1, name);
		}
		Err(Error::RuntimeError(0, "Environment not found".to_string()))
	}

	// Assign a value to a variable in our hashmap. If it doesn't exist, check the enclosing
	// environment
	pub fn assign(&mut self, token: &Token, value: LiteralType) -> Result<(), Error> {
		if self.values.contains_key(&token.lexeme) {
			self.values.insert(token.lexeme.clone(), value);
			Ok(())
		} else if let Some(enclosing) = &mut self.enclosing {
			enclosing.borrow_mut().assign(token, value)
		} else {
			Err(Error::RuntimeError(token.line, format!("Undefined variable '{}'", token.lexeme)))
		}
	}

	/// Assigns a value a fixed distance away from the current environment
	pub fn assign_at(
		&mut self,
		distance: u64,
		token: &Token,
		value: LiteralType,
	) -> Result<(), Error> {
		if distance == 0 {
			self.values.insert(token.lexeme.clone(), value.clone());
			return Ok(());
		}

		if let Some(enclosing) = &self.enclosing {
			return enclosing.borrow_mut().assign_at(distance - 1, token, value);
		}

		Err(Error::RuntimeError(0, "Environment not found".to_string()))
	}
}
