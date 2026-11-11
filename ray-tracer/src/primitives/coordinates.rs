/// Texture coordinates: `u` horizontal and `v` vertical, normally in `0..=1`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Coordinates {
    pub u: f64,
    pub v: f64,
}

impl Coordinates {
    pub fn new(u: impl Into<f64>, v: impl Into<f64>) -> Self {
        return Self {
            u: u.into(),
            v: v.into(),
        };
    }
}
