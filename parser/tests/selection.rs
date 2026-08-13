use lexer::position::{PositionInfo, TokenPosition};
use lexer::token::{Token, TokenType};
use parser::ast::{
    BlockStatement, Expression, Identifier, IfStatement, InfixExpression, ProgramRoot, Statement,
    StringLiteral,
};
use parser::parser::Parser;

#[test]
fn test_selection() {
    let mut parser = Parser::from(include_str!("../../spec/selection.txt"));
    let program = parser.parse().expect("Failed to parse program");
    println!("{:#?}", program);
    assert_eq!(
        program,
        ProgramRoot(BlockStatement {
            body: vec![Statement::from(IfStatement {
                condition: Expression::from(InfixExpression {
                    left: Expression::from(Identifier {
                        value: "entry".to_string(),
                        token_position: TokenPosition {
                            from: PositionInfo { line: 2, index: 3 },
                            to: PositionInfo { line: 2, index: 7 }
                        }
                    }),
                    operator: Token {
                        token_type: TokenType::Equal,
                        literal: "==".to_string(),
                        pos: TokenPosition {
                            from: PositionInfo { line: 2, index: 8 },
                            to: PositionInfo { line: 2, index: 9 }
                        },
                    },
                    right: Expression::from(StringLiteral {
                        value: "a".to_string(),
                        token_position: TokenPosition {
                            from: PositionInfo { line: 2, index: 10 },
                            to: PositionInfo { line: 2, index: 12 }
                        }
                    })
                }),
                consequence: BlockStatement::empty(),
                alternitive: BlockStatement {
                    body: vec![Statement::from(IfStatement {
                        condition: Expression::from(InfixExpression {
                            left: Expression::from(Identifier {
                                value: "entry".to_string(),
                                token_position: TokenPosition {
                                    from: PositionInfo { line: 4, index: 8 },
                                    to: PositionInfo { line: 4, index: 12 }
                                }
                            }),
                            operator: Token {
                                token_type: TokenType::Equal,
                                literal: "==".to_string(),
                                pos: TokenPosition {
                                    from: PositionInfo { line: 4, index: 13 },
                                    to: PositionInfo { line: 4, index: 14 }
                                },
                            },
                            right: Expression::from(StringLiteral {
                                value: "b".to_string(),
                                token_position: TokenPosition {
                                    from: PositionInfo { line: 4, index: 15 },
                                    to: PositionInfo { line: 4, index: 17 }
                                }
                            })
                        }),
                        consequence: BlockStatement::empty(),
                        alternitive: BlockStatement::empty(),
                    })]
                }
            })]
        })
    );
}
