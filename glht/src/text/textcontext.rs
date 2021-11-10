use crate::scrbox::ScrBox;
use crate::colourcell::ColourCell;
use crate::colour::Colour;
use crate::v2::*;
use super::Dimensions;

pub trait TextRenderer {
    fn get_window_dims(&self) -> ScrBox;

    fn draw_text(&self, pos : &V2<isize>, text : &str, col : &Colour);
    fn draw_box(&self, pos : &V2<isize>, dims : &V2<usize>, col : &Colour);
    fn draw_with_clip_rect<F>(&self, scr_box : &ScrBox, f: F) 
        where F: FnOnce();

    fn draw_text_with_bg(&self, pos : &V2<isize>, text : &str, cols : &ColourCell) { 
        let dims = V2::new(text.len(), 1);
        self.draw_box(pos, &dims, &cols.bg);
        self.draw_text( pos, text, &cols.fg);
    }

    fn draw_char(&self, pos : &V2<isize>, ch : char, col : &Colour) {
        let mut s = String::new();
        s.push(ch);
        self.draw_text( &pos, &s, col);
    }

    fn draw_char_with_bg(&self, pos : &V2<isize>, ch : char, cols : &ColourCell) {
        let mut s = String::new();
        s.push(ch);
        self.draw_text_with_bg( pos, &s, cols);
    }
}

pub struct TextContext<TR : TextRenderer> {
    dims : ScrBox,
    tr : TR,
}

impl<TR : TextRenderer> super::Dimensions<isize> for TextContext<TR> {
    fn pos(&self) -> V2<isize> {
        panic!("")

    }
    fn dims(&self) -> V2<isize> {
        panic!("")
    }

}

impl<TR : TextRenderer> TextContext< TR> {
    pub fn new(tr : TR) -> Self {
        let dims = tr.get_window_dims();

        Self {
            tr, dims
        }
    }

    fn clip(&self, scr_box : &ScrBox) -> Option<ScrBox> {
        ScrBox::clip_box(&self.dims, &scr_box)
    }

    pub fn clear(&self, col : &Colour) {
        self.tr.draw_box( &self.dims.pos, &self.dims.dims, col);
    }

    pub fn clear_line(&self, col : &Colour, line : usize) {
        let pos = V2::new(0,line).as_isizes();
        let dims = V2::new(self.width(),1);
        self.tr.draw_box( &pos, &dims.as_usizes(), col);
    }


    pub fn draw_text(&self, pos : &V2<isize>, text : &str, col : &Colour) { 
        self.tr.draw_text( pos, text, col);
    }

    pub fn draw_text_with_bg(&self, pos : &V2<isize>, text : &str, cols : &ColourCell) { 
        self.tr.draw_text_with_bg( pos, text, cols);
    }

    pub fn draw_box(&self, pos : &V2<isize>, dims : &V2<usize>, col : &Colour) { 
        self.tr.draw_box( pos, dims, col);
    }

    fn draw_char(&self, pos : &V2<isize>, ch : char, col : &Colour) {
        self.tr.draw_char( pos, ch, col);
    }
}

pub struct LinePrinter<'a, TR : TextRenderer> {
    pub tc : &'a TR,
    cols : ColourCell,
    pos : V2<isize>
}

impl<'a, TR : TextRenderer + Dimensions<isize> > LinePrinter<'a, TR> {

    pub fn new(tc : &'a TR) -> Self {
        let cols = ColourCell::new_bw();
        let pos = V2::new(0,0);
        Self { tc, cols, pos}
    }

    pub fn cols(&mut self, cols : &ColourCell) -> &mut Self {
        self.cols = *cols;
        self
    }

    fn chars_left(&self) -> isize {
        self.tc.width() as isize - self.pos.x
    }

    pub fn print(&mut self, text : &str)  -> &mut Self {
        self.tc.draw_text_with_bg(&self.pos,&text, &self.cols);
        self.pos = self.pos + V2::new(text.len(), 0).as_isizes();
        self
    }

    pub fn print_col(&mut self, text : &str, cols : &ColourCell)  -> &mut Self{
        self.cols(cols).print(text)
    }

    pub fn cr(&mut self) -> &mut Self {
        let chars_left = self.chars_left();

        if chars_left > 0 {
            let dims = V2::new(chars_left, 1).as_usizes();
            self.tc.draw_box(&self.pos, &dims, &self.cols.bg);
        }

        self.pos = V2::new(0, self.pos.y + 1);
        self
    }
}

