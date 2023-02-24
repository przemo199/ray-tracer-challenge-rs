use crate::color::Color;
use crate::point::Point;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub position: Point,
    pub intensity: Color,
}

impl Light {
    pub fn new(position: Point, intensity: Color) -> Light {
        return Light { position, intensity };
    }
}

impl Default for Light {
    fn default() -> Self {
        return Light::new(Point::new(-10.0, 10.0, -10.0), Color::white());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_has_position_and_intensity() {
        let position = Point::new(0.0, 0.0, 0.0);
        let intensity = Color::white();
        let light = Light::new(position, intensity);
        assert_eq!(light.position, position);
        assert_eq!(light.intensity, intensity);
    }
}
