#[derive(Debug, Clone, PartialEq, Copy)]
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
    fn sub(&self, rhs : &Self) -> Self;

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
    fn blend(&self, rhs : &Self, frac : f64) -> Self {
        let t_frac = frac.fract();
        let diff = rhs.sub(self).mul_scalar(t_frac as f32);
        let col = self.add(&diff);
        col
    }
}

impl ColourOps for Colour {
    fn sub(&self, rhs : &Self) -> Self {
        let r = self.data[0] - rhs.data[0];
        let g = self.data[1] - rhs.data[1];
        let b = self.data[2] - rhs.data[2];
        let a = self.data[3] - rhs.data[3];
        Colour::new(r,g,b,a)
    }

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
        let mut ret = *self;

        for c in &mut ret.data {
            *c *=  amount;
        }

        ret
    }

    fn saturate(&self) -> Self {
        let mut ret = *self;
        for c in &mut ret.data {
            *c = (*c).min(1.0).max(0.0);
        }
        ret
    }

    fn add_scalar(&self, amount : f32) -> Self {
        let mut ret = *self;
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

    pub fn set_red(&mut self, v : f32) { self.data[0] = v }
    pub fn set_green(&mut self, v: f32) {self.data[1] = v}
    pub fn set_blue(&mut self, v: f32) {self.data[2] = v}
    pub fn set_alpha(&mut self, v: f32) {self.data[3] = v}

    pub fn get_red(&self) -> f32 { self.as_array()[0] }
    pub fn get_green(&self) -> f32 { self.as_array()[1] }
    pub fn get_blue(&self) -> f32 { self.as_array()[2] }
    pub fn get_alpha(&self) -> f32 { self.as_array()[3] }

}

pub static WHITE :&'static Colour  = &col!(1.0,1.0,1.0,1.0);
pub static BLACK :&'static Colour = &col!(0.0,0.0,0.0,1.0);
pub static RED :&'static Colour = &col!(1.0,0.0,0.0,1.0);
pub static GREEN :&'static Colour = &col!(0.0,1.0,0.0,1.0);
pub static BLUE :&'static Colour = &col!(0.0,0.0,1.0,1.0);
pub static YELLOW :&'static Colour = &col!(1.0,1.0,0.0,1.0);
pub static PURPLE :&'static Colour = &col!(1.0,0.0,1.0,1.0);


