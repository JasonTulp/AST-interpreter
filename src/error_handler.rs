use crate::token::{Token, TokenType};

pub struct ErrorHandler {
    pub had_error: bool,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self { had_error: false }
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
}