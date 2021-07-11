
// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//
use super::imgui;
use romloader::SourceStore;
use super::docwin;

use super::window::*;

use super::styles::TextStyles;

use super::textscreen::{TextScreen, ColourCell, Cell, CursorTrait};

use vector2d::Vector2D  as V2;

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

pub enum Zone {
    TOP,
    MIDDLE,
    BOTTOM,
}

pub fn text_render(ui : &imgui::Ui, view : &dyn docwin::Doc) {
    let wind_dims = TextWinDims::new(ui);

    let char_box_dims = V2{x:1, y:1};
    let V2{x : _, y : rows} = wind_dims.get_window_char_dims();
    let draw_list = ui.get_window_draw_list();

    for y in 0..rows {
        if let Some(row) = view.get_line(y) {
            for Cell {col, pos, text} in row.iter() {
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


impl SourceWin {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn dims(&self) -> V2<isize> {
        self.text_screen.dims
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
        use romloader::Location;

        let window_info = TextWinDims::new(ui);

        if !window_info.is_visible() {
            let cd = window_info.get_char_dims();
            let pd = window_info.get_pixel_dims();

            println!("char dims {} {}", cd.x, cd.y);
            println!("pixel dims {} {}", pd.x, pd.y);
            panic!("Ficled");

            // return
        }

        if self.source_file.is_none() {
            self.source_file = source_store.add_to_loc(pc).map(|l| l.file.clone());
        }

        if let Some(sf) = self.get_source_file(source_store) {
            let file = self.source_file.clone().unwrap_or(String::from("No file"));

            let text_styles = TextStyles::new(&self.styles);
            let window_dims = window_info.get_window_char_dims();

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
                    let mut is_debug_line = scroll_zones.in_scroll_zone(line);

                    if is_cursor_line && is_debug_line {
                        is_debug_line = false;
                    }

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
                    c.write(&format!( "{} {} *LINE NOT FOUND*",file, line  ));
                    break;
                }

                loc.inc_line_number();
                c.cr();
            }

            screen.render(ui);
        } else {
            panic!("can't fine file!")
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

