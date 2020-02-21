use crate::colour::*;
use super::textscreen::{ ColourCell, };

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

        let normal = ColourCell::new(WHITE.clone(),BLUE.clone()).add_scalar_sat(-0.25);
        let addr = normal.add_scalar_sat(0.1);
        let pc = ColourCell::new(YELLOW.clone(), RED.clone()).add_scalar_sat(-0.15);
        let debug = ColourCell::new(WHITE.clone(), PURPLE.clone());


        ret.add("normal", &normal);
        ret.add("pc", &pc);
        ret.add("addr", &addr);

        ret.add("cursor", &normal.add_scalar(0.25));
        ret.add("addr_cursor", &addr.add_scalar_sat(0.2));
        ret.add("addr_pc", &pc.add_scalar_sat(0.2));

        ret.add("deubg", &debug);

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
            ColourCell::new(BLACK.clone(), WHITE.clone())
        }
    }
}
