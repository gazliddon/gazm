use crate::colour::{ Colour, WHITE, BLACK};
pub use crate::colour::ColourOps;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ColourCell {
    pub fg : Colour,
    pub bg : Colour,
}

impl ColourCell {
    pub fn new_bw() -> Self {
        Self { fg : WHITE.clone(), 
            bg : BLACK.clone() 
    }}

    pub fn new(fg : &Colour, bg : &Colour) -> Self {
        Self { fg: fg.clone(), bg: bg.clone() }
    }

    fn fmap<F>(&self, func : F) -> Self  
        where F : Fn(&Colour) -> Colour
        {
            Self {
                bg : func(&self.bg),
                fg : func(&self.fg)
            }
        }

    fn cross<F>(&self, rhs : &Self, func : F) -> Self 
        where F : Fn(&Colour, &Colour) -> Colour
        {

            Self {
                fg : func(&self.fg, &rhs.fg ),
                bg : func(&self.bg, &rhs.bg ),

            }

        }
}

impl ColourOps for ColourCell { 
    fn mul(&self, rhs : &Self) -> Self {
        self.cross(rhs, |lhs, rhs| lhs.mul(rhs))
    }
    fn sub(&self, rhs : &Self) -> Self {
        self.cross(rhs, |lhs, rhs| lhs.sub(rhs))
    }

    fn add(&self, rhs : &Self) -> Self {
        self.cross(rhs, |lhs, rhs| lhs.add(rhs))
    }

    fn add_scalar(&self, n : f32 ) -> Self {
        self.fmap(|c| c.add_scalar(n))
    }

    fn mul_scalar(&self, n : f32 ) -> Self {
        self.fmap(|c| c.mul_scalar(n))
    }

    fn saturate(&self ) -> Self {
        self.fmap(|c| c.saturate())
    }
}

impl Default for ColourCell {
    fn default()  -> Self {
        Self {
            fg: WHITE.clone(),
            bg: BLACK.clone(),
        }
    }
}
