
use super::v2::*;
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ScrBox {
    pub pos : V2<isize>,
    pub dims : V2<usize>
}

impl ScrBox {
    pub fn new(pos : &V2<isize>, dims : &V2<usize>) -> Self {
        Self {pos : *pos, dims: *dims}
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
                return Some( Self::new(&tl, &V2{x: w,y: h}.as_usizes()) )
            }
        }

        None
    }

}
