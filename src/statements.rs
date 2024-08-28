use crate::error_handler::Error;
use crate::expressions::Expr;
use crate::token::Token;

pub trait Visitor {
    fn visit_block(&mut self, block: &Block) -> Result<(), Error>;
    // fn visit_class(&mut self, class: &Class) -> Result<(), Error>;
    fn visit_expression(&mut self, expression: &ExpressionStatement) -> Result<(), Error>;
    fn visit_function(&mut self, function: &Function) -> Result<(), Error>;
    fn visit_if(&mut self, if_stmt: &If) -> Result<(), Error>;
    fn visit_print(&mut self, print: &Print) -> Result<(), Error>;
    fn visit_return(&mut self, return_stmt: &Return) -> Result<(), Error>;
    fn visit_variable(&mut self, variable: &Variable) -> Result<(), Error>;
    fn visit_while(&mut self, while_stmt: &While) -> Result<(), Error>;
}

pub enum Stmt {
    Block(Block),
    // Class(Class),
    Expression(ExpressionStatement),
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
            // Stmt::Class(class) => visitor.visit_class(class),
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
pub struct Block {
    statements: Vec<Stmt>,
}

// // Class statement
// pub struct Class {
//     name: Token,
//     superclass: Option<Variable>,
//     methods: Vec<Function>,
// }

// Expression statement
pub struct ExpressionStatement {
    expression: Expr,
}

// Function statement
pub struct Function {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
}

// If statement
pub struct If {
    condition: Expr,
    then_branch: Stmt,
    else_branch: Option<Stmt>,
}

// Print statement
pub struct Print {
    expression: Expr,
}

// Return statement
pub struct Return {
    keyword: Token,
    value: Option<Expr>,
}

// Variable statement
pub struct Variable {
    name: Token,
    initializer: Option<Expr>,
}

// While statement
pub struct While {
    condition: Expr,
    body: Stmt,
}
