// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//
use super::colour::*;
use super::events::Events;
use super::styles::TextStyles;
use super::styles::*;
use super::text::Dimensions;
use super::text::*;
use super::v2::*;
use romloader::AnnotatedSourceFile;
use Events::*;

use super::simple::SimState;

struct Cycler {
    cols: Vec<Colour>,
    per_entry: f64,
    t_mul: f64,
}

pub struct SourceCtx<'a> {
    sources: &'a romloader::SourceStore,
    pc: u16,
}

impl Cycler {
    pub fn new(speed: f64, cols: Vec<Colour>) -> Self {
        let total_t = speed * cols.len() as f64;
        let t_mul = 1.0 / total_t;

        Self {
            cols,
            per_entry: speed,
            t_mul,
        }
    }

    fn select(&self, t: f64) -> &Colour {
        &self.cols[(t.abs() as usize) % self.cols.len()]
    }

    pub fn get_col(&self, t: f64) -> Colour {
        let t = t * self.t_mul;
        let c1 = self.select(t);
        let c2 = self.select(t + 1.0);
        c1.blend(&c2, t.fract())
    }
}

trait RenderDoc<'a> {
    fn render_line(&mut self, cursor: usize, win_ypos: usize, doc_ypos: usize);

    fn render_doc(&mut self, num_of_lines: usize, offset: usize, cursor: usize) -> usize {
        let mut lines_rendered = 0;

        let range = (0..num_of_lines).map(|y| (y, y + offset));

        for (win_ypos, doc_ypos) in range {
            self.render_line(cursor, win_ypos, doc_ypos);
            lines_rendered = lines_rendered + 1;
        }
        lines_rendered
    }
}

struct SourceRenderer<'a, IR: TextRenderer> {
    sf: &'a AnnotatedSourceFile,
    pc: u16,
    text_styles: &'a TextStyles,
    blank: String,
    lp: LinePrinter<'a, IR>,
}

impl<'a, TR: TextRenderer> SourceRenderer<'a, TR> {
    pub fn new(
        pc: u16,
        sf: &'a romloader::AnnotatedSourceFile,
        text_styles: &'a TextStyles,
        tc: &'a TR,
    ) -> Self {
        let blank = String::new();
        let lp = LinePrinter::new(tc);
        Self {
            blank,
            sf,
            pc,
            text_styles,
            lp,
        }
    }
}

impl<'a, TR: TextRenderer> RenderDoc<'a> for SourceRenderer<'a, TR> {
    fn render_line(&mut self, cursor: usize, win_ypos: usize, doc_ypos: usize) {
        if let Some(sl) = self.sf.line(doc_ypos) {
            let addr_str = sl
                .addr
                .map(|x| format!("{:04X}", x))
                .unwrap_or(self.blank.clone());

            let source_text = sl.line.as_ref().unwrap_or(&self.blank);

            let is_pc_line = sl.addr.map(|p| p == self.pc).unwrap_or(false);
            let is_cursor_line = win_ypos == cursor;
            let is_debug_line = false;

            let (line_col, addr_col) =
                self.text_styles
                    .get_source_win_style(is_cursor_line, is_pc_line, is_debug_line);

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
    scroll_zone_height: usize,
    cursor: usize,
    scroll_offset: usize,
    styles: StylesDatabase,
    source_file: Option<romloader::AnnotatedSourceFile>,
    frame_time: FrameTime,
    pc: u16,
    win_dims: V2<usize>,
    state: Option<SimState>,
}

impl Default for SourceWin {
    fn default() -> Self {
        Self {
            scroll_zone_height: 6,
            cursor: 0,
            scroll_offset: 0,
            styles: StylesDatabase::default(),
            source_file: None,
            frame_time: FrameTime::from_now(),
            pc: 0,
            win_dims: V2::new(0, 0),
            state: None,
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

    pub fn is_empty(&self) -> bool {
        self.source_file.is_none()
    }

    pub fn has_source_file(&self) -> bool {
        !self.is_empty()
    }

    pub fn event(&mut self, event: Events) -> Option<Events> {
        let mut cursor = self.cursor as isize;
        let mut scroll_offset = self.scroll_offset as isize;

        if let Some(sf) = &self.source_file {
            let st = ScrollTriggers::new(
                self.scroll_offset,
                sf.num_of_lines(),
                self.win_dims,
                self.scroll_zone_height,
            );

            match event {
                CursorUp => cursor -= 1,
                CursorDown => cursor += 1,
                ScrollUp => scroll_offset += 1,
                ScrollDown => scroll_offset -= 1,
                PageUp => scroll_offset += 20,
                PageDown => scroll_offset -= 20,
                _ => (),
            }

            let h = self.win_dims.height() as isize;

            if cursor >= h {
                cursor = h - 1;
            }

            if cursor < 0 {
                cursor = 0;
            }
            if scroll_offset < 0 {
                scroll_offset = 0;
            }

            let mut cursor = cursor as usize;

            if self.cursor != cursor {
                if st.in_bottom_zone(cursor) {
                    scroll_offset = scroll_offset + 1;
                    cursor = self.cursor;
                }

                if st.in_top_zone(cursor) {
                    scroll_offset = scroll_offset - 1;
                    cursor = self.cursor;
                    self.event(ScrollUp);
                }
            }

            self.scroll_offset = scroll_offset as usize;
            self.cursor = cursor;
        }
        None
    }

    pub fn resize(&mut self, dims: V2<usize>) {
        info!("Resizing! rs: {:?} ", dims);
    }

    pub fn update<D: Dimensions<usize>>(
        &mut self,
        dims: &D,
        frame_time: &FrameTime,
        pc: u16,
        state: SimState,
    ) {
        if self.has_source_file() {
            // FIX : dims being passed is wrong
            self.win_dims = dims.dims();
            self.frame_time = *frame_time;
            self.pc = pc;
            self.state = Some(state);
        }
    }

    pub fn set_source_file(&mut self, sf: romloader::AnnotatedSourceFile) {
        self.source_file = Some(sf);
    }

    pub fn clear_source_file(&mut self) {
        self.source_file = None;
    }

    pub fn render<TR: TextRenderer>(&self, tc: &TR) {
        if self.has_source_file() {
            let w = self.win_dims.x;
            let h = self.win_dims.y;

            let text_styles = TextStyles::new(&self.styles);
            let offset = self.scroll_offset;

            if let Some(sf) = &self.source_file {
                let mut renderer = SourceRenderer::new(self.pc, sf, &text_styles, tc);
                renderer.render_doc(h, offset, self.cursor);

                // let cyc = Cycler::new(0.1, vec![*WHITE, *RED, *BLUE, *GREEN]);
                // let flash_col = cyc.get_col(self.frame_time.now_as_seconds());
                // let norm_col = &GREEN;
                // let st = ScrollTriggers::new(offset, sf.num_of_lines(), self.win_dims, self.scroll_zone_height);
                // st.draw(tc, self.cursor, &norm_col, &flash_col);

                let st = format!(
                    "st: {:?}, o: {:04} c: {:04} wh:{},{}",
                    self.state, offset, self.cursor, w, h
                );
                let ccel = super::colourcell::ColourCell::new(RED, YELLOW);
                tc.draw_text_with_bg(&V2::new(w - st.len(), 0).as_isizes(), &st, &ccel);
            }
        }
    }
}
use super::scrbox::ScrBox;

struct ScrollTriggers {
    top_zone: Option<ScrBox>,
    bottom_zone: Option<ScrBox>,
}

impl ScrollTriggers {
    pub fn in_bottom_zone(&self, cursor: usize) -> bool {
        self.intersects(&self.bottom_zone, cursor)
    }

    pub fn in_top_zone(&self, cursor: usize) -> bool {
        self.intersects(&self.top_zone, cursor)
    }

    fn intersects(&self, scr_box: &Option<ScrBox>, cursor: usize) -> bool {
        let c_box = &ScrBox::new(&V2::new(0, cursor as isize), &V2::new(1, 0));
        scr_box.map(|z| z.intersects(c_box)).unwrap_or(false)
    }

    pub fn new<D: Dimensions<usize>>(
        _doc_offset: usize,
        _doc_height: usize,
        dims: D,
        _zone_size: usize,
    ) -> Self {
        let mut top_zone = None;
        let mut bottom_zone = None;
        let szdims = &V2::new(dims.width(), _zone_size);

        if _doc_offset > 0 {
            top_zone = Some(ScrBox::new(&V2::new(0, 0), szdims));
        }

        let lines_to_print = _doc_height - _doc_offset;

        if lines_to_print > dims.height() {
            bottom_zone = Some(ScrBox::new(
                &V2::new(0, (dims.height() - _zone_size) as isize),
                szdims,
            ));
        }

        Self {
            top_zone,
            bottom_zone,
        }
    }

    pub fn draw<TR: TextRenderer>(&self, tr: &TR, cursor: usize, norm: &Colour, flash: &Colour) {
        let ccel = super::colourcell::ColourCell::new(RED, YELLOW);

        let mut norm = *norm;
        let mut flash = *flash;
        norm.set_alpha(0.5);
        flash.set_alpha(0.5);

        let tz = format!("{:?}", self.top_zone);
        let bz = format!("{:?}", self.bottom_zone);
        tr.draw_text_with_bg(&V2::new(0, 0), &tz, &ccel);
        tr.draw_text_with_bg(&V2::new(0, (tr.height() - 1) as isize), &bz, &ccel);

        if let Some(top) = self.top_zone {
            let ccol = if self.in_top_zone(cursor) {
                &flash
            } else {
                &norm
            };

            tr.draw_box(&top.pos, &top.dims, &ccol);
        }

        if let Some(bottom) = self.bottom_zone {
            let ccol = if self.in_bottom_zone(cursor) {
                &flash
            } else {
                &norm
            };
            tr.draw_box(&bottom.pos, &bottom.dims, &ccol);
        }
    }
}
