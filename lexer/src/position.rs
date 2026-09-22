use crate::cursor::FancyChar;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TokenPosition {
    pub from: PositionInfo,
    pub to: PositionInfo,
}
impl TokenPosition {
    #[inline(always)]
    pub(crate) fn from_to(a: &FancyChar, b: &FancyChar) -> TokenPosition {
        TokenPosition {
            from: a.pos,
            to: b.pos,
        }
    }

    pub fn concat(a: TokenPosition, b: TokenPosition) -> TokenPosition {
        TokenPosition { from: a.from, to: b.to }
    }

    // todo: retain position information in ast
    pub fn temp_default() -> TokenPosition {
        PositionInfo {
            line: 0,
            index: 0,
        }.to_single_letter_token_position()
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PositionInfo {
    /// Line number, counting from 0
    pub line: usize,
    /// Index of the character relative to the start of the line
    pub index: usize,
}
impl PositionInfo {
    pub fn to_single_letter_token_position(self) -> TokenPosition {
        TokenPosition {
            from: self,
            to: self,
        }
    }
}
