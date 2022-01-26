use crate::colour::*;
use super::colourcell::ColourCell;

use std::collections::HashMap;

pub struct StylesDatabase {
    styles : HashMap<String,ColourCell>,
}

impl std::default::Default for StylesDatabase {
    fn default() -> Self {
        let styles = HashMap::new();

        let mut ret  = Self {
            styles
        };

        let fg = &WHITE.add_scalar_sat(-0.15);
        let bg = &BLUE.add_scalar_sat(-0.3);
        let red = &RED.mul_scalar(0.5);

        let normal = ColourCell::new(fg,bg);
        let pc = ColourCell::new(fg, red);
        let debug = ColourCell::new(WHITE, PURPLE);

        let mut cursor = normal.add_scalar_sat(-0.1);
        cursor.fg = normal.fg;
        let addr = normal.add_scalar_sat(0.05);

        let cursor_pc = cursor.add_scalar_sat(0.25);

        let addr_pc = pc.blend(&addr, 0.1);
        let addr_cursor = addr.add_scalar_sat(0.2);

        let bp = ColourCell::new(WHITE, RED).mul_scalar(0.7);
        let bp_pc = bp;
        let bp_cursor = bp.blend(&cursor, 0.25);

        ret.add("bp",&bp  );
        ret.add("bp_pc",&bp_pc  );
        ret.add("bp_cursor",&bp_cursor  );

        ret.add("normal", &normal);
        ret.add("pc", &pc);

        ret.add("cursor", &cursor);
        ret.add("cursor_pc", &cursor_pc);

        ret.add("addr", &addr);
        ret.add("addr_pc", &addr_pc);
        ret.add("addr_cursor", &addr_cursor);
        ret.add("debug", &debug);

        ret
    }
}

impl StylesDatabase {

    pub fn add(&mut self, name : &str, cell : &ColourCell) {
        let mut cell = *cell;
        cell.set_alpha(( 1.0, 1.0 ));
        self.styles.insert(name.to_string(), cell);
    }

    pub fn get(&self, name : &str) -> ColourCell {
        if let Some(col_cell) = self.styles.get(name) {
            *col_cell
        } else {
            ColourCell::new(BLACK, WHITE)
        }
    }
}


pub struct TextStyles {
    pub normal : ColourCell,
    pub pc : ColourCell,
    pub cursor : ColourCell,
    pub cursor_addr : ColourCell,
    pub addr : ColourCell,
    pub debug: ColourCell,
    pub addr_pc: ColourCell,
            hm : HashMap<(bool, bool, bool),(ColourCell, ColourCell)>
}

impl TextStyles {

    pub fn new(styles : &StylesDatabase) -> Self {

        let normal = styles.get("normal");
        let pc = styles.get("pc");
        let addr_pc = styles.get("addr_pc");
        let cursor = styles.get("cursor");
        let cursor_addr = styles.get("addr_cursor");
        let addr = styles.get("addr");
        let debug = styles.get("debug");
        let bp = styles.get("bp");
        let bp_pc = styles.get("bp_pc");
        let bp_cursor = styles.get("bp_cursor");

            let mut hm = HashMap::<(bool, bool, bool),(ColourCell, ColourCell)>::new();

            // no cursor, pc
            hm.insert((false,false, true),(pc, addr_pc));
            // cursor, no pc
            hm.insert((false,true, false),(cursor, cursor_addr));
            // cursor,  pc
            hm.insert((false,true, true),(cursor, addr_pc));


            hm.insert((true,false, false),(bp, addr));
            // no cursor, pc
            hm.insert((true,false, true),(bp_pc, addr_pc));
            // cursor, no pc
            hm.insert((true,true, false),(bp_cursor, cursor_addr));
            // cursor,  pc
            hm.insert((true,true, true),(bp_cursor, addr_pc));

        Self {
            normal ,
            pc,
            cursor,
            cursor_addr,
            addr,
            debug,
            addr_pc,
            hm,
        }
    }

    pub fn get_source_win_style(&self,  is_cursor_line : bool , is_pc_line : bool, is_debug_line : bool ) -> (ColourCell, ColourCell) {
            let defaults = &(self.normal, self.addr);
            *self.hm.get(&(is_debug_line, is_cursor_line, is_pc_line)).unwrap_or(defaults)
    }
}
