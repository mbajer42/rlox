use crate::error::{LoxError, Result};
use crate::statement::{Expr, ExprId, Stmt};

use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq)]
enum FunctionType {
    None,
    Method,
    Function,
    Initializer,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub type Depth = u64;

struct Resolver<'a> {
    scopes: Vec<HashMap<&'a str, bool>>,
    expr_id_to_depth: HashMap<ExprId, Depth>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    fn new() -> Self {
        Self {
            scopes: Vec::new(),
            expr_id_to_depth: HashMap::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    fn resolve(&mut self, statements: &'a [Stmt]) -> Result<HashMap<ExprId, Depth>> {
        self.resolve_statements(statements)?;
        Ok(std::mem::take(&mut self.expr_id_to_depth))
    }

    fn resolve_statements(&mut self, stmts: &'a [Stmt]) -> Result<()> {
        for stmt in stmts {
            self.resolve_statement(stmt)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &'a Stmt) -> Result<()> {
        match stmt {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_statements(statements.as_ref())?;
                self.end_scope();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                self.define(name);
                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer)?;
                }
            }
            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                self.resolve_function(name, parameters, body, FunctionType::Function)?;
            }
            Stmt::Expression { expression } => {
                self.resolve_expression(expression)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch.as_ref())?;
                if let Some(stmt) = else_branch {
                    self.resolve_statement(stmt)?;
                }
            }
            Stmt::Print { expression } => self.resolve_expression(expression)?,
            Stmt::Return { value } => {
                if self.current_function == FunctionType::None {
                    return Err(LoxError::ResolverError(
                        "Cannot return from top-level code.",
                    ));
                }
                if let Some(value) = value {
                    if self.current_function == FunctionType::Initializer {
                        return Err(LoxError::ResolverError(
                            "Cannot return a value from an initializer.",
                        ));
                    }
                    self.resolve_expression(value)?;
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;
                self.declare(name);
                self.define(name);

                if let Some(superclass) = superclass {
                    if let Expr::Variable {
                        id: _,
                        name: superclass_name,
                    } = superclass.as_ref()
                    {
                        if name == superclass_name {
                            return Err(LoxError::ResolverError(
                                "A class cannot inherit from itself.",
                            ));
                        }
                    }

                    self.current_class = ClassType::SubClass;
                    self.resolve_expression(superclass)?;

                    self.begin_scope();
                    self.scopes
                        .last_mut()
                        .map(|scope| scope.insert("super", true));
                }
                self.begin_scope();
                self.scopes
                    .last_mut()
                    .map(|scope| scope.insert("this", true));

                for method in methods.as_ref() {
                    if let Stmt::Function {
                        name,
                        parameters,
                        body,
                    } = method
                    {
                        let function_type = if name == "init" {
                            FunctionType::Initializer
                        } else {
                            FunctionType::Method
                        };
                        self.resolve_function(name, parameters, body, function_type)?;
                    } else {
                        unreachable!()
                    }
                }

                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
            }
        };
        Ok(())
    }

    fn resolve_function(
        &mut self,
        name: &'a str,
        parameters: &'a Vec<String>,
        body: &'a [Stmt],
        function_type: FunctionType,
    ) -> Result<()> {
        self.declare(name);
        self.define(name);
        let enclosing_function = self.current_function;
        self.current_function = function_type;
        self.begin_scope();
        for param in parameters {
            self.declare(&param);
            self.define(&param);
        }
        self.resolve_statements(body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn resolve_expression(&mut self, expr: &'a Expr) -> Result<()> {
        match expr {
            Expr::Variable { id, name } => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get::<str>(name) == Some(&false) {
                        return Err(LoxError::ResolverError(
                            "Cannot read local variable in ints own initializer",
                        ));
                    }
                    self.resolve_local(*id, name);
                }
            }
            Expr::This { id, keyword } => {
                if self.current_class == ClassType::None {
                    return Err(LoxError::ResolverError(
                        "Cannot use 'this' outside of a class.",
                    ));
                }
                self.resolve_local(*id, keyword);
            }
            Expr::Super {
                id,
                keyword,
                method: _,
            } => {
                if self.current_class == ClassType::None {
                    return Err(LoxError::ResolverError(
                        "Cannot use 'super' outside of a class.",
                    ));
                }
                if self.current_class != ClassType::SubClass {
                    return Err(LoxError::ResolverError(
                        "Cannot use 'super' in a class with no superclass.",
                    ));
                }
                self.resolve_local(*id, keyword);
            }
            Expr::Assign { id, value, name } => {
                self.resolve_expression(value)?;
                self.resolve_local(*id, name);
            }
            Expr::Binary {
                left,
                token_type: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expr::Call { callee, arguments } => {
                self.resolve_expression(callee)?;
                for arg in arguments.as_ref() {
                    self.resolve_expression(arg)?;
                }
            }
            Expr::Get { object, name: _ } => {
                self.resolve_expression(object)?;
            }
            Expr::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expression(object)?;
                self.resolve_expression(value)?;
            }
            Expr::Grouping { expression } => {
                self.resolve_expression(expression)?;
            }
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expr::Unary {
                token_type: _,
                right,
            } => {
                self.resolve_expression(right)?;
            }
            Expr::Nil | Expr::Boolean(_) | Expr::Number(_) | Expr::String(_) => {}
        };
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &'a str) {
        self.scopes
            .last_mut()
            .map(|scope| scope.insert(name, false));
    }

    fn define(&mut self, name: &'a str) {
        self.scopes.last_mut().map(|scope| scope.insert(name, true));
    }

    fn resolve_local(&mut self, expr_id: ExprId, name: &'a str) {
        self.scopes
            .iter_mut()
            .rev()
            .enumerate()
            .find(|(_, scope)| scope.contains_key(name))
            .map(|(depth, _)| (expr_id, depth as u64))
            .map(|(expr_id, depth)| self.expr_id_to_depth.insert(expr_id, depth));
    }
}

pub fn resolve(statements: &[Stmt]) -> Result<HashMap<ExprId, Depth>> {
    let mut resolver = Resolver::new();
    resolver.resolve(statements)
}

#[cfg(test)]
mod tests {

    use super::{resolve, Depth};

    use crate::error::{LoxError, Result};
    use crate::lexer;
    use crate::parser;
    use crate::statement::ExprId;

    use std::collections::HashMap;

    fn scopes(source: &'static str) -> Result<HashMap<ExprId, Depth>> {
        let (tokens, lexer_errors) = lexer::lex(source);
        assert_eq!(lexer_errors.len(), 0);
        let (statements, parser_errors) = parser::parse(&tokens);
        assert_eq!(parser_errors.len(), 0);

        resolve(&statements)
    }

    #[test]
    fn invalid_return_statement() {
        let source = "return 42;";
        let scopes = scopes(source);
        assert_eq!(scopes.is_err(), true);
        assert_eq!(
            scopes.unwrap_err(),
            LoxError::ResolverError("Cannot return from top-level code.")
        );
    }

    #[test]
    fn valid_return_statement() {
        let source = r#"
            fun test() {
                return 42;
            }
        "#;
        let scopes = scopes(source);
        assert_eq!(scopes.is_ok(), true);
    }

    #[test]
    fn invalid_this() {
        let source = "var a = this;";
        let scopes = scopes(source);
        assert_eq!(scopes.is_err(), true);
        assert_eq!(
            scopes.unwrap_err(),
            LoxError::ResolverError("Cannot use 'this' outside of a class.")
        );
    }

    #[test]
    fn cannot_return_from_initializer() {
        let source = r#"
            class Foo {
                init() {
                    return "invalid";
                }
            }
        "#;
        let scopes = scopes(source);
        assert_eq!(scopes.is_err(), true);
        assert_eq!(
            scopes.unwrap_err(),
            LoxError::ResolverError("Cannot return a value from an initializer.")
        );
    }

    #[test]
    fn cannot_use_super_outside_of_class() {
        let source = "super.foo();";
        let scopes = scopes(source);
        assert_eq!(scopes.is_err(), true);
        assert_eq!(
            scopes.unwrap_err(),
            LoxError::ResolverError("Cannot use 'super' outside of a class.")
        );
    }

    #[test]
    fn cannot_use_super_in_non_subclass() {
        let source = r#"
            class Foo {
                foo() {
                    super.foo();
                }
            }
        "#;
        let scopes = scopes(source);
        assert_eq!(scopes.is_err(), true);
        assert_eq!(
            scopes.unwrap_err(),
            LoxError::ResolverError("Cannot use 'super' in a class with no superclass.")
        );
    }
}
