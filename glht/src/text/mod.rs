mod imguitext;
mod textcontext;
mod window;

use crate::v2::*;
pub use imguitext::*;
pub use textcontext::{LinePrinter, TextContext, TextRenderer};
pub use window::*;

use num::traits::Num;

pub trait FontInfo {
    fn char_extents(&self) -> V2<f32>;
}

pub trait Dimensions<I : Num> {
    fn pos(&self) -> V2<I>;
    fn dims(&self) -> V2<I>;
    fn width(&self) -> I {
        self.dims().x
    }

    fn height(&self) -> I {
        self.dims().y
    }
}
