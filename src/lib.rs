use crate::{
	environment::{EnvRef, Environment},
	error_handler::ErrorHandler,
	interpreter::Interpreter,
	native_functions::NativeFunctions,
	resolver::Resolver,
};
use parser::Parser;
use scanner::Scanner;
use std::{cell::RefCell, fs::File, io, io::prelude::*, path::Path, process, rc::Rc};

mod callable;
mod environment;
mod error_handler;
mod expressions;
mod interpreter;
mod native_functions;
mod parser;
mod resolver;
mod scanner;
mod statements;
mod token;

// Start the REPL and handle incoming prompts
pub fn run_prompt() {
	let error_handler = Rc::new(RefCell::new(ErrorHandler::new()));
	let mut interpreter = Interpreter::new(Rc::clone(&error_handler));
	loop {
		print!("==> ");
		let mut line = String::new();
		let _ = io::stdout().flush();
		io::stdin().read_line(&mut line).unwrap();
		run(line.as_bytes().to_vec(), &mut interpreter);
		error_handler.borrow_mut().reset();
	}
}

// Load and run a file, reading the entire contents into a buffer
pub fn run_file(path: &str) -> io::Result<()> {
	let ext = Path::new(path).extension();
	match ext {
		Some(e) =>
			if e != "jasn" {
				println!("Invalid file extension. Please provide a .jasn file.");
				return Ok(());
			},
		None => {
			println!("Invalid file extension. Please provide a .jasn file.");
			return Ok(());
		},
	}
	let error_handler = Rc::new(RefCell::new(ErrorHandler::new()));
	let mut interpreter = Interpreter::new(Rc::clone(&error_handler));
	let mut file = File::open(path)?;
	let mut buffer = Vec::new();
	file.read_to_end(&mut buffer)?;
	run(buffer, &mut interpreter);
	Ok(())
}

// Actually run the interpreter
fn run(source: Vec<u8>, interpreter: &mut Interpreter) {
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
	// Stop if there was a parsing error
	if error_handler.borrow().had_error {
		return;
	}
	// let parse_time = std::time::Instant::now();
	// println!("Parsing took: {:?}", parse_time.duration_since(scan_time));

	// Execute the parsed statements
	let mut resolver = Resolver::new(interpreter, Rc::clone(&error_handler));
	resolver.resolve_block(&statements);

	if error_handler.borrow().had_error {
		return;
	}
	interpreter.interpret(statements);
	// let end_time = std::time::Instant::now();
	// println!("Execution took: {:?}", end_time.duration_since(parse_time));
}
