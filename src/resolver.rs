use crate::error_handler::Error;
use crate::expressions::*;
use crate::interpreter::Interpreter;
use crate::statements::*;
use crate::token::{LiteralType, Token};
use crate::{expressions, statements};
use std::collections::HashMap;
use std::thread::scope;

pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Resolver {
        Resolver { interpreter }
    }

    /// Resolve a block of statements
    fn resolve_block(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    /// Resolve an individual statement by calling the accept method
    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), Error> {
        statement.accept(self)
    }

    /// Resolve an expression by calling accept
    fn resolve_expr(&mut self, expression: &Expr) -> Result<(), Error> {
        expression.accept(self)
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
    fn declare(&mut self, name: &Token) {
        match self.scopes.last().as_mut() {
            None => return, // Empty scopes
            Some(scope) => scope.insert(name.lexeme.to_owned(), false),
        };
    }

    /// This sets the variable in the same scope as above to true which shows that it is
    /// initialized and ready
    fn define(&mut self, name: &Token) {
        match self.scopes.last().as_mut() {
            None => return, // Empty scopes
            Some(scope) => scope.insert(name.lexeme.to_owned(), true),
        };
    }

    /// Resolve a local variable by checking the scopes from inner to outer
    fn resolve_local(&mut self, expr: &Expr, name: &Token) -> Result<(), Error> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                // Pass through the number of scopes between the variable and the innermost scope
                self.interpreter.resolve(expr, i)?;
                return Ok(());
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, function: Function) -> Result<(), Error> {
        self.begin_scope();
        function.params.iter().for_each(|param| {
            self.declare(param);
            self.define(param);
        });
        self.resolve(function.body);
        self.end_scope();
        Ok(())
    }
}

impl statements::Visitor for Resolver {
    fn visit_block(&mut self, block: &Block) -> Result<(), Error> {
        self.egin_scope();
        self.resolve_block(&block.statements)?;
        self.end_scope();
    }

    fn visit_expression(&mut self, expression: &Expression) -> Result<(), Error> {
        todo!()
    }

    fn visit_function(&mut self, function: &Function) -> Result<(), Error> {
        self.declare(&function.name);
        self.define(&function.name);
        self.resolve_function(function);
        Ok(())
    }

    fn visit_if(&mut self, if_stmt: &If) -> Result<(), Error> {
        todo!()
    }

    fn visit_print(&mut self, print: &Print) -> Result<(), Error> {
        todo!()
    }

    fn visit_return(&mut self, return_stmt: &Return) -> Result<(), Error> {
        todo!()
    }

    fn visit_variable(&mut self, variable: &statements::Variable) -> Result<(), Error> {
        self.declare(&variable.name);
        if let Some(initializer) = &variable.initializer {
            self.resolve_expr(initializer)?;
        }
        self.define(&variable.name);
        Ok(())
    }

    fn visit_while(&mut self, while_stmt: &While) -> Result<(), Error> {
        todo!()
    }
}

impl expressions::Visitor for Resolver {
    type Value = LiteralType;

    fn visit_assign(&mut self, assign: &Assign) -> Result<Self::Value, Error> {
        self.resolve(&assign.value)?;
        self.resolve_local(&Expr::Assign(Box::new(assign.clone())), &assign.name)?;
        Ok(LiteralType::Null)
    }

    fn visit_binary(&mut self, binary: &Binary) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_call(&mut self, call: &Call) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_get(&mut self, get: &Get) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_array(&mut self, array: &Array) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_index(&mut self, index: &Index) -> Result<Self::Value, Error> {
        todo!()
    }

    fn visit_literal(&mut self, literal: &Literal) -> Result<Self::Value, Error> {
        todo!()
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
        todo!()
    }

    fn visit_variable(&mut self, variable: &expressions::Variable) -> Result<Self::Value, Error> {
        if let Some(scope) = self.scopes.last() {
            if scope.get(&variable.name.lexeme) == false {
                Error::ResolverError(
                    variable.name.to_owned(),
                    "Can't read local variables in its own initializer.".to_string(),
                )?;
            }
        };
        self.resolve_local(variable, variable.name);
        Ok(LiteralType::Null)
    }
}
