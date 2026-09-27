use compiler::compiler::Compiler;
use compiler::optimiser::Optimiser;
use lexer::lexer::Lexer;
use lexer::token::TokenType;
use parser::ast::{
    AssignStatement, BlockStatement, BooleanLiteral, ExpressionStatement, ForLoopStatement,
    FunctionCallExpression, Identifier, IfStatement, InfixExpression, IntegerLiteral,
    PrefixExpression, ProcedureStatement, ReturnStatement, StringLiteral, WhileStatement,
};
use parser::parser::Parser;
use parser::{PrettyPrint, Visitor};
use std::panic;
use type_checker::TypeChecker;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn init_panic_hook() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}


#[wasm_bindgen]
pub fn transpile(code: &str) -> String {
    let mut parser = Parser::from(&*code);
    match parser.parse() {
        Ok(ast) => {
            let errors = TypeChecker::new().check(&ast);
            if let Some(errors) = errors {
                format!("Type error: {:?}", errors)
            } else {
                let mut result = Compiler::compile(ast);
                result = Optimiser::new(result).optimise();
                format!("{}", result.get_program())
            }
        }
        Err(err) => {
            format!("Parser error: {:?}", err)
        }
    }
}

#[wasm_bindgen]
pub fn pretty_print(code: &str) -> String {
    let mut parser = Parser::from(&*code);
    match parser.parse() {
        Ok(ast) => {
            format!("{}", ast.pretty_print())
        }
        Err(err) => {
            format!("Parser error: {:?}", err)
        }
    }
}

#[wasm_bindgen]
pub fn get_tokens(code: &str) -> String {
    let mut buf = String::new();
    let mut lexer = Lexer::from(code);
    loop {
        match lexer.bump() {
            Ok(token) => {
                if token.token_type == TokenType::Eof {
                    buf.push_str(&*format!("{:?}", token.token_type));
                    break;
                }
                buf.push_str(&*format!("{:?}, ", token.token_type));
            }
            Err(err) => {
                return format!("Error: {:?}", err);
            }
        }
    }
    buf
}

struct Grapher {}

impl Visitor<(), ()> for Grapher {
    fn visit_identifier(&mut self, ident: &Identifier) -> () {
        todo!()
    }

    fn visit_integer(&mut self, literal: &IntegerLiteral) -> () {
        todo!()
    }

    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> () {
        todo!()
    }

    fn visit_string(&mut self, literal: &StringLiteral) -> () {
        todo!()
    }

    fn visit_function_call(&mut self, call: &FunctionCallExpression) -> () {
        todo!()
    }

    fn visit_addition(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_subtraction(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_division(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_multiplication(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_equal(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_not_equal(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_greater(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_greater_equal(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_lesser(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_lesser_equal(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_positive(&mut self, prefix: &PrefixExpression, right: ()) -> () {
        todo!()
    }

    fn visit_negation(&mut self, prefix: &PrefixExpression, right: ()) -> () {
        todo!()
    }

    fn visit_not(&mut self, prefix: &PrefixExpression, right: ()) -> () {
        todo!()
    }

    fn visit_assign(&mut self, stmt: &AssignStatement) -> () {
        todo!()
    }

    fn visit_return(&mut self, stmt: &ReturnStatement) -> () {
        todo!()
    }

    fn visit_expression_statement(&mut self, expr: &ExpressionStatement) -> () {
        todo!()
    }

    fn visit_block(&mut self, stmt: &BlockStatement) -> () {
        todo!()
    }

    fn visit_if(&mut self, stmt: &IfStatement) -> () {
        todo!()
    }

    fn visit_procedure(&mut self, stmt: &ProcedureStatement) -> () {
        todo!()
    }

    fn visit_while(&mut self, stmt: &WhileStatement) -> () {
        todo!()
    }

    fn visit_for_loop(&mut self, stmt: &ForLoopStatement) -> () {
        todo!()
    }
}
