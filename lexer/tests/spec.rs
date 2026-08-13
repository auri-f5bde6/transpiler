use lexer::lexer::Lexer;
use lexer::position::{PositionInfo, TokenPosition};
use lexer::token::{Token, TokenType};

mod common;

#[test]
fn test_string() {
    test_helper!(
        include_str!("../../spec/string.txt"),
        [
            Token {
                token_type: TokenType::Identifier,
                literal: "someText".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 1, index: 0 },
                    to: PositionInfo { line: 1, index: 7 }
                },
            },
            Token {
                token_type: TokenType::Assign,
                literal: "=".to_string(),
                pos: PositionInfo { line: 1, index: 8 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::String,
                literal: "Computer Science".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 1, index: 9 },
                    to: PositionInfo { line: 1, index: 26 }
                },
            },
            Token {
                token_type: TokenType::Newline,
                literal: "\n".to_string(),
                pos: PositionInfo { line: 1, index: 27 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::String,
                literal: "Test\ning".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 5, index: 0 },
                    to: PositionInfo { line: 6, index: 3 }
                },
            },
            Token {
                token_type: TokenType::Newline,
                literal: "\n".to_string(),
                pos: PositionInfo { line: 6, index: 4 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::String,
                literal: "Hello\nWorld\n!".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 7, index: 0 },
                    to: PositionInfo { line: 9, index: 1 }
                },
            }
        ]
    )
}
#[test]
fn test_variable() {
    test_helper!(
        include_str!("../../spec/variable.txt"),
        [
            Token {
                token_type: TokenType::Identifier,
                literal: "x".to_string(),
                pos: PositionInfo { line: 1, index: 0 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Assign,
                literal: "=".to_string(),
                pos: PositionInfo { line: 1, index: 1 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Number,
                literal: "3".to_string(),
                pos: PositionInfo { line: 1, index: 2 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Newline,
                literal: "\n".to_string(),
                pos: PositionInfo { line: 1, index: 3 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Identifier,
                literal: "y".to_string(),
                pos: PositionInfo { line: 2, index: 0 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Assign,
                literal: "=".to_string(),
                pos: PositionInfo { line: 2, index: 1 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Number,
                literal: "1".to_string(),
                pos: PositionInfo { line: 2, index: 2 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Newline,
                literal: "\n".to_string(),
                pos: PositionInfo { line: 2, index: 3 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Identifier,
                literal: "name".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 3, index: 0 },
                    to: PositionInfo { line: 3, index: 3 }
                },
            },
            Token {
                token_type: TokenType::Assign,
                literal: "=".to_string(),
                pos: PositionInfo { line: 3, index: 4 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::String,
                literal: "Bob".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 3, index: 5 },
                    to: PositionInfo { line: 3, index: 9 }
                },
            },
            Token {
                token_type: TokenType::Newline,
                literal: "\n".to_string(),
                pos: PositionInfo { line: 3, index: 10 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Global,
                literal: "global".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 4, index: 0 },
                    to: PositionInfo { line: 4, index: 5 }
                },
            },
            Token {
                token_type: TokenType::Identifier,
                literal: "userid".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 4, index: 7 },
                    to: PositionInfo { line: 4, index: 12 }
                },
            },
            Token {
                token_type: TokenType::Assign,
                literal: "=".to_string(),
                pos: PositionInfo { line: 4, index: 14 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Number,
                literal: "123".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 4, index: 16 },
                    to: PositionInfo { line: 4, index: 18 }
                },
            },
            Token {
                token_type: TokenType::Newline,
                literal: "\n".to_string(),
                pos: PositionInfo { line: 4, index: 19 }.to_single_letter_token_position(),
            },
            Token {
                token_type: TokenType::Eof,
                literal: "\0".to_string(),
                pos: PositionInfo { line: 4, index: 19 }.to_single_letter_token_position(),
            }
        ]
    )
}
