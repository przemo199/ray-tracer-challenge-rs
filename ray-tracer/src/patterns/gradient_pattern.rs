use crate::patterns::Pattern;
use crate::primitives::{Color, Point, Transformation};
use crate::shapes::Transform;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct GradientPattern {
    color_a: Color,
    color_b: Color,
    transformation_inverse: Transformation,
}

impl GradientPattern {
    pub const fn new(color_a: Color, color_b: Color) -> Self {
        return Self {
            color_a,
            color_b,
            transformation_inverse: Transformation::IDENTITY,
        };
    }
}

impl Pattern for GradientPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = self.color_b - self.color_a;
        let mut fraction = point.x.fract().abs();
        if point.x as i64 % 2 != 0 {
            fraction = 1.0 - fraction;
        }
        return self.color_a + (distance * fraction);
    }
}

impl Transform for GradientPattern {
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

impl Display for GradientPattern {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        return formatter
            .debug_struct("GradientPattern")
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
    fn gradient_interpolates_between_colors() {
        let pattern = GradientPattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(
            pattern.color_at(&Point::new(0.25, 0, 0)),
            Color::new(0.75, 0.75, 0.75)
        );
        assert_eq!(
            pattern.color_at(&Point::new(0.5, 0, 0)),
            Color::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            pattern.color_at(&Point::new(0.75, 0, 0)),
            Color::new(0.25, 0.25, 0.25)
        );
        assert_eq!(pattern.color_at(&Point::new(1, 0, 0)), Color::BLACK);
    }
}
