
#[derive(Debug, Clone)]
pub struct Colour {
    data: [f32;4]
}

impl Default for Colour {
    fn default() -> Self {
        Self::make_white()
    }
}

impl Colour {
    pub fn as_array(&self) -> &[f32;4] {
        &self.data
    }

    pub fn new(r : f32, g : f32, b: f32, a : f32) -> Self {
        Self { data : [r,g,b,a] }
    }
    pub fn new_rgb(r : f32, g : f32, b: f32) -> Self {
        Self { data : [r,g,b,0.0] }
    }

    pub fn make_white() -> Self {
        Self::new_rgb(1.0,1.0,1.0)
    }

    pub fn make_red() -> Self {
        Self::new_rgb(1.0,0.0,0.0)
    }

    pub fn make_green() -> Self {
        Self::new_rgb(0.0,1.0,0.0)
    }

    pub fn make_blue() -> Self {
        Self::new_rgb(0.0,0.0,1.0)
    }

    pub fn make_yellow() -> Self {
        Self::new_rgb(1.0,1.0,0.0)
    }

    pub fn make_black() -> Self {
        Self::new_rgb(0.0,0.0,0.0)
    }

    pub fn red(&self) -> f32 { self.as_array()[0] }
    pub fn green(&self) -> f32 { self.as_array()[1] }
    pub fn blue(&self) -> f32 { self.as_array()[2] }
    pub fn alpha(&self) -> f32 { self.as_array()[3] }
}

lazy_static! {
    pub static ref WHITE : Colour = Colour::make_white();
    pub static ref RED : Colour = Colour::make_white();
    pub static ref GREEN : Colour = Colour::make_green();
    pub static ref BLUE : Colour = Colour::make_blue();
    pub static ref YELLOW : Colour = Colour::make_yellow();
}
