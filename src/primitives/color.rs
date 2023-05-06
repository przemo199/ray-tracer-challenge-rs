use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul, Sub};

use crate::utils::CloseEnough;

/// Struct representing RGB values of a color
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Color {
    pub const WHITE: Color = Color { red: 1.0, green: 1.0, blue: 1.0 };

    pub const BLACK: Color = Color { red: 0.0, green: 0.0, blue: 0.0 };

    pub const RED: Color = Color { red: 1.0, green: 0.0, blue: 0.0 };

    pub const GREEN: Color = Color { red: 0.0, green: 1.0, blue: 0.0 };

    pub const BLUE: Color = Color { red: 0.0, green: 0.0, blue: 1.0 };

    /// Creates new instance of struct [Color]
    /// # Examples
    /// ```
    ///     use raytracer::primitives::Color;
    ///     let color = Color::new(1.0, 0.5, 0.0);
    ///
    ///     assert_eq!(color.red, 1.0);
    ///     assert_eq!(color.green, 0.5);
    ///     assert_eq!(color.blue, 0.0);
    /// ```
    pub fn new(red: f64, green: f64, blue: f64) -> Color {
        return Color { red, green, blue };
    }

    /// Returns array of RGB values representing color
    /// # Examples
    /// ```
    ///     use raytracer::primitives::Color;
    ///     let color = Color::new(1.0, 0.5, 0.0);
    ///
    ///     assert_eq!(color.get_channels(), [1.0, 0.5, 0.0]);
    /// ```
    pub fn get_channels(&self) -> [f64; 3] {
        return [self.red, self.green, self.blue];
    }
}

impl Default for Color {
    fn default() -> Color {
        return Color::BLACK;
    }
}

impl Display for Color {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("Color")
            .field("red", &self.red)
            .field("green", &self.green)
            .field("blue", &self.blue)
            .finish();
    }
}

impl PartialEq for Color {
    fn eq(&self, rhs: &Color) -> bool {
        return self.red.close_enough(rhs.red) &&
            self.green.close_enough(rhs.green) &&
            self.blue.close_enough(rhs.blue);
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        return Color::new(self.red + rhs.red, self.green + rhs.green, self.blue + rhs.blue);
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Color) -> Self::Output {
        return Color::new(self.red - rhs.red, self.green - rhs.green, self.blue - rhs.blue);
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        return Color::new(self.red * rhs.red, self.green * rhs.green, self.blue * rhs.blue);
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        return Color::new(self.red * rhs, self.green * rhs, self.blue * rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_exist() {
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0));
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0));
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0));
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0));
        assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn new_color() {
        let color = Color::new(-0.5, 0.4, 1.7);
        assert_eq!(color.red, -0.5);
        assert_eq!(color.green, 0.4);
        assert_eq!(color.blue, 1.7);
    }

    #[test]
    fn add_color() {
        let color_1 = Color::new(0.9, 0.6, 0.75);
        let color_2 = Color::new(0.7, 0.1, 0.25);
        assert_eq!(color_1 + color_2, Color::new(1.6, 0.7, 1.0));
    }

    #[test]
    fn sub_color() {
        let color_1 = Color::new(0.9, 0.6, 0.75);
        let color_2 = Color::new(0.7, 0.1, 0.25);
        assert_eq!(color_1 - color_2, Color::new(0.2, 0.5, 0.5));
    }

    #[test]
    fn mul_color_by_scalar() {
        let color = Color::new(0.2, 0.3, 0.4);
        assert_eq!(color * 2.0, Color::new(0.4, 0.6, 0.8));
    }

    #[test]
    fn mul_color_by_color() {
        let color_1 = Color::new(1.0, 0.2, 0.4);
        let color_2 = Color::new(0.9, 1.0, 0.1);
        assert_eq!(color_1 * color_2, Color::new(0.9, 0.2, 0.04));
    }
}
