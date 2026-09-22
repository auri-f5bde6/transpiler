use crate::ast::{
    AssignStatement, BlockStatement, BooleanLiteral, Expression, ExpressionStatement,
    ForLoopStatement, FunctionCallExpression, Identifier, IfStatement, InfixExpression,
    IntegerLiteral, PrefixExpression, ProcedureStatement, ProgramRoot, ReturnStatement,
    StringLiteral, WhileStatement,
};
use crate::{Statement, Visitor};

const SPACE_INDENTATION: usize = 4;

pub trait PrettyPrint {
    fn pretty_print(&self) -> String;
}
impl PrettyPrint for Expression {
    fn pretty_print(&self) -> String {
        let mut buffer = String::new();
        let mut printer = PrettyPrinter::new(&mut buffer);
        printer.visit_expression(self);
        buffer
    }
}
impl PrettyPrint for Statement {
    fn pretty_print(&self) -> String {
        let mut buffer = String::new();
        let mut printer = PrettyPrinter::new(&mut buffer);
        printer.visit_statement(self);
        buffer
    }
}
impl PrettyPrint for ProgramRoot {
    fn pretty_print(&self) -> String {
        let mut buffer = String::new();
        let mut printer = PrettyPrinter::new(&mut buffer);
        printer.visit_program_root(&self);
        buffer
    }
}

struct PrettyPrinter<'a> {
    buffer: &'a mut String,
    indentation: usize,
}
impl<'a> PrettyPrinter<'a> {
    fn new(buffer: &'a mut String) -> Self {
        Self {
            buffer,
            indentation: 0,
        }
    }
}

impl<'a> Visitor<(), String> for PrettyPrinter<'a> {
    fn visit_identifier(&mut self, ident: &Identifier) -> String {
        ident.value.clone()
    }

    fn visit_integer(&mut self, literal: &IntegerLiteral) -> String {
        literal.value.to_string()
    }

    fn visit_boolean(&mut self, literal: &BooleanLiteral) -> String {
        if literal.value {
            String::from("true")
        } else {
            String::from("false")
        }
    }

    fn visit_string(&mut self, literal: &StringLiteral) -> String {
        format!("\"{}\"", literal.value)
    }

    fn visit_function_call(&mut self, call: &FunctionCallExpression) -> String {
        let mut buffer = String::new();
        buffer.push_str(&*call.identifier.value);
        buffer.push('(');
        for (i, v) in call.arguments.arguments.iter().enumerate() {
            let expression = self.visit_expression(v);
            buffer.push_str(&*expression);
            if (i < call.arguments.arguments.len() - 1) {
                buffer.push(',')
            }
        }
        buffer.push(')');
        buffer
    }

    fn visit_addition(&mut self, infix: &InfixExpression, left: String, right: String) -> String {
        format!("({} + {})", left, right)
    }

    fn visit_subtraction(
        &mut self,
        infix: &InfixExpression,
        left: String,
        right: String,
    ) -> String {
        format!("({} - {})", left, right)
    }

    fn visit_division(&mut self, infix: &InfixExpression, left: String, right: String) -> String {
        format!("({} / {})", left, right)
    }

    fn visit_multiplication(
        &mut self,
        infix: &InfixExpression,
        left: String,
        right: String,
    ) -> String {
        format!("({} * {})", left, right)
    }

    fn visit_equal(&mut self, infix: &InfixExpression, left: String, right: String) -> String {
        format!("({} == {})", left, right)
    }

    fn visit_not_equal(&mut self, infix: &InfixExpression, left: String, right: String) -> String {
        format!("({} != {})", left, right)
    }

    fn visit_greater(&mut self, infix: &InfixExpression, left: String, right: String) -> String {
        format!("({} > {})", left, right)
    }

    fn visit_greater_equal(
        &mut self,
        infix: &InfixExpression,
        left: String,
        right: String,
    ) -> String {
        format!("({} >= {})", left, right)
    }

    fn visit_lesser(&mut self, infix: &InfixExpression, left: String, right: String) -> String {
        format!("({} < {})", left, right)
    }

    fn visit_lesser_equal(
        &mut self,
        infix: &InfixExpression,
        left: String,
        right: String,
    ) -> String {
        format!("({} <= {})", left, right)
    }

    fn visit_positive(&mut self, prefix: &PrefixExpression, right: String) -> String {
        format!("(+{})", right)
    }

    fn visit_negation(&mut self, prefix: &PrefixExpression, right: String) -> String {
        format!("(-{})", right)
    }

    fn visit_not(&mut self, prefix: &PrefixExpression, right: String) -> String {
        format!("(NOT {})", right)
    }

    fn visit_assign(&mut self, stmt: &AssignStatement) {
        let line = format!(
            "{}{} = {}\n",
            " ".repeat(self.indentation * SPACE_INDENTATION),
            self.visit_identifier(&stmt.variable),
            self.visit_expression(&stmt.value)
        );
        self.buffer.push_str(&*line);
    }

    fn visit_return(&mut self, stmt: &ReturnStatement) {
        let line = format!(
            "{}return {}\n",
            " ".repeat(self.indentation * SPACE_INDENTATION),
            self.visit_expression(&stmt.expression)
        );
        self.buffer.push_str(&*line);
    }

    fn visit_expression_statement(&mut self, expr: &ExpressionStatement) -> () {
        let line = format!(
            "{}{}\n",
            " ".repeat(self.indentation * SPACE_INDENTATION),
            self.visit_expression(&expr.expression)
        );
        self.buffer.push_str(&*line);
    }

    fn visit_block(&mut self, stmt: &BlockStatement) {
        self.indentation += 1;
        for i in &stmt.body {
            self.visit_statement(i)
        }
        self.indentation -= 1
    }

    fn visit_if(&mut self, stmt: &IfStatement) {
        let if_line = format!(
            "{}if {} then\n",
            " ".repeat(self.indentation * SPACE_INDENTATION),
            self.visit_expression(&stmt.condition)
        );
        self.buffer.push_str(&*if_line);
        self.visit_block(&stmt.consequence);
        if (!stmt.alternitive.body.is_empty()) {
            self.buffer.push_str(&*format!(
                "{}else\n",
                " ".repeat(self.indentation * SPACE_INDENTATION)
            ));
            self.visit_block(&stmt.alternitive);
        }
        self.buffer.push_str(&*format!(
            "{}endif\n",
            " ".repeat(self.indentation * SPACE_INDENTATION)
        ));
    }

    fn visit_procedure(&mut self, stmt: &ProcedureStatement) {
        let mut buffer = String::new();
        for (i, v) in stmt.parameters.iter().enumerate() {
            let expression = self.visit_identifier(v);
            buffer.push_str(&*expression);
            if (i < stmt.parameters.len() - 1) {
                buffer.push(',')
            }
        }
        let line = format!("{}procedure {}({})", " ".repeat(self.indentation * SPACE_INDENTATION), self.visit_identifier(&stmt.name), buffer);
        self.buffer.push_str(&*line);
        self.visit_block(&stmt.body);
        self.buffer.push_str("endprocedure")
    }

    fn visit_while(&mut self, stmt: &WhileStatement) {
        let line = format!(
            "{}while {}",
            " ".repeat(self.indentation * SPACE_INDENTATION),
            self.visit_expression(&stmt.condition)
        );
        self.buffer.push_str(&*line);
        self.visit_block(&stmt.body);
        self.buffer.push_str(&*format!(
            "{}endwhile",
            " ".repeat(self.indentation * SPACE_INDENTATION)
        ))
    }

    fn visit_for_loop(&mut self, stmt: &ForLoopStatement) {
        let for_line = format!(
            "{}for {} = {} to {}",
            " ".repeat(self.indentation * SPACE_INDENTATION),
            self.visit_identifier(&stmt.variable),
            self.visit_expression(&stmt.initial_value),
            self.visit_expression(&stmt.end_value)
        );
        self.buffer.push_str(&*for_line);
        self.visit_block(&stmt.body);
        let next_line = format!(
            "{}next {}",
            " ".repeat(self.indentation * SPACE_INDENTATION),
            self.visit_identifier(&stmt.next_variable)
        );
        self.buffer.push_str(&*next_line);
    }
}
