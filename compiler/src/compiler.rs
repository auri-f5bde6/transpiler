use crate::lmc::*;
use indexmap::{IndexMap, IndexSet};
use lexer::token::TokenType;
use parser::Statement;
use parser::ast::{Expression, ForLoopStatement, InfixExpression, PrefixExpression, ProgramRoot};
use std::collections::HashSet;
use std::collections::btree_set::Intersection;

pub struct Compiler {
    program: Program,
    variables: HashSet<String>,
    variables_dat: Program,
    temp_count: usize,
    label_count: usize,
    next_label: Option<String>,
}
impl Compiler {
    pub fn compile(program_root: ProgramRoot) -> Program {
        let mut compiler = Compiler {
            program: Program::new(),
            variables: HashSet::new(),
            variables_dat: Program::new(),
            temp_count: 0,
            label_count: 0,
            next_label: None,
        };
        for stm in program_root.0.body {
            let label = compiler.next_label.take();
            compiler.compile_statement(label, &stm);
        }
        compiler.program.push_hlt(compiler.next_label.take());
        let Compiler {
            program,
            variables,
            variables_dat,
            temp_count,
            label_count,
            next_label: next_block,
        } = compiler;
        program.merge(variables_dat)
    }

    fn get_temp(&mut self) -> String {
        let name = format!("temp_{}", self.temp_count);
        if !self.variables.contains(&name) {
            self.variables_dat.push_dat(Some(name.clone()), 0);
            self.variables.insert(name.clone());
        }
        self.temp_count += 1;
        name
    }

    fn release_temp(&mut self) {
        self.temp_count -= 1;
    }

    fn get_label(&mut self) -> String {
        let result = format!("label_{}", self.label_count);
        self.label_count += 1;
        result
    }

    fn get_literal(&mut self, literal: u16) -> String {
        let string = format!("{}_{}", "literal", literal);
        if !self.variables.contains(&string) {
            self.variables_dat.push_dat(Some(string.clone()), literal);
            self.variables.insert(string.clone());
        }
        string
    }

    // Remember to run self.next_label.take() after this is ran, if any instruction is manually added afterward
    fn compile_expression(&mut self, label: Option<String>, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral(literal) => {
                let l = self.get_literal(literal.value as u16);
                self.program.push_lda(label, l)
            }
            Expression::BooleanLiteral(literal) => {
                let l = self.get_literal(literal.value as u16);
                self.program.push_lda(label, l)
            }
            Expression::Identifier(identifier) => {
                if !self.variables.contains(&identifier.value) {
                    // Todo: proper error reporting
                    panic!("Variable '{}' not found", identifier.value);
                }
                self.program.push_lda(label, identifier.value.clone());
            }
            Expression::Infix(infix) => {
                self.compile_expression(label, &infix.right);
                let temp_rhs = self.get_temp();
                self.program.push_sta(self.next_label.take(), temp_rhs.clone());
                self.compile_expression(None, &infix.left);
                match infix.operator.token_type {
                    TokenType::Plus => self.program.push_add(self.next_label.take(), temp_rhs),
                    TokenType::Minus => self.program.push_sub(self.next_label.take(), temp_rhs),
                    TokenType::Divide => todo!(),
                    TokenType::Multiply => {
                        // a * b
                        //        LDA a
                        //        STA temp_3
                        //        LDA literal_0
                        //        STA temp_2
                        //        LDA temp_3
                        // label1 BRZ label2
                        //        SUB literal_1
                        //        STA temp_1
                        //        LDA temp_2
                        //        ADD b
                        //        STA temp_2
                        //        LDA temp_1
                        //        BRZ label2
                        //        BRA label1
                        // label2 LDA temp_2
                        let one = self.get_literal(1);

                        let label1 = self.get_label();
                        let label2 = self.get_label();

                        let temp1 = self.get_temp();
                        let temp2 = self.get_temp();
                        let temp3 = self.get_temp();

                        self.program.push_sta(self.next_label.take(), temp3.clone());
                        self.program.push_lda(None, one.clone());
                        self.program.push_sta(None, temp2.clone());
                        self.program.push_lda(None, temp3.clone());
                        self.program.push_brz(Some(label1.clone()), label2.clone());
                        self.program.push_sub(None, one);
                        self.program.push_sta(None, temp1.clone());
                        self.program.push_lda(None, temp2.clone());
                        self.program.push_add(None, temp_rhs);
                        self.program.push_sta(None, temp2.clone());
                        self.program.push_lda(None, temp1);
                        self.program.push_brz(None, label2.clone());
                        self.program.push_bra(None, label1);
                        self.program.push_lda(Some(label2), temp2);
                        self.release_temp();
                        self.release_temp();
                    }
                    TokenType::Modulo => todo!(),
                    TokenType::Equal => {
                        // a == b
                        // b - a
                        //       LDA a
                        //       SUB b
                        //       BRZ label1
                        //       LDA literal_0
                        //       BRA label2
                        // label1 LDA literal1
                        // label2 ...
                        let one = self.get_literal(1);
                        let zero = self.get_literal(0);
                        let label1 = self.get_label();
                        let label2 = self.get_label();
                        self.program.push_sub(self.next_label.take(), temp_rhs);
                        self.program.push_brz(None, label1.clone());
                        self.program.push_lda(None, zero);
                        self.program.push_bra(None, label2.clone());
                        self.program.push_lda(Some(label1), one);
                        self.next_label = Some(label2);
                    }
                    TokenType::NotEqual => {
                        let one = self.get_literal(1);
                        let zero = self.get_literal(0);
                        let label1 = self.get_label();
                        let label2 = self.get_label();
                        self.program.push_sub(self.next_label.take(), temp_rhs);
                        self.program.push_brz(None, label1.clone());
                        self.program.push_lda(None, one);
                        self.program.push_bra(None, label2.clone());
                        self.program.push_lda(Some(label1), zero);
                        self.next_label = Some(label2);
                    }
                    TokenType::Greater => {
                        // a > b
                        // (a-1) - b
                        //        LDA a
                        //        SUB 1
                        //        SUB b
                        //        BRP label1
                        //        LDA literal_0
                        //        BRA label2
                        // label1 LDA literal_1
                        // label2 ...

                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let label1 = self.get_label();
                        let label2 = self.get_label();

                        self.program.push_sub(self.next_label.take(), one.clone());
                        self.program.push_sub(None, temp_rhs);
                        self.program.push_brp(None, label1.clone());
                        self.program.push_lda(None, zero);
                        self.program.push_bra(None, label2.clone());
                        self.program.push_lda(Some(label1), one);
                        self.next_label = Some(label2);
                    }

                    TokenType::GreaterEqual => {
                        // a >= b
                        // b - a
                        //        LDA a
                        //        SUB b
                        //        BRP label1
                        //        LDA literal_0
                        //        BRA label2
                        // label1 LDA literal_1
                        // label2 ...

                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let label1 = self.get_label();
                        let label2 = self.get_label();

                        self.program.push_sub(self.next_label.take(), temp_rhs);
                        self.program.push_sub(None, one.clone());
                        self.program.push_brp(None, label1.clone());
                        self.program.push_lda(None, zero);
                        self.program.push_bra(None, label2.clone());
                        self.program.push_lda(Some(label1), one);
                        self.next_label = Some(label2);
                    }
                    TokenType::Lesser => {
                        // a <= b == !(a > b)
                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let label1 = self.get_label();
                        let label2 = self.get_label();

                        self.program.push_sub(self.next_label.take(), temp_rhs);
                        self.program.push_sub(None, one.clone());
                        self.program.push_brp(None, label1.clone());
                        self.program.push_lda(None, one);
                        self.program.push_bra(None, label2.clone());
                        self.program.push_lda(Some(label1), zero);
                        self.next_label = Some(label2);
                    }
                    TokenType::LesserEqual => {
                        // a < b == !(a >= b)
                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let label1 = self.get_label();
                        let label2 = self.get_label();

                        self.program.push_sub(self.next_label.take(), one.clone());
                        self.program.push_sub(None, temp_rhs);
                        self.program.push_brp(None, label1.clone());
                        self.program.push_lda(None, one);
                        self.program.push_bra(None, label2.clone());
                        self.program.push_lda(Some(label1), zero);
                        self.next_label = Some(label2);
                    }
                    _ => todo!(),
                };
                self.release_temp();
            }
            Expression::FunctionCall(call) => match call.identifier.value.as_str() {
                "print" => {
                    self.compile_expression(label, &call.arguments.arguments[0]);
                    self.program.push_out(self.next_label.take())
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    fn new_variable(&mut self, name: String) {
        if !self.variables.contains(&name) {
            self.variables_dat.push_dat(Some(name.clone()), 0);
            self.variables.insert(name);
        }
    }

    fn compile_statement(&mut self, mut label: Option<String>, stm: &Statement) {
        if (self.next_label.is_some()) {
            if (label.is_some()) {
                todo!("no idea, ill fix this when it happens")
            } else {
                println!("Is this ever ran?");
                label = self.next_label.take()
            }
        }
        match stm {
            Statement::Assign(stm) => {
                self.compile_expression(label, &stm.value);
                self.new_variable(stm.variable.value.clone());
                self.program.push_sta(self.next_label.take(), stm.variable.to_string());
            }

            Statement::Return(_) => todo!(),
            Statement::Expression(expr) => self.compile_expression(label, &expr.expression),
            Statement::Block(block) => {
                if !block.body.is_empty() {
                    self.compile_statement(label, &block.body[0]);
                    for s in block.body.iter().skip(1) {
                        let l = self.next_label.take();
                        self.compile_statement(l, &s);
                    }
                }
            }
            Statement::If(stm) => {
                let consequence = self.get_label();
                self.compile_expression(None, &stm.condition);
                self.program.push_brp(self.next_label.take(), consequence.clone());
                self.compile_statement(None, &stm.alternitive.clone().into());
                self.program.push_bra(self.next_label.take(), consequence.clone());
                self.compile_statement(Some(consequence), &stm.consequence.clone().into())
            }
            Statement::Procedure(_) => todo!(),
            Statement::While(stm) => {
                let cond = label.unwrap_or(self.get_label());
                let end = self.get_label();
                self.compile_expression(Some(cond.clone()), &stm.condition);
                self.program.push_brz(self.next_label.take(), end.clone());
                self.compile_statement(None, &stm.body.clone().into());
                self.program.push_bra(self.next_label.take(), cond);
                self.next_label = Some(end);
            }
            Statement::ForLoop(stm) => {
                self.new_variable(stm.variable.value.clone());
            }
        }
    }
}
