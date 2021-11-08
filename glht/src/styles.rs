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

        let normal = ColourCell::new(WHITE,BLUE).add_scalar_sat(-0.25);
        let addr = normal.add_scalar_sat(0.1);
        let pc = ColourCell::new(YELLOW, RED).add_scalar_sat(-0.15);
        let debug = ColourCell::new(WHITE, PURPLE);


        ret.add("normal", &normal);
        ret.add("pc", &pc);
        ret.add("addr", &addr);

        ret.add("cursor", &normal.add_scalar(0.25));
        ret.add("addr_cursor", &addr.add_scalar_sat(0.2));
        ret.add("addr_pc", &pc.add_scalar_sat(0.2));

        ret.add("debug", &debug);

        ret
    }
}

impl StylesDatabase {

    pub fn add(&mut self, name : &str, cell : &ColourCell) {
        self.styles.insert(name.to_string(), cell.clone());
    }

    pub fn get(&self, name : &str) -> ColourCell {
        if let Some(col_cell) = self.styles.get(name) {
            col_cell.clone()
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
}

impl TextStyles {

    pub fn new(styles : &StylesDatabase) -> Self {

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
