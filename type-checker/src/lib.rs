use lexer::position::TokenPosition;
use lexer::token::TokenType;
use parser::Visitor;
use parser::ast::{
    AssignStatement, BlockStatement, BooleanLiteral, Expression, ExpressionStatement,
    ForLoopStatement, FunctionCallExpression, Identifier, IfStatement, InfixExpression,
    IntegerLiteral, PrefixExpression, ProcedureStatement, ProgramRoot, ReturnStatement,
    StringLiteral, WhileStatement,
};
use std::collections::{HashMap, HashSet};
use std::iter::Extend;
use std::process::id;
use std::sync::LazyLock;

static EQUALITY_INFIX: LazyLock<HashMap<(Type, Type), Type>> = LazyLock::new(|| {
    HashMap::from([
        ((Type::Int, Type::Int), Type::Boolean),
        ((Type::Boolean, Type::Boolean), Type::Boolean),
        ((Type::String, Type::String), Type::Boolean),
    ])
});

static ORDERING_INFIX: LazyLock<HashMap<(Type, Type), Type>> =
    LazyLock::new(|| HashMap::from([((Type::Int, Type::Int), Type::Boolean)]));

static MATH_INFIX: LazyLock<HashMap<(Type, Type), Type>> =
    LazyLock::new(|| HashMap::from([((Type::Int, Type::Int), Type::Int)]));

static MATH_PREFIX: LazyLock<HashMap<Type, Type>> =
    LazyLock::new(|| HashMap::from([(Type::Int, Type::Int)]));

static NOT_PREFIX: LazyLock<HashMap<Type, Type>> =
    LazyLock::new(|| HashMap::from([(Type::Boolean, Type::Boolean)]));

#[derive(PartialEq, Clone, Debug)]
pub enum TypeErrors {
    SingleErr(TypeError),
    MultipleErrs(Vec<TypeError>),
}
impl TypeErrors {
    fn merge(&self, b: &TypeErrors) -> TypeErrors {
        let mut x = vec![];
        match self {
            TypeErrors::SingleErr(err) => x.push(err.clone()),
            TypeErrors::MultipleErrs(errs) => x.extend(errs.clone()),
        }
        match b {
            TypeErrors::SingleErr(err) => x.push(err.clone()),
            TypeErrors::MultipleErrs(errs) => x.extend(errs.clone()),
        }
        Self::MultipleErrs(x)
    }

    fn merge_with<T>(a: &mut Option<TypeErrors>, b: Result<T, TypeErrors>) {
        if let Err(b_err) = b {
            *a = match a {
                Some(errs) => Some(errs.merge(&b_err)),
                None => Some(b_err),
            };
        }
    }
}
impl From<TypeError> for TypeErrors {
    fn from(value: TypeError) -> Self {
        Self::SingleErr(value)
    }
}
trait SomeTypeErrorsToResult {
    fn get_result(self) -> Result<(), TypeErrors>;
}
impl SomeTypeErrorsToResult for Option<TypeErrors> {
    fn get_result(self) -> Result<(), TypeErrors> {
        match self {
            Some(errs) => Err(errs),
            None => Ok(()),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum TypeError {
    UndeclaredVariable {
        variable: String,
        range: TokenPosition,
    },
    UnapplicableTypeForInfix {
        operator: TokenType,
        left: Type,
        right: Type,
        range: TokenPosition,
    },
    UnapplicableTypeForPrefix {
        operator: TokenType,
        right: Type,
        range: TokenPosition,
    },
    ExpectedBooleanForCondition {
        got: Type,
        range: TokenPosition,
    },
}

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
pub enum Type {
    Int,
    Boolean,
    String,
    Other(String),
}

pub struct TypeChecker {
    env_map: HashMap<String, Type>,
    broken_var: HashSet<String>,
}
impl TypeChecker {
    pub fn new() -> TypeChecker {
        TypeChecker {
            env_map: HashMap::new(),
            broken_var: HashSet::new(),
        }
    }
    pub fn check(&mut self, root: &ProgramRoot) -> Option<TypeErrors> {
        let mut error: Option<TypeErrors> = None;
        for i in &root.0.body {
            TypeErrors::merge_with(&mut error, self.visit_statement(i));
        }
        error
    }
    fn infix_helper(
        &mut self,
        map: &HashMap<(Type, Type), Type>,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        match (left, right) {
            (Some(left), Some(right)) => {
                let left = match left {
                    Ok(t) => t,
                    Err(err) => return Some(Err(err)),
                };
                let right = match right {
                    Ok(t) => t,
                    Err(err) => return Some(Err(err)),
                };

                if let Some(t) = map.get(&(left.clone(), right.clone())) {
                    Some(Ok(t.clone()))
                } else {
                    Some(Err(TypeError::UnapplicableTypeForInfix {
                        operator: infix.operator.token_type.clone(),
                        left,
                        right,
                        range: TokenPosition::temp_default(),
                    }))
                }
            }
            _ => None,
        }
    }
    fn prefix_helper(
        &mut self,
        map: &HashMap<Type, Type>,
        prefix: &PrefixExpression,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        match right {
            Some(right) => {
                let right = match right {
                    Ok(t) => t,
                    Err(err) => return Some(Err(err)),
                };

                if let Some(t) = map.get(&(right.clone())) {
                    Some(Ok(t.clone()))
                } else {
                    Some(Err(TypeError::UnapplicableTypeForPrefix {
                        operator: prefix.operator.token_type.clone(),
                        right,
                        range: TokenPosition::temp_default(),
                    }))
                }
            }
            _ => None,
        }
    }
    fn check_cond(&mut self, condition: &Expression) -> Result<(), TypeErrors> {
        let cond = self.visit_expression(condition);
        let cond = match cond {
            Some(c) => c,
            None => return Ok(()),
        }?;
        if cond != Type::Boolean {
            Err(TypeError::ExpectedBooleanForCondition {
                got: cond,
                range: TokenPosition::temp_default(),
            }
                .into())
        } else {
            Ok(())
        }
    }
}

// When `E`, the inferred type of expression is `None`, it means it contains a variable whose type inference have failed previously,
// to avoiding spamming error
// (e.g. assigment having an invalid type on rhs, causing the variable to never be added to environment map and spam unknown variable error),
// we pretend the type of expression is correct when `None` is returned.
// This is fine because the program shouldn't be compiled due to the previous error anyway
impl Visitor<Result<(), TypeErrors>, Option<Result<Type, TypeError>>> for TypeChecker {
    fn visit_identifier(&mut self, ident: &Identifier) -> Option<Result<Type, TypeError>> {
        if (self.broken_var.contains(&ident.value)) {
            None
        } else if let Some(t) = self.env_map.get(&ident.value).cloned() {
            Some(Ok(t))
        } else {
            Some(Err(TypeError::UndeclaredVariable {
                range: TokenPosition::temp_default(),
                variable: ident.value.clone(),
            }))
        }
    }

    fn visit_integer(&mut self, literal: &IntegerLiteral) -> Option<Result<Type, TypeError>> {
        Some(Ok(Type::Int))
    }

    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> Option<Result<Type, TypeError>> {
        Some(Ok(Type::Boolean))
    }

    fn visit_string(&mut self, literal: &StringLiteral) -> Option<Result<Type, TypeError>> {
        Some(Ok(Type::String))
    }

    fn visit_function_call(
        &mut self,
        call: &FunctionCallExpression,
    ) -> Option<Result<Type, TypeError>> {
        todo!()
    }

    fn visit_addition(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*MATH_INFIX, infix, left, right)
    }

    fn visit_subtraction(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*MATH_INFIX, infix, left, right)
    }

    fn visit_division(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*MATH_INFIX, infix, left, right)
    }

    fn visit_multiplication(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*MATH_INFIX, infix, left, right)
    }

    fn visit_equal(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*EQUALITY_INFIX, infix, left, right)
    }

    fn visit_not_equal(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*EQUALITY_INFIX, infix, left, right)
    }

    fn visit_greater(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*ORDERING_INFIX, infix, left, right)
    }

    fn visit_greater_equal(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*ORDERING_INFIX, infix, left, right)
    }

    fn visit_lesser(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*ORDERING_INFIX, infix, left, right)
    }

    fn visit_lesser_equal(
        &mut self,
        infix: &InfixExpression,
        left: Option<Result<Type, TypeError>>,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.infix_helper(&*ORDERING_INFIX, infix, left, right)
    }

    fn visit_positive(
        &mut self,
        prefix: &PrefixExpression,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.prefix_helper(&*MATH_PREFIX, prefix, right)
    }

    fn visit_negation(
        &mut self,
        prefix: &PrefixExpression,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.prefix_helper(&*MATH_PREFIX, prefix, right)
    }

    fn visit_not(
        &mut self,
        prefix: &PrefixExpression,
        right: Option<Result<Type, TypeError>>,
    ) -> Option<Result<Type, TypeError>> {
        self.prefix_helper(&*NOT_PREFIX, prefix, right)
    }

    fn visit_assign(&mut self, stmt: &AssignStatement) -> Result<(), TypeErrors> {
        let inferred_type = match self.visit_expression(&stmt.value) {
            Some(r) => r,
            None => return Ok(()),
        };
        match inferred_type {
            Ok(t) => {
                self.env_map.insert(stmt.variable.value.clone(), t);
                Ok(())
            }
            Err(err) => {
                self.broken_var.insert(stmt.variable.value.clone());
                Err(err.into())
            }
        }
    }

    fn visit_return(&mut self, stmt: &ReturnStatement) -> Result<(), TypeErrors> {
        todo!()
    }

    fn visit_expression_statement(&mut self, expr: &ExpressionStatement) -> Result<(), TypeErrors> {
        match self.visit_expression(&expr.expression) {
            Some(result) => result.map_err(Into::into).map(|_| ()),
            None => Ok(()),
        }
    }

    fn visit_block(&mut self, stmt: &BlockStatement) -> Result<(), TypeErrors> {
        let mut errs = None;
        for i in &stmt.body {
            TypeErrors::merge_with(&mut errs, self.visit_statement(&i));
        }
        errs.get_result()
    }

    fn visit_if(&mut self, stmt: &IfStatement) -> Result<(), TypeErrors> {
        let mut errs = None;
        TypeErrors::merge_with(&mut errs, self.check_cond(&stmt.condition));
        TypeErrors::merge_with(&mut errs, self.visit_block(&stmt.consequence));
        TypeErrors::merge_with(&mut errs, self.visit_block(&stmt.alternitive));
        errs.get_result()
    }

    fn visit_procedure(&mut self, stmt: &ProcedureStatement) -> Result<(), TypeErrors> {
        todo!()
    }

    fn visit_while(&mut self, stmt: &WhileStatement) -> Result<(), TypeErrors> {
        let mut errs = None;
        TypeErrors::merge_with(&mut errs, self.check_cond(&stmt.condition));
        TypeErrors::merge_with(&mut errs, self.visit_block(&stmt.body));
        errs.get_result()
    }

    fn visit_for_loop(&mut self, stmt: &ForLoopStatement) -> Result<(), TypeErrors> {
        todo!()
    }
}
