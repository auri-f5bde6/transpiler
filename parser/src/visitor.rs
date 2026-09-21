use crate::Statement;
use crate::ast::{
    AssignStatement, BlockStatement, BooleanLiteral, Expression, ForLoopStatement,
    FunctionCallExpression, Identifier, IfStatement, InfixExpression, IntegerLiteral,
    PrefixExpression, ProcedureStatement, ReturnStatement, StringLiteral, WhileStatement,
};
use lexer::token::TokenType;

pub trait Visitor<T> {
    fn visit_expression(&mut self, expression: &Expression) -> T {
        match expression {
            Expression::Identifier(ident) => self.visit_identifier(ident),
            Expression::IntegerLiteral(literal) => self.visit_integer(literal),
            Expression::BooleanLiteral(literal) => self.visit_boolean(literal),
            Expression::StringLiteral(literal) => self.visit_string(literal),
            Expression::Prefix(prefix) => self.visit_prefix(prefix),
            Expression::Infix(infix) => self.visit_infix(infix),
            Expression::FunctionCall(call) => self.visit_function_call(call),
        }
    }
    fn visit_identifier(&mut self, ident: &Identifier) -> T;
    fn visit_integer(&mut self, literal: &IntegerLiteral) -> T;
    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> T;
    fn visit_string(&mut self, literal: &StringLiteral) -> T;
    fn visit_prefix(&mut self, prefix: &PrefixExpression) -> T;
    fn visit_infix(&mut self, infix: &InfixExpression) -> T {
        let left = self.visit_expression(&infix.left);
        let right = self.visit_expression(&infix.right);
        self.visit_infix_default(infix, left, right)
    }
    fn visit_infix_default(&mut self, infix: &InfixExpression, left: T, right: T) -> T {
        match infix.operator.token_type {
            TokenType::Plus => self.visit_plus(infix, left, right),
            TokenType::Minus => self.visit_minus(infix, left, right),
            TokenType::Divide => self.visit_divide(infix, left, right),
            TokenType::Multiply => self.visit_multiply(infix, left, right),
            TokenType::Modulo => todo!(),
            TokenType::Equal => self.visit_equal(infix, left, right),
            TokenType::NotEqual => self.visit_not_equal(infix, left, right),
            TokenType::Greater => self.visit_greater(infix, left, right),
            TokenType::GreaterEqual => self.visit_greater_equal(infix, left, right),
            TokenType::Lesser => self.visit_lesser(infix, left, right),
            TokenType::LesserEqual => self.visit_lesser_equal(infix, left, right),
            _ => todo!(),
        }
    }
    fn visit_function_call(&mut self, call: &FunctionCallExpression) -> T;

    fn visit_plus(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_minus(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_divide(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_multiply(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_equal(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_not_equal(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_greater(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_greater_equal(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_lesser(&mut self, infix: &InfixExpression, left: T, right: T) -> T;
    fn visit_lesser_equal(&mut self, infix: &InfixExpression, left: T, right: T) -> T;

    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Assign(stmt) => self.visit_assign(stmt),
            Statement::Return(stmt) => self.visit_return(stmt),
            Statement::Expression(expr) => _ = self.visit_expression(&expr.expression),
            Statement::Block(stmt) => self.visit_block(stmt),
            Statement::If(stmt) => self.visit_if(stmt),
            Statement::Procedure(stmt) => self.visit_procedure(stmt),
            Statement::While(stmt) => self.visit_while(stmt),
            Statement::ForLoop(stmt) => self.visit_for_loop(stmt),
        }
    }
    fn visit_assign(&mut self, stmt: &AssignStatement);
    fn visit_return(&mut self, stmt: &ReturnStatement);
    fn visit_block(&mut self, stmt: &BlockStatement);
    fn visit_if(&mut self, stmt: &IfStatement);
    fn visit_procedure(&mut self, stmt: &ProcedureStatement);
    fn visit_while(&mut self, stmt: &WhileStatement);
    fn visit_for_loop(&mut self, stmt: &ForLoopStatement);
}
