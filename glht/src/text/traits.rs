use crate::scrbox::ScrBox;
use crate::v2::*;
use crate::colour::Colour;
use crate::colourcell::ColourCell;


pub trait Dimensions<I: num::traits::Num> {
    fn dims(&self) -> V2<I>;

    fn width(&self) -> I {
        self.dims().x
    }

    fn height(&self) -> I {
        self.dims().y
    }

}

pub trait Extents<I: num::traits::Num>  : Dimensions<I> {
    fn pos(&self) -> V2<I>;
}

pub trait TextRenderer : Dimensions<usize> {

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
        self.draw_text( pos, &s, col);
    }

    fn draw_char_with_bg(&self, pos : &V2<isize>, ch : char, cols : &ColourCell) {
        let mut s = String::new();
        s.push(ch);
        self.draw_text_with_bg( pos, &s, cols);
    }
}

