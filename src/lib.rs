mod scanner;
mod token;
mod expressions;
mod statements;
mod parser;

use std::{io, process};
use std::io::prelude::*;
use std::fs::File;
use scanner::Scanner;
use parser::Parser;

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
    // TODO Create error handler struct which prints all types of errors and keeps track of errors
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();
    scanner.print_tokens();
    if scanner.had_error {
        process::exit(65);
    }

    let mut parser = Parser::new(scanner.tokens);
    let expr = parser.parse();
    if parser.had_error {
        process::exit(65);
    }
    println!("{:?}", expr);
}