
// A source window shows
// source for an address
//
// so it would need
// A Source Store
// A PC
//

use romloader::{ SourceFile, Rom, Location };
use crate::window::*;

use crate::colour::{Colour};



pub struct SourceWin {
    addr : Option<u16>,
    line : usize,
    current_file : Option<String>,
}

fn render_file( ui : &imgui::Ui, source : &SourceFile, rom : &Rom, loc : Location ) {
    let wind_dims = TextWinDims::new(ui);

    let lines = wind_dims.char_dims[1];
    let line_height = wind_dims.line_height;
    let draw_list = ui.get_window_draw_list();

    let mut loc = loc;

    let mut addr_pos = ui.cursor_screen_pos();
    let mut src_pos = ui.cursor_screen_pos();

    src_pos[0] += line_height * 10.0;

    for (i, line) in source.lines.iter().enumerate() {
        if i == lines {
            break;
        }

        let addr_str;

        if let Some(r) = rom.get_location_addr_range(&loc) {
            addr_str = format!("{:04x}", r.start);
        } else {
            addr_str = "".to_string();
        };

        draw_list.add_text( addr_pos.clone(), Colour::make_yellow(), addr_str);
        draw_list.add_text( src_pos.clone(), Colour::make_white(), line);

        addr_pos[1] += line_height;
        src_pos[1] += line_height;
        loc.line_number+=1;
    }
}

impl SourceWin {
    pub fn new() -> Self {
        Self {
            addr : None,
            line : 0,
            current_file: Some( "demo/all.68".to_string() )
        }
    }

    pub fn render(&mut self, _ui: &imgui::Ui, rom : &Rom ) {
        if let Some(ref file) = self.current_file {
            rom.sources.get(&file, |source| {
                let loc = Location::new(file,1);
                render_file(_ui, source, rom, loc)
            });
        }
    }
}

