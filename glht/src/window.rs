use crate::colour::*;

impl std::convert::From<Colour> for imgui::ImColor {
    fn from(v: Colour) -> Self {
        Self::from(*v.as_array())
    }
}

#[derive(Debug, Clone)]
pub struct TextWinDims {
    pub pixel_dims : [f32;2 ],
    pub char_dims : [usize;2],
    pub line_height: f32,
}

impl TextWinDims {
    pub fn new(ui : &imgui::Ui) -> Self {
        let [_,line_height] = ui.calc_text_size(im_str!( " " ), false, std::f32::MAX);
        let [ww,wh] = ui.content_region_avail();
        let lines = (wh / line_height ) - 2.0;

        let lines : usize = if lines < 0.0 {
            0 as usize
        } else {
            lines as usize
        };

        TextWinDims {
            pixel_dims: [ww,wh],
            char_dims: [0,lines],
            line_height
        }
    }
}


