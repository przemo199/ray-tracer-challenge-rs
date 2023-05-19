//! Collection of basic types and methods useful for modelling world

pub use color::Color;
pub use light::Light;
pub use matrix::Matrix;
pub use point::Point;
pub use transformations::Transformation;
pub use vector::Vector;

mod color;
mod light;
mod matrix;
mod point;
mod vector;
pub mod transformations;

