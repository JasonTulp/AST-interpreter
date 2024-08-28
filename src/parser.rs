use crate::error;
use crate::error_handler::{Error, ErrorHandler};
use crate::expressions::*;
use crate::token::{LiteralType, Token, TokenType};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    error_handler: Rc<RefCell<ErrorHandler>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, error_handler: Rc<RefCell<ErrorHandler>>) -> Self {
        Self {
            tokens,
            current: 0,
            error_handler,
        }
    }

    // Method called to parse the tokens
    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(e) => {
                self.error_handler.borrow_mut().report_error(e);
                None
            }
        }
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.equality()
    }

    // Not equal and equal
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(Binary {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    // Greater than, greater than or equal, less than, less than or equal
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(Binary {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    // Addition and subtraction
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(Binary {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    // Multiplication and division
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(Binary {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    // Unary negation
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Box::new(Unary { operator, right })));
        }

        self.primary()
    }

    // Primary expression
    fn primary(&mut self) -> Result<Expr, Error> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal {
                value: LiteralType::Bool(false),
            }));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal {
                value: LiteralType::Bool(true),
            }));
        }
        if self.match_token(&[TokenType::Null]) {
            return Ok(Expr::Literal(Literal {
                value: LiteralType::Empty,
            }));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(Literal {
                value: self.previous().literal,
            }));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(Grouping { expression: expr })));
        }
        let token = self.peek();
        Err(Error::SyntaxError(token, "Expect expression.".to_string()))
    }

    // Since we have thrown an error, we need to synchronize the parser to the next
    // statement boundary. We will catch the exception there and continue parsing
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            // Advance until we meet a statement boundary
            match self.peek().token_type {
                TokenType::Class
                | TokenType::Funk
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    let _ = self.advance();
                }
            };
        }
    }

    // Check to see if the current token has any of the given types
    fn match_token(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    // returns true if the current token is of the given type. It never consumes the token,
    // only looks at it
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return &self.peek().token_type == token_type;
    }

    // Advance the current token and return the previous token
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    // Consume a token if it is the expected token, if not we throw an error
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, Error> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        return Err(Error::SyntaxError(self.peek(), message.to_string()));
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
