#[derive(Clone)]
pub struct TextPos {
    pub line: usize,
    pub character: usize,
}

/// Contains information for an edit to the in memrory text file
/// start..end is half open, end = the character after the last char to edit

pub struct TextEdit<'a> {
    pub start: TextPos,
    pub end: TextPos,
    pub text: &'a str,
}

impl<'a> TextEdit<'a> {
    pub fn new(start: TextPos, end: TextPos, text: &'a str) -> Self {
        Self { start, end, text }
    }
}

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum EditErrorKind {
    #[error("Index out of range")]
    IndexOutOfRange
}

pub type EditResult<T> = Result<T,EditErrorKind>;

pub trait TextEditTrait {
    fn edit(&mut self, _edit: &TextEdit) -> EditResult<()>;
}
