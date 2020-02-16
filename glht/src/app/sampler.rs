////////////////////////////////////////////////////////////////////////////////

pub trait CanSample: PartialEq + Ord + Copy + num::Num + num::NumCast {}

#[allow(dead_code)]
pub struct Sampler<T: CanSample> {
    samples: Vec<T>,
    max_size: usize,
}

#[allow(dead_code)]
impl<T: CanSample> Sampler<T> {
    pub fn length(&self) -> T {
        num::NumCast::from(self.samples.len()).unwrap()
    }

    pub fn sum(&self) -> T {
        self.samples.iter().fold(T::zero(), |a, b| a + *b)
    }

    pub fn avg(&self) -> T {
        if self.samples.is_empty() {
            self.length()
        } else {
            self.sum() / self.length()
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, s: T) {
        if self.samples.len() == self.max_size {
            self.samples.rotate_left(1);
            self.samples.truncate(self.samples.len() - 1);
        }

        self.samples.push(s);
    }

    pub fn max(&self) -> T {
        let v = self.samples.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
        *v.unwrap()
    }

    pub fn min(&self) -> T {
        let v = self.samples.iter().min_by(|a, b| a.partial_cmp(b).unwrap());
        *v.unwrap()
    }
}

impl<T : CanSample> Default for Sampler<T> {
    fn default() -> Self {
        let mut samples = Vec::new();
        let max_size = 60;
        samples.reserve_exact(max_size);
        Self { samples, max_size }

    }
}
