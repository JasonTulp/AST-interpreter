use crate::{error_handler::Error, expressions::Expr, token::Token};

pub trait Visitor {
	fn visit_block(&mut self, block: &Block) -> Result<(), Error>;
	fn visit_class(&mut self, class: &Class) -> Result<(), Error>;
	fn visit_expression(&mut self, expression: &Expression) -> Result<(), Error>;
	fn visit_function(&mut self, function: &Function) -> Result<(), Error>;
	fn visit_if(&mut self, if_stmt: &If) -> Result<(), Error>;
	fn visit_print(&mut self, print: &Print) -> Result<(), Error>;
	fn visit_return(&mut self, return_stmt: &Return) -> Result<(), Error>;
	fn visit_variable(&mut self, variable: &Variable) -> Result<(), Error>;
	fn visit_while(&mut self, while_stmt: &While) -> Result<(), Error>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
	Block(Box<Block>),
	Class(Class),
	Expression(Expression),
	Function(Function),
	If(Box<If>),
	Print(Print),
	Return(Return),
	Variable(Variable),
	While(Box<While>),
}

impl Stmt {
	pub fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), Error> {
		match self {
			Stmt::Block(block) => visitor.visit_block(block),
			Stmt::Class(class) => visitor.visit_class(class),
			Stmt::Expression(expression) => visitor.visit_expression(expression),
			Stmt::Function(function) => visitor.visit_function(function),
			Stmt::If(if_stmt) => visitor.visit_if(if_stmt),
			Stmt::Print(print) => visitor.visit_print(print),
			Stmt::Return(return_stmt) => visitor.visit_return(return_stmt),
			Stmt::Variable(variable) => visitor.visit_variable(variable),
			Stmt::While(while_stmt) => visitor.visit_while(while_stmt),
		}
	}
}

// Block statement
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
	pub statements: Vec<Stmt>,
}

// Class statement
#[derive(Debug, PartialEq, Clone)]
pub struct Class {
	pub name: Token,
	// pub superclass: Option<Variable>,
	pub methods: Vec<Function>,
}

// Expression statement
#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
	pub expression: Expr,
}

// Function statement
#[derive(Debug, PartialEq, Clone)]
pub struct Function {
	pub name: Token,
	pub params: Vec<Token>,
	pub body: Vec<Stmt>,
}

// If statement
#[derive(Debug, PartialEq, Clone)]
pub struct If {
	pub condition: Expr,
	pub then_branch: Stmt,
	pub else_branch: Option<Stmt>,
}

// Print statement
#[derive(Debug, PartialEq, Clone)]
pub struct Print {
	pub expression: Expr,
}

// Return statement
#[derive(Debug, PartialEq, Clone)]
pub struct Return {
	pub keyword: Token,
	pub value: Option<Expr>,
}

// Variable statement
#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
	pub name: Token,
	pub initializer: Option<Expr>,
}

// While statement
#[derive(Debug, PartialEq, Clone)]
pub struct While {
	pub condition: Expr,
	pub body: Stmt,
}
