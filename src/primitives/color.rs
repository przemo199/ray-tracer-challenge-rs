use crate::utils::CoarseEq;
use bincode::Encode;
use core::fmt::{Display, Formatter};
use core::ops::{Add, Mul, Sub};

/// Struct representing RGB values of a color
#[derive(Clone, Copy, Debug, Encode)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Color {
    pub const WHITE: Self = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };

    pub const BLACK: Self = Self {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    pub const RED: Self = Self {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
    };

    pub const GREEN: Self = Self {
        red: 0.0,
        green: 1.0,
        blue: 0.0,
    };

    pub const BLUE: Self = Self {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
    };

    /// Creates new instance of struct [Color]
    /// # Examples
    /// ```
    /// use raytracer::primitives::Color;
    ///
    /// let color = Color::new(1, 0.5, 0);
    ///
    /// assert_eq!(color.red, 1.0);
    /// assert_eq!(color.green, 0.5);
    /// assert_eq!(color.blue, 0.0);
    /// ```
    pub fn new(red: impl Into<f64>, green: impl Into<f64>, blue: impl Into<f64>) -> Self {
        return Self {
            red: red.into(),
            green: green.into(),
            blue: blue.into(),
        };
    }

    /// Returns array of RGB values representing color
    /// # Examples
    /// ```
    /// use raytracer::primitives::Color;
    ///
    /// let color = Color::new(1.0, 0.5, 0.0);
    ///
    ///  assert_eq!(color.get_channels(), [1.0, 0.5, 0.0]);
    /// ```
    pub const fn get_channels(&self) -> [f64; 3] {
        return [self.red, self.green, self.blue];
    }
}

impl Default for Color {
    fn default() -> Self {
        return Color::BLACK;
    }
}

impl Display for Color {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("Color")
            .field("red", &self.red)
            .field("green", &self.green)
            .field("blue", &self.blue)
            .finish();
    }
}

impl PartialEq for Color {
    fn eq(&self, rhs: &Self) -> bool {
        return self.red.coarse_eq(rhs.red)
            && self.green.coarse_eq(rhs.green)
            && self.blue.coarse_eq(rhs.blue);
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self::new(
            self.red + rhs.red,
            self.green + rhs.green,
            self.blue + rhs.blue,
        );
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Self::new(
            self.red - rhs.red,
            self.green - rhs.green,
            self.blue - rhs.blue,
        );
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        return Self::new(
            self.red * rhs.red,
            self.green * rhs.green,
            self.blue * rhs.blue,
        );
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        return Self::new(self.red * rhs, self.green * rhs, self.blue * rhs);
    }
}

impl<T: Into<f64>> From<[T; 3]> for Color {
    fn from(value: [T; 3]) -> Self {
        let [x, y, z] = value;
        return Self::new(x, y, z);
    }
}

impl<T: Into<f64>> From<[T; 4]> for Color {
    fn from(value: [T; 4]) -> Self {
        let [x, y, z, ..] = value;
        return Self::new(x, y, z);
    }
}

impl Into<[f64; 4]> for Color {
    fn into(self) -> [f64; 4] {
        return self.values();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_exist() {
        assert_eq!(Color::WHITE, Color::new(1, 1, 1));
        assert_eq!(Color::BLACK, Color::new(0, 0, 0));
        assert_eq!(Color::RED, Color::new(1, 0, 0));
        assert_eq!(Color::GREEN, Color::new(0, 1, 0));
        assert_eq!(Color::BLUE, Color::new(0, 0, 1));
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
        assert_eq!(color_1 + color_2, Color::new(1.6, 0.7, 1));
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
    fn mul_colors() {
        let color_1 = Color::new(1, 0.2, 0.4);
        let color_2 = Color::new(0.9, 1, 0.1);
        assert_eq!(color_1 * color_2, Color::new(0.9, 0.2, 0.04));
    }
}
