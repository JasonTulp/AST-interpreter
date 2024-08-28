use crate::callable::{Callable, NativeFunction};
use crate::environment::{EnvRef, Environment};
use crate::error_handler::Error;
use crate::token::{LiteralType, Token, TokenType};

pub struct NativeFunctions {}

impl NativeFunctions {
    // Define all the native functions in the environment
    pub fn define_native_functions(environment: EnvRef) {
        Self::define_clock(environment.clone());
        Self::define_input(environment.clone());
        Self::define_len(environment.clone());
        Self::define_print(environment.clone());
        Self::define_sleep(environment.clone());
    }

    /// The clock function will return the time in seconds since the UNIX Epoch
    fn define_clock(environment: EnvRef) {
        let clock = LiteralType::Callable(Callable::NativeFunction(NativeFunction {
            arity: 0,
            function: |_, _| {
                Ok(LiteralType::Number(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                ))
            },
        }));
        environment.borrow_mut().define("clock".to_string(), clock);
    }

    /// The input function will read a line from the standard input
    fn define_input(environment: EnvRef) {
        let input = LiteralType::Callable(Callable::NativeFunction(NativeFunction {
            arity: 0,
            function: |_, _| {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                Ok(LiteralType::String(input.trim().to_string()))
            },
        }));
        environment.borrow_mut().define("input".to_string(), input);
    }

    /// Print will print the argument, eventually replacing the print statement
    fn define_print(environment: EnvRef) {
        let print = LiteralType::Callable(Callable::NativeFunction(NativeFunction {
            arity: 1,
            function: |_, args| {
                println!("{:?}", args[0]);
                Ok(LiteralType::Null)
            },
        }));
        environment.borrow_mut().define("print".to_string(), print);
    }

    fn define_len(environment: EnvRef) {
        let len = LiteralType::Callable(Callable::NativeFunction(NativeFunction {
            arity: 1,
            function: |_env, args| match &args[0] {
                LiteralType::String(s) => Ok(LiteralType::Number(s.len() as f64)),
                LiteralType::Array(a) => Ok(LiteralType::Number(a.len() as f64)),
                LiteralType::Callable(c) => Ok(LiteralType::Number(c.arity() as f64)),
                _ => Ok(LiteralType::Null),
            },
        }));
        environment.borrow_mut().define("len".to_string(), len);
    }

    fn define_sleep(environment: EnvRef) {
        let sleep = LiteralType::Callable(Callable::NativeFunction(NativeFunction {
            arity: 1,
            function: |_env, args| {
                let LiteralType::Number(secs) = args[0] else {
                    return Err(Error::RuntimeError(
                        0,
                        "sleep only accepts a number as an argument".to_string(),
                    ));
                };
                std::thread::sleep(std::time::Duration::from_secs_f64(secs));
                Ok(LiteralType::Null)
            },
        }));
        environment.borrow_mut().define("sleep".to_string(), sleep);
    }
}
