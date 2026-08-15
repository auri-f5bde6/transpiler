use crate::lmc::*;
use indexmap::{IndexMap, IndexSet};
use lexer::token::TokenType;
use parser::Statement;
use parser::ast::{Expression, ForLoopStatement, InfixExpression, PrefixExpression, ProgramRoot};
use paste::paste;
use std::collections::HashSet;
use std::collections::btree_set::Intersection;

macro_rules! implement_with_operand {
    ($name:ident) => {
        paste! {
            pub fn [<push_$name:lower>](&mut self, label: Option<&str>, operand: &str) {
                let final_label =
                    if self.next_label.is_some() && label.is_some(){
                        self.program.push(self.next_label.take(), Bra::new(label.map(String::from).unwrap()));
                        label.map(String::from)
                    } else if self.next_label.is_some() || label.is_some(){
                        Some(self.next_label.take().unwrap_or_else(||label.map(String::from).unwrap()))
                    }else{
                        None
                    };
                self.program.push(final_label, [<$name:camel>]::new(operand.to_owned()))
            }
        }
    };
}
macro_rules! implement_no_operand {
    ($name:ident) => {
        paste! {
            pub fn [<push_$name:lower>](&mut self, label: Option<&str>) {
                let final_label =
                    if self.next_label.is_some() && label.is_some(){
                        self.program.push(self.next_label.take(), Bra::new(label.map(String::from).unwrap()));
                        label.map(String::from)
                    } else if self.next_label.is_some() || label.is_some(){
                        Some(self.next_label.take().unwrap_or_else(||label.map(String::from).unwrap()))
                    }else{
                        None
                    };
                self.program.push(final_label, [<$name:camel>]::new())
            }
        }
    };
}

struct LabelGenerator<'a> {
    name: &'a str,
    label_count: usize,
}

impl<'a> LabelGenerator<'a> {
    pub(crate) fn get_hinted_label(&self, hint: &str) -> String {
        let result = format!("label_{}_{}_{}", self.name, hint, self.label_count);
        result
    }
}

pub struct Compiler {
    program: Program,
    variables: HashSet<String>,
    variables_dat: Program,
    temp_count: usize,
    label_count: usize,
    next_label: Option<String>,
}
impl Compiler {
    implement_with_operand!(ADD);
    implement_with_operand!(SUB);
    implement_with_operand!(STA);
    implement_with_operand!(LDA);
    implement_with_operand!(BRA);
    implement_with_operand!(BRZ);
    implement_with_operand!(BRP);
    implement_no_operand!(INP);
    implement_no_operand!(OUT);
    implement_no_operand!(HLT);

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
            compiler.compile_statement(None, &stm);
        }
        compiler.push_hlt(None);
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

    fn get_hinted_labels<'a>(&mut self, name: &'a str) -> LabelGenerator<'a> {
        let generator = LabelGenerator {
            name,
            label_count: self.label_count,
        };
        self.label_count += 1;
        generator
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
    fn compile_expression(&mut self, label: Option<&str>, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral(literal) => {
                let l = &self.get_literal(literal.value as u16);
                self.push_lda(label, l)
            }
            Expression::BooleanLiteral(literal) => {
                let l = &self.get_literal(literal.value as u16);
                self.push_lda(label, l)
            }
            Expression::Identifier(identifier) => {
                if !self.variables.contains(&identifier.value) {
                    // Todo: proper error reporting
                    panic!("Variable '{}' not found", identifier.value);
                }
                self.push_lda(label, &identifier.value);
            }
            Expression::Infix(infix) => {
                self.compile_expression(label, &infix.right);
                let temp_rhs = self.get_temp();
                self.push_sta(None, &temp_rhs);
                self.compile_expression(None, &infix.left);
                match infix.operator.token_type {
                    TokenType::Plus => self.push_add(None.take(), &temp_rhs),
                    TokenType::Minus => self.push_sub(None.take(), &temp_rhs),
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
                        let one = &*self.get_literal(1);

                        let label1 = &*self.get_label();
                        let label2 = &*self.get_label();

                        let temp1 = &self.get_temp();
                        let temp2 = &*self.get_temp();
                        let temp3 = &*self.get_temp();

                        self.push_sta(None, temp3);
                        self.push_lda(None, one);
                        self.push_sta(None, temp2);
                        self.push_lda(None, temp3);
                        self.push_brz(Some(label1), label2);
                        self.push_sub(None, one);
                        self.push_sta(None, temp1);
                        self.push_lda(None, temp2);
                        self.push_add(None, &temp_rhs);
                        self.push_sta(None, temp2);
                        self.push_lda(None, temp1);
                        self.push_brz(None, label2);
                        self.push_bra(None, label1);
                        self.push_lda(Some(label2), temp2);
                        self.release_temp();
                        self.release_temp();
                    }
                    TokenType::Modulo => todo!(),
                    TokenType::Equal => {
                        // lhs == rhs
                        //               (LDA lhs)
                        //                SUB rhs
                        //                BRZ label_true
                        //                BRP label_false
                        //    label_false LDA literal_0
                        //                BRA label_continue
                        //     label_true LDA literal_1
                        // label_continue ...

                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let generator = self.get_hinted_labels("equal");
                        let l_true = generator.get_hinted_label("true");
                        let l_false = generator.get_hinted_label("false");
                        let l_continue = generator.get_hinted_label("continue");

                        self.push_sub(None, &temp_rhs);
                        self.push_brz(None, &l_true);
                        self.push_brp(None, &l_false);
                        self.push_lda(Some(&l_false), &zero);
                        self.push_bra(None, &l_continue);
                        self.push_lda(Some(&l_true), &one);

                        self.next_label = Some(l_continue);
                    }
                    TokenType::NotEqual => {
                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let generator = self.get_hinted_labels("equal");
                        let l_true = generator.get_hinted_label("true");
                        let l_false = generator.get_hinted_label("false");
                        let l_continue = generator.get_hinted_label("continue");

                        self.push_sub(None, &temp_rhs);
                        self.push_brz(None, &l_true);
                        self.push_brp(None, &l_false);
                        self.push_lda(Some(&l_false), &one);
                        self.push_bra(None, &l_continue);
                        self.push_lda(Some(&l_true), &zero);

                        self.next_label = Some(l_continue);
                    }
                    TokenType::Greater => {
                        // lhs > rhs
                        //               (LDA lhs)
                        //                SUB rhs
                        //                BRZ label_false
                        //                BRP label_true
                        //    label_false LDA literal_0
                        //                BRA label_continue
                        //     label_true LDA literal_1
                        // label_continue ...

                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let generator = self.get_hinted_labels("greater");
                        let l_false = generator.get_hinted_label("false");
                        let l_true = generator.get_hinted_label("true");
                        let l_continue = generator.get_hinted_label("continue");

                        self.push_sub(None, &temp_rhs);
                        self.push_brz(None, &l_false);
                        self.push_brp(None, &l_true);
                        self.push_lda(Some(&l_false), &zero);
                        self.push_bra(None, &l_continue);
                        self.push_lda(Some(&l_true), &one);

                        self.next_label = Some(l_continue);
                    }

                    TokenType::GreaterEqual => {
                        // lhs > rhs
                        //               (LDA lhs)
                        //                SUB rhs
                        //                BRP label_true
                        //    label_false LDA literal_0
                        //                BRA label_continue
                        //     label_true LDA literal_1
                        // label_continue ...

                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let generator = self.get_hinted_labels("greater_equal");
                        let l_false = generator.get_hinted_label("false");
                        let l_true = generator.get_hinted_label("true");
                        let l_continue = generator.get_hinted_label("continue");

                        self.push_sub(None, &temp_rhs);
                        self.push_brp(None, &l_true);
                        self.push_lda(Some(&l_false), &zero);
                        self.push_bra(None, &l_continue);
                        self.push_lda(Some(&l_true), &one);

                        self.next_label = Some(l_continue);
                    }
                    TokenType::Lesser => {
                        // lhs < rhs == !(lhs >= rhs)
                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let generator = self.get_hinted_labels("lesser");
                        let l_false = generator.get_hinted_label("false");
                        let l_true = generator.get_hinted_label("true");
                        let l_continue = generator.get_hinted_label("continue");

                        self.push_sub(None, &temp_rhs);
                        self.push_brp(None, &l_true);
                        self.push_lda(Some(&l_false), &one);
                        self.push_bra(None, &l_continue);
                        self.push_lda(Some(&l_true), &zero);

                        self.next_label = Some(l_continue);
                    }
                    TokenType::LesserEqual => {
                        // lhs <= rhs == !(lhs > rhs)
                        let zero = self.get_literal(0);
                        let one = self.get_literal(1);

                        let generator = self.get_hinted_labels("lesser_equal");
                        let l_false = generator.get_hinted_label("false");
                        let l_true = generator.get_hinted_label("true");
                        let l_continue = generator.get_hinted_label("continue");

                        self.push_sub(None, &temp_rhs);
                        self.push_brz(None, &l_false);
                        self.push_brp(None, &l_true);
                        self.push_lda(Some(&l_false), &one);
                        self.push_bra(None, &l_continue);
                        self.push_lda(Some(&l_true), &zero);

                        self.next_label = Some(l_continue);
                    }
                    _ => todo!(),
                };
                self.release_temp();
            }
            Expression::FunctionCall(call) => match call.identifier.value.as_str() {
                "print" => {
                    self.compile_expression(label, &call.arguments.arguments[0]);
                    self.push_out(None)
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

    fn compile_statement(&mut self, label: Option<&str>, stm: &Statement) {
        if (self.next_label.is_some()) {
            if (label.is_some()) {
                todo!("no idea, ill fix this when it happens")
            } else {
                println!("Is this ever ran?");
                // label = self.next_label.take()
            }
        }
        match stm {
            Statement::Assign(stm) => {
                self.compile_expression(label, &stm.value);
                self.new_variable(stm.variable.value.clone());
                self.push_sta(None, &stm.variable.value);
            }

            Statement::Return(_) => todo!(),
            Statement::Expression(expr) => self.compile_expression(label, &expr.expression),
            Statement::Block(block) => {
                if !block.body.is_empty() {
                    self.compile_statement(label, &block.body[0]);
                    for s in block.body.iter().skip(1) {
                        self.compile_statement(None, &s);
                    }
                }
            }
            Statement::If(stm) => {
                let consequence = &self.get_label();
                self.compile_expression(label, &stm.condition);
                self.push_brp(None, consequence);
                self.compile_statement(None, &stm.alternitive.clone().into());
                self.push_bra(None, consequence);
                self.compile_statement(Some(consequence), &stm.consequence.clone().into())
            }
            Statement::Procedure(_) => todo!(),
            Statement::While(stm) => {
                let mut cond = &*self.get_label();
                cond = label.unwrap_or(&cond);
                let end = self.get_label();
                self.compile_expression(Some(cond), &stm.condition);
                self.push_brz(None, &end);
                self.compile_statement(None, &stm.body.clone().into());
                self.push_bra(None, cond);
                self.next_label = Some(end);
            }
            Statement::ForLoop(stm) => {
                self.new_variable(stm.variable.value.clone());
            }
        }
    }
}
