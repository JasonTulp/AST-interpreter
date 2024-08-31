use crate::{
	error,
	error_handler::{Error, ErrorHandler},
	expressions,
	expressions::*,
	statements,
	statements::*,
	token::{LiteralType, Token, TokenType},
};
use std::{cell::RefCell, rc::Rc};

/// The parser struct handles incoming token streams and converts them into statements and
/// expressions
pub struct Parser {
	tokens: Vec<Token>,
	current: usize,
	error_handler: Rc<RefCell<ErrorHandler>>,
}

impl Parser {
	pub fn new(tokens: Vec<Token>, error_handler: Rc<RefCell<ErrorHandler>>) -> Self {
		Self { tokens, current: 0, error_handler }
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
				},
			}
		}

		statements
	}

	/// ================================  STATEMENTS

	/// Parse a declaration
	fn declaration(&mut self) -> Result<Stmt, Error> {
		if self.match_token(&[TokenType::Var]) {
			return self.variable_declaration();
		}
		if self.match_token(&[TokenType::Funk]) {
			return Ok(Stmt::Function(self.function("function")?));
		}
		if self.match_token(&[TokenType::Class]) {
			return self.class_declaration();
		}
		self.statement()
	}

	/// Parse a statement
	fn statement(&mut self) -> Result<Stmt, Error> {
		if self.match_token(&[TokenType::For]) {
			return self.for_statement();
		}
		if self.match_token(&[TokenType::If]) {
			return self.if_statement();
		}
		if self.match_token(&[TokenType::Print]) {
			return self.print_statement();
		}
		if self.match_token(&[TokenType::Return]) {
			return self.return_statement();
		}
		if self.match_token(&[TokenType::While]) {
			return self.while_statement();
		}

		if self.match_token(&[TokenType::LeftBrace]) {
			return Ok(Stmt::Block(Box::new(Block { statements: self.block()? })));
		}

		self.expression_statement()
	}

	/// Parse a variable declaration
	fn variable_declaration(&mut self) -> Result<Stmt, Error> {
		let name = self.consume(TokenType::Identifier, "Expected variable name.")?;
		let initializer =
			if self.match_token(&[TokenType::Equal]) { Some(self.expression()?) } else { None };
		self.check_statement_end()?;
		Ok(Stmt::Variable(statements::Variable { name, initializer }))
	}

	/// Parse a function declaration
	fn function(&mut self, kind: &str) -> Result<Function, Error> {
		let name = self.consume(TokenType::Identifier, &format!("Expected {} name.", kind))?;
		self.consume(TokenType::LeftParen, &format!("Expected '(' after {} name.", kind))?;
		let mut parameters = Vec::new();
		if !self.check(&TokenType::RightParen) {
			loop {
				if parameters.len() > 255 {
					return Err(Error::ParseError(
						self.peek(),
						"Cannot have more than 255 parameters.".to_string(),
					));
				}
				parameters.push(self.consume(TokenType::Identifier, "Expected parameter name.")?);
				if !self.match_token(&[TokenType::Comma]) {
					break;
				}
			}
		}
		self.consume(TokenType::RightParen, "Expected ')' after parameters.")?;
		self.consume(TokenType::LeftBrace, &format!("Expected '{{' before {} body.", kind))?;
		let body = self.block()?;

		Ok(statements::Function { name, params: parameters, body })
	}

	fn class_declaration(&mut self) -> Result<Stmt, Error> {
		let name = self.consume(TokenType::Identifier, "Expected class name.")?;
		self.consume(TokenType::LeftBrace, "Expected '{' before class body.")?;
		let mut methods = Vec::new();
		while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
			methods.push(self.function("method")?);
		}
		self.consume(TokenType::RightBrace, "Expected '}' after class body.")?;
		Ok(Stmt::Class(Class { name, methods }))
	}

	/// Parse an if statement
	fn if_statement(&mut self) -> Result<Stmt, Error> {
		self.consume(TokenType::LeftParen, "Expected '(' after 'if'.")?;
		let condition = self.expression()?;
		self.consume(TokenType::RightParen, "Expected ')' after if condition.")?;
		let then_branch = self.statement()?;
		let else_branch =
			if self.match_token(&[TokenType::Else]) { Some(self.statement()?) } else { None };
		Ok(Stmt::If(Box::new(If { condition, then_branch, else_branch })))
	}

	/// Parse a for statement. We essentially craft a while loop with a declaration and an iterator
	/// This is called "desugaring"
	fn for_statement(&mut self) -> Result<Stmt, Error> {
		self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;

		// Check for initializer, if it has been omitted, we will set it to None
		let initializer: Option<Stmt> = if self.match_token(&[TokenType::Semicolon]) {
			None
		} else if self.match_token(&[TokenType::Var]) {
			Some(self.variable_declaration()?)
		} else {
			Some(self.expression_statement()?)
		};

		// Check for condition, if it has been omitted, we will set it to None
		let condition = if !self.check(&TokenType::Semicolon) {
			self.expression()?
		} else {
			Expr::Literal(Literal { value: LiteralType::Bool(true) })
		};
		self.consume(TokenType::Semicolon, "Expected ';' after loop condition.")?;

		// Check for increment, if it has been omitted, we will set it to None
		let increment =
			if !self.check(&TokenType::RightParen) { Some(self.expression()?) } else { None };
		self.consume(TokenType::RightParen, "Expected ')' after for clauses.")?;

		// Parse the body of the for loop
		let mut body = self.statement()?;

		// If there is an increment, we will add it to the end of the body
		if let Some(increment) = increment {
			body = Stmt::Block(Box::new(Block {
				statements: vec![body, Stmt::Expression(Expression { expression: increment })],
			}));
		}

		// Add in the while condition
		body = Stmt::While(Box::new(While { condition, body }));

		// If there is an initializer, we will add it to the beginning of the body
		if let Some(initializer) = initializer {
			body = Stmt::Block(Box::new(Block { statements: vec![initializer, body] }));
		}

		Ok(body)
	}

	/// Parse a print statement
	fn print_statement(&mut self) -> Result<Stmt, Error> {
		let expression = self.expression()?;
		self.check_statement_end()?;
		Ok(Stmt::Print(Print { expression }))
	}

	fn return_statement(&mut self) -> Result<Stmt, Error> {
		let keyword = self.previous();
		let value = match self.check(&TokenType::Semicolon) {
			true => None,
			false => Some(self.expression()?),
		};
		self.check_statement_end()?;
		Ok(Stmt::Return(Return { keyword, value }))
	}

	/// Parse a while statement
	fn while_statement(&mut self) -> Result<Stmt, Error> {
		self.consume(TokenType::LeftParen, "Expected '(' after 'while'.")?;
		let condition = self.expression()?;
		self.consume(TokenType::RightParen, "Expected ')' after while condition.")?;
		let body = self.statement()?;
		Ok(Stmt::While(Box::new(While { condition, body })))
	}

	/// Return a list of statements between curly braces.
	/// Note, this returns a Vec<Stmt> instead of a Block as we will reuse this code for
	/// function bodies
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

	/// Check whether we are at the end of a statement. We accept both semi-colons and new lines
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
		Err(Error::ParseError(token, "Expected ';' or new line after expression.".to_string()))
	}

	/// ================================  EXPRESSIONS

	/// Parse an expression
	fn expression(&mut self) -> Result<Expr, Error> {
		self.assignment()
	}

	fn assignment(&mut self) -> Result<Expr, Error> {
		let expr = self.or()?;

		if self.match_token(&[
			TokenType::Equal,
			TokenType::PlusEqual,
			TokenType::MinusEqual,
			TokenType::StarEqual,
			TokenType::SlashEqual,
			TokenType::MinusMinus,
			TokenType::PlusPlus,
		]) {
			let operator = self.previous();
			let value = if operator.token_type == TokenType::Equal {
				self.or()?
			} else if operator.token_type == TokenType::MinusMinus ||
				operator.token_type == TokenType::PlusPlus
			{
				// Value is ++ or --, so we will operate on 1
				let right = Expr::Literal(Literal { value: LiteralType::Number(1.0) });
				Expr::Binary(Box::new(Binary {
					left: expr.clone(),
					operator: operator.clone(),
					right,
				}))
			} else {
				// Value is an operation as well as an assignment (+= 1 etc)
				let right = self.or()?;
				Expr::Binary(Box::new(Binary {
					left: expr.clone(),
					operator: operator.clone(),
					right,
				}))
			};

			// Check if we are assigning a variable or a property on an instance
			if let Expr::Variable(variable) = expr.clone() {
				return Ok(Expr::Assign(Box::new(Assign { name: variable.name, value })));
			} else if let Expr::Get(assign) = &expr {
				return Ok(Expr::Set(Box::new(Set {
					object: assign.object.clone(),
					name: assign.name.clone(),
					value,
				})));
			} else if let Expr::Index(index) = &expr {
				// return Ok(Expr::Set(Box::new(Set {
				// 	object: Expr::Index(index.clone()),
				// 	name: Token {
				// 		token_type: TokenType::String,
				// 		lexeme: "LEXEME".to_string(),
				// 		literal: LiteralType::String("Idk".to_string()),
				// 		line: operator.line,
				// 	},
				// 	value,
				// })));
				// return Ok(Expr::Set(Box::new(Set {
				// 	object: index.object.clone(),
				// 	name: index.index.clone(),
				// 	value,
				// })));
				return Err(Error::ParseError(
					operator,
					"Ruh roh, Jason hasn't implemented Array assignment.".to_string(),
				));
			}
			return Err(Error::ParseError(operator, "Invalid assignment target.".to_string()));
		}
		Ok(expr)
	}

	fn or(&mut self) -> Result<Expr, Error> {
		let mut expr = self.and()?;

		while self.match_token(&[TokenType::Or]) {
			let operator = self.previous();
			let right = self.and()?;
			expr = Expr::Logical(Box::new(Logical { left: expr, operator, right }));
		}

		Ok(expr)
	}

	fn and(&mut self) -> Result<Expr, Error> {
		let mut expr = self.equality()?;

		while self.match_token(&[TokenType::And]) {
			let operator = self.previous();
			let right = self.equality()?;
			expr = Expr::Logical(Box::new(Logical { left: expr, operator, right }));
		}

		Ok(expr)
	}

	/// Not equal and equal
	fn equality(&mut self) -> Result<Expr, Error> {
		let mut expr = self.comparison()?;

		while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
			let operator = self.previous();
			let right = self.comparison()?;
			expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
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
			expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
		}

		Ok(expr)
	}

	/// Addition and subtraction
	fn term(&mut self) -> Result<Expr, Error> {
		let mut expr = self.factor()?;

		while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
			let operator = self.previous();
			let right = self.factor()?;
			expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
		}

		Ok(expr)
	}

	/// Multiplication and division
	fn factor(&mut self) -> Result<Expr, Error> {
		let mut expr = self.modulo()?;

		while self.match_token(&[TokenType::Slash, TokenType::Star]) {
			let operator = self.previous();
			let right = self.unary()?;
			expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
		}

		Ok(expr)
	}

	fn modulo(&mut self) -> Result<Expr, Error> {
		let mut expr = self.unary()?;

		while self.match_token(&[TokenType::Modulo]) {
			let operator = self.previous();
			let right = self.unary()?;
			expr = Expr::Binary(Box::new(Binary { left: expr, operator, right }));
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

		self.call()
	}

	fn call(&mut self) -> Result<Expr, Error> {
		let mut expr = self.index_array()?;

		loop {
			if self.match_token(&[TokenType::LeftParen]) {
				expr = self.finish_call(expr)?;
			} else if self.match_token(&[TokenType::Dot]) {
				let name =
					self.consume(TokenType::Identifier, "Expected property name after '.'.")?;
				expr = Expr::Get(Box::new(Get { object: expr, name }));
			} else {
				break;
			}
		}

		Ok(expr)
	}

	fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
		let mut arguments = Vec::with_capacity(255);
		if !self.check(&TokenType::RightParen) {
			loop {
				if arguments.len() > 255 {
					return Err(Error::ParseError(
						self.peek(),
						"Cannot have more than 255 arguments.".to_string(),
					));
				}
				arguments.push(self.expression()?);
				if !self.match_token(&[TokenType::Comma]) {
					break;
				}
			}
		}
		let paren = self.consume(TokenType::RightParen, "Expected ')' after call arguments.")?;
		Ok(Expr::Call(Box::new(Call { callee, paren, arguments })))
	}

	fn index_array(&mut self) -> Result<Expr, Error> {
		let mut expr = self.primary()?;

		while self.match_token(&[TokenType::LeftSquare]) {
			let index = self.expression()?;
			self.consume(TokenType::RightSquare, "Expected ']' after index.")?;
			expr = Expr::Index(Box::new(Index { object: expr, index }));
		}

		Ok(expr)
	}

	/// Primary expression
	fn primary(&mut self) -> Result<Expr, Error> {
		if self.match_token(&[TokenType::LeftSquare]) {
			return self.array();
		}
		if self.match_token(&[TokenType::False]) {
			return Ok(Expr::Literal(Literal { value: LiteralType::Bool(false) }));
		}
		if self.match_token(&[TokenType::True]) {
			return Ok(Expr::Literal(Literal { value: LiteralType::Bool(true) }));
		}
		if self.match_token(&[TokenType::Null]) {
			return Ok(Expr::Literal(Literal { value: LiteralType::Null }));
		}

		if self.match_token(&[TokenType::Number, TokenType::String]) {
			return Ok(Expr::Literal(Literal { value: self.previous().literal }));
		}

		if self.match_token(&[TokenType::Identifier]) {
			return Ok(Expr::Variable(expressions::Variable { name: self.previous() }));
		}

		if self.match_token(&[TokenType::LeftParen]) {
			let expr = self.expression()?;
			self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
			return Ok(Expr::Grouping(Box::new(Grouping { expression: expr })));
		}
		let token = self.peek();
		Err(Error::ParseError(token, "Expected expression.".to_string()))
	}

	fn array(&mut self) -> Result<Expr, Error> {
		let mut elements = Vec::new();
		if !self.check(&TokenType::RightSquare) {
			loop {
				elements.push(self.expression()?);
				if !self.match_token(&[TokenType::Comma]) {
					break;
				}
			}
		}

		self.consume(TokenType::RightSquare, "Expected ']' after array elements.")?;
		Ok(Expr::Array(Box::new(Array { values: elements })))
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
				TokenType::Class |
				TokenType::Funk |
				TokenType::Var |
				TokenType::For |
				TokenType::If |
				TokenType::While |
				TokenType::Print |
				TokenType::Return => return,
				_ => {
					let _ = self.advance();
				},
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
