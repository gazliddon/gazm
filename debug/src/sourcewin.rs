// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//

use std::error::Error;

use log::info;
use utils::sources::{ItemType, LocationTrait, SourceFileAccess};

use super::{
    colour::*, colourcell::*, events::Events, scrbox::ScrBox, styles::TextStyles, styles::*,
    text::Dimensions, text::Extents, text::*, v2::*,
};

use Events::*;

use super::simple::{Machine, SimState, SimpleMachine};
use emu::cpu::Regs;
use emu::mem::MemoryIO;
use utils::Location;

trait RenderDoc<'a> {
    fn render_line(&mut self, is_cursor_line: bool, win_ypos: usize, doc_ypos: usize);

    fn num_of_lines(&self) -> usize;

    fn render_doc(&mut self, offset: usize, cursor: usize) -> usize {
        let mut lines_rendered = 0;
        let num_of_lines = self.num_of_lines();

        let range = (0..num_of_lines).map(|y| (y, y + offset));

        for (win_ypos, doc_ypos) in range {
            self.render_line(cursor == win_ypos, win_ypos, doc_ypos);
            lines_rendered += 1;
        }
        lines_rendered
    }
}

struct SourceRenderer<'a, IR: TextRenderer> {
    // sf: &'a Vec<SourceLine>,
    machine: &'a dyn Machine,
    text_styles: &'a TextStyles,
    blank: String,
    lp: LinePrinter<'a, IR>,
    sources: &'a SourceFileAccess<'a>,
}

impl<'a, TR: TextRenderer> SourceRenderer<'a, TR> {
    pub fn new(
        machine: &'a dyn Machine,
        // sf: &'a Vec<SourceLine>,
        text_styles: &'a TextStyles,
        tc: &'a TextContext<'a, TR>,
        sources: &'a SourceFileAccess<'a>,
    ) -> Self {
        let blank = String::new();
        let lp = LinePrinter::new(tc);
        Self {
            blank,
            // sf,
            text_styles,
            lp,
            machine,
            sources,
        }
    }
}

impl<'a, TR: TextRenderer> RenderDoc<'a> for SourceRenderer<'a, TR> {
    fn num_of_lines(&self) -> usize {
        self.sources.num_of_lines()
    }

    fn render_line(&mut self, is_cursor_line: bool, _win_ypos: usize, doc_ypos: usize) {
        let source_line_number = doc_ypos + 1;

        if let Some(sl) = self.sources.get_line(source_line_number) {
            // let mem_range = sl.mapping.map(|m| &m.mem_range);

            let addr_str = sl
                .mapping
                .map(|m| {
                    let mut mem_str = self.machine.get_mem().get_mem_as_str(&m.mem_range, "");

                    if mem_str.len() > 8 {
                        mem_str = format!("{:.8}..", mem_str)
                    }
                    format!("{:04X} {:10}", m.mem_range.start, mem_str)
                })
                .unwrap_or_else(|| self.blank.clone());

            let info_str = sl
                .mapping
                .and_then(|m| {
                    if m.item_type == ItemType::OpCode {
                        let ins = self.machine.inspect_instruction(m.mem_range.start).unwrap();
                        Some(format!("{} {}", ins.size, ins.cycles))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| self.blank.clone());

            let source_text = &sl.text;
            let pc = self.machine.get_regs().pc as usize;
            let is_pc_line = sl.mapping.map(|p| p.mem_range.start == pc).unwrap_or(false);
            let (line_col, addr_col) =
                self.text_styles
                    .get_source_win_style(is_cursor_line, is_pc_line, false);

            let mut bp_str = format!(" {:>3} ", source_line_number);

            if is_cursor_line {
                bp_str = format!(">{:>3} ", source_line_number);
            }

            let mut cyc_col = addr_col.clone();
            cyc_col.fg = cyc_col.fg.mul_scalar(0.75);

            self.lp.cols(&addr_col);
            self.lp.print(&bp_str);

            self.lp.cols(&addr_col);
            self.lp.print(&format!("{:20}", addr_str,));
            self.lp.cols(&cyc_col);
            self.lp.print(&format!("{:6}", info_str));

            self.lp.cols(&line_col);
            self.lp.print("  ");
            self.lp.print(source_text);
        }

        self.lp.cr();
    }
}

#[derive(Clone)]
pub struct LoadedSource {
    pub file_id: u64,
    pub num_of_lines: usize,
}

pub struct SourceWin {
    scroll_zone_height: usize,
    cursor: usize,
    scroll_offset: usize,
    styles: StylesDatabase,
    frame_time: FrameTime,
    win_dims: V2<usize>,
    current_source_file: Option<LoadedSource>,
    has_stepped: bool,
}

impl Default for SourceWin {
    fn default() -> Self {
        Self {
            scroll_zone_height: 6,
            cursor: 0,
            scroll_offset: 0,
            styles: StylesDatabase::default(),
            frame_time: FrameTime::from_now(),
            win_dims: V2::new(0, 0),
            current_source_file: None,
            has_stepped: false,
        }
    }
}

use super::app::frametime::FrameTime;

impl SourceWin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event(&mut self, machine: &dyn Machine, event: Events) -> Option<Events> {
        let mut cursor = self.cursor as isize;
        let mut scroll_offset = self.scroll_offset as isize;

        if let Some(source) = &self.current_source_file {
            let st = ScrollTriggers::new(
                self.scroll_offset,
                source.num_of_lines,
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
                Step => self.has_stepped = true,
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
                    scroll_offset += 1;
                    cursor = self.cursor;
                }

                if st.in_top_zone(cursor) {
                    scroll_offset -= 1;
                    cursor = self.cursor;
                    self.event(machine, ScrollUp);
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

    /// Return access to the current source file if we have one
    fn get_source_file_access<'a>(&self, machine: &'a dyn Machine) -> Option<SourceFileAccess<'a>> {
        let sfa = self
            .current_source_file
            .as_ref()
            .and_then(|sf| machine.get_sources().get_source_file(sf.file_id));
        sfa
    }

    fn is_pc_on_screen<D: Dimensions<usize>>(&mut self, dims: &D, machine: &dyn Machine) -> bool {
        let pc = machine.get_regs().pc as usize;

        if let Some(source_file) = self.get_source_file_access(machine) {
            for y in self.scroll_offset..self.scroll_offset + dims.height() {
                if let Some(mapping) = source_file.get_line(y + 1).and_then(|sl| sl.mapping) {
                    if mapping.mem_range.contains(&pc) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn update<D: Dimensions<usize>>(
        &mut self,
        dims: &D,
        frame_time: &FrameTime,
        machine: &dyn Machine,
    ) {
        let pc = machine.get_regs().pc as usize;

        // if we have a current_source_file
        // is the PC on the page?

        if !self.is_pc_on_screen(dims, machine) && self.has_stepped {
            self.has_stepped = false;
            self.current_source_file = None;
        }

        if !self.has_source_file() {
            let sources = machine.get_sources();

            if let Some((source_file, source_line)) = sources
                .get_source_info_from_address(pc)
                .and_then(|source_line| {
                    sources
                        .get_source_file(source_line.file_id)
                        .map(|sf| (sf, source_line))
                })
            {
                let offset = source_line.line_number as isize - (dims.height() / 2) as isize;
                self.scroll_offset = std::cmp::max(0, offset) as usize;

                self.cursor = (source_line.line_number - 1) - self.scroll_offset;

                self.current_source_file = Some(LoadedSource {
                    num_of_lines: source_file.num_of_lines(),
                    file_id: source_file.file_id,
                })
            }
        }

        self.win_dims = dims.dims();
        self.frame_time = *frame_time;
    }

    fn has_source_file(&self) -> bool {
        self.current_source_file.is_some()
    }

    fn set_source_file(&mut self, file_id: u64, num_of_lines: usize) {
        self.current_source_file = Some(LoadedSource {
            file_id,
            num_of_lines,
        })
    }

    pub fn render<TR: TextRenderer>(&self, renderer: &TR, machine: &dyn Machine) {
        let mut sc_box = renderer.get_scr_box();

        sc_box.pos.y += 4;
        sc_box.dims.y -= 4;

        let tc = TextContext::new_with_dims(renderer, &sc_box);

        if let Some(sf) = self
            .current_source_file
            .clone()
            .and_then(|sf| machine.get_sources().get_source_file(sf.file_id))
        {
            let text_styles = TextStyles::new(&self.styles);
            let offset = self.scroll_offset;
            let mut sr = SourceRenderer::new(machine, &text_styles, &tc, &sf);
            sr.render_doc(offset, self.cursor);
        }

        let sc_box = ScrBox::new(&V2::new(0, 0), &V2::new(sc_box.dims.x, 5));

        let tc = TextContext::new(renderer);
        let new_tc = tc.child_context(&sc_box);
        let reg_win = RegWin::new();
        reg_win.render(&new_tc, machine.get_regs());
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
}

struct RegWin {}

pub fn boxer<'a, TR: TextRenderer>(render: &'a TextContext<TR>) -> TextContext<'a, TR> {
    let cel_col = ColourCell::new(&WHITE, BLUE);
    let h = render.height() as usize;
    let w = render.width() as usize;

    let horiz = "─".repeat(w - 2);
    let vert = "│";

    let tr = '┐';
    let br = '┘';
    let tl = '┌';
    let bl = '└';

    let top = format!("{}{}{}", tl, horiz, tr);
    let mid = format!("{}{}{}", vert, " ".repeat(w - 2), vert);
    let bottom = format!("{}{}{}", bl, horiz, br);

    let mut v = LinePrinter::new(render);
    v.cols_alpha(&cel_col, 1.0);
    v.println(&top);
    for _ in 0..h - 2 {
        v.println(&mid);
    }
    v.println(&bottom);

    let dims = render.dims() - V2::new(4, 2);
    let pos = render.pos() + V2::new(2, 1);
    let dims = V2::new(dims.x as usize, dims.y as usize);
    let sc_box = ScrBox::new(&pos, &dims);
    render.child_context(&sc_box)
}

impl RegWin {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<TR: TextRenderer>(&self, tr: &TextContext<TR>, regs: &Regs) {
        let hdr = &regs.get_hdr();
        let txt = &regs.get_text();
        let sc_box = ScrBox::new(&tr.pos(), &V2::new(tr.width() as usize, 4));
        let tr = tr.child_context(&sc_box);
        let tr = boxer(&tr);

        let cel_col = ColourCell::new(&YELLOW, &RED.mul_scalar(0.2));
        let mut v = LinePrinter::new(&tr);

        v.cols(&cel_col);
        v.println(hdr);
        v.println(txt);
    }
}
