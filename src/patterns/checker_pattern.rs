use crate::consts::BINCODE_CONFIG;
use crate::patterns::pattern::Pattern;
use crate::primitives::{transformations, Transformation};
use crate::primitives::{Color, Point};
use bincode::Encode;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct CheckerPattern {
    color_a: Color,
    color_b: Color,
    transformation: Transformation,
}

impl CheckerPattern {
    const PATTERN_IDENTIFIER: &'static [u8] = b"CheckerPattern";

    pub const fn new(color_a: Color, color_b: Color) -> Self {
        return Self {
            color_a,
            color_b,
            transformation: transformations::IDENTITY,
        };
    }
}

impl Pattern for CheckerPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = (point.x.floor() + point.y.floor() + point.z.floor()) as i64;
        return if distance % 2 == 0 {
            self.color_a
        } else {
            self.color_b
        };
    }

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation = transformation;
    }

    fn encoded(&self) -> Vec<u8> {
        let mut encoded = Self::PATTERN_IDENTIFIER.to_vec();
        encoded.extend(bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap());
        return encoded;
    }
}

impl Display for CheckerPattern {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("CheckerPattern")
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
    fn checker_pattern_repeats_in_x() {
        let pattern = CheckerPattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0.99, 0, 0)), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(1.01, 0, 0)), Color::BLACK);
    }

    #[test]
    fn checker_pattern_repeats_in_y() {
        let pattern = CheckerPattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0, 0.99, 0)), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0, 1.01, 0)), Color::BLACK);
    }

    #[test]
    fn checker_pattern_repeats_in_z() {
        let pattern = CheckerPattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0, 0, 0.99)), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0, 0, 1.01)), Color::BLACK);
    }
}
