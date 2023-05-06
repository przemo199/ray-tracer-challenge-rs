use std::fmt::{Display, Formatter};

use crate::primitives::{Color, Point};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub position: Point,
    pub intensity: Color,
}

impl Light {
    /// Creates new instance of struct [Light]
    /// # Examples
    /// ```
    ///     use raytracer::primitives::{Color, Light, Point};
    ///     let light = Light::new(Point::default(), Color::BLACK);
    ///
    ///     assert_eq!(light.position, Point::default());
    ///     assert_eq!(light.intensity, Color::BLACK);
    /// ```
    pub fn new(position: Point, intensity: Color) -> Light {
        return Light { position, intensity };
    }
}

impl Default for Light {
    fn default() -> Self {
        return Light::new(Point::new(-10.0, 10.0, -10.0), Color::WHITE);
    }
}

impl Display for Light {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("Light")
            .field("position", &self.position)
            .field("intensity", &self.intensity)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_has_position_and_intensity() {
        let position = Point::new(0.0, 0.0, 0.0);
        let intensity = Color::WHITE;
        let light = Light::new(position, intensity);
        assert_eq!(light.position, position);
        assert_eq!(light.intensity, intensity);
    }
}
