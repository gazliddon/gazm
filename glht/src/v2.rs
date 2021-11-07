pub use vector2d::Vector2D as V2;

pub type V2F = V2<f32>;

pub trait ToArray<U> {
    fn as_array(&self) -> [U;2];
}

impl<U> ToArray<U> for V2<U> 
where U : Copy + Clone
{
    fn as_array(&self) -> [U;2] {
        [self.x, self.y]
    }
}

