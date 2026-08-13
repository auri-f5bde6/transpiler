use crate::ast::Expression;
use lexer::lexer::LexerError;
use lexer::token::{Token, TokenType};

#[derive(Debug, PartialEq)]
pub enum ParserError {
    LexerError(LexerError),
    /// Expected 1, got 2
    ExpectedAGotB(TokenType, Token),
    /// Expected end of a file or a newline, got
    ExpectedEofOrNewline(Token),
    /// Expected a prefix (e.g., number, identifier) but got something else
    ExpectedExpressionGot(Token),
    UnexpectedToken(Token),
    /// Attempts to convert a non-identifier token to an identifier struct
    TokenIsNotAnIdentifier(Token),
    ExpressionIsNotAnIdentifier(Expression),
    /// Attempts to convert a non-numer token to an integer literal
    NotANumber(Token),
    /// Unclosed parenthesis, 1. Opening parenthesis
    UnclosedParen(Token),
    /// Expected a Boolean but got something else
    NotABoolean(Token),
    /// Expected a String but got something else
    NotAString(Token),
    /// Expected a prefix operator but got something else
    NotAPrefixOperator(Token),
    /// Failed to parse statement because of an unexpected token
    FailedToParseStatementGot(Token),
}
impl From<LexerError> for ParserError {
    fn from(e: LexerError) -> ParserError {
        ParserError::LexerError(e)
    }
}
