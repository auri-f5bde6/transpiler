use crate::error::ParserError;
use lexer::position::TokenPosition;
use lexer::token::{Token, TokenType};
use std::fmt::{Display, Formatter, Write};

trait PreetyPrint {
    /// * `buffer` buffer to write formatted string to
    /// * `indentation` how many level of indentation to apply (not the size)
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result;
}
const INDENTATION_SPACE: usize = 4;
/// Enum for all possible Statement, and contains a pointer to a heap allocated instance of the object
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assign(Box<AssignStatement>),
    Return(Box<ReturnStatement>),
    Expression(Box<ExpressionStatement>),
    Block(Box<BlockStatement>),
    If(Box<IfStatement>),
    Procedure(Box<ProcedureStatement>),
    While(Box<WhileStatement>),
    ForLoop(Box<ForLoopStatement>),
}
impl PreetyPrint for Statement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        match self {
            Statement::Assign(assign_statement) => assign_statement.preety_fmt(buffer, indentation),
            Statement::Return(return_statement) => return_statement.preety_fmt(buffer, indentation),
            Statement::Expression(expression_statement) => {
                expression_statement.preety_fmt(buffer, indentation)
            }
            Statement::Block(block_statement) => block_statement.preety_fmt(buffer, indentation),
            Statement::If(if_statement) => if_statement.preety_fmt(buffer, indentation),
            Statement::Procedure(procedure_statement) => {
                procedure_statement.preety_fmt(buffer, indentation)
            }
            Statement::While(while_statement) => while_statement.preety_fmt(buffer, indentation),
            Statement::ForLoop(forloop_statement) => {
                forloop_statement.preety_fmt(buffer, indentation)
            }
        }
    }
}
impl From<AssignStatement> for Statement {
    fn from(assign: AssignStatement) -> Self {
        Statement::Assign(Box::new(assign))
    }
}
impl From<ReturnStatement> for Statement {
    fn from(ret: ReturnStatement) -> Self {
        Statement::Return(Box::new(ret))
    }
}
impl From<ExpressionStatement> for Statement {
    fn from(expr: ExpressionStatement) -> Self {
        Statement::Expression(Box::new(expr))
    }
}
impl From<BlockStatement> for Statement {
    fn from(expr: BlockStatement) -> Self {
        Statement::Block(Box::new(expr))
    }
}
impl From<IfStatement> for Statement {
    fn from(expr: IfStatement) -> Self {
        Statement::If(Box::new(expr))
    }
}
impl From<ProcedureStatement> for Statement {
    fn from(expr: ProcedureStatement) -> Self {
        Statement::Procedure(Box::new(expr))
    }
}
impl From<WhileStatement> for Statement {
    fn from(expr: WhileStatement) -> Self {
        Statement::While(Box::new(expr))
    }
}
impl From<ForLoopStatement> for Statement {
    fn from(expr: ForLoopStatement) -> Self {
        Statement::ForLoop(Box::new(expr))
    }
}
/// Enum for all possible Expression, and contains a pointer to a heap allocated instance of the object
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(Box<Identifier>),
    IntegerLiteral(Box<IntegerLiteral>),
    BooleanLiteral(Box<BooleanLiteral>),
    StringLiteral(Box<StringLiteral>),
    Prefix(Box<PrefixExpression>),
    Infix(Box<InfixExpression>),
    FunctionCall(Box<FunctionCallExpression>),
}
impl From<Identifier> for Expression {
    fn from(identifier: Identifier) -> Self {
        Expression::Identifier(Box::new(identifier))
    }
}
impl From<IntegerLiteral> for Expression {
    fn from(integer: IntegerLiteral) -> Self {
        Expression::IntegerLiteral(Box::new(integer))
    }
}
impl From<PrefixExpression> for Expression {
    fn from(prefix: PrefixExpression) -> Self {
        Expression::Prefix(Box::new(prefix))
    }
}
impl From<InfixExpression> for Expression {
    fn from(infix: InfixExpression) -> Self {
        Expression::Infix(Box::new(infix))
    }
}
impl From<BooleanLiteral> for Expression {
    fn from(boolean: BooleanLiteral) -> Self {
        Expression::BooleanLiteral(Box::new(boolean))
    }
}
impl From<StringLiteral> for Expression {
    fn from(literal: StringLiteral) -> Self {
        Expression::StringLiteral(Box::new(literal))
    }
}
impl From<FunctionCallExpression> for Expression {
    fn from(literal: FunctionCallExpression) -> Self {
        Expression::FunctionCall(Box::new(literal))
    }
}
impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Identifier(identifier) => write!(f, "{}", identifier),
            Expression::IntegerLiteral(literal) => write!(f, "{}", literal),
            Expression::Prefix(expr) => write!(f, "{}", expr),
            Expression::Infix(expr) => write!(f, "{}", expr),
            Expression::BooleanLiteral(literal) => write!(f, "{}", literal),
            Expression::StringLiteral(literal) => write!(f, "{}", literal),
            Expression::FunctionCall(expr) => write!(f, "{}", expr),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub value: String,
    pub token_position: TokenPosition,
}
impl TryFrom<Token> for Identifier {
    type Error = ParserError;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        if value.token_type == TokenType::Identifier {
            Ok(Self {
                value: value.literal,
                token_position: value.pos,
            })
        } else {
            Err(ParserError::TokenIsNotAnIdentifier(value))
        }
    }
}
impl TryFrom<Expression> for Identifier {
    type Error = ParserError;
    fn try_from(value: Expression) -> Result<Self, Self::Error> {
        match value {
            Expression::Identifier(ident) => Ok(*ident),
            exp => Err(ParserError::ExpressionIsNotAnIdentifier(exp)),
        }
    }
}
impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct IntegerLiteral {
    pub value: i64,
    pub token_position: TokenPosition,
}
impl Display for IntegerLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
impl TryFrom<Token> for IntegerLiteral {
    type Error = ParserError;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        if value.token_type == TokenType::Number {
            Ok(Self {
                value: value.literal.parse().unwrap(),
                token_position: value.pos,
            })
        } else {
            Err(ParserError::NotANumber(value))
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct StringLiteral {
    pub value: String,
    pub token_position: TokenPosition,
}
impl Display for StringLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
impl TryFrom<Token> for StringLiteral {
    type Error = ParserError;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        if value.token_type == TokenType::String {
            Ok(Self {
                value: value.literal,
                token_position: value.pos,
            })
        } else {
            Err(ParserError::NotAString(value))
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct BooleanLiteral {
    pub value: bool,
    pub token_position: TokenPosition,
}
impl Display for BooleanLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
impl TryFrom<Token> for BooleanLiteral {
    type Error = ParserError;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        if value.token_type == TokenType::True {
            Ok(Self {
                value: true,
                token_position: value.pos,
            })
        } else if value.token_type == TokenType::False {
            Ok(Self {
                value: false,
                token_position: value.pos,
            })
        } else {
            Err(ParserError::NotABoolean(value))
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ProgramRoot(pub BlockStatement);
impl ProgramRoot {
    pub fn write_preety_string(&self, buffer: &mut String) -> std::fmt::Result {
        self.0.preety_fmt(buffer, 0)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct AssignStatement {
    pub variable: Identifier,
    pub value: Expression,
}
impl PreetyPrint for AssignStatement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        let indent = " ".repeat(indentation * INDENTATION_SPACE);
        writeln!(buffer, "{}{} = {}", indent, self.variable, self.value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStatement {
    pub expression: Expression,
}
impl PreetyPrint for ReturnStatement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        let indent = " ".repeat(indentation * INDENTATION_SPACE);
        writeln!(buffer, "{}return {}", indent, self.expression)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: BlockStatement,
}
impl PreetyPrint for WhileStatement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        let indent = " ".repeat(indentation * INDENTATION_SPACE);
        writeln!(buffer, "{}while {}", indent, self.condition)?;
        self.body.preety_fmt(buffer, indentation + 1)?;
        writeln!(buffer, "{}endwhile", indent)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForLoopStatement {
    pub variable: Identifier,
    pub initial_value: Expression,
    pub end_value: Expression,
    pub body: BlockStatement,
    pub next_variable: Identifier,
}
impl PreetyPrint for ForLoopStatement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        let indent = " ".repeat(indentation * INDENTATION_SPACE);
        write!(
            buffer,
            "{}for {} = {}",
            indent, self.variable, self.initial_value
        )?;
        writeln!(buffer, " to {}", self.end_value)?;
        self.body.preety_fmt(buffer, indentation + 1)?;
        writeln!(buffer, "{}next {}", indent, self.next_variable)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStatement {
    pub body: Vec<Statement>,
}
impl BlockStatement {
    pub fn empty() -> Self {
        BlockStatement { body: vec![] }
    }
}
impl PreetyPrint for BlockStatement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        for stm in &self.body {
            stm.preety_fmt(buffer, indentation)?;
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub consequence: BlockStatement,
    pub alternitive: BlockStatement,
}

impl PreetyPrint for IfStatement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        let indent = " ".repeat(indentation * INDENTATION_SPACE);
        writeln!(buffer, "{}if {} then", indent, self.condition)?;
        self.consequence.preety_fmt(buffer, indentation + 1)?;
        if self.alternitive.body.len() > 0 {
            writeln!(buffer, "{}else", indent)?;
            self.alternitive.preety_fmt(buffer, indentation + 1)?;
        }
        writeln!(buffer, "{}endif", indent)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ProcedureStatement {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

impl PreetyPrint for ProcedureStatement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        let indent = " ".repeat(indentation * INDENTATION_SPACE);
        write!(buffer, "{}procedure {}(", indent, self.name)?;
        if self.parameters.len() > 0 {
            write!(buffer, "{}", self.parameters[0])?;
            if self.parameters.len() > 1 {
                for i in self.parameters[1..self.parameters.len() - 1].iter() {
                    write!(buffer, ",{}", i)?;
                }
            }
        }
        writeln!(buffer, ")")?;
        self.body.preety_fmt(buffer, indentation + 1)?;
        writeln!(buffer, "{}endprocedure", indent)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionStatement {
    pub expression: Expression,
}
impl PreetyPrint for ExpressionStatement {
    fn preety_fmt(&self, buffer: &mut String, indentation: usize) -> std::fmt::Result {
        let indent = " ".repeat(indentation * INDENTATION_SPACE);
        writeln!(buffer, "{}{}", indent, self.expression)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrefixExpression {
    pub operator: Token,
    pub right: Expression,
}
impl Display for PrefixExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator.literal, self.right)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InfixExpression {
    pub left: Expression,
    pub operator: Token,
    pub right: Expression,
}
impl Display for InfixExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.left, self.operator.literal, self.right
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionArguments {
    pub arguments: Vec<Expression>,
}
impl Display for FunctionArguments {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, arg) in self.arguments.iter().enumerate() {
            write!(
                f,
                "{}{}",
                arg,
                if i == self.arguments.len() - 1 {
                    ""
                } else {
                    ", "
                }
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCallExpression {
    pub identifier: Identifier,
    pub arguments: FunctionArguments,
}
impl Display for FunctionCallExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.identifier, self.arguments)
    }
}
