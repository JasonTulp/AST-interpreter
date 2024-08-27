use std::cell::RefCell;
use std::rc::Rc;
use crate::error_handler::ErrorHandler;
use crate::expressions::*;
use crate::token::{LiteralType, Token, TokenType};

pub enum InterpreterError {
    Error(Token, String),
    Return,
    RuntimeError,
}

pub struct Interpreter {
    error_handler: Rc<RefCell<ErrorHandler>>,
}

impl Interpreter {
    pub fn new(error_handler: Rc<RefCell<ErrorHandler>>) -> Self {
        Self { error_handler }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LiteralType, InterpreterError> {
        expr.accept(self)
    }
}

impl crate::expressions::Visitor for Interpreter {
    type Value = LiteralType;
    type Error = InterpreterError;

    fn visit_assign(&mut self, assign: &Assign) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_binary(&mut self, binary: &Binary) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_call(&mut self, call: &Call) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_get(&mut self, get: &Get) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> Result<Self::Value, Self::Error> {
        self.evaluate(&grouping.expression)
    }

    fn visit_literal(&mut self, literal: &Literal) -> Result<Self::Value, Self::Error> {
        Ok(literal.value.clone())
    }

    fn visit_logical(&mut self, logical: &Logical) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_set(&mut self, set: &Set) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_super(&mut self, super_: &Super) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_this(&mut self, this: &This) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_unary(&mut self, unary: &Unary) -> Result<Self::Value, Self::Error> {
        let right: Self::Value = self.evaluate(&unary.right)?;

        match &unary.operator.token_type {
            TokenType::Minus => {
                match right {
                    Self::Value::Number(num) => Ok(LiteralType::Number(-num)),
                    _ => {
                        self.error_handler.borrow_mut().report(unary.operator.line, "", "Unary - operator can only be used with numbers.");
                        Err(Self::Error::RuntimeError)
                    }
                }
            }
            TokenType::Bang => Ok(LiteralTypes::Bool(!self.is_truthy(&right))),
            _ => {
                self.error_handler.borrow_mut().report(unary.operator.line, "", "Invalid unary operator.");
                Err(Self::Error::RuntimeError)
            }
        }
    }

    fn visit_variable(&mut self, variable: &Variable) -> Result<Self::Value, Self::Error> {
        todo!()
    }
}