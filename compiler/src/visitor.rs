use std::collections::btree_set::Intersection;

use lexer::token::TokenType;
use parser::Statement;
use parser::ast::{Expression, ProgramRoot};
use qbe::{Block, Cmp, DataDef, DataItem, Function, Instr, Linkage, Module, Type, Value};

pub struct Compiler<'a> {
    module: Module<'a>,
    current_function_index: usize,
    temporary_variable_count: u64,
    jump_label_count: u64,
}
impl<'a> Compiler<'a> {
    fn get_new_jump_label(&mut self) -> String {
        let temp = format!("b{}", self.jump_label_count);
        self.jump_label_count += 1;
        temp
    }
    fn get_temporary_variable(&mut self) -> Value {
        let temp = format!("t{}", self.temporary_variable_count);
        self.temporary_variable_count += 1;
        Value::Temporary(temp)
    }
    pub fn compile(program_root: ProgramRoot) -> Module<'a> {
        let module = Module::new();
        let mut main = Function::new(Linkage::public(), "main", vec![], Some(Type::Word));
        let mut compiler = Compiler {
            module,
            current_function_index: 0,
            temporary_variable_count: 0,
            jump_label_count: 0,
        };
        compiler.module.add_data(DataDef {
            linkage: Linkage::private(),
            name: "fmt".to_string(),
            align: None,
            items: vec![(Type::Byte, DataItem::Str("%d\n".to_string()))],
        });
        main.add_block("start");
        compiler.module.add_function(main);
        for stm in program_root.0.body {
            compiler.compile_statement(&stm);
        }
        compiler
            .get_current_block()
            .add_instr(Instr::Ret(Some(Value::Const(0))));

        compiler.module
    }
    fn compile_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::IntegerLiteral(literal) => Value::Const(literal.value as u64),
            Expression::Identifier(ident) => Value::Temporary(ident.value.clone()),
            Expression::Infix(infix) => {
                let temp = self.get_temporary_variable();
                let lhs = self.compile_expression(&infix.left);
                let rhs = self.compile_expression(&infix.right);
                let instruction;
                match infix.operator.token_type {
                    TokenType::Plus => {
                        instruction = Instr::Add(lhs, rhs);
                    }
                    TokenType::Minus => {
                        instruction = Instr::Sub(lhs, rhs);
                    }
                    TokenType::Divide => instruction = Instr::Div(lhs, rhs),
                    TokenType::Multiply => instruction = Instr::Mul(lhs, rhs),
                    TokenType::Modulo => instruction = Instr::Rem(lhs, rhs),
                    TokenType::Equal => instruction = Instr::Cmp(Type::Word, Cmp::Eq, lhs, rhs),
                    TokenType::NotEqual => instruction = Instr::Cmp(Type::Word, Cmp::Ne, lhs, rhs),
                    TokenType::Greater=>instruction=Instr::Cmp(Type::Word, Cmp::Sgt, lhs, rhs),
                    TokenType::GreaterEqual=>instruction=Instr::Cmp(Type::Word, Cmp::Sge, lhs, rhs),
                    TokenType::Lesser=>instruction=Instr::Cmp(Type::Word, Cmp::Slt, lhs, rhs),
                    TokenType::LesserEqual=>instruction=Instr::Cmp(Type::Word, Cmp::Sle, lhs, rhs),
                    _ => todo!(),

                }
                self.get_current_block()
                    .assign_instr(temp.clone(), Type::Word, instruction);
                temp
            }
            Expression::Prefix(prefix)=>{
                let temp = self.get_temporary_variable();

                let instruction;
                let rhs = self.compile_expression(&prefix.right);
                match prefix.operator.token_type {
                    TokenType::Plus => {
                        instruction=Instr::Copy(rhs)
                    },
                    TokenType::Minus => {
                        instruction=Instr::Sub(Value::Const(0), rhs)
                    },
                    TokenType::Not=>{
                        instruction=Instr::Cmp(Type::Word, Cmp::Ne, rhs, Value::Const(1)); // can't fins XOR?
                    }
                    _ => todo!(),
                }
                self.get_current_block()
                    .assign_instr(temp.clone(), Type::Word, instruction);
                temp
            }
            Expression::FunctionCall(call) => match &*call.identifier.value {
                "print" => {
                    assert_eq!(
                        call.arguments.arguments.len(),
                        1,
                        "print must have exactly 1 argument"
                    );
                    let val = self.compile_expression(&call.arguments.arguments[0]);
                    let fmt_ptr = Value::Global("fmt".to_string());
                    self.get_current_block().add_instr(Instr::Call(
                        "printf".to_string(),
                        vec![(Type::Long, fmt_ptr), (Type::Word, val)],
                        None,
                    ));

                    Value::Const(0)
                }
                _ => panic!("Unknown function {:#?}", call.identifier),
            },
            _ => todo!(),
        }
    }
    fn get_current_block(&mut self) -> &mut Block<'a> {
        self.module.functions[self.current_function_index]
            .blocks
            .last_mut()
            .unwrap()
    }
    fn add_block(&mut self, name: String) {
        self.module.functions[self.current_function_index].add_block(name);
    }
    fn compile_statement(&mut self, stm: &Statement) {
        match stm {
            Statement::Expression(expr) => _ = self.compile_expression(&expr.expression),
            Statement::Assign(assignment) => {
                let intr = Instr::Copy(self.compile_expression(&assignment.value));
                self.get_current_block().assign_instr(
                    Value::Temporary(assignment.variable.value.clone()),
                    Type::Word,
                    intr,
                )
            }
            Statement::If(stm) => {
                let expr = self.compile_expression(&stm.condition);
                let consequence = self.get_new_jump_label();
                let alternative = self.get_new_jump_label();
                let end = self.get_new_jump_label();
                self.get_current_block().add_instr(Instr::Jnz(
                    expr,
                    consequence.clone(),
                    alternative.clone(),
                ));
                self.add_block(consequence);
                for i in &stm.consequence.body {
                    self.compile_statement(i);
                }
                self.get_current_block().add_instr(Instr::Jmp(end.clone()));

                self.add_block(alternative);
                for i in &stm.alternitive.body {
                    self.compile_statement(i);
                }
                self.get_current_block().add_instr(Instr::Jmp(end.clone()));
                self.add_block(end)
            }
            Statement::While(while_stmt) => {
                let cond_start = self.get_new_jump_label();
                let body = self.get_new_jump_label();
                let end = self.get_new_jump_label();
                self.add_block(cond_start.clone());
                let expr = self.compile_expression(&while_stmt.condition);
                self.get_current_block()
                    .add_instr(Instr::Jnz(expr, body.clone(), end.clone()));
                self.add_block(body.clone());
                for i in &while_stmt.body.body {
                    self.compile_statement(i);
                }
                self.get_current_block().add_instr(Instr::Jmp(cond_start));
                self.add_block(end);
            }
            Statement::ForLoop(for_stm) => {
                let start = self.compile_expression(&for_stm.initial_value);
                let end = self.compile_expression(&for_stm.end_value);
                let cond_variable = Value::Temporary(for_stm.variable.value.clone());

                let next_variable = Value::Temporary(for_stm.next_variable.value.clone());

                let cond_block=self.get_new_jump_label();
                let body = self.get_new_jump_label();
                let finish = self.get_new_jump_label();
                self.get_current_block().assign_instr(
                    cond_variable.clone(),
                    Type::Word,
                    Instr::Copy(start.clone()),
                );
                self.add_block(cond_block.clone());

                let end_le_start = self.get_temporary_variable();
                let a = self.get_temporary_variable();
                let b=self.get_temporary_variable();
                self.get_current_block().assign_instr(
                    a.clone(),
                    Type::Word,
                    Instr::Cmp(Type::Word, Cmp::Sge, cond_variable.clone(), start.clone()),
                );

                self.get_current_block().assign_instr(
                    b.clone(),
                    Type::Word,
                    Instr::Cmp(Type::Word, Cmp::Slt, cond_variable.clone(), end.clone()),
                );
                self.get_current_block().assign_instr(
                    end_le_start.clone(),
                    Type::Word,
                    Instr::Cmp(Type::Word, Cmp::Eq, a, b),
                );

                self.get_current_block().add_instr(Instr::Jnz(
                    end_le_start,
                    body.clone(),
                    finish.clone(),
                ));
                self.add_block(body.clone());
                for i in &for_stm.body.body {
                    self.compile_statement(i);
                }
                self.get_current_block().assign_instr(
                    next_variable.clone(),
                    Type::Word,
                    Instr::Add(next_variable, Value::Const(1)),
                );
                self.get_current_block()
                    .add_instr(Instr::Jmp(cond_block));
                self.add_block(finish);
            }
            Statement::Block(block) => {
                unreachable!("This should never happen")
            }
            _ => todo!(),
        }
    }
}
