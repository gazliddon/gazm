
mod imguitext;
mod textcontext;
mod traits;

pub use imguitext::*;
pub use textcontext::*;
pub use traits::*;

impl<I: num::traits::Num + Clone> Dimensions<I> for super::V2<I> {
    fn dims(&self) -> super::V2<I> {
        self.clone()
    }
}

