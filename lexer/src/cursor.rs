use std::{
    ops::Deref,
    str::{CharIndices, Lines},
};

use crate::position::{PositionInfo, TokenPosition};

/// A struct that stores char and its position,
/// while being able to be dereferenced into a char when needed
#[derive(Debug, Clone, PartialEq)]
pub struct FancyChar {
    pub v: char,
    pub pos: PositionInfo,
}
#[allow(clippy::from_over_into)]
impl Into<char> for FancyChar {
    fn into(self) -> char {
        self.v
    }
}
impl Deref for FancyChar {
    type Target = char;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}
#[derive(Debug, Clone)]
struct CharsHelper<'a> {
    lines: Lines<'a>, // The array of characters must outlive the Cursor struct
    chars: CharIndices<'a>,
    last_index: usize,
    line: usize,
}
impl<'a> Iterator for CharsHelper<'a> {
    type Item = FancyChar;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((i, v)) = self.chars.next() {
            self.last_index = i;
            Some(FancyChar {
                v,
                pos: PositionInfo {
                    line: self.line,
                    index: i,
                },
            })
        } else {
            // if the current line reached the end, try the next line instead
            if let Some(l) = self.lines.next() {
                // Increment the line number and set the line content
                self.chars = l.char_indices();
                self.line += 1;
                // Instead of bumping and getting the first character, emmit a newline character
                Some(FancyChar {
                    v: '\n',
                    pos: PositionInfo {
                        line: self.line,
                        index: self.last_index + 1,
                    },
                })
            } else {
                None
            }
        }
    }
}
impl<'a> From<&'a str> for CharsHelper<'a> {
    fn from(value: &'a str) -> Self {
        let mut lines = value.lines();
        Self {
            chars: lines.next().unwrap_or("").char_indices(),
            lines,
            line: 0,
            last_index: 0,
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Cursor<'a> {
    characters: CharsHelper<'a>, // The array of characters must outlive the Cursor struct
}

impl<'a> Cursor<'a> {
    /// Consume characters as long as the predicate function returns `true`
    ///
    /// `None` is used as an argument for predicate when the iterator is finished
    pub fn eat_while(&mut self, predicate: impl Fn(&mut Self, Option<FancyChar>) -> bool) {
        while predicate(self, self.characters.clone().next()) {
            self.characters.next();
        }
    }
    /// Return true is the next `bump` will result in advancing to the next line
    pub fn is_next_newline(&self) -> bool {
        let mut new = self.clone();
        !new.characters.chars.next().is_some()
    }
    /// Consume whitespace until a non-whitespace character is hit
    pub fn eat_whitespace(&mut self) {
        // If `None` (the iterator is finished), return `false` and stop consuming characters
        // If the character is a whitespace character, return true and continue
        self.eat_while(|_cursor, arg| arg.is_some_and(|c| *c == ' '));
    }
    /// Advance and return the next character
    pub fn bump(&mut self) -> Option<FancyChar> {
        self.characters.next()
    }
    /// Collect character onto a specified buffer
    /// and advance characters while the predicate function returns `true`.
    ///
    /// If no characters are collected, `None` is returned.
    /// Otherwise, the `TokenPosition` is returned.
    ///
    /// Thus,
    /// this function guarantees the returned value matches the predicate function's requirement
    /// and won't silently cause issue by returning empty string when parsing,
    /// for example, identifier
    ///
    /// `None` is used as an argument for predicate when the iterator is finished
    pub fn collect_while(
        &mut self,
        buffer: &mut String,
        predicate: impl Fn(&mut Self, Option<FancyChar>) -> bool,
    ) -> Option<TokenPosition> {
        let old_len = buffer.len();
        let first_char = self.characters.clone().next()?;
        let from = first_char.pos;

        let mut to = PositionInfo {
            line: usize::MAX,
            index: usize::MAX,
        }; // Should be overwritten if buffer.len is > 0, if it's not > 0, to will never be used
        while predicate(self, self.characters.clone().next()) {
            let val = self.bump().expect("Cannot collect None");
            to = val.pos;
            buffer.push(val.into());
        }
        if buffer.len() > old_len {
            // if the while loop pushed any new character onto the string, return Some, if it is unmodified, return None
            Some(TokenPosition { from, to })
        } else {
            None
        }
    }
    /// Peek the next character without advancing the cursor
    pub fn peek(&mut self) -> Option<FancyChar> {
        self.clone().bump() // Create a new instance of iterator and advance that instead
    }
    /// The line number of the line the cursor is on currently
    pub fn lines_number(&self) -> usize {
        self.characters.line
    }
    ///  Offset relative to the beginning of line
    pub fn offset(&self) -> usize {
        self.characters.chars.offset()
    }
}

impl<'a> From<&'a str> for Cursor<'a> {
    /// Convert from a string slice to a cursor
    fn from(value: &'a str) -> Self {
        Self {
            characters: value.into(),
        }
    }
}

#[cfg(test)]
mod test {
    /// Simple function that makes `char::is_ascii_alphabetic` a usable predicate
    #[inline]
    pub fn is_ascii_alphabetic(_c: &mut Cursor, arg: Option<FancyChar>) -> bool {
        arg.is_some_and(|c| c.is_ascii_alphabetic())
    }
    use super::*;
    #[test]
    fn test_bump_and_peek() {
        let test_str = "ab\ncc\nd";
        let mut cursor = Cursor::from(test_str);
        assert_eq!(
            cursor.bump(),
            Some(FancyChar {
                v: 'a',
                pos: PositionInfo { line: 0, index: 0 }
            })
        );
        assert_eq!(
            cursor.bump(),
            Some(FancyChar {
                v: 'b',
                pos: PositionInfo { line: 0, index: 1 }
            })
        );
        assert_eq!(
            cursor.bump(),
            Some(FancyChar {
                v: 'c',
                pos: PositionInfo { line: 1, index: 0 }
            })
        );
        assert_eq!(
            cursor.bump(),
            Some(FancyChar {
                v: 'c',
                pos: PositionInfo { line: 1, index: 1 }
            })
        );
        assert_eq!(
            cursor.bump(),
            Some(FancyChar {
                v: 'd',
                pos: PositionInfo { line: 2, index: 0 }
            })
        );
        assert_eq!(cursor.peek(), None);
        assert_eq!(cursor.bump(), None);
    }
    #[test]
    fn test_all() {
        let test_str = " \n  \r\n\t ==============Hello World!".to_string();
        let mut cursor = Cursor::from(&*test_str);
        cursor.eat_whitespace();
        cursor.eat_while(|_cursor, c| c.map_or(false, |c| *&*c == '='));
        assert_eq!(
            cursor.peek(),
            Some(FancyChar {
                v: 'H',
                pos: PositionInfo { line: 2, index: 16 }
            })
        );
        let mut buf = String::new();
        assert_eq!(
            cursor.collect_while(&mut buf, is_ascii_alphabetic),
            Some(TokenPosition {
                from: PositionInfo { line: 2, index: 16 },
                to: PositionInfo { line: 2, index: 20 }
            })
        );
        assert_eq!(buf, "Hello");
        let mut buf = String::new();
        assert_eq!(cursor.collect_while(&mut buf, is_ascii_alphabetic), None);
        assert_eq!(buf, "");
        assert_eq!(
            cursor.peek(),
            Some(FancyChar {
                v: ' ',
                pos: PositionInfo { line: 2, index: 21 }
            })
        );
        cursor.eat_whitespace();
        let mut buf = String::from("Testing ");
        assert_eq!(
            cursor.collect_while(&mut buf, is_ascii_alphabetic),
            Some(TokenPosition {
                from: PositionInfo { line: 2, index: 22 },
                to: PositionInfo { line: 2, index: 26 }
            })
        );
        assert_eq!(buf, "Testing World");
        assert_eq!(
            cursor.bump(),
            Some(FancyChar {
                v: '!',
                pos: PositionInfo { line: 2, index: 27 }
            })
        );
        assert_eq!(cursor.bump(), None);
    }
}
