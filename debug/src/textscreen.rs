// use crate::colour::*;
use super::v2::*;
use super::colourcell::ColourCell;
use super::scrbox::ScrBox;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub cols : ColourCell,
    pub glyph : char
}

impl Glyph {
    pub fn new(glyph : char, cols : &ColourCell) -> Self {
        Self {
            glyph, cols: *cols
        }
    }
    pub fn new_bw(glyph : char) -> Self {
        Glyph::new(glyph, &ColourCell::new_bw())
    }
}


////////////////////////////////////////////////////////////////////////////////

pub struct Cursor<'a> {
    screen : &'a mut TextScreen,
    dims : V2<isize>,
    pos : V2<isize>,
    current_color : ColourCell,
}

pub trait CursorTrait {
    fn get_dims(&self) -> V2<isize>;
    fn get_cell(&self) -> Option<&Glyph>;
    fn get_pos(&self) -> V2<isize>;

    fn write_cells(&mut self, text : &[Glyph])-> &mut Self;

    fn write(&mut self, text : &str)-> &mut Self;
    fn set_col(&mut self, c : &ColourCell) -> &mut Self;
    fn set_pos(&mut self, pos : V2<isize> ) -> &mut Self;
    fn get_col(&self) -> &ColourCell;

    fn clear_line(&mut self) -> &mut Self {
        let V2{x:w,..} = self.get_dims();

        let blank : String = vec![' ';w as usize].into_iter().collect();
        let old_pos = self.get_pos();

        self.set_pos(V2{x:0, y: old_pos.y});
        self.write(&blank);
        self.set_pos(old_pos);

        self
    }

    fn write_col(&mut self, text : &str, col : &ColourCell) -> &mut Self {
        self.set_col(col);
        self.write(text)
    }

    fn cr(&mut self) -> &mut Self {
        let new_pos = V2 { x : 0, y: self.get_pos().y + 1 };
        self.set_pos(new_pos);
        self
    }
}


impl<'a> CursorTrait for Cursor<'a> {
    fn write_cells(&mut self, _text : &[Glyph])-> &mut Self {
        self
    }

    fn get_col(&self) -> &ColourCell {
        &self.current_color
    }

    fn get_dims(&self) -> V2<isize> {
        self.screen.dims
    }

    fn write(&mut self, text : &str)-> &mut Self {
        self.pos = self.screen.write_with_colour(&self.pos.clone(), text, &self.current_color.clone());
        self
    }


    fn set_col(&mut self, c : &ColourCell) -> &mut Self{
        self.current_color = *c;
        self
    }

    fn set_pos(&mut self, pos : V2<isize> ) -> &mut Self{
        self.pos = pos;
        self
    }

    fn get_cell(&self) -> Option<&Glyph>{
        self.screen.get_cell(self.pos)
    }

    fn get_pos(&self) -> V2<isize> {
        self.pos
    }
}


////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TextScreen {
    pub text : Vec<String>,
    pub colours : Vec<ColourCell>,
    pub dims : V2<isize>,
    pub dim_box : ScrBox,
    pub glyph_text: Vec<Vec<Glyph>>
}


impl TextScreen {
    pub fn get_glyph_from_index(&self, index  : usize) -> Option<&Glyph> {
        if let Some(pos) = self.index_to_coords(index) {
            self.get_cell(pos.as_isizes())
        } else {
            None
        }
    }

    pub fn get_cell_from_index(&self, index  : usize) -> Option<&Glyph> {
        if let Some(pos) = self.index_to_coords(index) {
            self.get_cell(pos.as_isizes())
        } else {
            None
        }
    }

    fn index_to_coords(&self, index : usize) -> Option<V2<usize>> {
        let V2{x: rows, y: lines} = self.dims.as_usizes();

        let x = index % rows;
        let y = index / lines;

        let pos = V2{x,y};

        if self.in_bounds(&pos.as_isizes()) {
            Some(pos)
        } else {
            None
        }
    }

    fn coords_to_index(&self, pos : &V2<isize>) -> Option<usize> {
        if self.in_bounds(pos) {
            let V2{x:w,..} = self.dims;
            Some( ( pos.y * w +  pos.x ) as usize)
        } else {
            None
        }
    }

    pub fn write_colour(&mut self, pos : &V2<isize>, col : &ColourCell) {
        if let Some(idx) = self.coords_to_index(pos) {
            self.colours[idx] = *col;
        }
    }

    pub fn fill_colour_line(&mut self, line : usize, col : &ColourCell) -> Option<ScrBox> {
        let col_box = ScrBox::new(&V2{x:0,y:line as isize}, &V2{x:self.dims.x as usize,y:1});
        self.fill_colour_box(&col_box,col)
    }

    pub fn fill_colour_box(&mut self, scr_box : &ScrBox, col : &ColourCell) -> Option<ScrBox> {
        let clipped_box = Self::get_cliped_box(&self.dim_box, scr_box);

        if let Some((scr_box,_)) = clipped_box {
            let tl = &scr_box.pos;
            let br = scr_box.get_br();

            for y in tl.y..=br.y {
                for x in tl.x..=br.x {
                    if let Some(idx) = self.coords_to_index(&V2{x,y}) {
                        self.colours[idx] = *col;
                    }
                }
            }
            Some( scr_box )
        } else {
            None
        }
    }

    fn get_cliped_box(dim_box : &ScrBox, scr_box : &ScrBox) -> Option<(ScrBox, ScrBox)> {
        ScrBox::clip_box(dim_box, scr_box).map(|write_box| {
            let mut clipped_box = write_box;
            clipped_box.pos = write_box.pos - clipped_box.pos;
            (write_box, clipped_box)
        })
    }

    fn write_clipped<F>(dim_box : &ScrBox, pos : &V2<isize>, txt_len : usize, mut func : F) -> V2<isize>
        where F : FnMut(usize, std::ops::Range<usize>, std::ops::Range<usize>)

        {
            let text_box = ScrBox::new(pos, &V2{x:txt_len, y:1});

            if let Some((scr_box, text_box)) = Self::get_cliped_box(dim_box,&text_box) {
                let V2{x,y} = scr_box.pos.as_usizes();
                let V2{x:w,..} = text_box.dims;
                let V2{x: tx,..} = text_box.pos.as_usizes();
                let scr_r = x..x+w;
                let txt_r = tx..tx+w;

                func(y, scr_r, txt_r);
            }

            let mut pos = *pos;
            pos.x += txt_len as isize;
            pos

        }

    pub fn write(&mut self, pos : &V2<isize>, txt : &str) -> V2<isize> {
        let db = &self.dim_box.clone();

        let func = |y : usize, scr_r, txt_r| {
            self.text[y].replace_range(scr_r, &txt[txt_r]);
        };

        Self::write_clipped(db, pos, txt.len(), func )
    }

    pub fn write_cells(&mut self, _pos : &V2<isize>, _txt : &[Glyph]) -> V2<isize> {
        panic!("TBD")
    }


    pub fn write_with_colour(&mut self, pos : &V2<isize>, txt : &str, _col : &ColourCell) -> V2<isize> {
        let db = &self.dim_box.clone();
        use std::ops::Range;

        let func = |y : usize, scr_r : Range<usize>, txt_r : Range<usize>| {
            let x = scr_r.start;
            self.text[y].replace_range(scr_r, &txt[txt_r.clone()]);
            let ix = self.coords_to_index(&V2{x , y}.as_isizes()).unwrap();
            for i in txt_r {
                self.colours[ix+i] = *_col;
            }
        };

        Self::write_clipped(db, pos, txt.len(), func )
    }

    pub fn get_cell(&self, _pos : V2<isize>) -> Option<&Glyph> {
        panic!("TBD")
    }

    pub fn new(_dims : V2<usize>) -> Self {
        panic!("TBD")
    }

    pub fn resize(&mut self, dims : V2<usize> ) {
        self.dim_box.dims = dims;
        self.clear(' ', &ColourCell::default());
    }

    pub fn in_bounds(&self, pos : &V2<isize>) -> bool {
        let V2{x,y} = pos;
        let V2{x : w, y: h} = self.dims;
        *x >= 0 && *x < w && *y >= 0 && *y < h
    }

    pub fn cursor(& mut self) -> Cursor {
        let dims = self.dims;
        Cursor {
            screen : self,
            pos: V2{x:0, y:0},
            current_color: ColourCell::default(),
            dims 
        }
    }

    pub fn clear(&mut self, c : char, col : &ColourCell) {
        let V2{x : w, y: h} = self.dims.as_usizes();

        let blank = vec![c;w].into_iter().collect();
        self.text = vec![blank;h];

        let colours = vec![*col;w*h];
        self.colours = colours;
    }
}

// Sourcewin


////////////////////////////////////////////////////////////////////////////////



////////////////////////////////////////////////////////////////////////////////