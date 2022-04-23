use super::textscreen::TextScreen;
pub use super::textscreen::Glyph;

use super::styles::StylesDatabase;

use super::v2::*;

pub trait Doc {
    fn num_of_lines(&self) -> usize;
    fn get_line_chars(&self, line : usize) -> Option<&str>;
    fn get_line(&self, line : usize) -> Option<Box<dyn Iterator<Item = Glyph>>>;
}

pub struct DocWin {
    top : usize,
    dims : V2<usize>,
    text_screen : TextScreen,
    styles : StylesDatabase,
}

impl DocWin {
    pub fn new(_dims : V2<usize>) -> Self {
        panic!("TBD")
    }

    pub fn render<I: Doc>(&mut self, _doc : &I) {
        panic!("TBD")
    }
}


