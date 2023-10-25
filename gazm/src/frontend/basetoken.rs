#![deny(unused_imports)]
use super::TokenKind;

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Token<X: Clone> {
    pub kind: TokenKind,
    pub extra: X,
}

impl<X: Clone> Token<X> {
    pub fn new(kind: TokenKind, extra: X) -> Self {
        Self {
            kind,
            extra,
        }
    }
}

