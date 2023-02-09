use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul, Sub};
use crate::consts::EPSILON;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Color {
    pub fn new(red: f64, green: f64, blue: f64) -> Color {
        return Color { red, green, blue };
    }

    pub fn black() -> Color {
        return Color::new(0.0, 0.0, 0.0);
    }

    pub fn red() -> Color {
        return Color::new(1.0, 0.0, 0.0);
    }

    pub fn green() -> Color {
        return Color::new(0.0, 1.0, 0.0);
    }

    pub fn blue() -> Color {
        return Color::new(0.0, 0.0, 1.0);
    }

    pub fn white() -> Color {
        return Color::new(1.0, 1.0, 1.0);
    }

    pub fn get_colors(&self) -> [f64; 3] {
        return [self.red, self.green, self.blue];
    }
}

impl Default for Color {
    fn default() -> Color {
        return Color::black();
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
        return (self.red - rhs.red).abs() < EPSILON &&
            (self.green - rhs.green).abs() < EPSILON &&
            (self.blue - rhs.blue).abs() < EPSILON;
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
        assert_eq!(Color::white(), Color::new(1.0, 1.0, 1.0));
        assert_eq!(Color::black(), Color::new(0.0, 0.0, 0.0));
        assert_eq!(Color::red(), Color::new(1.0, 0.0, 0.0));
        assert_eq!(Color::green(), Color::new(0.0, 1.0, 0.0));
        assert_eq!(Color::blue(), Color::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn new_color() {
        let color1 = Color::new(-0.5, 0.4, 1.7);
        assert_eq!(color1.red, -0.5);
        assert_eq!(color1.green, 0.4);
        assert_eq!(color1.blue, 1.7);
    }

    #[test]
    fn add_color() {
        let color1 = Color::new(0.9, 0.6, 0.75);
        let color2 = Color::new(0.7, 0.1, 0.25);
        assert_eq!(color1 + color2, Color::new(1.6, 0.7, 1.0));
    }

    #[test]
    fn sub_color() {
        let color1 = Color::new(0.9, 0.6, 0.75);
        let color2 = Color::new(0.7, 0.1, 0.25);
        assert_eq!(color1 - color2, Color::new(0.2, 0.5, 0.5));
    }

    #[test]
    fn mul_color_by_scalar() {
        let color1 = Color::new(0.2, 0.3, 0.4);
        assert_eq!(color1 * 2.0, Color::new(0.4, 0.6, 0.8));
    }

    #[test]
    fn mul_color_by_color() {
        let color1 = Color::new(1.0, 0.2, 0.4);
        let color2 = Color::new(0.9, 1.0, 0.1);
        assert_eq!(color1 * color2, Color::new(0.9, 0.2, 0.04));
    }
}
