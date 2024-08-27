mod scanner;
mod token;
mod expressions;
mod statements;
mod parser;
mod interpreter;
mod error_handler;

use std::{io, process};
use std::cell::RefCell;
use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use scanner::Scanner;
use parser::Parser;
use crate::error_handler::ErrorHandler;

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
    scanner.print_tokens();
    if error_handler.borrow().had_error {
        process::exit(65);
    }

    let mut parser = Parser::new(scanner.tokens, Rc::clone(&error_handler));
    let expr = parser.parse();
    if parser.had_error {
        process::exit(65);
    }
    println!("{:?}", expr);
}