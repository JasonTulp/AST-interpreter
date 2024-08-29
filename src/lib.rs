mod callable;
mod environment;
mod error_handler;
mod expressions;
mod interpreter;
mod native_functions;
mod parser;
mod scanner;
mod statements;
mod token;

use crate::environment::{EnvRef, Environment};
use crate::error_handler::ErrorHandler;
use crate::native_functions::NativeFunctions;
use parser::Parser;
use scanner::Scanner;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;
use std::{io, process};

// Start the REPL and handle incoming prompts
pub fn run_prompt() {
    let environment = Rc::new(RefCell::new(Environment::new(None)));
    NativeFunctions::define_native_functions(environment.clone());
    loop {
        print!("==> ");
        let mut line = String::new();
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut line).unwrap();
        run(line.as_bytes().to_vec(), Rc::clone(&environment));
    }
}

// Load and run a file, reading the entire contents into a buffer
pub fn run_file(path: &str) -> io::Result<()> {
    // let ext = Path::new(path).extension();
    // match ext {
    //     Some(e) => {
    //         if e != "jasn" {
    //             return Err("Only '.jasn' file supported.".into());
    //         }
    //     }
    //     None => return Err("Cannot identify file extension.".into()),
    // }
    let environment = Rc::new(RefCell::new(Environment::new(None)));
    NativeFunctions::define_native_functions(environment.clone());
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    run(buffer, Rc::clone(&environment));
    Ok(())
}

// Actually run the interpreter
fn run(source: Vec<u8>, environment: EnvRef) {
    // let start_time = std::time::Instant::now();
    // Create a re-usable error handler
    let error_handler = Rc::new(RefCell::new(ErrorHandler::new()));

    // Scan the input text and convert to a list of tokens
    let mut scanner = Scanner::new(source, Rc::clone(&error_handler));
    scanner.scan_tokens();
    // We don't want to continue if there was an error scanning the tokens
    if error_handler.borrow().had_error {
        return;
    }
    // let scan_time = std::time::Instant::now();
    // println!("Scanning took: {:?}", scan_time.duration_since(start_time));

    // Parse the token stream
    let mut parser = Parser::new(scanner.tokens, Rc::clone(&error_handler));
    let statements = parser.parse();
    // let parse_time = std::time::Instant::now();
    // println!("Parsing took: {:?}", parse_time.duration_since(scan_time));

    // Execute the parsed statements
    let mut interpreter =
        interpreter::Interpreter::new(Rc::clone(&error_handler), Rc::clone(&environment));
    interpreter.interpret(statements);
    // let end_time = std::time::Instant::now();
    // println!("Execution took: {:?}", end_time.duration_since(parse_time));
}
