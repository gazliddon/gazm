use super::colour::*;
use super::imgui;
use imgui::im_str;

use super::v2::*;

impl std::convert::From<Colour> for imgui::color::ImColor32 {
    fn from(v: Colour) -> Self {
        Self::from(*v.as_array())
    }
}

impl std::convert::From<&Colour> for imgui::color::ImColor32 {
    fn from(v: &Colour) -> Self {
        Self::from(*v.as_array())
    }
}

#[derive(Debug, Clone)]
pub struct TextWinDims {
    window_pixel_dims : WindowDims,
    win_char_dims : V2<usize>,
    char_dims : V2<f32>,
}

#[derive(Debug, Clone)]
pub struct WindowDims {
    // Pixel base position of window
    base_pos : V2<f32>,
    // Pixel dimensions of the window
    pixel_dims : V2<f32>,
}

impl TextWinDims {
    pub fn height(&self) -> usize {
        self.win_char_dims.y
    }

    pub fn width(&self) -> usize {
        self.win_char_dims.x
    }

    pub fn as_pixel_extents_arrays(&self, tl : &V2<isize>, wh : &V2<usize>) -> [[f32;2];2] {
        let [tl,br] = self.as_pixel_extents(tl,wh);
        [[tl.x,tl.y], [br.x, br.y]]
    }

    pub fn as_pixel_extents(&self, tl : &V2<isize>, wh : &V2<usize>) -> [V2<f32>;2] {
        let tl = self.get_pixel_pos(tl);
        let br = tl + self.char_dims.mul_components(wh.as_f32s());
        [tl,br]
    }

    pub fn get_pixel_pos(&self, pos : &V2<isize>) -> V2<f32> {
        self.window_pixel_dims.base_pos + ( pos.as_f32s().mul_components(self.char_dims) )
    }

    pub fn get_window_dims_in_chars(&self) -> V2<usize> {
        self.win_char_dims
    }

    pub fn get_char_dims(&self) -> V2<f32> {
        self.char_dims
    }

    pub fn is_visible(&self) -> bool {
        self.win_char_dims.x > 0 && self.win_char_dims.y > 0
    }

    pub fn get_pixel_dims(&self) -> V2<f32> {
        self.window_pixel_dims.base_pos
    }

    pub fn new(window_pixel_dims : &WindowDims, char_dims : &V2<f32>) -> Self {
        let win_char_dims = ( window_pixel_dims.pixel_dims.div_components(*char_dims) ).as_usizes();

        TextWinDims {
            window_pixel_dims: window_pixel_dims.clone(),
            char_dims : *char_dims,
            win_char_dims}
    }

    pub fn from_ui(ui : &imgui::Ui) -> Self {
        let [ww,wh] = ui.content_region_max();
        let [char_width,char_height] = ui.calc_text_size(im_str!( "A" ), false, std::f32::MAX);
        let base_pos = ui.window_pos();

        let char_dims= V2{x:char_width, y:char_height};
        let window_pixel_dims = WindowDims {
            base_pos : V2::new(base_pos[0], base_pos[1]), pixel_dims : V2{x:ww,y:wh}
        };

        Self::new(&window_pixel_dims, &char_dims)
    }
}


