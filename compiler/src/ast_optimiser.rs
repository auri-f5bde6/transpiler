use lexer::position::TokenPosition;
use parser::ast::{
    AssignStatement, BlockStatement, BooleanLiteral, Expression, ExpressionStatement,
    ForLoopStatement, FunctionCallExpression, Identifier, IfStatement, InfixExpression,
    IntegerLiteral, PrefixExpression, ProcedureStatement, ProgramRoot, ReturnStatement,
    StringLiteral, WhileStatement,
};
use parser::{Statement, Visitor};

macro_rules! infix_helper {
    ($infix:ident, $left:ident, $right:ident, $({$type:ident, $func:expr}),+) => {
        match ($left, $right) {
            $(
            (Expression::$type(l), Expression::$type(r)) => Expression::$type(Box::new($type {
                value: ($func)(l.value, r.value),
                token_position: TokenPosition::concat(l.token_position, r.token_position),
            })),
            ),+
            _ => $infix.clone().into(),
        }
    };
    ($infix:ident, $left:ident, $right:ident, $({$type:ident, $output:ident, $func:expr}),+) => {
        match ($left, $right) {
            $(
            (Expression::$type(l), Expression::$type(r)) => Expression::$output(Box::new($output {
                value: ($func)(l.value, r.value),
                token_position: TokenPosition::concat(l.token_position, r.token_position),
            })),
            )+
            _ => $infix.clone().into(),
        }
    };
}
macro_rules! prefix_helper {
    ($type:ident, $prefix:ident, $right:ident, $func:expr) => {
        match $right {
            Expression::$type(expr) => Expression::$type(Box::new($type {
                value: ($func)(expr.value),
                token_position: expr.token_position,
            })),
            _ => $prefix.clone().into(),
        }
    };
}

pub struct AstOptimiser {}
impl AstOptimiser {
    pub fn visit_root(&mut self, program_root: ProgramRoot) -> ProgramRoot {
        let mut a = vec![];
        for i in program_root.0.body {
            a.push(self.visit_statement(&i));
        }
        ProgramRoot(BlockStatement { body: a })
    }
}
impl Visitor<Statement, Expression> for AstOptimiser {
    fn visit_identifier(&mut self, ident: &Identifier) -> Expression {
        ident.clone().into()
    }

    fn visit_integer(&mut self, literal: &IntegerLiteral) -> Expression {
        literal.clone().into()
    }

    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> Expression {
        literal.clone().into()
    }

    fn visit_string(&mut self, literal: &StringLiteral) -> Expression {
        literal.clone().into()
    }

    fn visit_function_call(&mut self, call: &FunctionCallExpression) -> Expression {
        call.clone().into()
    }

    fn visit_addition(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, |l, r| l + r})
    }

    fn visit_subtraction(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, |l, r| l - r})
    }

    fn visit_division(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, |l, r| l / r})
    }

    fn visit_multiplication(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, |l, r| l * r})
    }

    fn visit_modulo(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, |l, r| i64::rem_euclid(l,r)})
    }

    fn visit_equal(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, BooleanLiteral, |l, r| l == r},{BooleanLiteral, BooleanLiteral, |l, r| l == r})
    }

    fn visit_not_equal(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, BooleanLiteral, |l, r| l != r},{BooleanLiteral, BooleanLiteral, |l, r| l != r})
    }

    fn visit_greater(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, BooleanLiteral, |l, r| l > r})
    }

    fn visit_greater_equal(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, BooleanLiteral, |l, r| l >= r})
    }

    fn visit_lesser(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, BooleanLiteral, |l, r| l < r})
    }

    fn visit_lesser_equal(
        &mut self,
        infix: &InfixExpression,
        left: Expression,
        right: Expression,
    ) -> Expression {
        infix_helper!(infix, left, right, {IntegerLiteral, BooleanLiteral, |l, r| l <= r})
    }

    fn visit_positive(&mut self, prefix: &PrefixExpression, right: Expression) -> Expression {
        prefix_helper!(IntegerLiteral, prefix, right, |x: i64| x)
    }

    fn visit_negation(&mut self, prefix: &PrefixExpression, right: Expression) -> Expression {
        prefix_helper!(IntegerLiteral, prefix, right, |x: i64| -x)
    }

    fn visit_not(&mut self, prefix: &PrefixExpression, right: Expression) -> Expression {
        prefix_helper!(BooleanLiteral, prefix, right, |x: bool| !x)
    }

    fn visit_assign(&mut self, stmt: &AssignStatement, value: Expression) -> Statement {
        Statement::Assign(Box::new(AssignStatement {
            variable: stmt.variable.clone(),
            value,
        }))
    }

    fn visit_return(&mut self, stmt: &ReturnStatement, expression: Expression) -> Statement {
        Statement::Return(Box::new(ReturnStatement { expression }))
    }

    fn visit_expression_statement(
        &mut self,
        expr: &ExpressionStatement,
        expression: Expression,
    ) -> Statement {
        Statement::Expression(Box::new(ExpressionStatement { expression }))
    }

    fn visit_block(&mut self, stmt: &BlockStatement) -> Statement {
        let mut body = vec![];
        for i in stmt.body.clone() {
            body.push(self.visit_statement(&i));
        }
        Statement::Block(Box::new(BlockStatement { body }))
    }

    fn visit_if(&mut self, stmt: &IfStatement, condition: Expression) -> Statement {
        Statement::If(Box::new(IfStatement {
            condition,
            consequence: stmt.consequence.clone(),
            alternitive: stmt.alternitive.clone(),
        }))
    }

    fn visit_procedure(&mut self, stmt: &ProcedureStatement) -> Statement {
        todo!()
    }

    fn visit_while(&mut self, stmt: &WhileStatement, condition: Expression) -> Statement {
        Statement::While(Box::new(WhileStatement {
            condition,
            body: stmt.body.clone(),
        }))
    }

    fn visit_for_loop(
        &mut self,
        stmt: &ForLoopStatement,
        initial_value: Expression,
        end_value: Expression,
    ) -> Statement {
        Statement::ForLoop(Box::new(ForLoopStatement {
            variable: stmt.variable.clone(),
            initial_value,
            end_value,
            body: stmt.body.clone(),
            next_variable: stmt.next_variable.clone(),
        }))
    }
}
