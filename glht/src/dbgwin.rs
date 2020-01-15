use super::simple::simplecore::Machine;
use super::emu::diss::{  Disassembler };
// use romloader::Rom;

use crate::window::{TextWinDims};


////////////////////////////////////////////////////////////////////////////////
struct EdgeDistances {
    to_top: isize,
    to_top_scrollzone: isize,
    to_bottom: isize,
    to_bottom_scrollzone: isize,
    scroll_zone : isize,
    desired_scroll_zone: usize,
    cursor: isize,
    lines : usize,
}

pub enum ScrollAction {
    ScrollTo { distance : isize, cursor : isize },
    DontMove,
}

impl EdgeDistances {
    pub fn new(cursor : isize, lines: usize, desired_scroll_zone : usize) -> Self {
        let lines = lines as isize;
        let mut scroll_zone = desired_scroll_zone as isize;

        // do we have space for a scroll zone?

        if lines - (scroll_zone * 2) < 1 {
            scroll_zone = 0;
        }

        let to_bottom = (lines -1) - cursor;

        EdgeDistances {
            to_top: cursor,
            to_top_scrollzone: cursor - scroll_zone,
            to_bottom,
            to_bottom_scrollzone: to_bottom - 3,
            scroll_zone,
            desired_scroll_zone,
            cursor,
            lines : lines as usize,
        }
    }

    pub fn what_should_do(&self) -> ScrollAction {
        if self.to_top > 0 {
            return ScrollAction::ScrollTo{distance : self.to_top, cursor : 0};
        }

        if self.to_bottom < 0 {
            return ScrollAction::ScrollTo{distance : self.to_bottom, cursor : self.lines as isize - 1};
        }

        if self.to_top_scrollzone > 0 {
            return ScrollAction::ScrollTo{distance : self.to_top_scrollzone, cursor : self.to_top_scrollzone};
        }

        if self.to_bottom_scrollzone < 0 {
            return ScrollAction::ScrollTo{distance : self.to_bottom_scrollzone, cursor : ( self.lines as isize - self.scroll_zone ) - 1};
        }

        ScrollAction::DontMove
    }
} 

////////////////////////////////////////////////////////////////////////////////

pub struct DbgWin {
    addr : u16,
    cursor : isize,
    dims : Option<[usize;2]>
}

pub enum Events {
    CursorUp,
    CursorDown,
    Space,
    PageUp,
    PageDown,
}

type Cursor = [isize;2];

struct MessageBar {
    message : Option<String>,
    life : isize,
    cursor : [isize;2],
}

impl MessageBar {
}

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const YELLOW: [f32; 3] = [1.0, 1.0, 0.0];
const RED: [f32; 3] = [1.0, 0.0, 0.0];

struct DisassemblerIterator<'a> {
    addr : Option<u16>,
    disassmbler : Disassembler<'a>,
}

impl<'a> DisassemblerIterator<'a> {
    pub fn from_machine(machine : &'a dyn Machine, addr : u16) -> Self {
        let disassmbler  = machine.get_dissambler();

        let mut this_addr = None;

        if disassmbler.mem.is_valid_addr(addr) {
            this_addr = Some(addr)
        }

        Self {
            addr : this_addr,
            disassmbler,
        }
    }
}

use emu::diss::Dissembly;

impl<'a> Iterator for DisassemblerIterator<'a> {
    type Item = Dissembly;
    fn next(&mut self) -> Option<Dissembly> {
        // Is their a current address?

        if let Some(addr) = self.addr {
            // is it for a valid  address?
            if self.disassmbler.mem.is_valid_addr(addr) {
                // yes!
                let ret = self.disassmbler.diss(addr);
                self.addr = ret.next_instruction_addr;
                return Some(ret);
            }
        }
        None
    }
}


impl DbgWin {

    pub fn new(addr : u16) -> Self {
        Self {
            addr,
            cursor : 0,
            dims : None,
        }
    }


    fn iter<'a>(&self, machine : &'a dyn Machine) -> DisassemblerIterator<'a> {
        DisassemblerIterator::from_machine(machine, self.addr)
    }


    pub fn render(&mut self, ui: &imgui::Ui, machine : &dyn Machine) {

        let window_dims = TextWinDims::new(ui);
        let lines = window_dims.char_dims[1];
        let line_height = window_dims.line_height;

        self.dims = Some(window_dims.char_dims);

        // sort out cursor position

        if self.cursor < 0 {
            self.cursor = 0;
        }

        if self.cursor >= lines as isize {
            self.cursor = lines as isize - 1 ;
            self.next_instruction(machine);
        }

        // sort out cursor position

        let mut pos = ui.cursor_screen_pos();

        let draw_list = ui.get_window_draw_list();

        draw_list.add_text(
            pos.clone(),
            YELLOW,
            format!("wh : {:?} lines: {} line: {}", window_dims.pixel_dims, lines, self.cursor));

        pos[1] += line_height;

        draw_list.add_text(
            pos.clone(),
            YELLOW,
            "ADDR    CODE");

        pos[1] += line_height;

        for (i, diss) in self.iter(machine).enumerate() {

            if i == lines {
                break;
            }

            let addr = diss.addr;

            let rom = machine.get_rom();
            let src = rom.get_source_line(addr).unwrap_or_else(|| "".to_string());
            let text = format!("{:04x}    {:<20} {}", addr, diss.text, src);

            let col = if i as isize == self.cursor {
                let br = [pos[0] + window_dims.pixel_dims[0], pos[1] + line_height ];
                draw_list.add_rect_filled_multicolor(pos,br, RED, RED, RED, RED );
                YELLOW
            } else {
                WHITE
            };

            draw_list.add_text(
                pos.clone(),
                col,
                &text);

            pos[1] += line_height;
        }

    }

    fn scroll_up(&mut self) {
    }

    fn scroll_down(&mut self) {
    }

    fn cursor_up(&mut self) {
        self.cursor -= 1;
    }

    fn cursor_down(&mut self) {
        self.cursor += 1;
    }

    pub fn event(&mut self, event : Events) {
        match event {
            Events::CursorUp => self.cursor_up(),
            Events::CursorDown => self.cursor_down(),
            _ => ()
        }
    }

    pub fn next_instruction(&mut self, _machine : &dyn Machine) {
        // let diss = machine.get_dissambler();
        // let d = diss.diss(self.addr);
        panic!()
    }

    pub fn prev_instruction(&mut self, _machine : &dyn Machine) {
        panic!()
    }
}


