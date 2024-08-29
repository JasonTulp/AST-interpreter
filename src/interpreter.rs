use crate::callable::{Callable, JasnFunction, NativeFunction};
use crate::environment::{EnvRef, Environment};
use crate::error_handler::{Error, ErrorHandler};
use crate::expressions::*;
use crate::native_functions::*;
use crate::statements::*;
use crate::token::{LiteralType, Token, TokenType};
use crate::{callable, error};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    // The fixed global environment
    pub global: EnvRef,
    // The current environment we are in based on the current scope
    environment: EnvRef,
    error_handler: Rc<RefCell<ErrorHandler>>,
}

impl Interpreter {
    pub fn new(error_handler: Rc<RefCell<ErrorHandler>>, environment: EnvRef) -> Self {
        Self {
            global: environment.clone(),
            environment: environment.clone(),
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
    pub(crate) fn execute_block(&mut self, statements: &Vec<Stmt>, environment: EnvRef) {
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
        let jasn_function = JasnFunction {
            declaration: Box::new(function.clone()),
        };
        self.environment.borrow_mut().define(
            function.name.lexeme.clone(),
            LiteralType::Callable(Callable::Function(jasn_function)),
        );
        Ok(())
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
        let value: String = self.evaluate(&print.expression)?.to_string();
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
            TokenType::Minus | TokenType::MinusEqual | TokenType::MinusMinus => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Number(left_num - right_num))
            }
            TokenType::Slash | TokenType::SlashEqual => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                if right_num == 0.0 {
                    return Err(Error::RuntimeError(line, "Division by zero.".to_string()));
                }
                Ok(LiteralType::Number(left_num / right_num))
            }
            TokenType::Star | TokenType::StarEqual => {
                let left_num: f64 = left.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                let right_num: f64 = right.try_into().map_err(|e| Error::RuntimeError(line, e))?;
                Ok(LiteralType::Number(left_num * right_num))
            }
            TokenType::Plus | TokenType::PlusEqual | TokenType::PlusPlus => {
                match (left.clone(), right.clone()) {
                    (LiteralType::Number(left_num), LiteralType::Number(right_num)) => {
                        Ok(LiteralType::Number(left_num + right_num))
                    }
                    (LiteralType::String(left_str), _) => {
                        let right_str: String = right.to_string();
                        Ok(LiteralType::String(format!("{}{}", left_str, right_str)))
                    }
                    (_, LiteralType::String(right_str)) => {
                        let left_str: String = left.to_string();
                        Ok(LiteralType::String(format!("{}{}", left_str, right_str)))
                    }
                    _ => {
                        return Err(Error::RuntimeError(line, "Invalid Operands.".to_string()));
                    }
                }
            }
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
        let callee = self.evaluate(&call.callee)?;
        let mut arguments = Vec::new();
        for argument in &call.arguments {
            arguments.push(self.evaluate(argument)?);
        }
        let function = match callee {
            LiteralType::Callable(callable) => callable,
            _ => Err(Error::RuntimeError(
                call.paren.line,
                "Can only call functions and classes.".to_string(),
            ))?,
        };

        if arguments.len() as u8 != function.arity() {
            return Err(Error::RuntimeError(
                call.paren.line,
                format!(
                    "Expected {} arguments but found {}.",
                    function.arity(),
                    arguments.len()
                ),
            ));
        }

        function.call(self, arguments)
    }

    fn visit_get(&mut self, get: &Get) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> Result<Self::Value, Error> {
        self.evaluate(&grouping.expression)
    }

    fn visit_array(&mut self, array: &Array) -> Result<Self::Value, Error> {
        let mut values = Vec::new();
        for value in &array.values {
            values.push(self.evaluate(value)?);
        }
        Ok(LiteralType::Array(values))
    }

    fn visit_index(&mut self, index: &Index) -> Result<Self::Value, Error> {
        let array_value = self.evaluate(&index.object)?;
        let index_value = self.evaluate(&index.index)?;

        if let LiteralType::Array(elements) = array_value {
            if let LiteralType::Number(n) = index_value {
                let idx = n as usize;
                if idx < elements.len() {
                    return Ok(elements[idx].clone());
                } else {
                    return Err(Error::RuntimeError(
                        2,
                        "Array index out of bounds.".to_string(),
                    ));
                }
            } else {
                return Err(Error::RuntimeError(
                    2,
                    "Array index must be a number.".to_string(),
                ));
            }
        }

        Err(Error::RuntimeError(
            3,
            "Attempted to index a non-array value.".to_string(),
        ))
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
