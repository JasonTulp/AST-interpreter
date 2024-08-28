use crate::error_handler::Error;
use crate::token::{LiteralType, Token};

pub trait Visitor {
    type Value;

    fn visit_assign(&mut self, assign: &Assign) -> Result<Self::Value, Error>;
    fn visit_binary(&mut self, binary: &Binary) -> Result<Self::Value, Error>;
    fn visit_call(&mut self, call: &Call) -> Result<Self::Value, Error>;
    fn visit_get(&mut self, get: &Get) -> Result<Self::Value, Error>;
    fn visit_grouping(&mut self, grouping: &Grouping) -> Result<Self::Value, Error>;
    fn visit_literal(&mut self, literal: &Literal) -> Result<Self::Value, Error>;
    fn visit_logical(&mut self, logical: &Logical) -> Result<Self::Value, Error>;
    fn visit_set(&mut self, set: &Set) -> Result<Self::Value, Error>;
    fn visit_super(&mut self, super_: &Super) -> Result<Self::Value, Error>;
    fn visit_this(&mut self, this: &This) -> Result<Self::Value, Error>;
    fn visit_unary(&mut self, unary: &Unary) -> Result<Self::Value, Error>;
    fn visit_variable(&mut self, variable: &Variable) -> Result<Self::Value, Error>;
}

#[derive(Debug)]
pub enum Expr {
    Assign(Box<Assign>),
    Binary(Box<Binary>),
    Call(Box<Call>),
    Get(Box<Get>),
    Grouping(Box<Grouping>),
    Literal(Literal),
    Logical(Box<Logical>),
    Set(Box<Set>),
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
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Logical(logical) => visitor.visit_logical(logical),
            Expr::Set(set) => visitor.visit_set(set),
            Expr::Super(super_) => visitor.visit_super(super_),
            Expr::This(this) => visitor.visit_this(this),
            Expr::Unary(unary) => visitor.visit_unary(unary),
            Expr::Variable(variable) => visitor.visit_variable(variable),
        }
    }
}

// Variable assignment
#[derive(Debug)]
pub struct Assign {
    pub name: Token,
    pub value: Expr,
}

// Binary expression
#[derive(Debug)]
pub struct Binary {
    pub left: Expr,
    pub operator: Token,
    pub right: Expr,
}

// Call Expression
#[derive(Debug)]
pub struct Call {
    pub callee: Expr,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

// Get Expression
#[derive(Debug)]
pub struct Get {
    pub object: Expr,
    pub name: Token,
}

// Grouping expression
#[derive(Debug)]
pub struct Grouping {
    pub expression: Expr,
}

// Literal expression
#[derive(Debug)]
pub struct Literal {
    pub value: LiteralType,
}

// Logical expression
#[derive(Debug)]
pub struct Logical {
    pub left: Expr,
    pub operator: Token,
    pub right: Expr,
}

// Set Expression
#[derive(Debug)]
pub struct Set {
    pub object: Expr,
    pub name: Token,
    pub value: Expr,
}

// Super Expression
#[derive(Debug)]
pub struct Super {
    pub keyword: Token,
    pub method: Token,
}

// This Expression
#[derive(Debug)]
pub struct This {
    pub keyword: Token,
}

// Unary expression
#[derive(Debug)]
pub struct Unary {
    pub operator: Token,
    pub right: Expr,
}

// Variable expression
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Token,
}
