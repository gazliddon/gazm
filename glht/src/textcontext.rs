use super::imgui;
use super::window::*;
use super::scrbox::ScrBox;
use super::colourcell::ColourCell;
use super::colour::Colour;
use super::v2::*;

pub struct TextContext<'a> {
    ui : &'a imgui::Ui<'a>,
    pub win_dims : TextWinDims,
    text_dims : ScrBox,
    dl : imgui::DrawListMut<'a>,
}

enum Command<'a> {
    Box(V2<usize>),
    Char(char),
    String(&'a str),
}

pub struct DrawCommand<'a> {
    pos : V2<isize>,
    colour : &'a Colour,
    command : Command<'a>
}

impl<'a> DrawCommand<'a> {
    fn new(pos : &'a V2<isize>, colour : &'a Colour, command : Command<'a>)  -> Self {
        Self {
            pos : *pos, colour, command
        }
    }
}

impl<'a> TextContext<'a> {
    fn draw(&'a self, command : DrawCommand<'a>) {
        let pos = &command.pos;
        let col = command.colour;

        self.with_clip_rect(&self.text_dims, || {
            match &command.command {
                Command::Box(dims) => self.draw_box(pos, &dims, col),
                Command::Char(ch) => self.do_draw_char(*ch, col, &pos),
                Command::String(text) => self.draw_text(&pos,text, col),
            };
        });
    }

    fn clip(&self, scr_box : &'a ScrBox) -> Option<ScrBox> {
        ScrBox::clip_box(&self.text_dims, &scr_box)
    }

    fn do_draw_char(&self, ch : char, col : &'a Colour, pos : &'a V2<isize>) {
        let pos = *pos + self.text_dims.pos;
        let mut s = String::new();
        s.push(ch);
        self.draw_text(&pos, &s, col);
    }

    fn with_clip_rect<F>(&'a self, scr_box : &'a ScrBox, f: F)
    where
        F: FnOnce(), {
            if let Some(new_box) = self.clip(scr_box) {
                let [min, max] = self.win_dims.as_pixel_extents_arrays( &new_box.pos, &new_box.dims);
                self.dl.with_clip_rect_intersect(min,max, f);
            }
        }

    pub fn new(ui : &'a imgui::Ui<'a>) -> Self {
        let win_dims = TextWinDims::new(ui);
        let dl = ui.get_window_draw_list();
        let text_dims = ScrBox::new(&V2::new(0,0), &win_dims.get_window_char_dims());
        Self {
            dl, win_dims, ui, text_dims
        }
    }

    pub fn clear(&self, col : &Colour) {
        self.draw_box(&self.text_dims.pos, &self.text_dims.dims, col);
    }

    pub fn clear_line(&self, col : &Colour, line : usize) {

        let pos = V2::new(0,line).as_isizes();
        let dims = V2::new(self.width(),1);

        self.draw_box(&pos, &dims, col);
    }

    pub fn height(&self)->usize {
        self.text_dims.dims.y
    }

    pub fn width(&self)->usize {
        self.text_dims.dims.x
    }

    pub fn draw_text(&'a self, pos : &'a V2<isize>, text : &'a str, col : &'a Colour) { 
        let pos = *pos + self.text_dims.pos;
        let [tl, _] = self.win_dims.as_pixel_extents_arrays( &pos, &V2::new(1,1));
        self.dl.add_text(tl,col,text);
    }

    pub fn draw_box(&'a self, pos : &'a V2<isize>, dims : &'a V2<usize>, col : &Colour) { 
        let pos = *pos + self.text_dims.pos;
        let [tl, br] = self.win_dims.as_pixel_extents( &pos, &dims);
        let tl = [tl.x, tl.y];
        let br = [br.x, br.y];
        self.dl.add_rect_filled_multicolor(tl, br, col, col, col, col );
    }

    pub fn draw_text_with_bg(&'a self, pos : &'a V2<isize>, text : &'a str, cols : &ColourCell) { 
        let dims = V2::new(text.len(), 1);
        self.draw_box(pos, &dims, &cols.bg);
        self.draw_text(pos, text, &cols.fg);
    }
}

pub struct LinePrinter<'a> {
    pub tc : &'a TextContext<'a>,
    cols : ColourCell,
    pos : V2<isize>
}

impl<'a> LinePrinter<'a> {

    pub fn new(tc : &'a TextContext<'a>) -> Self {
        let cols = ColourCell::new_bw();
        let pos = V2::new(0,0);
        Self { tc, cols, pos}
    }

    pub fn cols(&mut self, cols : &ColourCell) -> &mut Self {
        self.cols = *cols;
        self
    }

    fn chars_left(&'a self) -> isize {
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
