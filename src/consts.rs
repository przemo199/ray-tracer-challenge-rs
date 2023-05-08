use bincode::config::{Configuration, standard};

/// Value for calculating smallest difference between floats
pub const EPSILON: f64 = 0.00000008;

pub const PI: f64 = std::f64::consts::PI;

pub const MAX_REFLECTION_ITERATIONS: u8 = 5;

pub const BINCODE_CONFIG: Configuration = standard();
