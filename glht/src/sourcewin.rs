
// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//
use romloader::SourceStore;
use super::imgui;
use romloader::SourceStore;
// use romloader::AnnotatedSourceFile;
use super::docwin;

use super::window::*;

use super::styles::TextStyles;

use super::textscreen::{TextScreen, ScrBox};
use super::colourcell::ColourCell;
use super::colour::Colour;

use vector2d::Vector2D  as V2;

use super::events::Events;
use Events::*;

use super::styles::*;

use docwin::Glyph;

pub struct SourceView {
    glyphs : Vec<Vec<Glyph>>,
}


impl docwin::Doc for SourceView {
    fn num_of_lines(&self) -> usize {
        self.glyphs.len()
    }

    fn get_line<'a>(&'a self, line : usize) -> Option<Box<dyn Iterator<Item = Glyph> + 'a>> {
        if let Some(l) = self.glyphs.get(line)  {
            let i = l.iter().cloned();
            Some(Box::new(i))
        } else {
            None
        }
    }

    fn get_line_chars<'a>(&'a self, _line : usize) -> Option<&'a str> {
        panic!("fucked")
    }
}

impl SourceView {
    pub fn new(src : &[String]) -> Self {
        let gnew = |glyph| Glyph::new(glyph,&ColourCell::new_bw());

        let  glyphs = src.iter().map(|line| {
            line.chars().map(gnew).collect() });

        SourceView { glyphs : glyphs.collect() }
    }
}

pub struct SourceWin {
    cursor : usize,
    scroll_offset : usize,
    styles : StylesDatabase,
    source_file : Option<String>,
    source_view : SourceView
}

impl Default for SourceWin {
    fn default() -> Self {

        let src = include_str!("sourcewin.rs");
        let lines : Vec<_> = src.split("\n").map(String::from).collect();
        let source_view = SourceView::new(&lines);

        Self {
            cursor : 0,
            scroll_offset : 0,
            styles : StylesDatabase::default(),
            source_file: None,
            source_view
        }
    }
}

pub enum Zone {
    TOP,
    MIDDLE,
    BOTTOM,
}

#[derive(Debug, Clone, PartialEq)]
struct PrintStr {
    pub text : String,
    pub cols : ColourCell,
}

impl PrintStr {
    pub fn new()-> Self {
        let mut text = String::new();
        text.reserve(100);
        Self {
            text,
            cols : ColourCell::new_bw()
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }

    pub fn is_empty(&self) -> bool {
        self.text.len() == 0
    }

    pub fn add(&mut self,  c : char, cols : ColourCell ) -> bool {

        if cols == self.cols || self.is_empty() {
            self.text.push(c);
            self.cols = cols;
            true
        } else  {
            false
        }
    }
}

struct TextContext<'a> {
    ui : &'a imgui::Ui<'a>,
    win_dims : TextWinDims,
    text_dims : ScrBox,
    dl : imgui::DrawListMut<'a>,
}

enum Command<'a> {
    Box(V2<usize>),
    Char(char),
    String(&'a str),
}

struct DrawCommand<'a> {
    pos : V2<isize>,
    colour : &'a Colour,
    command : Command<'a>
}

impl<'a> DrawCommand<'a> {
    pub fn new(pos : &'a V2<isize>, colour : &'a Colour, command : Command<'a>)  -> Self {
        Self {
            pos : *pos, colour, command
        }
    }
}


impl<'a> TextContext<'a> {

    pub fn draw(&'a self, command : DrawCommand<'a>) {
        let pos = &command.pos;
        let col = command.colour;

        self.with_clip_rect(&self.text_dims, || {
            match &command.command {
                Command::Box(dims) => self.do_draw_box(col, &ScrBox::new(pos, &dims)),
                Command::Char(ch) => self.do_draw_char(*ch, col, &pos),
                Command::String(text) => self.do_draw_text(text, col, &pos),
            };
        });
    }

    fn clip(&self, scr_box : &'a ScrBox) -> Option<ScrBox> {
        ScrBox::clip_box(&self.text_dims, &scr_box)
    }

    fn do_draw_box(&self, col : &'a Colour, scr_box : &'a ScrBox)  {
        let [tl, br] = self.win_dims.as_pixel_extents( &scr_box.pos, &scr_box.dims);
        let tl = [tl.x, tl.y];
        let br = [br.x, br.y];
        self.dl.add_rect_filled_multicolor(tl, br, col, col, col, col );
    }

    fn do_draw_char(&self, ch : char, col : &'a Colour, pos : &'a V2<isize>) {
        let mut s = String::new();
        s.push(ch);
        self.do_draw_text(&s, col, pos);
    }

    fn do_draw_text(&self, text : &'a str, col : &'a Colour, pos : &V2<isize>) {
        let [tl, _] = self.win_dims.as_pixel_extents_arrays( &pos, &V2::new(1,1));
        self.dl.add_text(tl,col,text);
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
        self.win_dims.height()
    }

    pub fn width(&self)->usize {
        self.win_dims.width()
    }

    pub fn draw_text(&'a self, pos : &'a V2<isize>, text : &'a str, col : &'a Colour) { 
        let text_command = DrawCommand::new(pos,col, Command::String(text));
        self.draw(text_command);
    }

    pub fn draw_box(&'a self, pos : &'a V2<isize>, dims : &'a V2<usize>, col : &Colour) { 
        let box_command = DrawCommand::new(&pos, &col,Command::Box(*dims));
        self.draw(box_command);
    }

    pub fn draw_text_with_bg(&'a self, pos : &'a V2<isize>, text : &'a str, cols : &ColourCell) { 
        let dims = V2::new(text.len(), 1);
        self.draw_box(pos, &dims, &cols.bg);
        self.draw_text(pos, text, &cols.fg);
    }

}

impl TextScreen {
    pub fn render(&self, _ui: &imgui::Ui ) {
        panic!("TBD")

            // let wind_dims = TextWinDims::new(ui);

            // let char_box_dims = V2{x:1, y:1};
            // let V2{x : cols, y : rows} = wind_dims.get_window_char_dims();

            // let draw_list = ui.get_window_draw_list();

            // for y in 0..rows {
            //     for x in 0..cols {

            //         if let Some(Cell {col, pos, text}) = self.get_cell(V2{x,y}.as_isizes()) {

            //             let ColourCell{bg,fg} = &col;

            //             let [tl, br] = wind_dims.get_box_dims(
            //                 pos.as_usizes(),
            //                 char_box_dims);

            //             let tl = [tl.x, tl.y];
            //             let br = [br.x, br.y];

            //             draw_list.add_rect_filled_multicolor(tl, br, bg, bg, bg, bg );
            //             draw_list.add_text(tl,fg,text);
            //         }
            //     }

            // }
    }
}

struct ScrollZones {
    top : V2<usize>,
    bottom : V2<usize>
}

impl ScrollZones {

    pub fn new(win_dims : &TextWinDims, top_line : usize, _lines_in_doc : usize, sz : usize) -> Self {

        let sz = sz as isize;
        let top_line = top_line as isize;

        let win_char_height = win_dims.get_window_char_dims().y as isize;

        let adj = if top_line < sz {
            sz - (sz - top_line)
        } else {
            sz
        };

        let top = V2::new(top_line, adj) ;

        let bottom_line = top_line + win_char_height - 1;

        let bottom_adj = if bottom_line < sz {
            0
        } else {
            sz
        };

        let bottom = V2::new(( bottom_line - bottom_adj ) + 1, bottom_adj);

        // println!("bottom_line: {:?}", bottom_line);
        // println!("wc+_dims:    {:?}", win_dims.get_window_char_dims());
        // println!("top:         {:?}", top);
        // println!("bottom:      {:?}", bottom);

        // panic!("lskalkssa");

        Self {
            top : top.as_usizes(),
            bottom : bottom.as_usizes(),
        }
    }

    fn in_span(line : usize, span : &V2<usize>) -> bool {
        let V2{x : y, y : h} = *span;
        h > 0 && (line >= y && line < (y+h))

    }

    pub fn in_top_zone(&self, line : usize) -> bool {
        Self::in_span(line, &self.top)
    }

    pub fn in_bottom_zone(&self, line : usize) -> bool {
        Self::in_span(line, &self.bottom)
    }

    pub fn in_scroll_zone(&self, line : usize) -> bool {
        self.in_bottom_zone(line) || self.in_top_zone(line)
    }
}


struct Formatter {
}

impl SourceWin {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn dims(&self) -> V2<isize> {
        panic!("TBD")
            // self.text_screen.dims
    }

    pub fn get_zone_from_cursor(&self, dims : &TextWinDims, cursor : usize) -> Zone {
        if cursor <= 3 {
            Zone::TOP
        } else if cursor >= ( dims.get_window_char_dims().y - 3 ) {
            Zone::BOTTOM
        } else {
            Zone::MIDDLE
        }
    }

    pub fn event(&mut self, event : Events) {

        let mut cursor = self.cursor as isize;
        let mut scroll_offset = self.scroll_offset as isize;

        match event {
            CursorUp => {
                cursor-=1;
            }

            CursorDown => {
                cursor+=1;
            }

            ScrollUp => {
                scroll_offset+=1;
                cursor+=1;
            }

            ScrollDown => {
                cursor-=1;
                scroll_offset-=1;
            }

            PageUp => {
                cursor+=1;
                scroll_offset+=20;
            }

            PageDown => {
                cursor-=20;
                scroll_offset-=20;
            }

            _ => ()
        }

        if cursor < 0 {
            cursor = 0;
        }

        if scroll_offset < 0 {
            scroll_offset = 0;
        }

        self.cursor = cursor as usize;
        self.scroll_offset = scroll_offset as usize;

    }

    pub fn resize(&mut self,  dims : V2<usize>) {
        info!("Resizing! rs: {:?} ",dims );
    }

    pub fn update(&mut self) {
    }

    fn get_scroll_zones(&self, win_dims : &TextWinDims, lines_in_doc : usize) -> ScrollZones {
        ScrollZones::new(win_dims, self.scroll_offset,lines_in_doc, 3 )
    }

    fn get_source_file<'a>(&'a self, source_store : &'a SourceStore) -> Option<&'a romloader::AnnotatedSourceFile> {
        self.source_file.as_ref().and_then(|f| source_store.get(f))
    }

    pub fn render(&mut self, ui: &imgui::Ui, source_store : &SourceStore, pc : u16) {
        // use romloader::Location;
        
        let text_styles = TextStyles::new(&self.styles);

        if self.source_file.is_none() {
            self.source_file = source_store.add_to_loc(pc).map(|l| l.file.clone());
        }

        let tc = TextContext::new(ui);
        let current_col = text_styles.normal;

        // tc.clear(&current_col.bg);

        if let Some(sf) = self.get_source_file(source_store) {

            for (y,sl) in sf.lines.iter().enumerate() {
                let mut chars_drawn = 0;

                // tc.clear_line(&current_col.bg, y);

                if let Some(ref line) = sl.line {
                    let pos = V2::new(0,y).as_isizes();
                    tc.draw_text_with_bg(&pos,&line, &current_col);
                    chars_drawn = chars_drawn + line.len();
                }

                let to_draw = tc.width() - chars_drawn;

                if to_draw > 0 {
                    let pos = V2::new(chars_drawn, y).as_isizes();
                    let dims = V2::new(to_draw, 1);
                    tc.draw_box(&pos, &dims, &current_col.bg);
                }
            }
        }

        // tc.render_it(x, &self.source_view);


        // if !window_info.is_visible() {
        //     let cd = window_info.get_char_dims();
        //     let pd = window_info.get_pixel_dims();

        //     println!("char dims {} {}", cd.x, cd.y);
        //     println!("pixel dims {} {}", pd.x, pd.y);
        //     panic!("Ficled");

        //     // return
        // }

        // if self.source_file.is_none() {
        //     self.source_file = source_store.add_to_loc(pc).map(|l| l.file.clone());
        // }

        // if let Some(sf) = self.get_source_file(source_store) {
        //     let file = self.source_file.clone().unwrap_or(String::from("No file"));

        //     let text_styles = TextStyles::new(&self.styles);
        //     let window_dims = window_info.get_window_char_dims();

        //     let scroll_zones = self.get_scroll_zones(&window_info, sf.num_of_lines());

        //     let mut screen = TextScreen::new(window_dims);

        //     screen.clear(' ',&text_styles.normal);

        //     let mut c = screen.cursor();
        //     c.set_col(&text_styles.normal);

        //     let blank = String::new();
        //     let mut loc = Location::new(&file, self.scroll_offset );

        //     for line in self.scroll_offset..(self.scroll_offset + window_dims.y) {

        //         if let Some(source_line) = source_store.loc_to_source_line(&loc) {

        //             let is_cursor_line  = self.cursor as usize == line;
        //             let is_pc_line = Some(pc) == source_line.addr;
        //             let mut is_debug_line = scroll_zones.in_scroll_zone(line);

        //             if is_cursor_line && is_debug_line {
        //                 is_debug_line = false;
        //             }

        //             let (line_style, addr_style) = text_styles.get_source_win_style(is_cursor_line, is_pc_line, is_debug_line);

        //             let addr_str = source_line.addr
        //                 .map(|addr|
        //                     format!("{:04x}", addr))
        //                 .unwrap_or_else(||blank.clone());

        //             let addr_str = format!("{:^8}", addr_str);

        //             let line_str = source_line.line.clone().unwrap_or_else(|| blank.clone());

        //             c.set_col(line_style).clear_line();
        //             c.set_col(addr_style).write(&addr_str);
        //             c.set_col(line_style).write(" ").write(&line_str);

        //         } else {
        //             c.write(&format!( "{} {} *LINE NOT FOUND*",file, line  ));
        //             break;
        //         }

        //         loc.inc_line_number();
        //         c.cr();
        //     }

        //     screen.render(ui);
        // } else {
        //     panic!("can't fine file!")
        // }
    }
}


use emu::cpu::Regs;

struct BreakPoints {
}

struct DebuggerState<'a> {
    pub regs : &'a Regs,
    pub break_points : &'a BreakPoints,
    pub rom : &'a romloader::Rom,
    pub styles : &'a StylesDatabase,
}

struct SourceLine {
    pub addr : String,
    pub source : String,

}

// impl<'a> DebuggerState<'a> {
//     pub fn get_source_line(&self, loc : &'a romloader::Location) -> Option<Vec<Cell>> {

//         let Regs{pc,..} = self.regs;

//         if let Some(line) = self.rom.sources.get_line(&loc) {

//             let mut ret = "normal";

//             let addr_string = if let Some(addr) = self.rom.get_location_addr_range(&loc) {
//                 if addr.start == *pc as usize {
//                     ret = "pc"
//                 }

//                 format!("{:04X}", addr.start)
//             } else {
//                 "".to_string()
//             };

//             let final_str = format!("{:<4} {}", addr_string, line);
//         }

//         None
//     }
// }

/*
   (location, debugger_state, styles) -> formatted line
   */

