mod graph;

use crate::graph::{AstDisplay, FormattedValue, NameValue, Row, Rows};
use compiler::ast_optimiser::AstOptimiser;
use compiler::compiler::Compiler;
use compiler::lmc_optimiser::LmcOptimiser;
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

#[wasm_bindgen(getter_with_clone)]
pub struct Output {
    pub tokens: String,
    pub pretty_printed: String,
    pub ast: String,
    pub folded_ast: String,
    pub lmc: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

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

#[wasm_bindgen]
pub fn generate_output(code: &str) -> Output {
    let mut parser = Parser::from(&*code);

    let pretty_printed;
    let ast;
    let folded_ast;
    let lmc;

    match parser.parse() {
        Ok(root) => {
            pretty_printed = root.pretty_print();
            ast = serde_json::to_string(&Grapher::new().visit_root(root.clone())).unwrap();
            let folded = AstOptimiser {}.visit_root(root);
            folded_ast = serde_json::to_string(&Grapher::new().visit_root(folded.clone())).unwrap();
            lmc = {
                let errors = TypeChecker::new().check(&folded);
                if let Some(errors) = errors {
                    format!("Type error: {:?}", errors)
                } else {
                    let mut result = Compiler::compile(folded);
                    result = LmcOptimiser::new(result).optimise();
                    format!("{}", result.get_program())
                }
            }
        }
        Err(err) => {
            let x = format!("Parser error: {:?}", err);
            pretty_printed = x.clone();
            ast = x.clone();
            folded_ast = x.clone();
            lmc = x;
        }
    }

    Output {
        tokens: get_tokens(code),
        pretty_printed,
        ast,
        folded_ast,
        lmc,
    }
}

struct Grapher {
    id: usize,
    rows: Rows<AstDisplay>,
}
impl Grapher {
    fn new() -> Self {
        Self {
            id: 0,
            rows: Vec::new(),
        }
    }
    fn infix_helper(
        &mut self,
        op: &str,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        let name = self.getName("InfixExpression", Some(op));
        self.rows.push(left.with_parent(name.get_name()));
        self.rows.push(right.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }
    fn prefix_helper(
        &mut self,
        op: &str,
        prefix: &PrefixExpression,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        let name = self.getName("PrefixExpression", Some(op));
        self.rows.push(right.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }
    fn visit_root(mut self, program_root: ProgramRoot) -> Rows<AstDisplay> {
        let root_node = self.visit_statements("ProgramRoot", &program_root.0);
        self.rows.push(root_node);
        self.rows
    }

    fn getName(&mut self, ast_name: &str, value: Option<&str>) -> NameValue<AstDisplay> {
        let x = NameValue::Formatted(FormattedValue {
            value: self.id.to_string(),
            formatting: AstDisplay {
                title: ast_name.into(),
                subtitle: value.map(String::from),
            },
        });
        self.id += 1;
        x
    }

    fn visit_statements(&mut self, stmt_name: &str, stmt: &BlockStatement) -> Row<AstDisplay> {
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
impl Visitor<Row<AstDisplay>, Row<AstDisplay>> for Grapher {
    fn visit_identifier(&mut self, ident: &Identifier) -> Row<AstDisplay> {
        Row {
            name: self.getName("Identifier", Some(&*ident.value)),
            parent: None,
            tooltip: None,
        }
    }

    fn visit_integer(&mut self, literal: &IntegerLiteral) -> Row<AstDisplay> {
        Row {
            name: self.getName("IntegerLiteral", Some(&*literal.value.to_string())),
            parent: None,
            tooltip: None,
        }
    }

    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> Row<AstDisplay> {
        Row {
            name: self.getName("BooleanLiteral", Some(&*literal.value.to_string())),
            parent: None,
            tooltip: None,
        }
    }

    fn visit_string(&mut self, literal: &StringLiteral) -> Row<AstDisplay> {
        Row {
            name: self.getName("StringLiteral", Some(&*literal.value)),
            parent: None,
            tooltip: None,
        }
    }

    fn visit_function_call(&mut self, call: &FunctionCallExpression) -> Row<AstDisplay> {
        let name = self.getName("FunctionCallExpression", None);

        let identifier = self.visit_identifier(&call.identifier);
        self.rows.push(identifier.with_parent(name.get_name()));
        for expr in &call.arguments.arguments {
            let expr = self.visit_expression(expr).with_parent(name.get_name());
            self.rows.push(expr)
        }
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_addition(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("+", infix, left, right)
    }

    fn visit_subtraction(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("-", infix, left, right)
    }

    fn visit_division(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("/", infix, left, right)
    }

    fn visit_multiplication(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("*", infix, left, right)
    }

    fn visit_modulo(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("MOD", infix, left, right)
    }

    fn visit_equal(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("==", infix, left, right)
    }

    fn visit_not_equal(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("!=", infix, left, right)
    }

    fn visit_greater(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper(">", infix, left, right)
    }

    fn visit_greater_equal(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper(">=", infix, left, right)
    }

    fn visit_lesser(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("<", infix, left, right)
    }

    fn visit_lesser_equal(
        &mut self,
        infix: &InfixExpression,
        left: Row<AstDisplay>,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.infix_helper("<=", infix, left, right)
    }

    fn visit_positive(
        &mut self,
        prefix: &PrefixExpression,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.prefix_helper("+", prefix, right)
    }

    fn visit_negation(
        &mut self,
        prefix: &PrefixExpression,
        right: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        self.prefix_helper("-", prefix, right)
    }

    fn visit_not(&mut self, prefix: &PrefixExpression, right: Row<AstDisplay>) -> Row<AstDisplay> {
        self.prefix_helper("NOT", prefix, right)
    }

    fn visit_assign(&mut self, stmt: &AssignStatement, value: Row<AstDisplay>) -> Row<AstDisplay> {
        let var_name = self.visit_identifier(&stmt.variable);
        let name = self.getName("AssignStatement", None);
        self.rows.push(var_name.with_parent(name.get_name()));
        self.rows.push(value.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_return(
        &mut self,
        _stmt: &ReturnStatement,
        expression: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        let name = self.getName("ReturnStatement", None);
        self.rows.push(expression.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_expression_statement(
        &mut self,
        _expr: &ExpressionStatement,
        expression: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        let name = self.getName("ExpressionStatement", None);
        self.rows.push(expression.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_block(&mut self, stmt: &BlockStatement) -> Row<AstDisplay> {
        self.visit_statements("BlockStatement", stmt)
    }

    fn visit_if(&mut self, stmt: &IfStatement, condition: Row<AstDisplay>) -> Row<AstDisplay> {
        let consequence = self.visit_block(&stmt.consequence).map_display_name(
            |_| unreachable!(),
            |v| v.with_subtitle(String::from("consequence")),
        );
        let alternative = self.visit_block(&stmt.alternitive).map_display_name(
            |_| unreachable!(),
            |v| v.with_subtitle(String::from("alternitive")),
        );
        let name = self.getName("IfStatement", None);
        self.rows.push(condition.with_parent(name.get_name()));
        self.rows.push(consequence.with_parent(name.get_name()));
        self.rows.push(alternative.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_procedure(&mut self, _stmt: &ProcedureStatement) -> Row<AstDisplay> {
        todo!()
    }

    fn visit_while(
        &mut self,
        stmt: &WhileStatement,
        condition: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        let body = self.visit_block(&stmt.body).map_display_name(
            |_| unreachable!(),
            |v| v.with_subtitle(String::from("body")),
        );
        let name = self.getName("WhileStatement", None);
        self.rows.push(condition.with_parent(name.get_name()));
        self.rows.push(body.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }

    fn visit_for_loop(
        &mut self,
        stmt: &ForLoopStatement,
        initial_value: Row<AstDisplay>,
        end_value: Row<AstDisplay>,
    ) -> Row<AstDisplay> {
        let variable = self.visit_identifier(&stmt.variable);
        let initial_value = initial_value.map_display_name(
            |_| unreachable!(),
            |v| v.map_subtitle(|s| s.map(|s| format!("initial value\n{}", s))),
        );
        let end_value = end_value.map_display_name(
            |_| unreachable!(),
            |v| v.map_subtitle(|s| s.map(|s| format!("end value\n{}", s))),
        );
        let body = self.visit_block(&stmt.body).map_display_name(
            |_| unreachable!(),
            |v| v.with_subtitle(String::from("body")),
        );
        let next_variable = self.visit_identifier(&stmt.next_variable).map_display_name(
            |_| unreachable!(),
            |v| v.map_subtitle(|s| s.map(|s| format!("next variable\n{}", s))),
        );
        let name = self.getName("ForLoopStatement", None);
        self.rows.push(variable.with_parent(name.get_name()));
        self.rows.push(initial_value.with_parent(name.get_name()));
        self.rows.push(end_value.with_parent(name.get_name()));
        self.rows.push(body.with_parent(name.get_name()));
        self.rows.push(next_variable.with_parent(name.get_name()));
        Row {
            name,
            parent: None,
            tooltip: None,
        }
    }
}
