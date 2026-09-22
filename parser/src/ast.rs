use crate::error::ParserError;
use lexer::position::TokenPosition;
use lexer::token::{Token, TokenType};
use std::fmt::{Display, Formatter, Write};

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


#[derive(Debug, PartialEq, Clone)]
pub struct IntegerLiteral {
    pub value: i64,
    pub token_position: TokenPosition,
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
#[derive(Debug, PartialEq, Clone)]
pub struct AssignStatement {
    pub variable: Identifier,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStatement {
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: BlockStatement,
}


#[derive(Debug, PartialEq, Clone)]
pub struct ForLoopStatement {
    pub variable: Identifier,
    pub initial_value: Expression,
    pub end_value: Expression,
    pub body: BlockStatement,
    pub next_variable: Identifier,
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

#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub consequence: BlockStatement,
    pub alternitive: BlockStatement,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcedureStatement {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionStatement {
    pub expression: Expression,
}


#[derive(Debug, PartialEq, Clone)]
pub struct PrefixExpression {
    pub operator: Token,
    pub right: Expression,
}


#[derive(Debug, PartialEq, Clone)]
pub struct InfixExpression {
    pub left: Expression,
    pub operator: Token,
    pub right: Expression,
}


#[derive(Debug, PartialEq, Clone)]
pub struct FunctionArguments {
    pub arguments: Vec<Expression>,
}


#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCallExpression {
    pub identifier: Identifier,
    pub arguments: FunctionArguments,
}
