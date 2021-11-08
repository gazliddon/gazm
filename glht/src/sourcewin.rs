
// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//
use romloader::SourceStore;
use super::scrbox::ScrBox;
use super::docwin;
use super::window::*;
use super::styles::TextStyles;
use super::colourcell::ColourCell;
use super::v2::*;

use super::events::Events;
use Events::*;
use super::styles::*;
use docwin::Glyph;
use super::textcontext::*;
use super::colour::*;

trait RenderDoc<'a> {
    fn render_line(&mut self, cursor : usize, win_ypos : usize, doc_ypos : usize);
    fn window_height(&self) -> usize;
    fn doc_height(&self) -> usize;

    fn render_doc(&mut self, offset : usize, cursor : usize)  -> usize {
        let mut lines_rendered = 0;

        let range = ( 0..self.window_height()).map(|y| (y, y + offset));

        for ( win_ypos, doc_ypos ) in range {
            self.render_line(cursor, win_ypos, doc_ypos);
            lines_rendered = lines_rendered + 1;
        }
        lines_rendered
    }
}

struct SourceRenderer<'a> {
    sf : &'a romloader::AnnotatedSourceFile,
    pc : u16,
    text_styles : &'a TextStyles,
    blank: String,
    lp : LinePrinter<'a>,

}

impl<'a> SourceRenderer<'a> {
    pub fn new(pc : u16, sf: &'a romloader::AnnotatedSourceFile, text_styles: &'a TextStyles, tc : &'a TextContext<'a>) -> Self {
        let blank = String::new();
        let lp = LinePrinter::new(tc);
        Self {
            blank, sf, pc, text_styles, lp
        }
    }
}

impl<'a> RenderDoc<'a> for SourceRenderer<'a> {
    fn window_height(&self) -> usize {
        self.lp.tc.height()
    }

    fn doc_height(&self) -> usize {
        self.sf.num_of_lines()
    }

    fn render_line(&mut self, cursor : usize, win_ypos : usize, doc_ypos : usize) {
        if let Some(sl) = self.sf.line(doc_ypos) {

            let addr_str = sl.addr.map(|x| format!("{:04X}",x)).unwrap_or(self.blank.clone());

            let source_text = sl.line.as_ref().unwrap_or(&self.blank);

            let is_pc_line = sl.addr.map(|p| p == self.pc).unwrap_or(false);
            let is_cursor_line = win_ypos == cursor;
            let is_debug_line = false;

            let (line_col, addr_col) = self.text_styles.get_source_win_style(is_cursor_line, is_pc_line, is_debug_line);

            self.lp.cols(addr_col);
            self.lp.print(&format!("{:4} ", addr_str));
            self.lp.cols(line_col);
            self.lp.print(" ");
            self.lp.print(source_text);
        }

        self.lp.cr();
    }
}


pub struct SourceView {
    glyphs : Vec<Vec<Glyph>>,
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
    source_view : SourceView,
    frame_time : FrameTime,
    pc : u16,

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
            source_view,
            frame_time : FrameTime::from_now(), 
            pc: 0
        }
    }
}

pub enum Zone {
    TOP,
    MIDDLE,
    BOTTOM,
}

struct ScrollZones {
    top : V2<usize>,
    bottom : V2<usize>
}

impl ScrollZones {

    pub fn new(win_dims : &ScrBox, top_line : usize, _lines_in_doc : usize, sz : usize) -> Self {

        let sz = sz as isize;
        let top_line = top_line as isize;

        let win_char_height = win_dims.dims.y as isize;

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

    pub fn get_top_zone(&self) -> Option<ScrBox> {
        None
    }

    pub fn get_bottom_zone(&self) -> Option<ScrBox> {
        None
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

// To print a document
// Doc line number 
// Screen cursor position

use super::app::frametime::FrameTime;

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

    pub fn update(&mut self, frame_time : &FrameTime, source_store : &SourceStore, pc: u16) {
        self.frame_time = *frame_time;
        self.pc = pc;

        if self.source_file.is_none() {
            self.source_file = source_store.add_to_loc(pc).map(|l| l.file.clone());
        }
    }

    fn get_scroll_zones(&self, win_dims : &ScrBox, lines_in_doc : usize) -> ScrollZones {
        ScrollZones::new(win_dims, self.scroll_offset,lines_in_doc, 3 )
    }

    fn get_source_file<'a>(&'a self, source_store : &'a SourceStore) -> Option<&'a romloader::AnnotatedSourceFile> {
        self.source_file.as_ref().and_then(|f| source_store.get(f))
    }

    fn bind_cursor(&mut self, tc : &TextContext) {
        if self.cursor >= tc.height() {
            self.cursor = tc.height() -1;
        }
    }

    pub fn render(&self, tc : &TextContext, source_store : &SourceStore) {
        // self.bind_cursor(&tc);

        let text_styles = TextStyles::new(&self.styles);
        let offset = 0;

        let mut lines = 0;

        if let Some(sf) = self.get_source_file(source_store) {
            let mut renderer = SourceRenderer::new(self.pc, sf, &text_styles, tc);
            lines = renderer.render_doc(offset,self.cursor);
        }

        let scroll_zone_height = 10;

        let sz_y = tc.height() - scroll_zone_height ;
        let w = tc.width();
        let dims = &V2::new(w,scroll_zone_height);
        let col = &Colour::new(1.0, 0.0, 0.0, 0.5);
        tc.draw_box(&V2::new(0,sz_y).as_isizes(), dims,col);
        tc.draw_box(&V2::new(0,0), dims, col);

        let ry = &ColourCell::new(WHITE, RED);

        let sline = format!("c: {} - wh:{} {} - lines : {}", self.cursor, tc.width(), tc.height(), lines);

        tc.draw_text_with_bg(&V2::new(0,0), &sline, ry );
    }
}

struct ScrollTriggers {
    top_zone : usize,
    bottom_zone : usize,
}

impl ScrollTriggers {
    pub fn new(doc_offset : usize, doc_height : usize, window_height : usize ) -> Self {
        panic!("TBD")
    }
}

