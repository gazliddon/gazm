// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//

use log::info;

use super::{
    colour::*, colourcell::*, events::Events, scrbox::ScrBox, styles::TextStyles, styles::*,
    text::Dimensions, text::*, v2::*,
};

use Events::*;

use super::simple::{Machine, SimState, SimpleMachine};
use emu::cpu::Regs;
use emu::mem::MemoryIO;
use romloader::{ Location, SourceLine };

trait RenderDoc<'a> {
    fn render_line(&mut self, cursor: usize, win_ypos: usize, doc_ypos: usize);

    fn render_doc(&mut self, num_of_lines: usize, offset: usize, cursor: usize) -> usize {
        let mut lines_rendered = 0;

        let range = (0..num_of_lines).map(|y| (y, y + offset));

        for (win_ypos, doc_ypos) in range {
            self.render_line(cursor, win_ypos, doc_ypos);
            lines_rendered +=  1;
        }
        lines_rendered
    }
}

struct SourceRenderer<'a, IR: TextRenderer> {
    sf: &'a Vec<SourceLine>,
    machine: &'a dyn Machine,
    text_styles: &'a TextStyles,
    blank: String,
    lp: LinePrinter<'a, IR>,
}

impl<'a, TR: TextRenderer> SourceRenderer<'a, TR> {
    pub fn new(
        machine: &'a dyn Machine,
        sf: &'a Vec<SourceLine>,
        text_styles: &'a TextStyles,
        tc: &'a TR,
    ) -> Self {
        let blank = String::new();
        let lp = LinePrinter::new(tc);
        Self {
            blank,
            sf,
            text_styles,
            lp,
            machine,
        }
    }
}

impl<'a, TR: TextRenderer> RenderDoc<'a> for SourceRenderer<'a, TR> {
    fn render_line(&mut self, cursor: usize, win_ypos: usize, doc_ypos: usize) {
        if let Some(sl) = self.sf.get(doc_ypos) {
            let addr_str = sl
                .addr
                .map(|x| format!("{:04X}", x))
                .unwrap_or_else(|| self.blank.clone());

            let source_text = sl.line.as_ref().unwrap_or(&self.blank);
            let pc = self.machine.get_regs().pc;

            let is_pc_line = sl.addr.map(|p| p == pc).unwrap_or(false);
            let is_cursor_line = win_ypos == cursor;
            let is_debug_line = false;

            let (line_col, addr_col) =
                self.text_styles
                    .get_source_win_style(is_cursor_line, is_pc_line, is_debug_line);

            let mut bp_str = " ";

            let break_points = self.machine.get_breakpoints();
            let has_bp = sl
                .addr
                .map(|addr| break_points.has_any_breakpoint(addr))
                .unwrap_or(false);
            if has_bp {
                bp_str = "*";
            }

            self.lp.cols(&addr_col);
            self.lp.print(&format!(" {} {:4}  ", bp_str, addr_str));
            self.lp.cols(&line_col);
            self.lp.print("  ");
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
    source_file: Option<Vec<SourceLine>>,
    frame_time: FrameTime,
    win_dims: V2<usize>,
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
            win_dims: V2::new(0, 0),
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

    pub fn get_cursor_file_loc(&self) -> Option<Location> {
        self.source_file
            .as_ref()
            .and_then(|sf| sf.get(self.cursor + self.scroll_offset))
            .map(|sl| sl.loc.clone())
    }

    pub fn event(&mut self, event: Events) -> Option<Events> {
        let mut cursor = self.cursor as isize;
        let mut scroll_offset = self.scroll_offset as isize;

        if let Some(sf) = &self.source_file {
            let st = ScrollTriggers::new(
                self.scroll_offset,
                sf.len(),
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
                    scroll_offset +=  1;
                    cursor = self.cursor;
                }

                if st.in_top_zone(cursor) {
                    scroll_offset -= 1;
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
        _machine: &dyn Machine,
    ) {
        if self.has_source_file() {
            // FIX : dims being passed is wrong
            self.win_dims = dims.dims();
            self.frame_time = *frame_time;
        }
    }

    pub fn set_source_file(&mut self, sf: Vec<SourceLine>) {
        self.source_file = Some(sf);
    }

    pub fn clear_source_file(&mut self) {
        self.source_file = None;
    }

    pub fn render<TR: TextRenderer>(&self, tc: &TR, machine: &dyn Machine) {
        if self.has_source_file() {
            let _w = self.win_dims.x;
            let h = self.win_dims.y;

            let text_styles = TextStyles::new(&self.styles);
            let offset = self.scroll_offset;

            if let Some(sf) = &self.source_file {
                let mut renderer = SourceRenderer::new(machine, sf, &text_styles, tc);
                renderer.render_doc(h, offset, self.cursor);
            }
        }

        let reg_win = RegWin::new();
        reg_win.render(&V2::new(0, 0), tc, machine.get_regs());
    }
}

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

            tr.draw_box(&top.pos, &top.dims, ccol);
        }

        if let Some(bottom) = self.bottom_zone {
            let ccol = if self.in_bottom_zone(cursor) {
                &flash
            } else {
                &norm
            };
            tr.draw_box(&bottom.pos, &bottom.dims, ccol);
        }
    }
}

struct RegWin {}

pub fn boxer<TR: TextRenderer>(render: &TR) {
    let cel_col = ColourCell::new(YELLOW, &RED.mul_scalar(0.2));
    let h = 3;
    let w = 10;

    let horiz = "─".repeat(w);
    let vert = "│";

    let tr = '┐';
    let br = '┘';
    let tl = '┌';
    let bl = '└';

    let top = format!("{}{}{}", tl, horiz, tr);
    let mid = format!("{}{}{}", vert, " ".repeat(w), vert);
    let bottom = format!("{}{}{}", bl, horiz, br);

    let mut v = LinePrinter::new(render);
    v.cols_alpha(&cel_col, 1.0);
    v.println(&top);
    for _ in 0..h {
        v.println(&mid);
    }
    v.println(&bottom);
}

impl RegWin {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<TR: TextRenderer>(&self, _pos: &V2<isize>, tr: &TR, _regs: &Regs) {
        // let cel_col = ColourCell::new(&YELLOW, &RED.mul_scalar(0.2));
        // let mut v = LinePrinter::new(tr);

        // v.cols(&cel_col);
        // v.println(&regs.get_hdr());
        // v.println(&regs.get_text());

        boxer(tr);
    }
}
