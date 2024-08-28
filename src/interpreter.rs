use crate::environment::Environment;
use crate::error;
use crate::error_handler::{Error, ErrorHandler};
use crate::expressions::*;
use crate::statements::*;
use crate::token::{LiteralType, Token, TokenType};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    // Environment stores and retrieves variables
    environment: Rc<RefCell<Environment>>,
    error_handler: Rc<RefCell<ErrorHandler>>,
}

impl Interpreter {
    pub fn new(
        error_handler: Rc<RefCell<ErrorHandler>>,
        environment: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            environment,
            error_handler,
        }
    }

    /// Interpret a list of statements,
    /// This is the main entry point for the interpreter
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for stmt in statements {
            if let Err(e) = self.execute(&stmt) {
                error!(self, e);
            }
        }
    }

    /// Execute a statement
    fn execute(&mut self, stmt: &Stmt) -> Result<(), Error> {
        stmt.accept(self)
    }

    /// Evaluate an expression
    fn evaluate(&mut self, expr: &Expr) -> Result<LiteralType, Error> {
        expr.accept(self)
    }

    // Execute a block of statements, throwing an error if one occurs
    fn execute_block(&mut self, statements: &Vec<Stmt>, environment: Rc<RefCell<Environment>>) {
        let previous = self.environment.clone();
        self.environment = environment;
        for stmt in statements {
            if let Err(e) = self.execute(stmt) {
                // Throw the error without exiting as we want to return to the previous environment
                error!(self, e);
            }
        }
        self.environment = previous;
    }
}

/// Statement Visitor will visit all types of statements
/// ^ Lol, what a nothing statement
impl crate::statements::Visitor for Interpreter {
    fn visit_block(&mut self, block: &Block) -> Result<(), Error> {
        self.execute_block(
            &block.statements,
            Rc::new(RefCell::new(Environment::new(Some(
                self.environment.clone(),
            )))),
        );
        Ok(())
    }

    fn visit_expression(&mut self, expression: &Expression) -> Result<(), Error> {
        let _ = self.evaluate(&expression.expression)?;
        Ok(())
    }

    fn visit_function(&mut self, function: &Function) -> Result<(), Error> {
        todo!()
    }

    fn visit_if(&mut self, if_stmt: &If) -> Result<(), Error> {
        if self.evaluate(&if_stmt.condition)?.is_truthy() {
            self.execute(&if_stmt.then_branch)?
        } else if let Some(else_branch) = &if_stmt.else_branch {
            self.execute(else_branch)?
        }
        Ok(())
    }

    fn visit_print(&mut self, print: &Print) -> Result<(), Error> {
        let value: String = self.evaluate(&print.expression)?.into();
        println!("{}", value);
        Ok(())
    }

    fn visit_return(&mut self, return_stmt: &Return) -> Result<(), Error> {
        todo!()
    }

    fn visit_variable(&mut self, variable: &crate::statements::Variable) -> Result<(), Error> {
        let value = if let Some(initializer) = &variable.initializer {
            self.evaluate(initializer)?
        } else {
            LiteralType::Null
        };
        self.environment
            .borrow_mut()
            .define(variable.name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_while(&mut self, while_stmt: &While) -> Result<(), Error> {
        while self.evaluate(&while_stmt.condition)?.is_truthy() {
            self.execute(&while_stmt.body)?;
        }
        Ok(())
    }
}

impl crate::expressions::Visitor for Interpreter {
    type Value = LiteralType;

    fn visit_assign(&mut self, assign: &Assign) -> Result<Self::Value, Error> {
        let value: LiteralType = self.evaluate(&assign.value)?;
        self.environment
            .borrow_mut()
            .assign(assign.name.clone(), value.clone())?;
        Ok(value)
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
            TokenType::Modulo => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Number(left_num % right_num))
            }
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
        let left = self.evaluate(&logical.left)?;

        if logical.operator.token_type == TokenType::Or {
            if left.is_truthy() {
                return Ok(left);
            }
        } else {
            if !left.is_truthy() {
                return Ok(left);
            }
        }

        self.evaluate(&logical.right)
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

    // Return a stored variable in the environment
    fn visit_variable(
        &mut self,
        variable: &crate::expressions::Variable,
    ) -> Result<Self::Value, Error> {
        self.environment.borrow().get(variable.name.clone())
    }
}
