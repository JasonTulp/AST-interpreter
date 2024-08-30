use crate::token::{LiteralType, Token, TokenType};
use colored::Colorize;

#[derive(Debug, Clone)]
pub enum Error {
    SyntaxError(u32, String),
    ParseError(Token, String),
    /// Error type for runtime errors (line, message)
    RuntimeError(u32, String),
    ResolverError(Token, String),
    Return(LiteralType),
    Unknown,
}

pub struct ErrorHandler {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn report_error(&mut self, error: Error) {
        match error {
            Error::SyntaxError(line, message) => {
                self.had_error = true;
                eprintln!(
                    "[line {}] {} {}",
                    line,
                    "Syntax Error:".red().italic(),
                    message.red()
                );
            }
            Error::ParseError(token, message) => {
                self.had_error = true;
                eprintln!(
                    "[line {}] {} {}",
                    token.get_line(),
                    "Parse Error:".red().italic(),
                    message.red()
                )
            }
            Error::RuntimeError(line, message) => {
                self.had_runtime_error = true;
                eprintln!(
                    "[line {}] {} {}",
                    line,
                    "Runtime Error:".red().italic(),
                    message.red()
                )
            }
            Error::ResolverError(token, message) => {
                eprintln!(
                    "[line {}] {} {}",
                    token.get_line(),
                    "Resolver Error:".red().italic(),
                    message.red()
                )
            }
            Error::Return(literal) => {
                // No need to throw an error
                return;
            }
            Error::Unknown => {
                self.had_error = true;
                eprintln!("{}", "An unknown error occurred. Sorry :(".red())
            }
        }
        eprintln!("{}", "(╯°□°)╯︵ ɹoɹɹƎ".red().bold());
    }
}

// Macro to simplify error handling from within structs
#[macro_export]
macro_rules! error {
    ($self:expr, $error:expr) => {
        $self.error_handler.borrow_mut().report_error($error)
    };
}
