//! Collection of basic types and methods useful for modelling world

pub use color::Color;
pub use coordinates::Coordinates;
pub use light::Light;
pub use matrix::Matrix;
pub use point::Point;
pub use transformations::Transformation;
pub use vector::Vector;

mod color;
mod coordinates;
mod light;
mod matrix;
mod point;
pub mod transformations;
mod vector;
