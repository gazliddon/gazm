use super::imgui;
use super::window::TextWinDims;
use super::scrbox::ScrBox;
use super::v2::*;
use super::colour::*;
use super::textcontext::TextRenderer;

pub struct ImgUiTextRender<'a> {
    dl : imgui::DrawListMut<'a>,
    pub win_dims : TextWinDims,
    pub text_dims : ScrBox
}

impl<'a> ImgUiTextRender<'a> {
    pub fn new(ui : &'a imgui::Ui<'a>) -> Self {
        let dl = ui.get_window_draw_list();
        let win_dims = TextWinDims::from_ui(ui);
        let text_dims = ScrBox::new(&V2::new(0,0), &win_dims.get_window_dims_in_chars());
        Self { text_dims, dl , win_dims }
    }
}

impl<'a> TextRenderer for ImgUiTextRender<'a> {

    fn get_window_dims(&self) -> ScrBox {
        self.text_dims
    }

    fn draw_text(&self,pos : &V2<isize>, text : &str, col : &Colour) {
        let pos = *pos + self.text_dims.pos;
        let [tl, _] = self.win_dims.as_pixel_extents_arrays( &pos, &V2::new(1,1));
        self.dl.add_text(tl,col,text);
    }

    fn draw_box(&self,  pos : &V2<isize>, dims : &V2<usize>, col : &Colour) {
        let pos = *pos + self.text_dims.pos;
        let [tl, br] = self.win_dims.as_pixel_extents( &pos, &dims);
        let tl = [tl.x, tl.y];
        let br = [br.x, br.y];
        self.dl.add_rect_filled_multicolor(tl, br, col, col, col, col );
    }


    fn draw_with_clip_rect<F>(&self, scr_box : &ScrBox, f: F) 
        where F: FnOnce(),
        {
            if let Some(new_box) = ScrBox::clip_box(&self.text_dims, &scr_box) {
                let [min, max] = self.win_dims.as_pixel_extents_arrays( &new_box.pos, &new_box.dims);
                self.dl.with_clip_rect_intersect(min,max, f);
            }
        }
}
