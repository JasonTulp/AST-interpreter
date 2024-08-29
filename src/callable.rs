use crate::environment::Environment;
use crate::error_handler::Error;
use crate::interpreter::Interpreter;
use crate::statements;
use crate::statements::Stmt;
use crate::token::LiteralType;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub enum Callable {
    NativeFunction(NativeFunction),
    Function(JasnFunction),
}

// Native functions are functions that are implemented in Rust and are callable from JASN
#[derive(PartialEq, Debug, Clone)]
pub struct NativeFunction {
    pub arity: u8,
    pub function: fn(&mut Interpreter, Vec<LiteralType>) -> Result<LiteralType, Error>,
}

// Functions are user-defined functions that are defined in JASN
#[derive(Debug, Clone, PartialEq)]
pub struct JasnFunction {
    pub declaration: Box<statements::Function>,
}

impl Callable {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LiteralType>,
    ) -> Result<LiteralType, Error> {
        match self {
            Callable::NativeFunction(native_function) => {
                (native_function.function)(interpreter, arguments)
            }
            Callable::Function(function) => {
                // Create a new environment whenever the function is called and pass the arguments
                // into that environment
                let mut environment = Rc::new(RefCell::new(Environment::new(Some(
                    interpreter.global.clone(),
                ))));
                for (i, argument) in arguments.iter().enumerate() {
                    environment.borrow_mut().define(
                        function.declaration.params[i].lexeme.to_string(),
                        argument.clone(),
                    );
                }
                interpreter.execute_block(&function.declaration.body, environment);
                Ok(LiteralType::Null)
            }
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
