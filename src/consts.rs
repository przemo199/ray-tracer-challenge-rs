use bincode::config::{standard, Configuration};

/// Value for calculating smallest difference between floats
pub const EPSILON: f64 = 0.00000008;

pub const PI: f64 = core::f64::consts::PI;

pub const MAX_REFLECTION_ITERATIONS: u8 = 6;

pub const BINCODE_CONFIG: Configuration = standard();

pub const PROGRESS_TEMPLATE: &str =
    "[{elapsed_precise}] {bar:50.white/gray}{percent}% {human_pos}/{human_len}";
