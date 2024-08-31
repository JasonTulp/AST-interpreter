use crate::{
	environment::Environment,
	error_handler::Error,
	interpreter::Interpreter,
	statements,
	statements::Stmt,
	token::{LiteralType, Token},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// FunctionType is an enum that represents the type of function that is being resolved
#[derive(Copy, Clone, PartialEq)]
pub enum FunctionType {
	None,
	Function,
	Method,
	// Initializer,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Callable {
	NativeFunction(NativeFunction),
	Function(JasnFunction),
	Class(JasnClass),
	Instance(JasnInstanceRef),
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

#[derive(Debug, PartialEq, Clone)]
pub struct JasnClass {
	pub name: String,
	pub methods: HashMap<String, Callable>,
}

impl JasnClass {
	pub fn new(name: String, methods: HashMap<String, Callable>) -> Self {
		Self { name, methods }
	}

	pub fn find_method(&self, name: &str) -> Option<Callable> {
		self.methods.get(name).cloned()
	}
}

pub type JasnInstanceRef = Rc<RefCell<JasnInstance>>;

#[derive(Debug, PartialEq, Clone)]
pub struct JasnInstance {
	pub class: JasnClass,
	fields: HashMap<String, LiteralType>,
}

impl JasnInstance {
	pub fn new(class: JasnClass) -> JasnInstanceRef {
		JasnInstanceRef::new(RefCell::new(Self { class, fields: Default::default() }))
	}

	pub fn get(&self, name: &Token) -> Result<LiteralType, Error> {
		if let Some(value) = self.fields.get(&name.lexeme) {
			Ok(value.clone())
		} else if let Some(method) = self.class.find_method(&name.lexeme) {
			Ok(LiteralType::Callable(method.clone()))
		} else {
			Err(Error::RuntimeError(
				name.get_line(),
				format!("Undefined property '{}'", name.lexeme),
			))
		}
	}

	pub fn set(&mut self, name: &str, value: LiteralType) {
		self.fields.insert(name.to_string(), value);
	}
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
			Callable::Class(class) => {
				let instance = JasnInstance::new(class.clone());
				Ok(LiteralType::Callable(Callable::Instance(instance.clone())))
			},
			Callable::Instance(instance) => {
				todo!()
			},
		}
	}

	pub fn arity(&self) -> u8 {
		match self {
			Callable::NativeFunction(native_function) => native_function.arity,
			Callable::Function(function) => function.declaration.params.len() as u8,
			Callable::Class(_) => 0,
			Callable::Instance(_) => 0,
		}
	}
}

impl ToString for Callable {
	fn to_string(&self) -> String {
		match self {
			Callable::NativeFunction(native_function) => "<fn native>".to_string(),
			Callable::Function(function) => format!("<fn {:?}>", function.declaration.name.lexeme),
			Callable::Class(class) => format!("<class {:?}>", class.name),
			Callable::Instance(instance) =>
				format!("<{:?} instance>", instance.borrow().class.name),
		}
	}
}
