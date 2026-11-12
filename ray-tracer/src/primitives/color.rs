use crate::utils::CoarseEq;
use core::fmt::{Display, Formatter, Result};
use core::ops::{Add, Div, Mul, Sub};

/// Struct representing RGB values of a color
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Color {
    pub const MIN_COLOR_VALUE: f64 = 0.0;

    pub const MAX_COLOR_VALUE: f64 = 1.0;

    pub const WHITE: Self = Self {
        red: Self::MAX_COLOR_VALUE,
        green: Self::MAX_COLOR_VALUE,
        blue: Self::MAX_COLOR_VALUE,
    };

    pub const BLACK: Self = Self {
        red: Self::MIN_COLOR_VALUE,
        green: Self::MIN_COLOR_VALUE,
        blue: Self::MIN_COLOR_VALUE,
    };

    pub const RED: Self = Self {
        red: Self::MAX_COLOR_VALUE,
        green: Self::MIN_COLOR_VALUE,
        blue: Self::MIN_COLOR_VALUE,
    };

    pub const GREEN: Self = Self {
        red: Self::MIN_COLOR_VALUE,
        green: Self::MAX_COLOR_VALUE,
        blue: Self::MIN_COLOR_VALUE,
    };

    pub const BLUE: Self = Self {
        red: Self::MIN_COLOR_VALUE,
        green: Self::MIN_COLOR_VALUE,
        blue: Self::MAX_COLOR_VALUE,
    };

    /// Creates new instance of [Color]
    /// # Examples
    /// ```
    /// use ray_tracer::primitives::Color;
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
    /// use ray_tracer::primitives::Color;
    ///
    /// let color = Color::new(1.0, 0.5, 0.0);
    ///
    ///  assert_eq!(color.channels(), [1.0, 0.5, 0.0]);
    /// ```
    pub const fn channels(&self) -> [f64; 3] {
        return [self.red, self.green, self.blue];
    }

    pub fn map(&self, f: impl Fn(f64) -> f64) -> Self {
        return Self {
            red: f(self.red),
            green: f(self.green),
            blue: f(self.blue),
        };
    }

    pub fn clamped(&self) -> Color {
        return self.map(Self::clamp);
    }

    fn clamp(color_value: f64) -> f64 {
        return color_value.clamp(Self::MIN_COLOR_VALUE, Self::MAX_COLOR_VALUE);
    }
}

impl Default for Color {
    fn default() -> Self {
        return Color::BLACK;
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return f
            .debug_struct("Color")
            .field("red", &self.red)
            .field("green", &self.green)
            .field("blue", &self.blue)
            .finish();
    }
}

impl CoarseEq for Color {
    fn coarse_eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || self.red.coarse_eq(&other.red)
                && self.green.coarse_eq(&other.green)
                && self.blue.coarse_eq(&other.blue);
    }
}

impl Add for Color {
    type Output = Color;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        return Self::Output::new(
            self.red + rhs.red,
            self.green + rhs.green,
            self.blue + rhs.blue,
        );
    }
}

impl Sub for Color {
    type Output = Color;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        return Self::Output::new(
            self.red - rhs.red,
            self.green - rhs.green,
            self.blue - rhs.blue,
        );
    }
}

impl Mul for Color {
    type Output = Color;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return Self::Output::new(
            self.red * rhs.red,
            self.green * rhs.green,
            self.blue * rhs.blue,
        );
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        return self.map(|value| value * rhs);
    }
}

impl Div<f64> for Color {
    type Output = Color;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        return self.map(|value| value / rhs);
    }
}

impl<T: Into<f64>> From<[T; 3]> for Color {
    fn from([x, y, z]: [T; 3]) -> Self {
        return Self::new(x, y, z);
    }
}

impl<T: Into<f64>> From<[T; 4]> for Color {
    fn from([x, y, z, ..]: [T; 4]) -> Self {
        return Self::new(x, y, z);
    }
}

impl From<Color> for [f64; 3] {
    fn from(color: Color) -> Self {
        return color.channels();
    }
}

impl<T: Into<f64>> From<(T, T, T)> for Color {
    fn from((x, y, z): (T, T, T)) -> Self {
        return Self::new(x, y, z);
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
        assert!((color_1 - color_2).coarse_eq(&Color::new(0.2, 0.5, 0.5)));
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
        assert!((color_1 * color_2).coarse_eq(&Color::new(0.9, 0.2, 0.04)));
    }
}
