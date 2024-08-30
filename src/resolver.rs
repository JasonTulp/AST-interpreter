use crate::{
	callable::FunctionType,
	error,
	error_handler::{Error, ErrorHandler},
	expressions,
	expressions::*,
	interpreter::Interpreter,
	statements,
	statements::*,
	token::{LiteralType, Token},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Resolver<'a> {
	pub interpreter: &'a mut Interpreter,
	scopes: Vec<HashMap<String, bool>>,
	current_function: FunctionType,
	// The error handler
	pub error_handler: Rc<RefCell<ErrorHandler>>,
}

impl Resolver<'_> {
	pub fn new(
		interpreter: &mut Interpreter,
		error_handler: Rc<RefCell<ErrorHandler>>,
	) -> Resolver {
		Resolver {
			interpreter,
			scopes: vec![],
			current_function: FunctionType::None,
			error_handler,
		}
	}

	/// Resolve a block of statements
	pub(crate) fn resolve_block(&mut self, statements: &Vec<Stmt>) {
		for statement in statements {
			if let Err(e) = self.resolve_stmt(statement) {
				error!(self, e);
			}
		}
	}

	/// Resolve an individual statement by calling the accept method
	fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), Error> {
		statement.accept(self)
	}

	/// Resolve an expression by calling accept
	fn resolve_expr(&mut self, expression: &Expr) -> Result<(), Error> {
		expression.accept(self)?;
		Ok(())
	}

	/// Begin a new scope, this pushes a Hashmap to the Vec pretending to be a stack
	fn begin_scope(&mut self) {
		self.scopes.push(HashMap::new());
	}

	/// End a scope by popping the last element from the stack
	fn end_scope(&mut self) {
		self.scopes.pop();
	}

	/// Declare a variable first, we need to check if we are inside the initializer so we split this
	/// into two steps.
	/// Declaration adds the variable to the inner most scope and shadows the outer one so we
	/// know that it exists, but the false says it's not ready to use yet
	fn declare(&mut self, name: &Token) -> Result<(), Error> {
		match self.scopes.last_mut() {
			None => Ok(()), // Empty scopes
			Some(scope) => {
				if scope.contains_key(&name.lexeme) {
					return Err(Error::ResolverError(
						name.to_owned(),
						"There's already a variable with this name in this scope.".to_string(),
					))
				}
				scope.insert(name.lexeme.to_owned(), false);
				Ok(())
			},
		}
	}

	/// This sets the variable in the same scope as above to true which shows that it is
	/// initialized and ready
	fn define(&mut self, name: &Token) {
		match self.scopes.last_mut() {
			None => return, // Empty scopes
			Some(scope) => scope.insert(name.lexeme.to_owned(), true),
		};
	}

	/// Resolve a local variable by checking the scopes from inner to outer
	fn resolve_local(&mut self, expr: &Expr, name: &Token) -> Result<(), Error> {
		for (i, scope) in self.scopes.iter_mut().rev().enumerate() {
			if scope.contains_key(&name.lexeme) {
				// Pass through the number of scopes between the variable and the innermost scope
				self.interpreter.resolve(expr.clone(), i as u64)?;
				return Ok(());
			}
		}
		Ok(())
	}

	fn resolve_function(
		&mut self,
		function: Function,
		function_type: FunctionType,
	) -> Result<(), Error> {
		let enclosing_function = self.current_function;
		self.current_function = function_type;
		self.begin_scope();
		for param in function.params.iter() {
			self.declare(param)?;
			self.define(param);
		}
		self.resolve_block(&function.body);
		self.end_scope();
		self.current_function = enclosing_function;
		Ok(())
	}
}

impl statements::Visitor for Resolver<'_> {
	fn visit_block(&mut self, block: &Block) -> Result<(), Error> {
		self.begin_scope();
		self.resolve_block(&block.statements);
		self.end_scope();
		Ok(())
	}

	fn visit_expression(&mut self, expression: &Expression) -> Result<(), Error> {
		self.resolve_expr(&expression.expression)?;
		Ok(())
	}

	fn visit_function(&mut self, function: &Function) -> Result<(), Error> {
		self.declare(&function.name)?;
		self.define(&function.name);
		self.resolve_function(function.clone(), FunctionType::Function)?;
		Ok(())
	}

	fn visit_if(&mut self, if_stmt: &If) -> Result<(), Error> {
		self.resolve_expr(&if_stmt.condition)?;
		self.resolve_stmt(&if_stmt.then_branch)?;
		if let Some(else_branch) = &if_stmt.else_branch {
			self.resolve_stmt(else_branch)?;
		}
		Ok(())
	}

	fn visit_print(&mut self, print: &Print) -> Result<(), Error> {
		self.resolve_expr(&print.expression)?;
		Ok(())
	}

	fn visit_return(&mut self, return_stmt: &Return) -> Result<(), Error> {
		if self.current_function == FunctionType::None {
			return Err(Error::ResolverError(
				return_stmt.keyword.to_owned(),
				"Can't return from top-level code.".to_string(),
			));
		}
		if let Some(value) = &return_stmt.value {
			self.resolve_expr(value)?;
		}
		Ok(())
	}

	fn visit_variable(&mut self, variable: &statements::Variable) -> Result<(), Error> {
		self.declare(&variable.name)?;
		if let Some(initializer) = &variable.initializer {
			self.resolve_expr(initializer)?;
		}
		self.define(&variable.name);
		Ok(())
	}

	fn visit_while(&mut self, while_stmt: &While) -> Result<(), Error> {
		self.resolve_expr(&while_stmt.condition)?;
		self.resolve_stmt(&while_stmt.body)?;
		Ok(())
	}
}

impl expressions::Visitor for Resolver<'_> {
	type Value = LiteralType;

	fn visit_assign(&mut self, assign: &Assign) -> Result<Self::Value, Error> {
		self.resolve_expr(&assign.value)?;
		self.resolve_local(&Expr::Assign(Box::new(assign.clone())), &assign.name)?;
		Ok(LiteralType::Null)
	}

	fn visit_binary(&mut self, binary: &Binary) -> Result<Self::Value, Error> {
		self.resolve_expr(&binary.left)?;
		self.resolve_expr(&binary.right)?;
		Ok(LiteralType::Null)
	}

	fn visit_call(&mut self, call: &Call) -> Result<Self::Value, Error> {
		self.resolve_expr(&call.callee)?;
		call.arguments.iter().for_each(|arg| self.resolve_expr(arg).unwrap());
		Ok(LiteralType::Null)
	}

	fn visit_get(&mut self, get: &Get) -> Result<Self::Value, Error> {
		todo!();
	}

	fn visit_grouping(&mut self, grouping: &Grouping) -> Result<Self::Value, Error> {
		self.resolve_expr(&grouping.expression)?;
		Ok(LiteralType::Null)
	}

	fn visit_array(&mut self, array: &Array) -> Result<Self::Value, Error> {
		array.values.iter().for_each(|val| self.resolve_expr(val).unwrap());
		Ok(LiteralType::Null)
	}

	fn visit_index(&mut self, index: &Index) -> Result<Self::Value, Error> {
		self.resolve_expr(&index.object)?;
		self.resolve_expr(&index.index)?;
		Ok(LiteralType::Null)
	}

	fn visit_literal(&mut self, _literal: &Literal) -> Result<Self::Value, Error> {
		Ok(LiteralType::Null)
	}

	fn visit_logical(&mut self, logical: &Logical) -> Result<Self::Value, Error> {
		self.resolve_expr(&logical.left)?;
		self.resolve_expr(&logical.right)?;
		Ok(LiteralType::Null)
	}

	fn visit_set(&mut self, set: &Set) -> Result<Self::Value, Error> {
		todo!()
	}

	fn visit_super(&mut self, super_: &Super) -> Result<Self::Value, Error> {
		todo!()
	}

	fn visit_this(&mut self, this: &This) -> Result<Self::Value, Error> {
		todo!()
	}

	fn visit_unary(&mut self, unary: &Unary) -> Result<Self::Value, Error> {
		self.resolve_expr(&unary.right)?;
		Ok(LiteralType::Null)
	}

	fn visit_variable(&mut self, variable: &expressions::Variable) -> Result<Self::Value, Error> {
		if let Some(scope) = self.scopes.last() {
			if scope.get(&variable.name.lexeme) == Some(&false) {
				return Err(Error::ResolverError(
					variable.name.to_owned(),
					"Can't read local variables in its own initializer.".to_string(),
				));
			}
		};
		self.resolve_local(&Expr::Variable(variable.clone()), &variable.name)?;
		Ok(LiteralType::Null)
	}
}
