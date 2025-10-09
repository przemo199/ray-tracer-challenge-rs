#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Coordinates {
    pub u: f64,
    pub v: f64,
}

impl Coordinates {
    pub fn new(u: f64, v: f64) -> Self {
        return Self { u, v };
    }
}
