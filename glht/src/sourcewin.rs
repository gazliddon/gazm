
// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//
use romloader::SourceStore;
use super::styles::TextStyles;
use super::v2::*;
use super::events::Events;
use Events::*;
use super::styles::*;
use super::text::*;
use super::colour::*;
use super::text::Dimensions;

struct Cycler {
    cols : Vec<Colour>,
    per_entry : f64,
    t_mul : f64,
}

impl Cycler {
    pub fn new(speed : f64, cols : Vec<Colour>) -> Self {
        let total_t = speed * cols.len() as f64;
        let t_mul = 1.0 / total_t;
        Self {
            cols, per_entry : speed, t_mul
        }
    }

    fn select(&self, t : f64) -> &Colour {
        &self.cols[( t.abs() as usize ) % self.cols.len()]
    }

    pub fn get_col(&self, t : f64) -> Colour{
        let t = t * self.t_mul;
        let c1 = self.select(t);
        let c2 = self.select(t + 1.0);
        c1.blend(&c2, t.fract())
    }
}


trait RenderDoc<'a> {
    fn render_line(&mut self, cursor : usize, win_ypos : usize, doc_ypos : usize);

    fn render_doc(&mut self, num_of_lines : usize, offset : usize, cursor : usize)  -> usize {
        let mut lines_rendered = 0;

        let range = ( 0..num_of_lines).map(|y| (y, y + offset));

        for ( win_ypos, doc_ypos ) in range {
            self.render_line(cursor, win_ypos, doc_ypos);
            lines_rendered = lines_rendered + 1;
        }
        lines_rendered
    }
}

struct SourceRenderer<'a,IR : TextRenderer  > {
    sf : &'a romloader::AnnotatedSourceFile,
    pc : u16,
    text_styles : &'a TextStyles,
    blank: String,
    lp : LinePrinter<'a, IR>,
}

impl<'a, TR : TextRenderer  > SourceRenderer<'a, TR> {
    pub fn new(pc : u16, sf: &'a romloader::AnnotatedSourceFile, text_styles: &'a TextStyles, tc : &'a TR) -> Self {
        let blank = String::new();
        let lp = LinePrinter::new(tc);
        Self {
            blank, sf, pc, text_styles, lp
        }
    }
}

impl<'a, TR : TextRenderer  > RenderDoc<'a> for SourceRenderer<'a, TR> {

    fn render_line(&mut self, cursor : usize, win_ypos : usize, doc_ypos : usize) {
        if let Some(sl) = self.sf.line(doc_ypos) {

            let addr_str = sl.addr.map(|x| format!("{:04X}",x)).unwrap_or(self.blank.clone());

            let source_text = sl.line.as_ref().unwrap_or(&self.blank);

            let is_pc_line = sl.addr.map(|p| p == self.pc).unwrap_or(false);
            let is_cursor_line = win_ypos == cursor;
            let is_debug_line = false;

            let (line_col, addr_col) = self.text_styles.get_source_win_style(is_cursor_line, is_pc_line, is_debug_line);

            self.lp.cols(&addr_col);
            self.lp.print(&format!("{:4} ", addr_str));
            self.lp.cols(&line_col);
            self.lp.print(" ");
            self.lp.print(source_text);
        }

        self.lp.cr();
    }
}

pub struct SourceWin {
    cursor : usize,
    scroll_offset : usize,
    styles : StylesDatabase,
    source_file : Option<String>,
    frame_time : FrameTime,
    pc : u16,
    win_dims : V2<usize>,
    ccol : Colour
}

impl Default for SourceWin {
    fn default() -> Self {

        Self {
            cursor : 0,
            scroll_offset : 0,
            styles : StylesDatabase::default(),
            source_file: None,
            frame_time : FrameTime::from_now(), 
            pc: 0,
            win_dims: V2::new(0,0),
            ccol : *WHITE
        }
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

    fn get_source_file<'a>(&'a self, source_store : &'a SourceStore) -> Option<&'a romloader::AnnotatedSourceFile> {
        self.source_file.as_ref().and_then(|f| source_store.get(f))
    }

    pub fn update<D : Dimensions<usize>>(&mut self, dims : &D, frame_time : &FrameTime, source_store : &SourceStore, pc: u16) {
        // FIX : dims being passed is wrong
        self.win_dims = dims.dims();
        self.frame_time = *frame_time;
        self.pc = pc;

        let cyc = Cycler::new(0.1, vec![
            *WHITE,
            *RED,
            *BLUE,
            *GREEN,
        ]);

        self.ccol = cyc.get_col(frame_time.now_as_seconds());

        if self.source_file.is_none() {
            self.source_file = source_store.add_to_loc(pc).map(|l| l.file.clone());
        }

        if self.cursor >= self.win_dims.y {
            self.cursor = self.win_dims.y -1;
        }
    }

    pub fn render<TR: TextRenderer >(&self, tc : &TR, source_store : &SourceStore) {
        let w = self.win_dims.x;
        let h = self.win_dims.y;


        let text_styles = TextStyles::new(&self.styles);
        let offset = 0;

        if let Some(sf) = self.get_source_file(source_store) {
            let mut renderer = SourceRenderer::new(self.pc, sf, &text_styles, tc);
            renderer.render_doc(h, offset,self.cursor);
        }

        // Pront scroll zones

        let scroll_zone_height = 1;

        let sz_y = h - scroll_zone_height ;
        let dims = &V2::new(w,scroll_zone_height);
        let mut col = self.ccol;
        col.set_alpha(0.5);

        tc.draw_box(&V2::new(0,sz_y).as_isizes(), dims,&col);
        tc.draw_box(&V2::new(0,0), dims, &self.ccol);

        // let ry = &ColourCell::new(WHITE, RED);
        // let sline = format!("c: {} - wh:{} {} - lines : {}", self.cursor, w, h, lines);
        // tc.draw_text_with_bg(&V2::new(0,0), &sline, ry );
        //
        // let mut pos = V2::new(0,0);
        // let dims = &V2::new(1,1);
        // tc.draw_box(&pos,dims, super::colour::WHITE);

        // pos.x = pos.x + 1;
        // tc.draw_box(&pos,dims, super::colour::RED);
        // pos.x = pos.x + 1;
        // tc.draw_box(&pos,dims, super::colour::GREEN);

        // pos.x = pos.x + 1;
        // tc.draw_text(&pos, "A", super::colour::WHITE);
        // pos.x = pos.x + 1;
        // tc.draw_text(&pos, "B", super::colour::WHITE);
        // pos.x = pos.x + 1;
        // tc.draw_text(&pos, "C", super::colour::WHITE);
    }
}

struct ScrollTriggers {
    top_zone : usize,
    bottom_zone : usize,
}

impl ScrollTriggers {
    pub fn new(_doc_offset : usize, _doc_height : usize, _window_height : usize ) -> Self {
        panic!("TBD")
    }
}

