use crate::colour::*;
use vector2d::Vector2D as V2;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct ColourCell {
    pub fg : Colour,
    pub bg : Colour,
}

impl ColourCell {
    pub fn new(fg : Colour, bg : Colour) -> Self {
        Self { fg, bg }
    }

    fn fmap<F>(&self, func : F) -> Self  
        where F : Fn(&Colour) -> Colour
        {
            Self {
                bg : func(&self.bg),
                fg : func(&self.fg)
            }
        }

    fn cross<F>(&self, rhs : &Self, func : F) -> Self 
        where F : Fn(&Colour, &Colour) -> Colour
        {

            Self {
                fg : func(&self.fg, &rhs.fg ),
                bg : func(&self.bg, &rhs.bg ),

            }

        }
}

impl ColourOps for ColourCell { 
    fn mul(&self, rhs : &Self) -> Self {
        self.cross(rhs, |lhs, rhs| lhs.mul(rhs))
    }

    fn add(&self, rhs : &Self) -> Self {
        self.cross(rhs, |lhs, rhs| lhs.add(rhs))
    }

    fn add_scalar(&self, n : f32 ) -> Self {
        self.fmap(|c| c.add_scalar(n))
    }

    fn mul_scalar(&self, n : f32 ) -> Self {
        self.fmap(|c| c.mul_scalar(n))
    }

    fn saturate(&self ) -> Self {
        self.fmap(|c| c.saturate())
    }
}

impl Default for ColourCell {
    fn default()  -> Self {
        Self {
            fg: WHITE.clone(),
            bg: BLACK.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScrBox {
    pub pos : V2<isize>,
    pub dims : V2<usize>
}

impl ScrBox {
    pub fn new(pos : V2<isize>, dims : V2<usize>) -> Self {
        Self {pos, dims}
    }

    pub fn get_br(&self) -> V2<isize> {
        ( self.dims.as_isizes() + self.pos ) - V2{x:1,y:1}
    }

    pub fn in_bounds(&self, pos : &V2<isize>) -> bool {
        let V2{x,y} = *pos;

        let tl = self.pos;
        let br = self.get_br();

        x >= tl.x && x <= br.x && y >=tl.y && y <= br.y
    }

    pub fn intersects(&self, a : &ScrBox) -> bool {
        Self::clip_box(self, a).is_some()
    }

    pub fn clip_box(clipper : &ScrBox, box_to_clip : &ScrBox) -> Option<Self> {
        use std::cmp::{ max,min };

        let clipper_br = clipper.get_br();

        let x = max(box_to_clip.pos.x, clipper.pos.x);
        let y = max(box_to_clip.pos.y, clipper.pos.y);

        let tl = V2{x,y};
        let br = box_to_clip.get_br();

        if clipper.in_bounds(&tl) {
            let brx = min(br.x, clipper_br.x);
            let bry = min(br.y, clipper_br.y);

            let w = ( brx-tl.x ) + 1;
            let h = ( bry-tl.y ) + 1;

            if w >= 0 && h >= 0  {
                return Some( Self::new(tl, V2{x: w,y: h}.as_usizes()) )
            }
        }

        None
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
    fn get_cell(&self) -> Option<Cell>;
    fn get_pos(&self) -> V2<isize>;

    fn write_cells(&mut self, text : &[Cell])-> &mut Self;

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
    fn write_cells(&mut self, _text : &[Cell])-> &mut Self {
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
        self.current_color = c.clone();
        self
    }

    fn set_pos(&mut self, pos : V2<isize> ) -> &mut Self{
        self.pos = pos;
        self
    }

    fn get_cell(&self) -> Option<Cell>{
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
}

#[derive(Debug, Clone)]
pub struct Cell<'a> {
    pub text : &'a str,
    pub col : &'a ColourCell,
    pub pos : V2<isize>,
}

impl TextScreen {

    pub fn get_cell_from_index(&self, index  : usize) -> Option<Cell> {
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
            self.colours[idx] = col.clone()
        }
    }

    pub fn fill_colour_line(&mut self, line : usize, col : &ColourCell) -> Option<ScrBox> {
        let col_box = ScrBox::new(V2{x:0,y:line as isize}, V2{x:self.dims.x as usize,y:1});
        self.fill_colour_box(&col_box,col)
    }

    pub fn fill_colour_box(&mut self, scr_box : &ScrBox, col : &ColourCell) -> Option<ScrBox> {
        let clipped_box = Self::get_cliped_box(&self.dim_box, &scr_box);

        if let Some((scr_box,_)) = clipped_box {
            let tl = &scr_box.pos;
            let br = scr_box.get_br();

            for y in tl.y..=br.y {
                for x in tl.x..=br.x {
                    if let Some(idx) = self.coords_to_index(&V2{x,y}) {
                        self.colours[idx] = col.clone();
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
            let mut clipped_box = write_box.clone();
            clipped_box.pos = write_box.pos - clipped_box.pos;
            (write_box, clipped_box)
        })
    }

    fn write_clipped<F>(dim_box : &ScrBox, pos : &V2<isize>, txt_len : usize, mut func : F) -> V2<isize>
        where F : FnMut(usize, std::ops::Range<usize>, std::ops::Range<usize>)

        {
            let text_box = ScrBox::new(*pos, V2{x:txt_len, y:1});

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

        Self::write_clipped(&db, pos, txt.len(), func )
    }

    pub fn write_cells(&mut self, pos : &V2<isize>, txt : &[Cell]) -> V2<isize> {
        let db = &self.dim_box.clone();

        let func = |y : usize, scr_r,  txt_r : std::ops::Range<usize>| {

            let _cols : Vec<ColourCell> = txt[txt_r.clone()].iter().map(|c| c.col.clone()).collect();
            let string : Vec<_> = txt[txt_r.clone()].iter().map(|c| c.text.as_bytes()[0]).collect();
            self.text[y].replace_range(scr_r,&( String::from_utf8(string).unwrap() ));

        };

        Self::write_clipped(&db, pos, txt.len(), func )
    }


    pub fn write_with_colour(&mut self, pos : &V2<isize>, txt : &str, _col : &ColourCell) -> V2<isize> {
        let db = &self.dim_box.clone();
        use std::ops::Range;

        let func = |y : usize, scr_r : Range<usize>, txt_r : Range<usize>| {
            let x = scr_r.start;
            self.text[y].replace_range(scr_r, &txt[txt_r.clone()]);
            let ix = self.coords_to_index(&V2{x , y}.as_isizes()).unwrap();
            for i in txt_r {
                self.colours[ix+i] = _col.clone()
            }
        };

        Self::write_clipped(&db, pos, txt.len(), func )
    }

    pub fn get_cell(&self, pos : V2<isize>) -> Option<Cell> {
        if let Some(idx) = self.coords_to_index(&pos) {
            let V2{x,y} = pos.as_usizes();
            let text = &self.text[y][x..=x];
            let col = &self.colours[idx];
            let ret = Cell { text, col, pos };
            Some(ret)
        } else {
            None
        }
    }

    pub fn new(dims : V2<usize>) -> Self {
        let mut ret = Self {
            text : vec![],
            colours: vec![],
            dims: dims.as_isizes(),
            dim_box : ScrBox::new(V2{x:0, y:0}, dims)
        };

        ret.clear(' ', &ColourCell::default());
        ret
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

        let colours = vec![col.clone();w*h];
        self.colours = colours;
    }
}


////////////////////////////////////////////////////////////////////////////////



////////////////////////////////////////////////////////////////////////////////
