mod environment;
mod error_handler;
mod expressions;
mod interpreter;
mod parser;
mod scanner;
mod statements;
mod token;

use crate::environment::Environment;
use crate::error_handler::ErrorHandler;
use parser::Parser;
use scanner::Scanner;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
use std::{io, process};

// Start the REPL and handle incoming prompts
pub fn run_prompt() {
    let environment = Rc::new(RefCell::new(Environment::new(None)));
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
    let environment = Rc::new(RefCell::new(Environment::new(None)));
    println!("Running JASN in file mode: {}", path);
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    run(buffer, Rc::clone(&environment));
    Ok(())
}

// Actually run the interpreter
fn run(source: Vec<u8>, environment: Rc<RefCell<Environment>>) {
    // Create a re-usable error handler
    let error_handler = Rc::new(RefCell::new(ErrorHandler::new()));

    // Scan the input text and convert to a list of tokens
    let mut scanner = Scanner::new(source, Rc::clone(&error_handler));
    scanner.scan_tokens();
    // scanner.print_tokens();
    // We don't want to continue if there was an error scanning the tokens
    if error_handler.borrow().had_error {
        return;
    }

    // Parse the token stream
    let mut parser = Parser::new(scanner.tokens, Rc::clone(&error_handler));
    let statements = parser.parse();
    // print!("{:?}", statements);

    // Execute the parsed statements
    let mut interpreter =
        interpreter::Interpreter::new(Rc::clone(&error_handler), Rc::clone(&environment));
    interpreter.interpret(statements);
}
