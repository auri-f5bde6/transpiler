use crate::cursor::FancyChar;
use crate::position::TokenPosition;
use crate::{
    cursor::Cursor,
    position::PositionInfo,
    token::{Token, TokenType},
};

#[derive(Debug, PartialEq, Clone)]
pub enum LexerError {
    UnexpectedCharacter(FancyChar),
    /// 1. The \
    ///
    /// 2. The escape character
    UnknownCharacterEscape(FancyChar, FancyChar),
    /// 1. Character that began the string
    ///
    /// 2. The position where the end of the string is expected
    UnterminatedString(FancyChar, FancyChar),
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}
impl<'a> From<&'a str> for Lexer<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            cursor: Cursor::from(value),
        }
    }
}
impl<'a> Lexer<'a> {
    pub fn peek(&self) -> Result<Token, LexerError> {
        self.clone().bump()
    }
    pub fn is_eof(&self) -> Result<bool, LexerError> {
        Ok(self.peek()?.token_type == TokenType::Eof)
    }
    /// Functionally identical to [`Cursor::collect_while`], but it require a first char and it return TokenPosition for the collected literal
    /// Require the first character of the string, and a predicate function
    /// Predicate should be the same as [`Cursor::collect_while`]'s predicate
    fn collect_literal(
        &mut self,
        c: FancyChar,
        predicate: impl Fn(&mut Cursor, Option<FancyChar>) -> bool,
    ) -> (String, TokenPosition) {
        let mut literal = c.to_string();
        let collected_pos = self.cursor.collect_while(&mut literal, predicate);
        let pos;
        if let Some(p) = collected_pos {
            pos = TokenPosition {
                from: c.pos,
                to: p.to,
            };
        } else {
            // Collect failed, which mean it's a single character identifier
            pos = c.pos.to_single_letter_token_position();
        }
        (literal, pos)
    }

    pub fn bump(&mut self) -> Result<Token, LexerError> {
        self.cursor.eat_whitespace();
        let tok = {
            match self.cursor.bump() {
                Some(c) => match c.v {
                    '+' => Ok(Token {
                        token_type: TokenType::Plus,
                        literal: "+".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '-' => Ok(Token {
                        token_type: TokenType::Minus,
                        literal: "-".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '*' => Ok(Token {
                        token_type: TokenType::Multiply,
                        literal: "*".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '^' => Ok(Token {
                        token_type: TokenType::Power,
                        literal: "^".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '/' if self.cursor.peek().is_some_and(|c| *c == '/') => {
                        // Skip everything until newline
                        self.cursor
                            .eat_while(|cursor, _c| !cursor.is_next_newline());
                        self.bump()
                    }
                    '/' => Ok(Token {
                        token_type: TokenType::Divide,
                        literal: "/".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '=' => match self.cursor.peek() {
                        Some(peek) if peek.v == '=' => {
                            self.cursor.bump();
                            Ok(Token {
                                token_type: TokenType::Equal,
                                literal: "==".to_string(),
                                pos: TokenPosition::from_to(&c, &peek),
                            })
                        }
                        Some(_) | None => Ok(Token {
                            token_type: TokenType::Assign,
                            literal: "=".to_string(),
                            pos: c.pos.to_single_letter_token_position(),
                        }),
                    },
                    '>' => match self.cursor.peek() {
                        Some(peek) if peek.v == '=' => {
                            self.cursor.bump();
                            Ok(Token {
                                token_type: TokenType::GreaterEqual,
                                literal: ">=".to_string(),
                                pos: TokenPosition::from_to(&c, &peek),
                            })
                        }
                        Some(_) | None => Ok(Token {
                            token_type: TokenType::Greater,
                            literal: ">".to_string(),
                            pos: c.pos.to_single_letter_token_position(),
                        }),
                    },
                    '<' => match self.cursor.peek() {
                        Some(peek) if peek.v == '=' => {
                            self.cursor.bump();
                            Ok(Token {
                                token_type: TokenType::LesserEqual,
                                literal: "<=".to_string(),
                                pos: TokenPosition::from_to(&c, &peek),
                            })
                        }
                        Some(_) | None => Ok(Token {
                            token_type: TokenType::Lesser,
                            literal: "<".to_string(),
                            pos: c.pos.to_single_letter_token_position(),
                        }),
                    },
                    '(' => Ok(Token {
                        token_type: TokenType::LeftParen,
                        literal: "(".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),

                    ')' => Ok(Token {
                        token_type: TokenType::RightParen,
                        literal: ")".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '"' => {
                        let mut literal = String::new();
                        let mut last_pos = c.pos;
                        let mut val = None;
                        while let Some(bump) = self.cursor.bump() {
                            if *bump == '\\' {
                                match self.cursor.bump() {
                                    Some(escape_character) if *escape_character == '\\' => {
                                        literal.push('\\');
                                    }
                                    Some(peek) if *peek == 'n' => {
                                        literal.push('\n');
                                    }
                                    Some(unknown) => {
                                        val = Some(Err(LexerError::UnknownCharacterEscape(
                                            bump, unknown,
                                        )));
                                        break;
                                    }
                                    None => {
                                        val = Some(Err(LexerError::UnexpectedCharacter(bump)));
                                        break;
                                    }
                                }
                            } else if *bump == '"' {
                                val = Some(Ok(Token {
                                    token_type: TokenType::String,
                                    literal,
                                    pos: TokenPosition::from_to(&c, &bump),
                                }));
                                break;
                            } else {
                                literal.push(*bump);
                            }
                            last_pos = bump.pos;
                            if self.cursor.is_next_newline() {
                                literal.push('\n');
                            }
                        }
                        if let Some(val) = val {
                            val
                        } else {
                            Err(LexerError::UnterminatedString(
                                c,
                                FancyChar {
                                    v: '"',
                                    pos: PositionInfo {
                                        line: last_pos.line + 1,
                                        index: last_pos.index + 1,
                                    },
                                },
                            ))
                        }
                    }

                    '!' if self.cursor.peek().is_some_and(|c| *c == '=') => Ok(Token {
                        token_type: TokenType::NotEqual,
                        literal: "!=".to_string(),
                        pos: TokenPosition::from_to(&c, &self.cursor.bump().unwrap()),
                    }),
                    ':' => Ok(Token {
                        token_type: TokenType::Colon,
                        literal: ":".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '.' => Ok(Token {
                        token_type: TokenType::Dot,
                        literal: ".".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    ',' => Ok(Token {
                        token_type: TokenType::Comma,
                        literal: ",".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '[' => Ok(Token {
                        token_type: TokenType::LeftSquareBracket,
                        literal: "[".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    ']' => Ok(Token {
                        token_type: TokenType::RightSquareBracket,
                        literal: "]".to_string(),
                        pos: c.pos.to_single_letter_token_position(),
                    }),
                    '\n' => Ok(Token {
                        token_type: TokenType::Newline,
                        literal: "\n".to_string(),
                        pos: PositionInfo {
                            line: self.cursor.lines_number(),
                            index: self.cursor.offset(),
                        }
                        .to_single_letter_token_position(),
                    }),
                    _ => {
                        if c.is_ascii_alphabetic() || (*c) == '_' {
                            // Collect identifier
                            // Can contain alphabet, digits, and _,
                            // but the first character must be an alphabet or _
                            let (literal, pos) = self.collect_literal(c, |cursor, c| {
                                !cursor.is_next_newline()
                                    && c.is_some_and(|c| c.is_ascii_alphanumeric() || (*c) == '_')
                            });
                            match literal.as_str() {
                                "MOD" => Ok(Token {
                                    token_type: TokenType::Modulo,
                                    literal: "MOD".to_string(),
                                    pos,
                                }),
                                "DIV" => Ok(Token {
                                    token_type: TokenType::Quotient,
                                    literal: "DIV".to_string(),
                                    pos,
                                }),
                                "AND" => Ok(Token {
                                    token_type: TokenType::And,
                                    literal: "AND".to_string(),
                                    pos,
                                }),
                                "OR" => Ok(Token {
                                    token_type: TokenType::Or,
                                    literal: "OR".to_string(),
                                    pos,
                                }),
                                "NOT" => Ok(Token {
                                    token_type: TokenType::Not,
                                    literal: "NOT".to_string(),
                                    pos,
                                }),
                                "for" => Ok(Token {
                                    token_type: TokenType::For,
                                    literal: "for".to_string(),
                                    pos,
                                }),
                                "to" => Ok(Token {
                                    token_type: TokenType::To,
                                    literal: "to".to_string(),
                                    pos,
                                }),
                                "next" => Ok(Token {
                                    token_type: TokenType::Next,
                                    literal: "next".to_string(),
                                    pos,
                                }),
                                "while" => Ok(Token {
                                    token_type: TokenType::While,
                                    literal: "while".to_string(),
                                    pos,
                                }),
                                "endwhile" => Ok(Token {
                                    token_type: TokenType::EndWhile,
                                    literal: "endwhile".to_string(),
                                    pos,
                                }),
                                "do" => Ok(Token {
                                    token_type: TokenType::Do,
                                    literal: "do".to_string(),
                                    pos,
                                }),
                                "until" => Ok(Token {
                                    token_type: TokenType::Until,
                                    literal: "until".to_string(),
                                    pos,
                                }),
                                "true" => Ok(Token {
                                    token_type: TokenType::True,
                                    literal: "true".to_string(),
                                    pos,
                                }),
                                "false" => Ok(Token {
                                    token_type: TokenType::False,
                                    literal: "false".to_string(),
                                    pos,
                                }),
                                "switch" => Ok(Token {
                                    token_type: TokenType::Switch,
                                    literal: "switch".to_string(),
                                    pos,
                                }),
                                "case" => Ok(Token {
                                    token_type: TokenType::Case,
                                    literal: "case".to_string(),
                                    pos,
                                }),
                                "default" => Ok(Token {
                                    token_type: TokenType::Default,
                                    literal: "default".to_string(),
                                    pos,
                                }),
                                "endswitch" => Ok(Token {
                                    token_type: TokenType::EndSwitch,
                                    literal: "endswitch".to_string(),
                                    pos,
                                }),
                                "if" => Ok(Token {
                                    token_type: TokenType::If,
                                    literal: "if".to_string(),
                                    pos,
                                }),
                                "else" => Ok(Token {
                                    token_type: TokenType::Else,
                                    literal,
                                    pos,
                                }),
                                "then" => Ok(Token {
                                    token_type: TokenType::Then,
                                    literal: "then".to_string(),
                                    pos,
                                }),
                                "elseif" => Ok(Token {
                                    token_type: TokenType::ElseIf,
                                    literal,
                                    pos,
                                }),
                                "endif" => Ok(Token {
                                    token_type: TokenType::EndIf,
                                    literal: "endif".to_string(),
                                    pos,
                                }),
                                "global" => Ok(Token {
                                    token_type: TokenType::Global,
                                    literal: "global".to_string(),
                                    pos,
                                }),
                                "return" => Ok(Token {
                                    token_type: TokenType::Return,
                                    literal: "return".to_string(),
                                    pos,
                                }),
                                "procedure" => Ok(Token {
                                    token_type: TokenType::Procedure,
                                    literal: "procedure".to_string(),
                                    pos,
                                }),
                                "endprocedure" => Ok(Token {
                                    token_type: TokenType::EndProcedure,
                                    literal: "procedure".to_string(),
                                    pos,
                                }),
                                _ => Ok(Token {
                                    token_type: TokenType::Identifier,
                                    literal,
                                    pos,
                                }),
                            }
                        } else if c.is_numeric() {
                            // Collect number
                            let (literal, pos) = self.collect_literal(c, |cursor, c| {
                                !cursor.is_next_newline() && c.is_some_and(|c| c.is_numeric())
                            });
                            Ok(Token {
                                token_type: TokenType::Number,
                                literal,
                                pos,
                            })
                        } else {
                            Err(LexerError::UnexpectedCharacter(c))
                        }
                    }
                },
                None => Ok(Token {
                    token_type: TokenType::Eof,
                    literal: "\0".to_string(),
                    pos: PositionInfo {
                        line: self.cursor.lines_number(),
                        index: self.cursor.offset(),
                    }
                    .to_single_letter_token_position(),
                }),
            }
        };
        tok
    }
}
#[cfg(test)]
mod test {
    macro_rules! test_helper {
        ($test_str: expr, [$($x:expr),*]) => {
            {
                let mut lexer= Lexer::from($test_str);
                $(
                    assert_eq!(lexer.bump(),Ok($x));
                )*
            }
        };
    }
    use super::*;
    /// Test identifier, string and number
    #[test]
    fn test_str_num() {
        let test_str = "identifier123 12345\"hello\\n\"";
        test_helper!(
            test_str,
            [
                Token {
                    token_type: TokenType::Identifier,
                    literal: "identifier123".to_string(),
                    pos: TokenPosition {
                        from: PositionInfo { index: 0, line: 0 },
                        to: PositionInfo { index: 12, line: 0 },
                    },
                },
                Token {
                    token_type: TokenType::Number,
                    literal: "12345".to_string(),
                    pos: TokenPosition {
                        from: PositionInfo { index: 14, line: 0 },
                        to: PositionInfo { index: 18, line: 0 },
                    },
                },
                Token {
                    token_type: TokenType::String,
                    literal: "hello\n".to_string(),
                    pos: TokenPosition {
                        from: PositionInfo { line: 0, index: 19 },
                        to: PositionInfo { line: 0, index: 27 },
                    },
                }
            ]
        );
    }
    /// Test comparison
    #[test]
    fn test_compare_and_logic() {
        let test_str = "(a>b OR a<=b) AND (a>=b OR b<a OR a==b) AND c!=d";
        test_helper!(
            test_str,
            [
                Token {
                    token_type: TokenType::LeftParen,
                    literal: "(".to_string(),
                    pos: PositionInfo { line: 0, index: 0 }.to_single_letter_token_position(),
                },
                Token {
                    token_type: TokenType::Identifier,
                    literal: "a".to_string(),
                    pos: PositionInfo { line: 0, index: 1 }.to_single_letter_token_position(),
                },
                Token {
                    token_type: TokenType::Greater,
                    literal: ">".to_string(),
                    pos: PositionInfo { line: 0, index: 2 }.to_single_letter_token_position(),
                }
            ]
        )
    }
}
