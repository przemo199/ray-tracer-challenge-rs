use crate::{Color, Tuple};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub position: Tuple,
    pub intensity: Color,
}

impl Light {
    pub fn new(position: Tuple, intensity: Color) -> Light {
        return Light { position, intensity };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_has_position_and_intensity() {
        let position = Tuple::point(0.0, 0.0, 0.0);
        let intensity = Color::new(1.0, 1.0, 1.0);
        let light = Light::new(position.clone(), intensity.clone());
        assert_eq!(light.position, position);
        assert_eq!(light.intensity, intensity);
    }
}
