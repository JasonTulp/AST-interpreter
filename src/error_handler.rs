use crate::token::{Token, TokenType};
use colored::Colorize;

pub enum Error {
    SyntaxError(u32, String),
    ParseError(Token, String),
    /// Error type for runtime errors (line, message)
    RuntimeError(u32, String),
    Return,
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

    pub fn error(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }

    pub fn report(&mut self, line: u32, location: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, location, message);
        self.had_error = true;
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
            Error::Return => {
                self.had_error = true;
                eprintln!("{}", "Return error".red())
            }
            Error::Unknown => {
                self.had_error = true;
                eprintln!("{}", "An unknown error occurred. Sorry :(".red())
            }
        }
        eprintln!("{}", "(╯°□°)╯︵ ɹoɹɹƎ".red().bold());
    }
}

// Macro to simplify error reporting from within structs
#[macro_export]
macro_rules! report_err {
    ($self:expr, $line:expr, $context:expr, $message:expr) => {
        $self
            .error_handler
            .borrow_mut()
            .report($line, $context, $message);
    };
}

// Macro to simplify error handling from within structs
#[macro_export]
macro_rules! error {
    ($self:expr, $token:expr, $message:expr) => {
        $self.error_handler.borrow_mut().error($token, $message);
    };
}
