use crate::lmc::*;
use indexmap::{IndexMap, IndexSet};
use lexer::token::TokenType;
use parser::ast::{
    AssignStatement, BlockStatement, BooleanLiteral, Expression, ForLoopStatement,
    FunctionCallExpression, Identifier, IfStatement, InfixExpression, IntegerLiteral,
    PrefixExpression, ProcedureStatement, ProgramRoot, ReturnStatement, StringLiteral,
    WhileStatement,
};
use parser::{Statement, Visitor};
use paste::paste;
use std::collections::btree_set::Intersection;
use std::collections::{HashSet, VecDeque};

macro_rules! implement_with_operand {
    ($name:ident) => {
        paste! {
            pub fn [<push_$name:lower>](&mut self, operand: &str) {
                let final_label = if self.next_label.is_empty() { None } else if self.next_label.len() == 1 {
                    self.next_label.pop_front()
                } else {
                    let mut current = self.next_label.pop_front().unwrap();
                    while self.next_label.len()>0{
                        let next = self.next_label.pop_front().unwrap();
                        self.program.push(Some(current), Bra::new(next.clone()));
                        current=next;
                    }
                    Some(current)
                };
                self.program.push(final_label, [<$name:camel>]::new(operand.to_owned()))
            }
        }
    };
}
macro_rules! implement_no_operand {
    ($name:ident) => {
        paste! {
            pub fn [<push_$name:lower>](&mut self) {
                let final_label = if self.next_label.is_empty() { None } else if self.next_label.len() == 1 {
                    self.next_label.pop_front()
                } else {
                    let mut current = self.next_label.pop_front().unwrap();
                    while self.next_label.len()>0{
                        let next = self.next_label.pop_front().unwrap();
                        self.program.push(Some(current), Bra::new(next.clone()));
                        current=next;
                    }
                    Some(current)
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
        format!("label_{}_{}_{}", self.name, hint, self.label_count)
    }
}

pub struct Compiler {
    program: Program,
    variables: HashSet<String>,
    variables_dat: Program,
    temp_count: usize,
    label_count: usize,
    next_label: VecDeque<String>,
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
            next_label: vec![].into(),
        };
        for stm in program_root.0.body {
            compiler.visit_statement(&stm);
        }
        compiler.push_hlt();
        let Compiler {
            program,
            variables,
            variables_dat,
            temp_count,
            label_count,
            next_label,
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

    fn new_variable(&mut self, name: String) -> String {
        let name = format!("variable_{}", name);
        if !self.variables.contains(&name) {
            self.variables_dat.push_dat(Some(name.clone()), 0);
            self.variables.insert(name.clone());
        }
        name
    }

    fn infix_helper<F: Fn(&mut Compiler, String) -> ()>(&mut self, infix: &InfixExpression, f: F) {
        self.visit_expression(&infix.right);
        let temp_rhs = self.get_temp();
        self.push_sta(&temp_rhs);
        self.visit_expression(&infix.left);
        (f)(self, temp_rhs);
        self.release_temp()
    }
}
impl Visitor<()> for Compiler {
    fn visit_identifier(&mut self, ident: &Identifier) -> () {
        let name = format!("variable_{}", ident);
        if !self.variables.contains(&name) {
            // Todo: proper error reporting
            panic!("Variable '{}' not found", name);
        }
        self.push_lda(&name);
    }

    fn visit_integer(&mut self, literal: &IntegerLiteral) -> () {
        let l = &self.get_literal(literal.value as u16);
        self.push_lda(l)
    }

    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> () {
        let l = &self.get_literal(literal.value as u16);
        self.push_lda(l)
    }

    fn visit_string(&mut self, literal: &StringLiteral) -> () {
        todo!()
    }

    fn visit_prefix(&mut self, prefix: &PrefixExpression) -> () {
        self.visit_prefix_default(prefix, ())
    }
    fn visit_infix(&mut self, infix: &InfixExpression) -> () {
        self.visit_infix_default(infix, (), ())
    }

    fn visit_function_call(&mut self, call: &FunctionCallExpression) -> () {
        match call.identifier.value.as_str() {
            "print" => {
                self.visit_expression(&call.arguments.arguments[0]);
                self.push_out()
            }
            _ => todo!(),
        }
    }

    fn visit_addition(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        self.infix_helper(infix, |c, temp_rhs| c.push_add(&temp_rhs))
    }

    fn visit_subtraction(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        self.infix_helper(infix, |c, temp_rhs| c.push_sub(&temp_rhs))
    }

    fn visit_division(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        todo!()
    }

    fn visit_multiplication(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        /*
                        lhs * rhs

                        result = 0
                        count = 10
                        while count > 0
                            result = result + 2
                            count=count-1
                        endwhile

                        STA temp_count
                        LDA literal_0
                        STA temp_result
        label_while_start LDA temp_count
                        BRZ label_comp_false
                        BRP label_comp_true
        label_comp_false LDA literal_0
                        BRA label_comp_done
        label_comp_true LDA literal_1
        label_comp_done BRZ label_finish
                        LDA temp_result
                        ADD rhs
                        STA temp_result
                        LDA temp_count
                        SUB literal_1
                        STA temp_count
                        BRA label_while_start
           label_finish LDA temp_result
                 */
        self.infix_helper(infix, |c, temp_rhs| {
            let zero = c.get_literal(0);
            let one = c.get_literal(1);

            let count = c.get_temp();
            let result = c.get_temp();

            let labels = c.get_hinted_labels("multiplication");
            let while_start = labels.get_hinted_label("while_start");
            let comp_true = labels.get_hinted_label("comp_true");
            let comp_false = labels.get_hinted_label("comp_false");
            let comp_done = labels.get_hinted_label("comp_done");
            let finish = labels.get_hinted_label("finish");

            c.push_sta(&count);
            c.push_lda(&zero);
            c.push_sta(&result);
            c.next_label.push_back(while_start.clone());
            c.push_lda(&count);
            c.push_brz(&comp_false);
            c.push_brp(&comp_true);
            c.next_label.push_back(comp_false);
            c.push_lda(&zero);
            c.push_bra(&comp_done);
            c.next_label.push_back(comp_true);
            c.push_lda(&one);
            c.next_label.push_back(comp_done);
            c.push_brz(&finish);
            c.push_lda(&result);
            c.push_add(&temp_rhs);
            c.push_sta(&result);
            c.push_lda(&count);
            c.push_sub(&one);
            c.push_sta(&count);
            c.push_bra(&while_start);
            c.next_label.push_back(finish);
            c.push_lda(&result);

            c.release_temp();
            c.release_temp();
        });
    }

    fn visit_equal(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        // lhs == rhs
        //               (LDA lhs)
        //                SUB rhs
        //                BRZ label_true
        //                BRP label_false
        //    label_false LDA literal_0
        //                BRA label_continue
        //     label_true LDA literal_1
        // label_continue ...
        self.infix_helper(infix, |c, temp_rhs| {
            let zero = c.get_literal(0);
            let one = c.get_literal(1);

            let generator = c.get_hinted_labels("equal");
            let l_true = generator.get_hinted_label("true");
            let l_false = generator.get_hinted_label("false");
            let l_continue = generator.get_hinted_label("continue");

            c.push_sub(&temp_rhs);
            c.push_brz(&l_true);
            c.push_brp(&l_false);
            c.next_label.push_back(l_false);
            c.push_lda(&zero);
            c.push_bra(&l_continue);
            c.next_label.push_back(l_true);
            c.push_lda(&one);

            c.next_label.push_back(l_continue);
        })
    }

    fn visit_not_equal(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        self.infix_helper(infix, |c, temp_rhs| {
            let zero = c.get_literal(0);
            let one = c.get_literal(1);

            let generator = c.get_hinted_labels("equal");
            let l_true = generator.get_hinted_label("true");
            let l_false = generator.get_hinted_label("false");
            let l_continue = generator.get_hinted_label("continue");

            c.push_sub(&temp_rhs);
            c.push_brz(&l_true);
            c.push_brp(&l_false);
            c.next_label.push_back(l_false);
            c.push_lda(&one);
            c.push_bra(&l_continue);
            c.next_label.push_back(l_true);
            c.push_lda(&zero);

            c.next_label.push_back(l_continue);
        })
    }

    fn visit_greater(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        // lhs > rhs
        //               (LDA lhs)
        //                SUB rhs
        //                BRZ label_false
        //                BRP label_true
        //    label_false LDA literal_0
        //                BRA label_continue
        //     label_true LDA literal_1
        // label_continue ...

        self.infix_helper(infix, |c, temp_rhs| {
            let zero = c.get_literal(0);
            let one = c.get_literal(1);

            let generator = c.get_hinted_labels("greater");
            let l_false = generator.get_hinted_label("false");
            let l_true = generator.get_hinted_label("true");
            let l_continue = generator.get_hinted_label("continue");

            c.push_sub(&temp_rhs);
            c.push_brz(&l_false);
            c.push_brp(&l_true);
            c.next_label.push_back(l_false);
            c.push_lda(&zero);
            c.push_bra(&l_continue);
            c.next_label.push_back(l_true);
            c.push_lda(&one);

            c.next_label.push_back(l_continue);
        });
    }

    fn visit_greater_equal(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        // lhs > rhs
        //               (LDA lhs)
        //                SUB rhs
        //                BRP label_true
        //    label_false LDA literal_0
        //                BRA label_continue
        //     label_true LDA literal_1
        // label_continue ...

        self.infix_helper(infix, |c, temp_rhs| {
            let zero = c.get_literal(0);
            let one = c.get_literal(1);

            let generator = c.get_hinted_labels("greater_equal");
            let l_false = generator.get_hinted_label("false");
            let l_true = generator.get_hinted_label("true");
            let l_continue = generator.get_hinted_label("continue");

            c.push_sub(&temp_rhs);
            c.push_brp(&l_true);
            c.next_label.push_back(l_false);
            c.push_lda(&zero);
            c.push_bra(&l_continue);
            c.next_label.push_back(l_true);
            c.push_lda(&one);

            c.next_label.push_back(l_continue);
        })
    }

    fn visit_lesser(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        // lhs < rhs == !(lhs >= rhs)
        self.infix_helper(infix, |c, temp_rhs| {
            let zero = c.get_literal(0);
            let one = c.get_literal(1);

            let generator = c.get_hinted_labels("lesser");
            let l_false = generator.get_hinted_label("false");
            let l_true = generator.get_hinted_label("true");
            let l_continue = generator.get_hinted_label("continue");

            c.push_sub(&temp_rhs);
            c.push_brp(&l_false);
            c.next_label.push_back(l_true);
            c.push_lda(&one);
            c.push_bra(&l_continue);
            c.next_label.push_back(l_false);
            c.push_lda(&zero);

            c.next_label.push_back(l_continue);
        })
    }

    fn visit_lesser_equal(&mut self, infix: &InfixExpression, left: (), right: ()) -> () {
        // lhs <= rhs == !(lhs > rhs)
        self.infix_helper(infix, |c, temp_rhs| {
            let zero = c.get_literal(0);
            let one = c.get_literal(1);

            let generator = c.get_hinted_labels("lesser_equal");
            let l_false = generator.get_hinted_label("false");
            let l_true = generator.get_hinted_label("true");
            let l_continue = generator.get_hinted_label("continue");

            c.push_sub(&temp_rhs);
            c.push_brz(&l_true);
            c.push_brp(&l_false);
            c.next_label.push_back(l_true);
            c.push_lda(&one);
            c.push_bra(&l_continue);
            c.next_label.push_back(l_false);
            c.push_lda(&zero);

            c.next_label.push_back(l_continue);
        })
    }

    fn visit_positive(&mut self, prefix: &PrefixExpression, right: ()) -> () {
        self.visit_expression(&prefix.right)
    }

    fn visit_negation(&mut self, prefix: &PrefixExpression, right: ()) -> () {
        todo!()
    }

    fn visit_not(&mut self, prefix: &PrefixExpression, right: ()) -> () {
        //               (LDA rhs)
        //                BRP label_true
        //                LDA literal_1
        //                BRA label_continue
        //     label_true LDA literal_0
        // label_continue ...
        let zero = self.get_literal(0);
        let one = self.get_literal(1);

        let labels = self.get_hinted_labels("not");
        let l_true = labels.get_hinted_label("true");
        let l_continue = labels.get_hinted_label("continue");


        self.visit_expression(&prefix.right);
        self.push_brp(&l_true);
        self.push_lda(&one);
        self.push_bra(&l_continue);
        self.next_label.push_back(l_true);
        self.push_lda(&zero);
        self.next_label.push_back(l_continue);
    }

    fn visit_assign(&mut self, stmt: &AssignStatement) {
        self.visit_expression(&stmt.value);
        let name = self.new_variable(stmt.variable.value.clone());
        self.push_sta(&name);
    }

    fn visit_return(&mut self, stmt: &ReturnStatement) {
        todo!()
    }

    fn visit_block(&mut self, block: &BlockStatement) {
        if !block.body.is_empty() {
            self.visit_statement(&block.body[0]);
            for s in block.body.iter().skip(1) {
                self.visit_statement(&s);
            }
        }
    }

    fn visit_if(&mut self, stmt: &IfStatement) {
        let consequence = &self.get_label();
        self.visit_expression(&stmt.condition);
        self.push_brp(consequence);
        self.visit_statement(&stmt.alternitive.clone().into());
        self.push_bra(consequence);
        self.next_label.push_back(consequence.clone());
        self.visit_statement(&stmt.consequence.clone().into())
    }

    fn visit_procedure(&mut self, stmt: &ProcedureStatement) {
        todo!()
    }

    fn visit_while(&mut self, stmt: &WhileStatement) {
        let generator = self.get_hinted_labels("while");
        let cond = generator.get_hinted_label("cond");
        let end = generator.get_hinted_label("end");
        self.next_label.push_back(cond.clone());
        self.visit_expression(&stmt.condition);
        self.push_brz(&end);
        self.visit_statement(&stmt.body.clone().into());
        self.push_bra(&cond);
        self.next_label.push_back(end);
    }

    fn visit_for_loop(&mut self, stmt: &ForLoopStatement) {
        /*
                             LDA initial_vale
                             STA i
                  label_cond LDA i
                             SUB end_value
                             BRZ label_continue_false
                             BRP label_continue_true
        label_continue_false LDA literal_1
                             BRA label_continue
         label_continue_true LDA literal_0
              label_continue BRZ label_1
                             ...
                             LDA i
                             ADD literal_1
                             STA i
                             BRA label_while_cond_0
                     label_1 ...


                */
        todo!()
    }
}
