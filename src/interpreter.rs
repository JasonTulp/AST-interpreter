use crate::error_handler::{Error, ErrorHandler};
use crate::expressions::*;
use crate::report_err;
use crate::token::{LiteralType, Token, TokenType};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    error_handler: Rc<RefCell<ErrorHandler>>,
}

impl Interpreter {
    pub fn new(error_handler: Rc<RefCell<ErrorHandler>>) -> Self {
        Self { error_handler }
    }

    pub fn interpret(&mut self, expr: &Expr) {
        match self.evaluate(expr) {
            Ok(value) => {
                let value: String = value.into();
                println!("{}", value);
            }
            Err(e) => {
                self.error_handler.borrow_mut().report_error(e);
            }
        }
        // let _ = self
        //     .evaluate(expr)
        //     .map_err(|e| self.error_handler.borrow_mut().report_error(e));
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LiteralType, Error> {
        expr.accept(self)
    }
}

impl crate::expressions::Visitor for Interpreter {
    type Value = LiteralType;

    fn visit_assign(&mut self, assign: &Assign) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_binary(&mut self, binary: &Binary) -> Result<Self::Value, Error> {
        let left = self.evaluate(&binary.left)?;
        let right = self.evaluate(&binary.right)?;
        let line = binary.operator.line;

        match binary.operator.token_type {
            TokenType::BangEqual => Ok(LiteralType::Bool(left != right)),
            TokenType::EqualEqual => Ok(LiteralType::Bool(left == right)),
            TokenType::Greater => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Bool(left_num > right_num))
            }
            TokenType::GreaterEqual => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Bool(left_num >= right_num))
            }
            TokenType::Less => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Bool(left_num < right_num))
            }
            TokenType::LessEqual => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Bool(left_num <= right_num))
            }
            TokenType::Minus => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Number(left_num - right_num))
            }
            TokenType::Slash => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                if right_num == 0.0 {
                    return Err(Error::RuntimeError(line, "Division by zero.".to_string()));
                }
                Ok(LiteralType::Number(left_num / right_num))
            }
            TokenType::Star => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Number(left_num * right_num))
            }
            TokenType::Plus => match (left.clone(), right.clone()) {
                (LiteralType::Number(left_num), LiteralType::Number(right_num)) => {
                    Ok(LiteralType::Number(left_num + right_num))
                }
                (LiteralType::String(left_str), _) => {
                    let right_str: String = right.into();
                    Ok(LiteralType::String(format!("{}{}", left_str, right_str)))
                }
                (_, LiteralType::String(right_str)) => {
                    let left_str: String = left.into();
                    Ok(LiteralType::String(format!("{}{}", left_str, right_str)))
                }
                _ => {
                    return Err(Error::RuntimeError(line, "Invalid Operands.".to_string()));
                }
            },
            _ => {
                return Err(Error::RuntimeError(
                    line,
                    "Invalid binary operator.".to_string(),
                ));
            }
        }
    }

    fn visit_call(&mut self, call: &Call) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_get(&mut self, get: &Get) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> Result<Self::Value, Error> {
        self.evaluate(&grouping.expression)
    }

    fn visit_literal(&mut self, literal: &Literal) -> Result<Self::Value, Error> {
        Ok(literal.value.clone())
    }

    fn visit_logical(&mut self, logical: &Logical) -> Result<Self::Value, Error> {
        todo!()
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
        let right: Self::Value = self.evaluate(&unary.right)?;
        let line = unary.operator.line;

        match &unary.operator.token_type {
            TokenType::Minus => {
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Number(-right_num))
            }
            TokenType::Bang => Ok(LiteralType::Bool(!right.is_truthy())),
            _ => Err(Error::RuntimeError(
                line,
                "Invalid unary operator.".to_string(),
            )),
        }
    }

    fn visit_variable(&mut self, variable: &Variable) -> Result<Self::Value, Error> {
        todo!()
    }
}
