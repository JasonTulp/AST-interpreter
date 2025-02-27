use crate::{
	error_handler::Error,
	token::{LiteralType, Token},
};

pub trait Visitor {
	type Value;

	fn visit_assign(&mut self, assign: &Assign) -> Result<Self::Value, Error>;
	fn visit_binary(&mut self, binary: &Binary) -> Result<Self::Value, Error>;
	fn visit_call(&mut self, call: &Call) -> Result<Self::Value, Error>;
	fn visit_get(&mut self, get: &Get) -> Result<Self::Value, Error>;
	fn visit_set(&mut self, set: &Set) -> Result<Self::Value, Error>;
	fn visit_grouping(&mut self, grouping: &Grouping) -> Result<Self::Value, Error>;
	fn visit_array(&mut self, array: &Array) -> Result<Self::Value, Error>;
	fn visit_index(&mut self, index: &Index) -> Result<Self::Value, Error>;
	fn visit_assign_index(&mut self, assign_index: &AssignIndex) -> Result<Self::Value, Error>;
	fn visit_literal(&mut self, literal: &Literal) -> Result<Self::Value, Error>;
	fn visit_logical(&mut self, logical: &Logical) -> Result<Self::Value, Error>;
	fn visit_super(&mut self, super_: &Super) -> Result<Self::Value, Error>;
	fn visit_this(&mut self, this: &This) -> Result<Self::Value, Error>;
	fn visit_unary(&mut self, unary: &Unary) -> Result<Self::Value, Error>;
	fn visit_variable(&mut self, variable: &Variable) -> Result<Self::Value, Error>;
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Expr {
	Assign(Box<Assign>),
	Binary(Box<Binary>),
	Call(Box<Call>),
	Get(Box<Get>),
	Set(Box<Set>),
	Grouping(Box<Grouping>),
	Array(Box<Array>),
	Index(Box<Index>),
	AssignIndex(Box<AssignIndex>),
	Literal(Literal),
	Logical(Box<Logical>),
	Super(Super),
	This(This),
	Unary(Box<Unary>),
	Variable(Variable),
}

impl Expr {
	pub fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<V::Value, Error> {
		match self {
			Expr::Assign(assign) => visitor.visit_assign(assign),
			Expr::Binary(binary) => visitor.visit_binary(binary),
			Expr::Call(call) => visitor.visit_call(call),
			Expr::Get(get) => visitor.visit_get(get),
			Expr::Set(set) => visitor.visit_set(set),
			Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
			Expr::Array(array) => visitor.visit_array(array),
			Expr::Index(index) => visitor.visit_index(index),
			Expr::AssignIndex(assign_index) => visitor.visit_assign_index(assign_index),
			Expr::Literal(literal) => visitor.visit_literal(literal),
			Expr::Logical(logical) => visitor.visit_logical(logical),
			Expr::Super(super_) => visitor.visit_super(super_),
			Expr::This(this) => visitor.visit_this(this),
			Expr::Unary(unary) => visitor.visit_unary(unary),
			Expr::Variable(variable) => visitor.visit_variable(variable),
		}
	}
}

// Variable assignment
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Assign {
	pub name: Token,
	pub value: Expr,
}

// Binary expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Binary {
	pub left: Expr,
	pub operator: Token,
	pub right: Expr,
}

// Call Expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Call {
	pub callee: Expr,
	pub paren: Token,
	pub arguments: Vec<Expr>,
}

// Get Expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Get {
	pub object: Expr,
	pub name: Token,
}

// Grouping expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Grouping {
	pub expression: Expr,
}

// Array Expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Array {
	pub values: Vec<Expr>,
}

// Index Expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Index {
	pub object: Expr,
	pub index: Expr,
}

// Variable assignment at index (For arrays)
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct AssignIndex {
	pub object: Expr,
	pub index: Expr,
	pub value: Expr,
}

// Literal expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Literal {
	pub value: LiteralType,
}

// Logical expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Logical {
	pub left: Expr,
	pub operator: Token,
	pub right: Expr,
}

// Set Expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Set {
	pub object: Expr,
	pub name: Token,
	pub value: Expr,
}

// Super Expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Super {
	pub keyword: Token,
	pub method: Token,
}

// This Expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct This {
	pub keyword: Token,
}

// Unary expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Unary {
	pub operator: Token,
	pub right: Expr,
}

// Variable expression
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Variable {
	pub name: Token,
}
