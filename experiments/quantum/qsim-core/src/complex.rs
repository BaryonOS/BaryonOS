#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub fn probability(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}
