use std::fmt::{Display, Formatter};

use crate::patterns::Pattern;
use crate::primitives::{Color, Point};
use crate::primitives::{Transformation, transformations};

#[derive(Clone, Debug, PartialEq)]
pub struct RingPattern {
    color_a: Color,
    color_b: Color,
    transformation: Transformation,
}

impl RingPattern {
    pub fn new(color_a: Color, color_b: Color) -> RingPattern {
        return RingPattern { color_a, color_b, transformation: transformations::IDENTITY };
    }
}

impl Pattern for RingPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = (point.x * point.x + point.z * point.z).sqrt().floor() as i64;
        return if distance % 2 == 0 { self.color_a } else { self.color_b };
    }

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation = transformation;
    }
}

impl Display for RingPattern {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("CircularPattern")
            .field("color_a", &self.color_a)
            .field("color_b", &self.color_b)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_pattern_extends_in_x_and_z() {
        let pattern = RingPattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(1.0, 0.0, 0.0)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 1.0)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0.708, 0.0, 0.708)), Color::BLACK);
    }
}
