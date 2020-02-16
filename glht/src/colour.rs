
#[derive(Debug, Clone, PartialEq)]
pub struct Colour {
    pub data: [f32;4]
}

impl Default for Colour {
    fn default() -> Self {
        WHITE.clone()
    }
}

macro_rules! col {
    ($r:expr,$g:expr,$b:expr,$a:expr) => {
        Colour {
            data: [$r,$g, $b, $a]
        }
    };
}

pub trait ColourOps  : Sized {
    fn add_scalar(&self, n : f32 ) -> Self;
    fn mul_scalar(&self, n : f32 ) -> Self;
    fn saturate(&self ) -> Self;

    fn add(&self, rhs : &Self) -> Self;
    fn mul(&self, rhs : &Self) -> Self;


    fn add_saturate(&self, rhs : &Self) -> Self {
        self.add(rhs).saturate()
    }

    fn mul_saturate(&self, rhs : &Self) -> Self {
        self.mul(rhs).saturate()
    }

    fn div_scalar(&self, n : f32) -> Self {
        self.mul_scalar(1.0 / n)
    }

    fn sub_scaler(& self, n : f32) -> Self {
        self.add_scalar(-n)
    }

    fn add_scalar_sat(& self, n : f32 ) -> Self {
        self.add_scalar(n).saturate()
    }

    fn mul_scalar_sat(& self, n : f32 ) -> Self {
        self.mul_scalar(n).saturate()
    }
}

impl ColourOps for Colour {

    fn mul(&self, rhs : &Self) -> Self {

        let r = self.data[0] * rhs.data[0];
        let g = self.data[1] * rhs.data[1];
        let b = self.data[2] * rhs.data[2];
        let a = self.data[3] * rhs.data[3];

        Colour::new(r,g,b,a)
    }

    fn add(&self, rhs : &Self) -> Self {

        let r = self.data[0] + rhs.data[0];
        let g = self.data[1] + rhs.data[1];
        let b = self.data[2] + rhs.data[2];
        let a = self.data[3] + rhs.data[3];

        Colour::new(r,g,b,a)
    }

    fn mul_scalar(&self, amount : f32) -> Self {
        let mut ret = self.clone();

        for c in &mut ret.data {
            *c *=  amount;
        }

        ret
    }

    fn saturate(&self) -> Self {
        let mut ret = self.clone();
        for c in &mut ret.data {
            *c = (*c).min(1.0).max(0.0);
        }
        ret
    }

    fn add_scalar(&self, amount : f32) -> Self {
        let mut ret = self.clone();
        for c in &mut ret.data {
            *c += amount
        }

        ret
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

    pub fn get_red(&self) -> f32 { self.as_array()[0] }
    pub fn get_green(&self) -> f32 { self.as_array()[1] }
    pub fn get_blue(&self) -> f32 { self.as_array()[2] }
    pub fn get_alpha(&self) -> f32 { self.as_array()[3] }

}



pub static WHITE : Colour = col!(1.0,1.0,1.0,1.0);
pub static BLACK : Colour = col!(0.0,0.0,0.0,1.0);
pub static RED : Colour = col!(1.0,0.0,0.0,1.0);
pub static GREEN : Colour = col!(0.0,1.0,0.0,1.0);
pub static BLUE : Colour = col!(0.0,0.0,1.0,1.0);
pub static YELLOW : Colour = col!(1.0,1.0,0.0,1.0);
pub static PURPLE : Colour = col!(1.0,0.0,1.0,1.0);


