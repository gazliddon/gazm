use crate::v2::*;
use super::{Dimensions, FontInfo };

#[derive(Debug, Clone)]
pub struct TextWinDims {
    window_pixel_dims : WindowDims,
    win_char_dims : V2<usize>,
    char_dims : V2<f32>,
}

#[derive(Debug, Clone)]
pub struct WindowDims {
    // Pixel base position of window
    pub base_pos : V2<f32>,
    // Pixel dimensions of the window
    pub pixel_dims : V2<f32>,
}
impl Dimensions<isize> for TextWinDims {
    fn pos(&self) -> V2<isize> {
        self.win_char_dims.as_isizes()
    }

    fn dims(&self) -> V2<isize> {
        self.win_char_dims.as_isizes()
    }

}

impl TextWinDims {

    pub fn xform(&self, pos : &V2<isize>) -> V2<f32> {
        self.window_pixel_dims.base_pos + ( pos.as_f32s().mul_components(self.char_dims) )
    }

    pub fn get_window_dims_in_chars(&self) -> V2<usize> {
        self.win_char_dims
    }

    pub fn get_char_dims(&self) -> V2<f32> {
        self.char_dims
    }

    pub fn get_pixel_dims(&self) -> V2<f32> {
        self.window_pixel_dims.base_pos
    }

    pub fn new<A>(wp : A) -> Self 
    where A : Dimensions<f32> + FontInfo {
        let wdims = wp.dims();
        let char_dims = wp.char_extents();

        let win_char_dims = ( wdims.div_components(char_dims) ).as_usizes();

        let window_pixel_dims= WindowDims {
            base_pos : wp.pos(),
            pixel_dims : wp.dims()
        };

        TextWinDims {
            window_pixel_dims,
            char_dims, 
            win_char_dims}
    }
}


