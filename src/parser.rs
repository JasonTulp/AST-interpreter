use crate::error_handler::{Error, ErrorHandler};
use crate::expressions::*;
use crate::statements::*;
use crate::token::{LiteralType, Token, TokenType};
use crate::{error, expressions, statements};
use std::cell::RefCell;
use std::rc::Rc;

/// The parser struct handles incoming token streams and converts them into statements and expressions
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

    /// Method called to parse the tokens
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    error!(self, e);
                    self.synchronize();
                }
            }
        }

        statements
    }

    fn declaration(&mut self) -> Result<Stmt, Error> {
        if self.match_token(&[TokenType::Var]) {
            return self.variable_declaration();
        }
        self.statement()
    }

    /// Parse a statement
    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(Box::new(Block {
                statements: self.block()?,
            })));
        }

        self.expression_statement()
    }

    /// Parse a variable declaration
    fn variable_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?;
        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.check_statement_end()?;
        Ok(Stmt::Variable(statements::Variable { name, initializer }))
    }

    /// Parse a print statement
    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expression = self.expression()?;
        self.check_statement_end()?;
        Ok(Stmt::Print(Print { expression }))
    }

    // Return a list of statements between curly braces.
    // Note, this returns a Vec<Stmt> instead of a Block as we will reuse this code for
    // function bodies
    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(statements)
    }

    /// Parse an expression statement
    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expression = self.expression()?;
        self.check_statement_end()?;
        Ok(Stmt::Expression(Expression { expression }))
    }

    // Check whether we are at the end of a statement. We accept both semi-colons and new lines
    fn check_statement_end(&mut self) -> Result<(), Error> {
        // Semi colon always ends a statement
        if self.check(&TokenType::Semicolon) {
            self.advance();
            return Ok(());
        }
        // If we are at a new line, we can end the statement
        let token: Token = self.peek();
        if self.previous().line < token.line {
            return Ok(());
        }
        Err(Error::ParseError(
            token,
            "Expected ';' or new line after expression.".to_string(),
        ))
    }

    /// Parse an expression
    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.equality()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            if let Expr::Variable(variable) = expr {
                return Ok(Expr::Assign(Box::new(Assign {
                    name: variable.name,
                    value,
                })));
            }
            return Err(Error::ParseError(
                equals,
                "Invalid assignment target.".to_string(),
            ));
        }
        Ok(expr)
    }

    /// Not equal and equal
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

    /// Greater than, greater than or equal, less than, less than or equal
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

    /// Addition and subtraction
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

    /// Multiplication and division
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

    /// Unary negation
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Box::new(Unary { operator, right })));
        }

        self.primary()
    }

    /// Primary expression
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
                value: LiteralType::Null,
            }));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(Literal {
                value: self.previous().literal,
            }));
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(expressions::Variable {
                name: self.previous(),
            }));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(Grouping { expression: expr })));
        }
        let token = self.peek();
        Err(Error::ParseError(token, "Expected expression.".to_string()))
    }

    /// Since we have thrown an error, we need to synchronize the parser to the next
    /// statement boundary. We will catch the exception there and continue parsing
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

    /// Check to see if the current token has any of the given types
    fn match_token(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// returns true if the current token is of the given type. It never consumes the token,
    /// only looks at it
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return &self.peek().token_type == token_type;
    }

    /// Advance the current token and return the previous token
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Consume a token if it is the expected token, if not we throw an error
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, Error> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        return Err(Error::ParseError(self.peek(), message.to_string()));
    }

    /// Are we at the end of the list of tokens
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    /// Get the next token we haven't consumed
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    /// Get the most recently consumed token
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
