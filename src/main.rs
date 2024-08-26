use std::{env, process};
use jasn::{run_prompt, run_file};

// Throw an error and exit the process from within the interpreter
fn handle_error(code: i32, err: &str) {
    eprintln!("{}", err);
    process::exit(code);
}

// Entry point for the Jasn AST Interpreter
fn main() {
    println!("Starting JASN-AST Interpreter...");
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]).map_err(|e| handle_error(64, &e.to_string())).unwrap(),
        _ => handle_error(64, "Usage: jasn [script]")
    }
}
