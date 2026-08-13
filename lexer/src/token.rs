use crate::position::TokenPosition;

/// Enum for types of token
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    Identifier,
    /// +
    Plus,
    /// -
    Minus,
    /// \*
    Multiply,
    /// /
    Divide,
    /// ^
    Power,
    /// DIV
    Quotient,
    /// MOD
    Modulo,
    /// ==
    Equal,
    // (
    LeftParen,
    /// )
    RightParen,
    /// =
    Assign,
    /// >
    Greater,
    /// >=
    GreaterEqual,
    /// <
    Lesser,
    /// <=
    LesserEqual,
    /// NOT
    Not,
    /// !=
    NotEqual,
    /// AND
    And,
    /// OR
    Or,
    /// if
    If,
    /// then
    Then,
    /// else
    Else,
    // elseif
    ElseIf,
    /// endif
    EndIf,
    /// for
    For,
    /// to
    To,
    /// next
    Next,
    /// while
    While,
    /// endwhile
    EndWhile,
    /// do
    Do,
    /// until
    Until,
    /// switch
    Switch,
    /// case
    Case,
    /// :
    Colon,
    /// ,
    Comma,
    /// [
    LeftSquareBracket,
    /// ]
    RightSquareBracket,
    /// default
    Default,
    /// endswitch
    EndSwitch,
    /// true
    True,
    /// false
    False,
    /// .
    Dot,
    Number,
    String,
    Eof,
    Procedure,
    EndProcedure,
    /// global
    Global,
    Newline,
    Return,
}

/// Struct that store the tokentype, literal and position of a token
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
    pub pos: TokenPosition,
}
