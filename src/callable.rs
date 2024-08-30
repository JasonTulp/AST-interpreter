use crate::{
	environment::Environment, error_handler::Error, interpreter::Interpreter, statements,
	statements::Stmt, token::LiteralType,
};
use std::{cell::RefCell, rc::Rc};

/// FunctionType is an enum that represents the type of function that is being resolved
#[derive(Copy, Clone, PartialEq)]
pub enum FunctionType {
	None,
	Function,
	// Initializer,
	// Method,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Callable {
	NativeFunction(NativeFunction),
	Function(JasnFunction),
}

// Native functions are functions that are implemented in Rust and are callable from JASN
#[derive(Debug, PartialEq, Clone)]
pub struct NativeFunction {
	pub arity: u8,
	pub function: fn(&mut Interpreter, Vec<LiteralType>) -> Result<LiteralType, Error>,
}

// Functions are user-defined functions that are defined in JASN
#[derive(Debug, PartialEq, Clone)]
pub struct JasnFunction {
	pub declaration: Box<statements::Function>,
	pub closure: Rc<RefCell<Environment>>,
}

impl Callable {
	pub fn call(
		&self,
		interpreter: &mut Interpreter,
		arguments: Vec<LiteralType>,
	) -> Result<LiteralType, Error> {
		match self {
			Callable::NativeFunction(native_function) =>
				(native_function.function)(interpreter, arguments),
			Callable::Function(function) => {
				// Create a new environment whenever the function is called and pass the arguments
				// into that environment
				let mut environment = function.closure.clone();
				for (i, argument) in arguments.iter().enumerate() {
					environment.borrow_mut().define(
						function.declaration.params[i].lexeme.to_string(),
						argument.clone(),
					);
				}
				match interpreter.execute_block(&function.declaration.body, environment) {
					Ok(_) => Ok(LiteralType::Null),
					Err(error) =>
						if let Error::Return(value) = error {
							Ok(value)
						} else {
							Err(error)
						},
				}
			},
		}
	}

	pub fn arity(&self) -> u8 {
		match self {
			Callable::NativeFunction(native_function) => native_function.arity,
			Callable::Function(function) => function.declaration.params.len() as u8,
		}
	}
}

impl ToString for Callable {
	fn to_string(&self) -> String {
		match self {
			Callable::NativeFunction(native_function) => "<fn native>".to_string(),
			Callable::Function(function) => format!("<fn {:?}>", function.declaration.name.lexeme),
		}
	}
}
