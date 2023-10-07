use crate::consts::BINCODE_CONFIG;
use crate::patterns::Pattern;
use crate::primitives::{transformations, Transformation};
use crate::primitives::{Color, Point};
use bincode::Encode;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct GradientPattern {
    color_a: Color,
    color_b: Color,
    pub transformation: Transformation,
}

impl GradientPattern {
    const PATTERN_IDENTIFIER: &'static [u8] = b"GradientPattern";

    pub const fn new(color_a: Color, color_b: Color) -> Self {
        return Self {
            color_a,
            color_b,
            transformation: transformations::IDENTITY,
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

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn encoded(&self) -> Vec<u8> {
        let mut encoded = Self::PATTERN_IDENTIFIER.to_vec();
        encoded.extend(bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap());
        return encoded;
    }
}

impl Display for GradientPattern {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("GradientPattern")
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
