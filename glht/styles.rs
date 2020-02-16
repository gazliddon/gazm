use super::textscreen::{ColourCell, Cell, CursorTrait};
use crate::colour::*;

use std::collections::HashMap;

struct StylesDatabase {
    styles : HashMap<String,ColourCell>,
}

impl std::default::Default for StylesDatabase {
    fn default() -> Self {
        let styles = HashMap::new();

        let mut ret  = Self {
            styles
        };

        ret.add("normal", &WHITE, &BLUE);
        ret.add("pc", &YELLOW, &RED);
        ret.add("cursor", &BLACK, &PURPLE);
        ret
    }
}

impl StylesDatabase {
    pub fn add(&mut self, name : &str, fg : &Colour, bg : &Colour) {
        self.styles.insert(name.to_string(), ColourCell::new(fg.clone(), bg.clone()));
    }

    pub fn get(&self, name : &str) -> ColourCell {
        if let Some(col_cell) = self.styles.get(name) {
            col_cell.clone()
        } else {
            ColourCell::new(BLACK.clone(), WHITE.clone())
        }
    }
}
