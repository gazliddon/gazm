
// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//

use romloader::{ SourceStore};

use super::window::*;

use super::styles;

use super::textscreen::{TextScreen, ColourCell, Cell, CursorTrait};

use vector2d::{ Vector2D  as V2};

use super::events::Events;
use Events::*;

use super::styles::*;

pub struct SourceWin {
    cursor : usize,
    scroll_offset : usize,
    text_screen: TextScreen,
    styles : StylesDatabase,
    source_file : Option<String>,
}



impl TextScreen {
    pub fn render(&self, ui: &imgui::Ui ) {

        let wind_dims = TextWinDims::new(ui);

        let char_box_dims = V2{x:1, y:1};
        let V2{x : cols, y : rows} = wind_dims.get_window_char_dims();

        let draw_list = ui.get_window_draw_list();

        for y in 0..rows {
            for x in 0..cols {
                if let Some(Cell {col, pos, text}) = self.get_cell(V2{x,y}.as_isizes()) {

                    let ColourCell{bg,fg} = &col;

                    let [tl, br] = wind_dims.get_box_dims(
                        pos.as_usizes(),
                        char_box_dims);

                    let tl = [tl.x, tl.y];
                    let br = [br.x, br.y];

                    draw_list.add_rect_filled_multicolor(tl, br, bg, bg, bg, bg );
                    draw_list.add_text(tl,fg,text);
                }
            }

        }
    }
}

impl Default for SourceWin {
    fn default() -> Self {

        let text_screen = TextScreen::new(
            V2 { x: 30, y :30 });

        Self {
            cursor : 0,
            scroll_offset : 0,
            text_screen ,
            styles : StylesDatabase::default(),
            source_file: None,

        }
    }
}


struct TextStyles {
    pub normal : ColourCell,
    pub pc : ColourCell,
    pub cursor : ColourCell,
    pub cursor_addr : ColourCell,
    pub addr : ColourCell,
    pub debug: ColourCell,
}

impl TextStyles {

    pub fn new(styles : &styles::StylesDatabase) -> Self {

        let normal = styles.get("normal");
        let pc = styles.get("pc");
        let cursor = styles.get("cursor");
        let cursor_addr = styles.get("addr_cursor");
        let addr = styles.get("addr");
        let debug = styles.get("debug");

        Self {
            normal ,
            pc,
            cursor,
            cursor_addr,
            addr,
            debug,
        }
    }

    pub fn get_source_win_style(&self,  is_cursor_line : bool , is_pc_line : bool, is_debug_line : bool ) -> (&ColourCell, &ColourCell) {

        if is_debug_line {
            (&self.debug, &self.debug)
        } else {

            let mut line_style;

            let addr_style = if is_cursor_line {
                line_style = &self.cursor;
                &self.cursor_addr
            } else {
                line_style = &self.normal;
                &self.addr
            };

            if is_pc_line {
                line_style = &self.pc;
            }

            (line_style, addr_style)
        }

    }
}

struct ScrollZones {
    top : V2<usize>,
    bottom : V2<usize>
}

impl ScrollZones {
    pub fn new(win_dims : &TextWinDims, top_line : usize, _lines_in_doc : usize, sz : usize) -> Self {
        let win_char_height = win_dims.get_window_char_dims().y;

        let top = V2::new(top_line, top_line + sz);
        let bottom = V2::new(( top_line + win_char_height ) - sz, sz);
        Self {
            top,
            bottom
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


impl SourceWin {

    pub fn new() -> Self {
        Self::default()
    }


    pub fn event(&mut self, event : Events) {
        match event {
            CursorUp => {
                if self.cursor > 0 { self.cursor-=1 }
            }

            CursorDown => {
                self.cursor+=1
            }

            ScrollUp => {
                self.scroll_offset += 1
            },

            ScrollDown => {
                if self.scroll_offset >= 1 {
                    self.scroll_offset -= 1
                }
            },

            _ => ()
        }
    }

    pub fn resize(&mut self,  dims : V2<usize>) {
        info!("Resizing! rs: {:?} ",dims );
    }

    pub fn update(&mut self) {
    }

    fn get_scroll_zones(&self, win_dims : &TextWinDims, lines_in_doc : usize) -> ScrollZones {
        ScrollZones::new(win_dims, self.cursor,lines_in_doc, 3 )
    }

    pub fn render(&mut self, ui: &imgui::Ui, source_store : &SourceStore, pc : u16) {
        use romloader::Location;

        let window_info = TextWinDims::new(ui);

        if window_info.is_visible() {

            if self.source_file.is_none() {
                self.source_file = source_store.add_to_loc(pc).map(|l| l.file.clone());
            }

            if let Some(file) = &self.source_file {

                let text_styles = TextStyles::new(&self.styles);

                let window_dims = window_info.get_window_char_dims();

                if let Some(sf) = source_store.get(&file) {

                    let scroll_zones = self.get_scroll_zones(&window_info, sf.num_of_lines());

                    let mut screen = TextScreen::new(window_dims);

                    screen.clear(' ',&text_styles.normal);

                    let mut c = screen.cursor();

                    c.set_col(&text_styles.normal);

                    let blank = String::new();
                    let mut loc = Location::new(&file, self.scroll_offset );

                    for line in self.scroll_offset..(self.scroll_offset + window_dims.y) {

                        if let Some(source_line) = source_store.loc_to_source_line(&loc) {

                            let is_cursor_line  = self.cursor as usize == line;
                            let is_pc_line = Some(pc) == source_line.addr;
                            let is_debug_line = scroll_zones.in_scroll_zone(line);

                            let (line_style, addr_style) = text_styles.get_source_win_style(is_cursor_line, is_pc_line, is_debug_line);

                            let addr_str = source_line.addr
                                .map(|addr|
                                    format!("{:04x}", addr))
                                .unwrap_or_else(||blank.clone());

                            let addr_str = format!("{:^8}", addr_str);

                            let line_str = source_line.line.clone().unwrap_or_else(|| blank.clone());

                            c.set_col(line_style).clear_line();
                            c.set_col(addr_style).write(&addr_str);
                            c.set_col(line_style).write(" ").write(&line_str);

                        } else {
                            c.write(&format!( "{} {} *LINE NOT FOUND*",&file, line  ));
                            break;
                        }

                        loc.inc_line_number();
                        c.cr();
                    }

                    screen.render(ui);
                }
            }
        }
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

