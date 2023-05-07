use std::fmt::{Display, Formatter};

use crate::patterns::Pattern;
use crate::primitives::{Color, Point};
use crate::primitives::{Transformation, transformations};

#[derive(Clone, Debug, PartialEq)]
pub struct GradientPattern {
    color_a: Color,
    color_b: Color,
    transformation: Transformation,
}

impl GradientPattern {
    pub fn new(color_a: Color, color_b: Color) -> GradientPattern {
        return GradientPattern { color_a, color_b, transformation: transformations::IDENTITY };
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

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation = transformation;
    }
}

impl Display for GradientPattern {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("GradientPattern")
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
    fn gradient_interpolates_between_colors() {
        let pattern = GradientPattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0, 0, 0)), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0.25, 0, 0)), Color::new(0.75, 0.75, 0.75));
        assert_eq!(pattern.color_at(&Point::new(0.5, 0, 0)), Color::new(0.5, 0.5, 0.5));
        assert_eq!(pattern.color_at(&Point::new(0.75, 0, 0)), Color::new(0.25, 0.25, 0.25));
        assert_eq!(pattern.color_at(&Point::new(1, 0, 0)), Color::BLACK);
    }
}
