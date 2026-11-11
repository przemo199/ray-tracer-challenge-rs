use crate::patterns::UvPattern;
use crate::primitives::{Color, Coordinates};

/// A test pattern for checking a mapping's orientation: one color with a differently colored square
/// in each corner (each a fifth of the width and height). If the corners land where expected, `u`
/// and `v` run the right way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UvAlignCheck {
    pub main: Color,
    pub upper_left: Color,
    pub upper_right: Color,
    pub bottom_left: Color,
    pub bottom_right: Color,
}

impl UvAlignCheck {
    pub const fn new(
        main: Color,
        upper_left: Color,
        upper_right: Color,
        bottom_left: Color,
        bottom_right: Color,
    ) -> Self {
        return Self {
            main,
            upper_left,
            upper_right,
            bottom_left,
            bottom_right,
        };
    }
}

impl UvPattern for UvAlignCheck {
    fn color_at(&self, coordinates: &Coordinates) -> Color {
        let (u, v) = (coordinates.u, coordinates.v);
        let (left, right, upper, bottom) = (u < 0.2, u > 0.8, v > 0.8, v < 0.2);
        return match (upper, bottom, left, right) {
            (true, _, true, _) => self.upper_left,
            (true, _, _, true) => self.upper_right,
            (_, true, true, _) => self.bottom_left,
            (_, true, _, true) => self.bottom_right,
            _ => self.main,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const YELLOW: Color = Color {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
    };
    const CYAN: Color = Color {
        red: 0.0,
        green: 1.0,
        blue: 1.0,
    };

    #[rstest]
    #[case(0.5, 0.5, Color::WHITE)]
    #[case(0.1, 0.9, Color::RED)]
    #[case(0.9, 0.9, YELLOW)]
    #[case(0.1, 0.1, Color::GREEN)]
    #[case(0.9, 0.1, CYAN)]
    #[case(0.5, 0.9, Color::WHITE)]
    #[case(0.1, 0.5, Color::WHITE)]
    fn layout_of_the_align_check_pattern(#[case] u: f64, #[case] v: f64, #[case] expected: Color) {
        let pattern = UvAlignCheck::new(Color::WHITE, Color::RED, YELLOW, Color::GREEN, CYAN);
        assert_eq!(pattern.color_at(&Coordinates::new(u, v)), expected);
    }
}
