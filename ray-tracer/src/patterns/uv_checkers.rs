use crate::patterns::UvPattern;
use crate::primitives::{Color, Coordinates};

/// Checkers in texture space: `width` × `height` squares across the `0..1` range of `u` and `v`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UvCheckers {
    pub width: u32,
    pub height: u32,
    pub color_a: Color,
    pub color_b: Color,
}

impl UvCheckers {
    pub const fn new(width: u32, height: u32, color_a: Color, color_b: Color) -> Self {
        return Self {
            width,
            height,
            color_a,
            color_b,
        };
    }
}

impl UvPattern for UvCheckers {
    fn color_at(&self, coordinates: &Coordinates) -> Color {
        // `floor`, not a cast (which rounds toward zero), so squares stay regular below zero
        let u = (coordinates.u * f64::from(self.width)).floor() as i64;
        let v = (coordinates.v * f64::from(self.height)).floor() as i64;
        return if (u + v).rem_euclid(2) == 0 {
            self.color_a
        } else {
            self.color_b
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0.0, 0.0, Color::BLACK)]
    #[case(0.5, 0.0, Color::WHITE)]
    #[case(0.0, 0.5, Color::WHITE)]
    #[case(0.5, 0.5, Color::BLACK)]
    #[case(1.0, 1.0, Color::BLACK)]
    fn checker_pattern_in_2d(#[case] u: f64, #[case] v: f64, #[case] expected: Color) {
        let checkers = UvCheckers::new(2, 2, Color::BLACK, Color::WHITE);
        assert_eq!(checkers.color_at(&Coordinates::new(u, v)), expected);
    }

    #[test]
    fn checker_pattern_uses_separate_width_and_height() {
        let checkers = UvCheckers::new(4, 2, Color::BLACK, Color::WHITE);
        assert_eq!(checkers.color_at(&Coordinates::new(0, 0.4)), Color::BLACK);
        assert_eq!(checkers.color_at(&Coordinates::new(0, 0.6)), Color::WHITE);
        assert_eq!(checkers.color_at(&Coordinates::new(0.3, 0)), Color::WHITE);
    }

    #[test]
    fn checkers_stay_regular_below_zero() {
        // a cast would put -0.25 in the same square as 0.25
        let checkers = UvCheckers::new(2, 2, Color::BLACK, Color::WHITE);
        assert_eq!(checkers.color_at(&Coordinates::new(0.25, 0)), Color::BLACK);
        assert_eq!(checkers.color_at(&Coordinates::new(-0.25, 0)), Color::WHITE);
        assert_eq!(checkers.color_at(&Coordinates::new(-0.75, 0)), Color::BLACK);
    }
}
