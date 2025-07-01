use crate::patterns::Pattern;
use crate::primitives::{Color, Point, Transformation};
use crate::shapes::Transform;
use crate::utils::Squared;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct RingPattern {
    color_a: Color,
    color_b: Color,
    transformation_inverse: Transformation,
}

impl RingPattern {
    pub const fn new(color_a: Color, color_b: Color) -> Self {
        return Self {
            color_a,
            color_b,
            transformation_inverse: Transformation::IDENTITY,
        };
    }
}

impl Pattern for RingPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = (point.x.squared() + point.z.squared()).sqrt().floor() as i64;
        return if distance % 2 == 0 {
            self.color_a
        } else {
            self.color_b
        };
    }
}

impl Transform for RingPattern {
    fn transformation(&self) -> Transformation {
        return self.transformation_inverse.inverse();
    }

    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation_inverse = transformation.inverse();
    }

    fn transformation_inverse(&self) -> Transformation {
        return self.transformation_inverse;
    }

    fn set_transformation_inverse(&mut self, transformation: Transformation) {
        self.transformation_inverse = transformation;
    }
}

impl Display for RingPattern {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        return formatter
            .debug_struct("CircularPattern")
            .field("color_a", &self.color_a)
            .field("color_b", &self.color_b)
            .field("transformation", &self.transformation())
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_pattern_extends_in_x_and_z() {
        let pattern = RingPattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(1, 0, 0)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0, 0, 1)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0.708, 0, 0.708)), Color::BLACK);
    }
}
