mod graph;

use crate::graph::{FormattedValue, NameValue, Row, Rows};
use compiler::compiler::Compiler;
use compiler::optimiser::Optimiser;
use lexer::lexer::Lexer;
use lexer::token::TokenType;
use log::log;
use parser::ast::{
    AssignStatement, BlockStatement, BooleanLiteral, ExpressionStatement, ForLoopStatement,
    FunctionCallExpression, Identifier, IfStatement, InfixExpression, IntegerLiteral,
    PrefixExpression, ProcedureStatement, ProgramRoot, ReturnStatement, StringLiteral,
    WhileStatement,
};
use parser::parser::Parser;
use parser::{PrettyPrint, Visitor};
use std::panic;
use type_checker::TypeChecker;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

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
        Ok(ast) => ast.pretty_print(),
        Err(err) => {
            format!("Parser error: {:?}", err)
        }
    }
}

#[wasm_bindgen]
pub fn ast(code: &str) -> String {
    let mut parser = Parser::from(&*code);
    match parser.parse() {
        Ok(ast) => serde_json::to_string(&Grapher::new().visit_root(ast)).expect("erm"),
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
                buf.push_str(&*format!(
                    "{:?}, {}",
                    token.token_type,
                    if (token.token_type == TokenType::Newline) {
                        "\n"
                    } else {
                        ""
                    }
                ));
            }
            Err(err) => {
                return format!("Error: {:?}", err);
            }
        }
    }
    buf
}
fn escapeHtml(html: &str) -> String {
    html.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
}
struct Grapher {
    id: usize,
    rows: Rows,
}
impl Grapher {
    fn new() -> Self {
        Self {
            id: 0,
            rows: Vec::new(),
        }
    }

    fn visit_root(mut self, program_root: ProgramRoot) -> Rows {
        let root_node = self.visit_statements("ProgramRoot", &program_root.0);
        self.rows.push(root_node);
        self.rows
    }

    fn getName(&mut self, ast_name: &str, value: Option<&str>) -> NameValue {
        let x = NameValue::Formatted(FormattedValue {
            value: self.id.to_string(),
            formatting: format!(
                "<div class='ast_name'>{}</div>{}",
                escapeHtml(ast_name),
                value.map_or(String::new(), |x| format!(
                    "<div class='ast_value'>{}</div>",
                    escapeHtml(x)
                ))
            ),
        });
        self.id += 1;
        x
    }

    fn visit_statements(&mut self, stmt_name: &str, stmt: &BlockStatement) -> Row {
        let name = self.getName(stmt_name, None);
        for stmt in &stmt.body {
            let child = self.visit_statement(stmt).with_parent(name.get_name());
            self.rows.push(child)
        }
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }
}
impl Visitor<Row, Row> for Grapher {
    fn visit_identifier(&mut self, ident: &Identifier) -> Row {
        Row {
            name: self.getName("Identifier", Some(&*ident.value)),
            parent: None,
            tooltip: None,
        }
    }

    fn visit_integer(&mut self, literal: &IntegerLiteral) -> Row {
        Row {
            name: self.getName("IntegerLiteral", Some(&*literal.value.to_string())),
            parent: None,
            tooltip: None,
        }
    }

    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> Row {
        Row {
            name: self.getName("BooleanLiteral", Some(&*literal.value.to_string())),
            parent: None,
            tooltip: None,
        }
    }

    fn visit_string(&mut self, literal: &StringLiteral) -> Row {
        Row {
            name: self.getName("StringLiteral", Some(&*literal.value)),
            parent: None,
            tooltip: None,
        }
    }

    fn visit_function_call(&mut self, call: &FunctionCallExpression) -> Row {
        todo!()
    }

    fn visit_addition(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        let name = self.getName("InfixExpression", Some("+"));
        self.rows.push(left.with_parent(name.get_name()));
        self.rows.push(right.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_subtraction(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_division(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_multiplication(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_equal(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_not_equal(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_greater(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_greater_equal(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_lesser(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_lesser_equal(&mut self, infix: &InfixExpression, left: Row, right: Row) -> Row {
        todo!()
    }

    fn visit_positive(&mut self, prefix: &PrefixExpression, right: Row) -> Row {
        todo!()
    }

    fn visit_negation(&mut self, prefix: &PrefixExpression, right: Row) -> Row {
        todo!()
    }

    fn visit_not(&mut self, prefix: &PrefixExpression, right: Row) -> Row {
        todo!()
    }

    fn visit_assign(&mut self, stmt: &AssignStatement) -> Row {
        let var_name = self.visit_identifier(&stmt.variable);
        let expr = self.visit_expression(&stmt.value);
        let name = self.getName("AssignStatement", None);
        self.rows.push(var_name.with_parent(name.get_name()));
        self.rows.push(expr.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_return(&mut self, stmt: &ReturnStatement) -> Row {
        todo!()
    }

    fn visit_expression_statement(&mut self, expr: &ExpressionStatement) -> Row {
        let expr = self.visit_expression(&expr.expression);
        let name = self.getName("ExpressionStatement", None);
        self.rows.push(expr.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_block(&mut self, stmt: &BlockStatement) -> Row {
        self.visit_statements("BlockStatement", stmt)
    }

    fn visit_if(&mut self, stmt: &IfStatement) -> Row {
        todo!()
    }

    fn visit_procedure(&mut self, stmt: &ProcedureStatement) -> Row {
        todo!()
    }

    fn visit_while(&mut self, stmt: &WhileStatement) -> Row {
        todo!()
    }

    fn visit_for_loop(&mut self, stmt: &ForLoopStatement) -> Row {
        todo!()
    }
}
