use super::textscreen::{ Cell, TextScreen};
use super::styles::{ StylesDatabase, TextStyles };

use vector2d::Vector2D  as V2;

pub trait Doc {
    fn get_line(&self,line : usize) -> Option<&[Cell]>;
    fn num_of_lines(&self) -> usize;
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

    pub fn render(&mut self, doc : &dyn Doc) {

        let text_styles = TextStyles::new(&self.styles);

        self.text_screen.clear(' ',&text_styles.normal);

        let mut _c = self.text_screen.cursor();

        for line in self.top..(self.top + self.dims.y) {
            if let Some(_text) = doc.get_line(line) {
                // c.write
            }
        }
    }
}


