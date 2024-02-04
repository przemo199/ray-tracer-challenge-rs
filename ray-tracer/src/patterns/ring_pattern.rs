use crate::consts::BINCODE_CONFIG;
use crate::patterns::Pattern;
use crate::primitives::{transformations, Transformation};
use crate::primitives::{Color, Point};
use crate::utils::Squared;
use bincode::Encode;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct RingPattern {
    color_a: Color,
    color_b: Color,
    pub transformation: Transformation,
}

impl RingPattern {
    const PATTERN_IDENTIFIER: &'static [u8] = b"RingPattern";

    pub const fn new(color_a: Color, color_b: Color) -> Self {
        return Self {
            color_a,
            color_b,
            transformation: transformations::IDENTITY,
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

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn encoded(&self) -> Vec<u8> {
        let mut encoded = Self::PATTERN_IDENTIFIER.to_vec();
        encoded.extend(bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap());
        return encoded;
    }
}

impl Display for RingPattern {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("CircularPattern")
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
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(1, 0, 0)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0, 0, 1)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0.708, 0, 0.708)), Color::BLACK);
    }
}
