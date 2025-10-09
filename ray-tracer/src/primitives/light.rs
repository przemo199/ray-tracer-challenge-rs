use crate::primitives::{Color, Point};
use crate::utils::CoarseEq;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub position: Point,
    pub intensity: Color,
}

impl Light {
    /// Creates new instance of struct [Light]
    /// # Examples
    /// ```
    /// use ray_tracer::primitives::{Color, Light, Point};
    ///
    /// let light = Light::new(Point::default(), Color::BLACK);
    ///
    /// assert_eq!(light.position, Point::default());
    /// assert_eq!(light.intensity, Color::BLACK);
    /// ```
    pub const fn new(position: Point, intensity: Color) -> Self {
        return Self {
            position,
            intensity,
        };
    }
}

impl CoarseEq for Light {
    fn coarse_eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || self.position.coarse_eq(&other.position)
                && self.intensity.coarse_eq(&other.intensity);
    }
}

impl Default for Light {
    fn default() -> Self {
        return Self::new(Point::new(-10, 10, -10), Color::WHITE);
    }
}

impl Display for Light {
    fn fmt(&self, f: &mut Formatter) -> Result {
        return f
            .debug_struct("Light")
            .field("position", &self.position)
            .field("intensity", &self.intensity)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::{Color, Light, Point};

    #[test]
    fn light_has_position_and_intensity() {
        let position = Point::ORIGIN;
        let intensity = Color::WHITE;
        let light = Light::new(position, intensity);
        assert_eq!(light.position, position);
        assert_eq!(light.intensity, intensity);
    }
}
