use crate::error::ParserError;
use lexer::lexer::{Lexer, LexerError};
use lexer::token::{Token, TokenType};

#[derive(Debug, Clone)]
pub(crate) struct Cursor<'a> {
    lexer: Lexer<'a>,
}
impl<'a> From<&'a str> for Cursor<'a> {
    fn from(src: &'a str) -> Self {
        Self {
            lexer: Lexer::from(src),
        }
    }
}
impl<'a> Cursor<'a> {
    pub(crate) fn is_eof(&self) -> Result<bool, LexerError> {
        self.lexer.is_eof()
    }
    pub(crate) fn bump(&mut self) -> Result<Token, LexerError> {
        self.lexer.bump()
    }
    pub(crate) fn expect_end_of_statement_or_eof(&mut self) -> Result<Token, ParserError> {
        /*let tok=self.bump()?;
        if tok.token_type == TokenType::Newline || tok.token_type == TokenType::Eof {
            Ok(tok)
        }else{
            Err(ExpectedEofOrNewline(tok))
        }*/
        match self.peek()? {
            tok if tok.token_type == TokenType::Newline => {
                self.bump()?;
                Ok(tok)
            }
            tok if tok.token_type == TokenType::Eof => {
                self.bump()?;
                Ok(tok)
            }
            tok if tok.token_type == TokenType::EndIf => Ok(tok),
            tok if tok.token_type == TokenType::Else => Ok(tok),
            tok => Err(ParserError::ExpectedAGotB(TokenType::Newline, tok)),
        }
    }
    pub(crate) fn expect_end_of_statement(&mut self) -> Result<Token, ParserError> {
        //self.expect(TokenType::Newline)
        match self.peek()? {
            tok if tok.token_type == TokenType::Newline => {
                self.bump()?;
                Ok(tok)
            }
            tok if tok.token_type == TokenType::Else => Ok(tok),
            tok if tok.token_type == TokenType::EndIf => Ok(tok),
            tok => Err(ParserError::ExpectedAGotB(TokenType::Newline, tok)),
        }
    }
    /// If the bumped token's token_type does not match the one specified, return error of [`ParserError::ExpectedAGotB`], otherwise, return the token.
    pub(crate) fn expect(&mut self, token_type: TokenType) -> Result<Token, ParserError> {
        let tok = self.bump();
        match tok {
            Ok(t) => {
                if t.token_type == token_type {
                    Ok(t)
                } else {
                    Err(ParserError::ExpectedAGotB(token_type, t))
                }
            }
            Err(err) => Err(ParserError::LexerError(err)),
        }
    }
    pub(crate) fn peek(&mut self) -> Result<Token, LexerError> {
        self.lexer.peek()
    }
    pub(crate) fn peek_n<const N: usize>(&mut self) -> Result<[Token; N], LexerError> {
        let mut lex = self.lexer.clone();
        let mut toks = Vec::with_capacity(N);
        for _ in 0..N {
            toks.push(lex.bump()?);
        }
        Ok(toks.try_into().unwrap())
    }
    pub(crate) fn eat_newline(&mut self) -> Result<(), LexerError> {
        while self.peek()?.token_type == TokenType::Newline {
            self.bump()?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod test {

    use lexer::position::{PositionInfo, TokenPosition};

    use super::*;
    fn test_expect() {
        let code = "\
        hello==true
        ";
        let mut cursor = Cursor::from(code);
        assert_eq!(
            cursor.expect(TokenType::Identifier),
            Ok(Token {
                token_type: TokenType::Identifier,
                literal: "hello".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 0, index: 0 },
                    to: PositionInfo { line: 0, index: 4 }
                }
            })
        );
        assert_eq!(
            cursor.expect(TokenType::Equal),
            Ok(Token {
                token_type: TokenType::Equal,
                literal: "==".to_string(),
                pos: TokenPosition {
                    from: PositionInfo { line: 0, index: 5 },
                    to: PositionInfo { line: 0, index: 6 }
                }
            })
        );
        assert_eq!(
            cursor.expect(TokenType::Lesser),
            Err(ParserError::ExpectedAGotB(
                TokenType::Lesser,
                Token {
                    token_type: TokenType::True,
                    literal: "true".to_string(),
                    pos: TokenPosition {
                        from: PositionInfo { line: 0, index: 6 },
                        to: PositionInfo { line: 0, index: 10 }
                    }
                }
            ))
        )
    }
}
