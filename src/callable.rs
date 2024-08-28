use crate::error_handler::Error;
use crate::interpreter::Interpreter;
use crate::token::LiteralType;

#[derive(PartialEq, Debug, Clone)]
pub enum Callable {
    NativeFunction(NativeFunction),
}

#[derive(PartialEq, Debug, Clone)]
pub struct NativeFunction {
    pub arity: u8,
    pub function: fn(&mut Interpreter, Vec<LiteralType>) -> Result<LiteralType, Error>,
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
        }
    }

    pub fn arity(&self) -> u8 {
        match self {
            Callable::NativeFunction(native_function) => native_function.arity,
        }
    }
}
