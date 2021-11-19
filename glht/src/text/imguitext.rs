use crate::colour::*;
use crate::imgui::{color, DrawListMut, Font, Ui};
use crate::scrbox::ScrBox;
use crate::v2::*;
use std::f32;

use super::{Dimensions, TextRenderer};

impl From<Colour> for color::ImColor32 {
    fn from(v: Colour) -> Self {
        Self::from(*v.as_array())
    }
}

impl From<&Colour> for color::ImColor32 {
    fn from(v: &Colour) -> Self {
        Self::from(*v.as_array())
    }
}

pub struct ImgUiTextRender<'a> {
    dl: DrawListMut<'a>,
    offset: V2<f32>,
    pos: V2<f32>,
    grid_spacing: V2<f32>,
    grid_dims: V2<usize>,
}

impl Dimensions<f32> for Font {
    fn dims(&self) -> V2<f32> {
        V2::new(self.fallback_advance_x, self.font_size)
    }
}

impl<'a> ImgUiTextRender<'a> {
    pub fn new(pos : &V2<f32>, grid_spacing: &V2<f32>, grid_dims: &V2<usize>, ui: &'a Ui<'a>) -> Self {
        let dl = ui.get_window_draw_list();

        // FIX
        // No idea why this is offset
        let offset = V2::new(grid_spacing.x / 2.0, 0.0);

        Self {
            dl,
            offset,
            grid_spacing: *grid_spacing,
            pos : *pos,
            grid_dims : *grid_dims
        }
    }

    fn xform(&self, pos: &V2<isize>) -> [f32; 2] {
        let pos = self.pos + (pos.as_f32s().mul_components(self.grid_spacing));
        let pos = pos + self.offset;
        pos.as_array()
    }

    fn xform_box(&self, pos: &V2<isize>, dims: &V2<usize>) -> [[f32; 2]; 2] {
        let br = *pos + dims.as_isizes();
        let tl = self.xform(pos);
        let br = self.xform(&br);
        [tl, br]
    }
}

impl<'a> Dimensions<usize> for ImgUiTextRender<'a> {
    fn dims(&self) -> V2<usize> {
        self.grid_dims
    }
}

impl<'a> TextRenderer for ImgUiTextRender<'a> {
    fn draw_text(&self, pos: &V2<isize>, text: &str, col: &Colour) {
        let tl = self.xform(pos);
        self.dl.add_text(tl, col, text);
    }

    fn draw_box(&self, pos: &V2<isize>, dims: &V2<usize>, col: &Colour) {
        let [tl, br] = self.xform_box(pos, dims);
        self.dl
            .add_rect_filled_multicolor(tl, br, col, col, col, col);
    }

    fn draw_with_clip_rect<F>(&self, scr_box: &ScrBox, f: F)
    where
        F: FnOnce(),
    {
        let whole_grid = ScrBox::new(&V2::new(0, 0), &self.grid_dims);
        if let Some(new_box) = ScrBox::clip_box(&whole_grid, &scr_box) {
            let [min, max] = self.xform_box(&new_box.pos, &new_box.dims);
            self.dl.with_clip_rect_intersect(min, max, f);
        }
    }
}
