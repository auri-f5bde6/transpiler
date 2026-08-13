use crate::ast::{
    AssignStatement, BlockStatement, BooleanLiteral, Expression, ExpressionStatement,
    ForLoopStatement, FunctionArguments, FunctionCallExpression, Identifier, IfStatement,
    InfixExpression, IntegerLiteral, PrefixExpression, ProcedureStatement, ProgramRoot,
    ReturnStatement, Statement, StringLiteral, WhileStatement,
};
use crate::cursor::Cursor;
use crate::error::ParserError;
use crate::error::ParserError::UnclosedParen;
use lexer::token::TokenType;
use std::cmp::Ordering;

#[derive(Debug)]
enum Precedence {
    Lowest,
    LogicalOr,
    LogicalAnd,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
}
impl Precedence {
    fn index(&self) -> usize {
        match self {
            Precedence::Lowest => 0,
            Precedence::LogicalOr => 1,
            Precedence::LogicalAnd => 2,
            Precedence::Equals => 3,
            Precedence::LessGreater => 4,
            Precedence::Sum => 5,
            Precedence::Product => 6,
            Precedence::Prefix => 7,
            Precedence::Call => 8,
        }
    }
}

impl Eq for Precedence {}

impl PartialEq<Self> for Precedence {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }
}

impl PartialOrd<Self> for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index().partial_cmp(&other.index())
    }
}

impl Ord for Precedence {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index().cmp(&other.index())
    }
}
fn get_precedence(token_type: TokenType) -> Precedence {
    match token_type {
        TokenType::Plus | TokenType::Minus => Precedence::Sum,
        TokenType::Or => Precedence::LogicalOr,
        TokenType::And => Precedence::LogicalAnd,
        TokenType::Equal
        | TokenType::NotEqual
        | TokenType::LesserEqual
        | TokenType::GreaterEqual => Precedence::Equals,
        TokenType::Lesser | TokenType::Greater => Precedence::LessGreater,
        TokenType::Divide | TokenType::Multiply | TokenType::Modulo | TokenType::Quotient => {
            Precedence::Product
        }
        TokenType::LeftParen => Precedence::Call,
        _ => Precedence::Lowest,
    }
}
pub struct Parser<'a> {
    cursor: Cursor<'a>,
}
impl<'a> From<&'a str> for Parser<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            cursor: Cursor::from(value),
        }
    }
}
impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> Result<ProgramRoot, ParserError> {
        let mut statements = Vec::new();
        self.cursor.eat_newline()?;
        while !self.cursor.is_eof()? {
            statements.push(self.parse_statement()?);
            self.cursor.eat_newline()?;
        }
        Ok(ProgramRoot(BlockStatement { body: statements }))
    }
    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        let [a, b] = self.cursor.peek_n()?;
        match a.token_type {
            TokenType::Identifier if b.token_type == TokenType::Assign => {
                self.parse_assign_statement()
            }
            TokenType::Return => self.parse_return_statement(),
            TokenType::If => self.parse_if_statement(),
            TokenType::Procedure => self.parse_procedure_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_loop_statement(),
            _ => match self.parse_expression_statement() {
                Err(ParserError::ExpectedExpressionGot(token)) => {
                    Err(ParserError::FailedToParseStatementGot(token))
                }
                rest => rest,
            },
        }
    }
    fn parse_assign_statement(&mut self) -> Result<Statement, ParserError> {
        let variable = self.parse_identifier()?;
        self.cursor.expect(TokenType::Assign)?;
        let expression = self.parse_expression(Precedence::Lowest)?;
        self.cursor.expect_end_of_statement_or_eof()?;
        Ok(Statement::Assign(Box::from(AssignStatement {
            variable,
            value: expression,
        })))
    }
    /*fn parse_block_statement(&mut self)->Result<Statement, ParserError>{
        self.cursor.expect(TokenType::Then)

    }*/
    fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
        self.cursor.expect(TokenType::Return)?;
        let expression = self.parse_expression(Precedence::Lowest)?;
        self.cursor.expect_end_of_statement_or_eof()?;
        Ok(Statement::Return(Box::from(ReturnStatement { expression })))
    }
    fn parse_parameters(&mut self) -> Result<Vec<Identifier>, ParserError> {
        let mut parameter = vec![];

        if self.cursor.peek()?.token_type == TokenType::RightParen {
            return Ok(parameter);
        }
        parameter.push(self.parse_identifier()?);
        while self.cursor.peek()?.token_type != TokenType::RightParen {
            self.cursor.expect(TokenType::Comma)?;
            parameter.push(self.parse_identifier()?);
        }
        Ok(parameter)
    }
    fn parse_procedure_statement(&mut self) -> Result<Statement, ParserError> {
        self.cursor.expect(TokenType::Procedure)?;
        let name = self.parse_identifier()?;
        self.cursor.expect(TokenType::LeftParen)?;
        let parameters = self.parse_parameters()?;
        self.cursor.expect(TokenType::RightParen)?;
        self.cursor.expect_end_of_statement()?;
        let mut body = vec![];
        while self.cursor.peek()?.token_type != TokenType::EndProcedure {
            body.push(self.parse_statement()?);
        }
        self.cursor.bump()?;
        self.cursor.expect_end_of_statement_or_eof()?;
        Ok(Statement::from(ProcedureStatement {
            name,
            parameters,
            body: BlockStatement { body },
        }))
    }
    fn parse_expression_statement(&mut self) -> Result<Statement, ParserError> {
        let expression = self.parse_expression(Precedence::Lowest)?;
        self.cursor.expect_end_of_statement_or_eof()?;

        Ok(Statement::Expression(Box::from(ExpressionStatement {
            expression,
        })))
    }
    fn parse_identifier(&mut self) -> Result<Identifier, ParserError> {
        self.cursor.expect(TokenType::Identifier)?.try_into()
    }
    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ParserError> {
        let a = self.cursor.peek()?;
        let mut lhs: Expression = match a.token_type {
            TokenType::Plus | TokenType::Not | TokenType::Minus => {
                Ok(Expression::from(self.parse_prefix_expression()?))
            }
            TokenType::Number => Ok(Expression::from(self.parse_integer_literal()?)),
            TokenType::LeftParen => Ok(Expression::from(self.parse_grouped_expression()?)),
            TokenType::Identifier => Ok(Expression::from(self.parse_identifier()?)),
            TokenType::True => Ok(Expression::from(self.parse_true_literal()?)),
            TokenType::False => Ok(Expression::from(self.parse_false_literal()?)),
            TokenType::String => Ok(Expression::from(self.parse_string_literal()?)),
            _ => Err(ParserError::ExpectedExpressionGot(a)),
        }?;
        loop {
            let peek = self.cursor.peek()?;
            match peek.token_type {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Multiply
                | TokenType::Divide
                | TokenType::Modulo
                | TokenType::Quotient
                | TokenType::Equal
                | TokenType::NotEqual
                | TokenType::Greater
                | TokenType::GreaterEqual
                | TokenType::LesserEqual
                | TokenType::Lesser
                | TokenType::Or
                | TokenType::And
                    if precedence < get_precedence(peek.token_type) =>
                {
                    lhs = Expression::from(self.parse_infix(lhs)?);
                }
                TokenType::LeftParen if precedence < get_precedence(peek.token_type) => {
                    self.cursor.bump()?;
                    lhs = Expression::from(self.parse_function_call(lhs)?);
                }
                _ => break,
            };
        }
        Ok(lhs)
    }
    fn parse_grouped_expression(&mut self) -> Result<Expression, ParserError> {
        let open = self.cursor.expect(TokenType::LeftParen)?;
        let expr = self.parse_expression(Precedence::Lowest)?;
        if self.cursor.bump()?.token_type != TokenType::RightParen {
            return Err(UnclosedParen(open));
        }
        Ok(expr)
    }
    fn parse_prefix_expression(&mut self) -> Result<PrefixExpression, ParserError> {
        let bump = self.cursor.bump()?;
        match bump.token_type {
            TokenType::Plus | TokenType::Not | TokenType::Minus => Ok(PrefixExpression {
                operator: bump,
                right: self.parse_expression(Precedence::Prefix)?,
            }),
            _ => Err(ParserError::NotAPrefixOperator(bump)),
        }
    }
    fn parse_integer_literal(&mut self) -> Result<IntegerLiteral, ParserError> {
        self.cursor.expect(TokenType::Number)?.try_into()
    }
    fn parse_true_literal(&mut self) -> Result<BooleanLiteral, ParserError> {
        self.cursor.expect(TokenType::True)?.try_into()
    }
    fn parse_false_literal(&mut self) -> Result<BooleanLiteral, ParserError> {
        self.cursor.expect(TokenType::False)?.try_into()
    }
    fn parse_string_literal(&mut self) -> Result<StringLiteral, ParserError> {
        self.cursor.expect(TokenType::String)?.try_into()
    }
    fn parse_infix(&mut self, lhs: Expression) -> Result<InfixExpression, ParserError> {
        let operator = self.cursor.bump()?;
        let precedence = get_precedence(operator.token_type);
        let rhs = self.parse_expression(precedence)?;
        Ok(InfixExpression {
            left: lhs,
            operator,
            right: rhs,
        })
    }
    fn parse_function_call(
        &mut self,
        lhs: Expression,
    ) -> Result<FunctionCallExpression, ParserError> {
        Ok(FunctionCallExpression {
            identifier: Identifier::try_from(lhs)?,
            arguments: self.parse_call_arguments()?,
        })
    }
    fn parse_call_arguments(&mut self) -> Result<FunctionArguments, ParserError> {
        let mut args = vec![];
        if self.cursor.peek()?.token_type == TokenType::RightParen {
            self.cursor.bump()?;
            return Ok(FunctionArguments { arguments: args });
        }
        args.push(self.parse_expression(Precedence::Lowest)?);
        while self.cursor.peek()?.token_type != TokenType::RightParen {
            self.cursor.expect(TokenType::Comma)?;
            args.push(self.parse_expression(Precedence::Lowest)?);
        }
        self.cursor.expect(TokenType::RightParen)?;
        Ok(FunctionArguments { arguments: args })
    }
    fn parse_if_statement(&mut self) -> Result<Statement, ParserError> {
        self.cursor.expect(TokenType::If)?;
        let condition = self.parse_expression(Precedence::Lowest)?;
        self.cursor.expect(TokenType::Then)?;
        // should
        // if 10==10 then a
        // endif
        // be valid?
        //self.cursor.expect_newline()?;
        let mut consequence = vec![];
        self.cursor.eat_newline()?;
        while !(self.cursor.peek()?.token_type == TokenType::EndIf
            || self.cursor.peek()?.token_type == TokenType::Else)
        {
            consequence.push(self.parse_statement()?);
            self.cursor.eat_newline()?;
        }
        let mut alternative = vec![];
        match self.cursor.bump()? {
            tok if tok.token_type == TokenType::Else => {
                if self.cursor.peek()?.token_type == TokenType::If {
                    alternative.push(self.parse_if_statement()?);
                } else {
                    //self.cursor.expect_newline()?;
                    self.cursor.eat_newline()?;
                    while !(self.cursor.peek()?.token_type == TokenType::EndIf) {
                        alternative.push(self.parse_statement()?);
                        self.cursor.eat_newline()?;
                    }
                    self.cursor.bump()?; // Consume the EndIf
                    self.cursor.expect_end_of_statement_or_eof()?;
                }
            }
            tok if tok.token_type == TokenType::EndIf => (),
            _ => panic!("Should be unreachable"),
        }
        Ok(Statement::If(Box::from(IfStatement {
            condition,
            consequence: BlockStatement { body: consequence },
            alternitive: BlockStatement { body: alternative },
        })))
    }
    fn parse_while_statement(&mut self) -> Result<Statement, ParserError> {
        self.cursor.expect(TokenType::While)?;
        let condition = self.parse_expression(Precedence::Lowest)?;
        self.cursor.expect(TokenType::Newline)?;
        let mut body = vec![];
        self.cursor.eat_newline()?;
        while !(self.cursor.peek()?.token_type == TokenType::EndWhile) {
            body.push(self.parse_statement()?);
            self.cursor.eat_newline()?;
        }
        self.cursor.bump()?;
        Ok(Statement::from(WhileStatement {
            condition: condition,
            body: BlockStatement { body },
        }))
    }
    fn parse_for_loop_statement(&mut self) -> Result<Statement, ParserError> {
        self.cursor.expect(TokenType::For)?;
        let variable = self.parse_identifier()?;
        self.cursor.expect(TokenType::Assign)?;
        let initial_value = self.parse_expression(Precedence::Lowest)?;
        self.cursor.expect(TokenType::To)?;
        let end_value = self.parse_expression(Precedence::Lowest)?;
        self.cursor.expect(TokenType::Newline)?;
        self.cursor.eat_newline()?;
        let mut body = vec![];
        while !(self.cursor.peek()?.token_type == TokenType::Next) {
            body.push(self.parse_statement()?);
            self.cursor.eat_newline()?;
        }
        self.cursor.bump()?;
        let next_variable = self.parse_identifier()?;
        return Ok(Statement::from(ForLoopStatement {
            variable,
            initial_value,
            end_value,
            body: BlockStatement { body },
            next_variable,
        }));
    }
}
