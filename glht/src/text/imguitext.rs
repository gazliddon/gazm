use super::TextRenderer;

use crate::colour::*;
use crate::scrbox::ScrBox;
use crate::v2::*;

use crate::imgui;
use imgui::im_str;

use super::{ Dimensions, FontInfo, TextWinDims };

impl From<Colour> for imgui::color::ImColor32 {
    fn from(v: Colour) -> Self {
        Self::from(*v.as_array())
    }
}

impl From<&Colour> for imgui::color::ImColor32 {
    fn from(v: &Colour) -> Self {
        Self::from(*v.as_array())
    }
}

pub struct ImgUiTextRender<'a> {
    dl: imgui::DrawListMut<'a>,
    offset : V2<f32>,
    pub win_dims: TextWinDims,
    pub text_dims: ScrBox,
}

impl<'a> Dimensions<f32> for &imgui::Ui<'a> {
    fn pos(&self) -> V2<f32> {
        let [x, y] = self.window_pos();
        V2::new(x, y)
    }

    fn dims(&self) -> V2<f32> {
        let [ww, wh] = self.content_region_max();
        V2::new(ww, wh)
    }
}



impl<'a> FontInfo for &imgui::Ui<'a> {
    fn char_extents(&self) -> V2<f32> {
        let [x,y] = self.calc_text_size(im_str!("A"), false, std::f32::MAX);
        let char_dims = V2 { x,y };
        char_dims
    }
}

impl<'a> Dimensions<isize> for ImgUiTextRender<'a> {
    fn pos(&self) -> V2<isize> {
        self.text_dims.pos
    }

    fn dims(&self) -> V2<isize> {
        self.text_dims.dims.as_isizes()
    }
}

impl<'a> ImgUiTextRender<'a> {
    pub fn new(ui: &'a imgui::Ui<'a>) -> Self {
        let dl = ui.get_window_draw_list();
        let win_dims =  TextWinDims::new(&*ui);

        // FIX
        // No idea why this is offset
        let offset = win_dims.get_char_dims().mul_components(V2::new(0.5, 0.0));

        let text_dims = ScrBox::new(&V2::new(0, 0), &win_dims.get_window_dims_in_chars());
        Self {
            text_dims,
            dl,
            win_dims,
            offset,
        }
    }

    fn xform(&self, pos: &V2<isize>) -> [f32; 2] {
        let pos = *pos + self.text_dims.pos;
        let r = self.win_dims.xform(&pos) + self.offset;
        r.as_array()
    }

    fn xform_box(&self, pos: &V2<isize>, dims: &V2<usize>) -> [[f32; 2]; 2] {
        let br = *pos + dims.as_isizes();
        let tl = self.xform(pos);
        let br = self.xform(&br);
        [tl, br]
    }
}

impl<'a> TextRenderer for ImgUiTextRender<'a> {
    fn get_window_dims(&self) -> ScrBox {
        self.text_dims
    }

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
        if let Some(new_box) = ScrBox::clip_box(&self.text_dims, &scr_box) {
            let [min, max] = self.xform_box(&new_box.pos, &new_box.dims);
            self.dl.with_clip_rect_intersect(min, max, f);
        }
    }
}
