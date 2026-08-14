use crate::lmc::*;
use lexer::token::TokenType;
use parser::Statement;
use parser::ast::{Expression, ForLoopStatement, InfixExpression, PrefixExpression, ProgramRoot};
use std::collections::HashSet;
use std::collections::btree_set::Intersection;
use indexmap::{IndexMap, IndexSet};

struct VariableDiscovery {
    variables: HashSet<String>,
}
impl VariableDiscovery {
    pub fn list_variable(program_root: ProgramRoot) -> HashSet<String> {
        let mut discovery = VariableDiscovery {
            variables: HashSet::new(),
        };
        discovery.try_statement(program_root.0.into());
        discovery.variables
    }
    fn try_expression(&mut self, expr: Expression) {
        match expr {
            Expression::Identifier(identifier) => _ = self.variables.insert(identifier.value),
            Expression::IntegerLiteral(literal) => {
                _ = self.variables.insert(literal.value.to_string())
            }
            Expression::BooleanLiteral(literal) => {
                _ = self.variables.insert(if literal.value {
                    String::from("1")
                } else {
                    String::from("0")
                })
            }
            Expression::StringLiteral(_) => {}
            Expression::Prefix(prefix) => {
                self.try_expression(prefix.right);
            }
            Expression::Infix(infix) => {
                let InfixExpression {
                    left,
                    operator,
                    right,
                } = *infix;
                self.try_expression(left);
                self.try_expression(right);
            }
            Expression::FunctionCall(expr) => {
                // TODO: Ignoring function name for now
                for arg in expr.arguments.arguments {
                    self.try_expression(arg);
                }
            }
        }
    }
    fn try_statement(&mut self, stmt: Statement) {
        match stmt {
            Statement::Assign(stm) => {
                _ = self.variables.insert(stm.variable.value);
                self.try_expression(stm.value);
            }
            Statement::Return(stm) => self.try_expression(stm.expression),
            Statement::Expression(stm) => self.try_expression(stm.expression),
            Statement::Block(stm) => {
                for s in stm.body {
                    self.try_statement(s);
                }
            }
            Statement::If(stm) => {
                self.try_expression(stm.condition);
                self.try_statement(stm.consequence.into());
                self.try_statement(stm.alternitive.into());
            }
            Statement::Procedure(_) => todo!(),
            Statement::While(stm) => {
                self.try_expression(stm.condition);
                self.try_statement(stm.body.into());
            }
            Statement::ForLoop(stm) => {
                let ForLoopStatement {
                    variable,
                    initial_value,
                    end_value,
                    body,
                    next_variable,
                } = *stm;
                self.try_expression(variable.into());
                self.try_expression(initial_value);
                self.try_expression(end_value);
                self.try_statement(body.into());
                self.try_expression(next_variable.into());
            }
        }
    }
}

pub struct Compiler {
    program: Program,
    variables: IndexSet<String>,
}
impl Compiler {
    pub fn compile(program_root: ProgramRoot) -> Program {
        let mut variables: Vec<String> = VariableDiscovery::list_variable(program_root.clone())
            .iter()
            .cloned()
            .collect();
        println!("{:?}", variables);
        let mut compiler = Compiler {
            program: Program::new(),
            variables,
        };
        for stm in program_root.0.body {
            compiler.compile_statement(&stm);
        }
        compiler.program
    }

    fn variable_index(&self, value: &String) -> u16 {
        self.variables.insert(value)
        self.variables.iter().position(|x| x == value).unwrap() as u16
    }
    fn load(&mut self, var: &String) {
        self.program.push(Lda::new(Some(self.variable_index(var))))
    }
    fn store(&mut self, var: &String) {
        self.program.push(Sta::new(Some(self.variable_index(var))))
    }
    fn compile_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral(literal) => {
                self.load(&literal.value.to_string())
            }
            Expression::BooleanLiteral(literal) => {
                self.load(&(literal.value as u16).to_string())
            }
            Expression::Identifier(identifier) => {
                self.load(&identifier.value)
            }
            Expression::Infix(infix) => {
                self.compile_expression(&infix.left);
                self.store(temp);
                self.compile_expression(&infix.right);
            }
            _ => todo!(),
        }
    }

    fn compile_statement(&mut self, stm: &Statement) {
        match stm {
            Statement::Assign(stm) => {
                self.compile_expression(&stm.value);
                self.program
                    .push(Sta::new(Some(self.variable_index(&stm.variable.value))))
            }
            _ => todo!(),
        }
    }
}
