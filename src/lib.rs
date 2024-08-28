mod error_handler;
mod expressions;
mod interpreter;
mod parser;
mod scanner;
mod statements;
mod token;

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
    loop {
        print!("==> ");
        let mut line = String::new();
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut line).unwrap();
        run(line.as_bytes().to_vec());
    }
}

// Load and run a file, reading the entire contents into a buffer
pub fn run_file(path: &str) -> io::Result<()> {
    println!("Running JASN in file mode: {}", path);
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    run(buffer);
    Ok(())
}

// Actually run the interpreter
fn run(source: Vec<u8>) {
    let mut error_handler = Rc::new(RefCell::new(ErrorHandler::new()));
    let mut scanner = Scanner::new(source, Rc::clone(&error_handler));
    scanner.scan_tokens();
    // scanner.print_tokens();
    if error_handler.borrow().had_error {
        return;
    }

    let mut parser = Parser::new(scanner.tokens, Rc::clone(&error_handler));
    let expr = parser.parse();
    if let Some(expression) = expr {
        let mut interpreter = interpreter::Interpreter::new(Rc::clone(&error_handler));
        interpreter.interpret(&expression);
    }
    // if error_handler.borrow().had_error {
    //     process::exit(65);
    // }

    // if error_handler.borrow().had_runtime_error {
    //     process::exit(70);
    // }
}
