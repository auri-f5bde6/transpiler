use crate::Statement;
use crate::ast::{AssignStatement, BlockStatement, BooleanLiteral, Expression, ExpressionStatement, ForLoopStatement, FunctionCallExpression, Identifier, IfStatement, InfixExpression, IntegerLiteral, PrefixExpression, ProcedureStatement, ProgramRoot, ReturnStatement, StringLiteral, WhileStatement};
use lexer::token::TokenType;

pub trait Visitor<S, E> {
    fn visit_program_root(&mut self, root: &ProgramRoot) {
        for i in &root.0.body {
            self.visit_statement(i);
        }
    }

    fn visit_expression(&mut self, expression: &Expression) -> E {
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
    fn visit_identifier(&mut self, ident: &Identifier) -> E;
    fn visit_integer(&mut self, literal: &IntegerLiteral) -> E;
    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> E;
    fn visit_string(&mut self, literal: &StringLiteral) -> E;
    fn visit_prefix(&mut self, prefix: &PrefixExpression) -> E {
        let right = self.visit_expression(&prefix.right);
        self.visit_prefix_default(prefix, right)
    }
    fn visit_prefix_default(&mut self, prefix: &PrefixExpression, right: E) -> E {
        match prefix.operator.token_type {
            TokenType::Plus => self.visit_positive(prefix, right),
            TokenType::Not => self.visit_not(prefix, right),
            TokenType::Minus => self.visit_negation(prefix, right),
            _ => todo!(),
        }
    }
    fn visit_infix(&mut self, infix: &InfixExpression) -> E {
        let left = self.visit_expression(&infix.left);
        let right = self.visit_expression(&infix.right);
        self.visit_infix_default(infix, left, right)
    }
    fn visit_infix_default(&mut self, infix: &InfixExpression, left: E, right: E) -> E {
        match infix.operator.token_type {
            TokenType::Plus => self.visit_addition(infix, left, right),
            TokenType::Minus => self.visit_subtraction(infix, left, right),
            TokenType::Divide => self.visit_division(infix, left, right),
            TokenType::Multiply => self.visit_multiplication(infix, left, right),
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
    fn visit_function_call(&mut self, call: &FunctionCallExpression) -> E;

    fn visit_addition(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_subtraction(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_division(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_multiplication(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_equal(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_not_equal(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_greater(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_greater_equal(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_lesser(&mut self, infix: &InfixExpression, left: E, right: E) -> E;
    fn visit_lesser_equal(&mut self, infix: &InfixExpression, left: E, right: E) -> E;

    fn visit_positive(&mut self, prefix: &PrefixExpression, right: E) -> E;
    fn visit_negation(&mut self, prefix: &PrefixExpression, right: E) -> E;
    fn visit_not(&mut self, prefix: &PrefixExpression, right: E) -> E;

    fn visit_statement(&mut self, statement: &Statement) -> S {
        match statement {
            Statement::Assign(stmt) => self.visit_assign(stmt),
            Statement::Return(stmt) => self.visit_return(stmt),
            Statement::Expression(expr) => self.visit_expression_statement(expr),
            Statement::Block(stmt) => self.visit_block(stmt),
            Statement::If(stmt) => self.visit_if(stmt),
            Statement::Procedure(stmt) => self.visit_procedure(stmt),
            Statement::While(stmt) => self.visit_while(stmt),
            Statement::ForLoop(stmt) => self.visit_for_loop(stmt),
        }
    }
    fn visit_assign(&mut self, stmt: &AssignStatement) -> S;
    fn visit_return(&mut self, stmt: &ReturnStatement) -> S;
    fn visit_expression_statement(&mut self, expr: &ExpressionStatement) -> S;
    fn visit_block(&mut self, stmt: &BlockStatement) -> S;
    fn visit_if(&mut self, stmt: &IfStatement) -> S;
    fn visit_procedure(&mut self, stmt: &ProcedureStatement) -> S;
    fn visit_while(&mut self, stmt: &WhileStatement) -> S;
    fn visit_for_loop(&mut self, stmt: &ForLoopStatement) -> S;
}
