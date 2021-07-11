use super::textscreen::{ Cell, TextScreen, CursorTrait};
use super::styles::{ StylesDatabase, TextStyles };
use romloader::AnnotatedSourceFile;

impl Doc for AnnotatedSourceFile {
    fn get_line(&self,_line : usize) -> Option<&[Cell]> {
        None
    }

    fn num_of_lines(&self) -> usize {
        self.num_of_lines()
    }
}

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

        let ts = &mut self.text_screen;

        ts.clear(' ',&text_styles.normal);

        let mut c = ts.cursor();

        for line in self.top..(self.top + self.dims.y) {
            if let Some(text) = doc.get_line(line) {
                c.write_cells(text);
            }
        }
    }
}


