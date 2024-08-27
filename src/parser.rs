use crate::token::{Token, TokenType, LiteralType};
use crate::expressions::*;

#[derive(Debug)]
pub enum ParseError {
    SyntaxError(String),
    ParseError,
}


pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub had_error: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            had_error: false,
        }
    }

    // Method called to parse the tokens
    pub fn parse(&mut self) -> Expr {
        self.expression().expect("Error parsing expression")
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    // Not equal and equal
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
        }

        Ok(expr)
    }

    // Greater than, greater than or equal, less than, less than or equal
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_token(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
        }

        Ok(expr)
    }

    // Addition and subtraction
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
        }

        Ok(expr)
    }

    // Multiplication and division
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
        }

        Ok(expr)
    }

    // Unary negation
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Box::new(Unary { operator, right })));
        }

        self.primary()
    }

    // Primary expression
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal { value: LiteralType::Bool(false) }));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal { value: LiteralType::Bool(true) }));
        }
        if self.match_token(&[TokenType::Null]) {
            return Ok(Expr::Literal(Literal { value: LiteralType::Empty }));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(Literal { value: self.previous().literal }));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(Grouping { expression: expr })));
        }
        let token = self.peek();
        self.error(&token, "Expect expression.");
        Err(ParseError::SyntaxError("Expect expression.".to_string()))
    }

    // Since we have thrown an error, we need to synchronize the parser to the next
    // statement boundary. We will catch the exception there and continue parsing
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return
            }

            // Advance until we meet a statement boundary
            match self.peek().token_type {
                TokenType::Class |
                TokenType::Funk |
                TokenType::Var |
                TokenType::For |
                TokenType::If |
                TokenType::While |
                TokenType::Print |
                TokenType::Return => return,
                _ => { let _ = self.advance(); }
            };
        }
    }

    // Check to see if the current token has any of the given types
    fn match_token(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true
            }
        }
        false
    }

    // returns true if the current token is of the given type. It never consumes the token,
    // only looks at it
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false
        }
        return &self.peek().token_type == token_type
    }

    // Advance the current token and return the previous token
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    // Consume a token if it is the expected token, if not we throw an error
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError>{
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        self.error(&self.peek(), message);
        return Err(ParseError::SyntaxError(message.to_string()));
    }

    // Are we at the end of the list of tokens
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    // Get the next token we haven't consumed
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    // Get the most recently consumed token
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn error(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }

    fn report(&mut self, line: u32, location: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, location, message);
        self.had_error = true;
    }
}

// impl crate::expressions::Visitor for Parser {
//     type Value = u32;
//
//     fn visit_assign(&mut self, assign: &Assign) -> Result<Self::Value, InterpreterError> {
//         let value = self.expression();
//         match assign.value {
//             Expr::Variable(var) => {
//                 self.environment.assign_at(var.name.lexeme.clone(), value.clone(), var.name.line)
//             },
//             _ => Err(InterpreterError::new(assign.name.line, "Invalid assignment target."))
//         }
//     }
// }