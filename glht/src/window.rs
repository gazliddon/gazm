use crate::colour::*;

use vector2d::{Vector2D as V2};
use super::imgui;

impl std::convert::From<Colour> for imgui::ImColor {
    fn from(v: Colour) -> Self {
        Self::from(*v.as_array())
    }
}

impl std::convert::From<&Colour> for imgui::ImColor {
    fn from(v: &Colour) -> Self {
        Self::from(*v.as_array())
    }
}

#[derive(Debug, Clone)]
pub struct TextWinDims {
    base_pos : V2<f32>,
    pixel_dims : V2<f32>,
    win_char_dims : V2<usize>,
    char_dims : V2<f32>,
}


impl TextWinDims {


    pub fn get_box_dims(&self, tl : V2<usize>, wh : V2<usize>) -> [V2<f32>;2] {
        let tl = self.get_pixel_pos(tl);
        let br = tl + self.char_dims.mul_components(wh.as_f32s());

        [tl,br]
    }

    pub fn get_pixel_pos(&self, pos : V2<usize>) -> V2<f32> {
        self.base_pos + ( pos.as_f32s().mul_components(self.char_dims) )
    }

    pub fn get_window_char_dims(&self) -> V2<usize> {
        self.win_char_dims
    }

    pub fn get_char_dims(&self) -> V2<f32> {
        self.char_dims
    }

    pub fn is_visible(&self) -> bool {
        self.win_char_dims.x > 0 && self.win_char_dims.y > 0
    }

    pub fn new(ui : &imgui::Ui) -> Self {
        let [char_width,char_height] = ui.calc_text_size(im_str!( " " ), false, std::f32::MAX);
        let [ww,wh] = ui.content_region_avail();

        let pixel_dims = V2{x:ww,y:wh};
        let char_dims= V2{x:char_width, y:char_height};
        let win_char_dims = ( pixel_dims.div_components(char_dims) ).as_usizes();

        let base_pos = ui.cursor_screen_pos();
        let base_pos = V2{x:base_pos[0], y:base_pos[1]};

        TextWinDims { pixel_dims, char_dims, win_char_dims, base_pos }
    }
}


