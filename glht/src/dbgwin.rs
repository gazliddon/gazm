use super::simple::simplecore::Machine;
use super::emu::diss::{  Disassembler };
// use romloader::Rom;

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

fn find_previous_instruction(addr : u16, disassembler : &Disassembler) -> Option<u16> {
    let mut ret = None;

    let mut prev_addr = addr;

    loop {
        prev_addr = prev_addr.wrapping_sub(1);
        if prev_addr > addr || !disassembler.mem.is_valid_addr(addr) {
            break;
        }

        let d = disassembler.diss(prev_addr);

        if d.next_instruction_addr == addr {
            ret = Some(prev_addr);
            break;
        }

        if d.next_instruction_addr < addr {
            ret = Some(addr.wrapping_sub(1));
            break;
        }
    }
    ret
}

type Cursor = [isize;2];

struct MessageBar {
    message : Option<String>,
    life : isize,
    cursor : [isize;2],
}

impl MessageBar {
}

struct TextWinDims {
    pixel_dims : [f32;2 ],
    char_dims : [usize;2],
    line_height: f32,

}
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const YELLOW: [f32; 3] = [1.0, 1.0, 0.0];
const RED: [f32; 3] = [1.0, 0.0, 0.0];

struct DisassemblerIterator<'a> {
    addr : u16,
    disassmbler : Disassembler<'a>,
}

impl<'a> DisassemblerIterator<'a> {
    pub fn from_machine(machine : &'a dyn Machine, addr : u16) -> Self {
        Self {
            addr,
            disassmbler : machine.get_dissambler()
        }
    }
}

use emu::diss::Dissembly;

impl<'a> Iterator for DisassemblerIterator<'a> {
    type Item = Dissembly;
    fn next(&mut self) -> Option<Dissembly> {
        if self.disassmbler.mem.is_valid_addr(self.addr) {
            let ret = self.disassmbler.diss(self.addr);
            self.addr = ret.next_instruction_addr;
            Some(ret)
        } else {
            None
        }
    }
}


pub fn render_source(ui: &imgui::Ui, machine: &dyn Machine, addr : u16, lines : usize, pos : [f32;2], line_height : f32, cursor : isize) {

    let draw_list = ui.get_window_draw_list();

    let mut addr =  addr;
    let mut pos = pos;

    let diss = machine.get_dissambler();

    let dissasemble = |addr : u16| {
        let rom = machine.get_rom();
        let d = diss.diss(addr);
        let src = rom.get_source_line(addr).unwrap_or_else(|| "".to_string());
        let text = format!("{:04x}    {:<20} {}", addr, d.text, src);
        (d.next_instruction_addr,text)
    };

    for _i in 0..lines {
        let (next_ins, text ) = dissasemble(addr);

        let col = if _i as isize == cursor {
            let br = [pos[0], pos[1] + line_height ];
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

        addr = next_ins;
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

    fn get_window_dimensions(&self, ui: &imgui::Ui) ->TextWinDims {

        let [_,line_height] = ui.calc_text_size(im_str!( " " ), false, std::f32::MAX);
        let [ww,wh] = ui.content_region_avail();
        let lines = (wh / line_height ) - 2.0;

        let lines : usize = if lines < 0.0 {
            0 as usize
        } else {
            lines as usize
        };

        TextWinDims {
            pixel_dims: [ww,wh],
            char_dims: [0,lines],
            line_height
        }
    }

    fn iter<'a>(&self, machine : &'a dyn Machine) -> DisassemblerIterator<'a> {
        DisassemblerIterator::from_machine(machine, self.addr)
    }


    pub fn render(&mut self, ui: &imgui::Ui, machine : &dyn Machine) {

        let window_dims = self.get_window_dimensions(&ui);
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

    pub fn next_instruction(&mut self, machine : &dyn Machine) {
        let diss = machine.get_dissambler();
        let d = diss.diss(self.addr);
        self.addr = d.next_instruction_addr;
    }

    pub fn prev_instruction(&mut self, machine : &dyn Machine) {
        let diss = machine.get_dissambler();
        if let Some(addr) = find_previous_instruction(self.addr, &diss) {
            self.addr = addr
        }
    }
}


