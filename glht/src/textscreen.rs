use crate::colour::*;
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ColourCell {
    fg : Colour,
    bg : Colour,
}

impl Default for ColourCell {
    fn default()  -> Self {
        Self {
            fg: Colour::make_white(),
            bg: Colour::make_black(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Cursor<'a> {
    screen : &'a mut TextScreen,
    pos : [isize; 2],
    dims : [isize; 2],
    current_color : ColourCell,
}

impl<'a> Cursor<'a> {

    pub fn write(&mut self, text : &str) {
        for c in text.chars() {
            let [x,y] = self.pos;
            if self.screen.in_bounds(&self.pos) {
                self.screen.text[y as usize][x as usize] = c;
                self.screen.colours[y as usize][x as usize] = self.current_color.clone();
                self.pos[0] += 1;
            }
        }
    }

    pub fn set_col(&mut self, c : &ColourCell) {
        self.current_color = c.clone();
    }

    pub fn set_pos(&mut self, pos : [isize;2] ) {
        self.pos = pos
    }

    pub fn write_col(&mut self, text : &str, col : ColourCell) {
        self.current_color = col;
        self.write(text)
    }

    pub fn cr(&mut self) {
        let [_,y] = self.pos;
        self.pos = [0,y+1];
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct TextScreen {
    text : Vec<Vec<char>>,
    colours : Vec<Vec<ColourCell>>,
    dims : [usize; 2]
}

impl TextScreen {
    pub fn iter(&self) -> TextScreenIterator {
        TextScreenIterator {
            text_screen : self,
            pos: [0,0],
            dims: [self.dims[0] as isize, self.dims[1] as isize]
        }
    }

    pub fn new(dims : [usize; 2]) -> Self {
        let mut ret = Self {
            text : vec![],
            colours: vec![],
            dims
        };
        ret.clear(' ', ColourCell::default());
        ret
    }

    pub fn in_bounds(&self, pos : &[isize;2]) -> bool {
        let [x,y] = pos;
        let [w,h] = self.dims;
        let [w,h] = [w as isize, h as isize];
        *x >= 0 && *x < w && *y >= 0 && *y < h
    }

    pub fn cursor(& mut self) -> Cursor {
        let [w,h] = self.dims;
        Cursor {
            screen : self,
            pos: [0,0],
            current_color: ColourCell::default(),
            dims : [w as isize, h as isize]
        }
    }

    pub fn clear(&mut self, c : char, col : ColourCell) {
        let [w,h] = self.dims;
        let text = vec![vec![c;w];h];
        let colours = vec![vec![col;w];h];
        self.text = text;
        self.colours = colours;
    }
}


////////////////////////////////////////////////////////////////////////////////
pub struct TextScreenIterator<'a> {
    text_screen : &'a TextScreen,
    pos : [isize;2],
    dims : [isize;2],
}

impl<'a> TextScreenIterator<'a> {
    fn bump(&mut self) {
        let [mut x,mut y] = self.pos;
        let w = self.dims[0];
        x += 1;
        if x == w { x = 0; y += 1; }
        self.pos = [x,y];
    }
}

pub struct TextChunk<'a> {
    text : &'a [char],
    col : &'a ColourCell,
    pos : [isize;2],
}

impl<'a> Iterator for TextScreenIterator<'a> {
    type Item = TextChunk<'a>;

    fn next(&mut self) -> Option<TextChunk<'a>> {
        if self.text_screen.in_bounds(&self.pos) {
            let [xind,yind] = [self.pos[0] as usize, self.pos[1] as usize];

            let col = &self.text_screen.colours[yind][xind];
            let text : &[char] = &self.text_screen.text[yind][xind..=xind+1];

            let ret = TextChunk { text, col, pos: self.pos };

            self.bump();

            Some(ret)

        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
