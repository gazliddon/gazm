use super::colour::*;

impl Cycler {
    pub fn new(speed: f64, cols: Vec<Colour>) -> Self {
        let total_t = speed * cols.len() as f64;
        let t_mul = 1.0 / total_t;

        Self {
            cols,
            per_entry: speed,
            t_mul,
        }
    }

    fn select(&self, t: f64) -> &Colour {
        &self.cols[(t.abs() as usize) % self.cols.len()]
    }

    pub fn get_col(&self, t: f64) -> Colour {
        let t = t * self.t_mul;
        let c1 = self.select(t);
        let c2 = self.select(t + 1.0);
        c1.blend(c2, t.fract())
    }
}

struct Cycler {
    cols: Vec<Colour>,
    per_entry: f64,
    t_mul: f64,
}
