#![deny(unused_imports)]
use super::{ TokenKind,Token };



#[derive(Clone,Copy,PartialEq,)]
pub struct ParseState {
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
        }
    }
}



pub type TSpan<'a> = unraveler::Span<'a, Token<'a>, ()>;

////////////////////////////////////////////////////////////////////////////////
impl unraveler::Item for Token<'_> {
    type Kind = TokenKind;

    fn get_kind(&self) -> Self::Kind {
        self.kind
    }
}

impl unraveler::Item for TokenKind {
    type Kind = TokenKind;

    fn get_kind(&self) -> Self::Kind {
        *self
    }
}

impl unraveler::Collection for TokenKind {
    type Item = TokenKind;

    fn at(&self, _index: usize) -> Option<&Self::Item> {
        Some(self)
    }

    fn length(&self) -> usize {
        1
    }
}
