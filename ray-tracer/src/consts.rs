use bincode::config::{Configuration, standard};

/// Value for calculating smallest difference between floats
pub const EPSILON: f64 = 0.00000008;

pub const MIN: f64 = f64::MIN;

pub const MAX: f64 = f64::MAX;

pub const PI: f64 = core::f64::consts::PI;

pub static BINCODE_CONFIG: Configuration = standard();
