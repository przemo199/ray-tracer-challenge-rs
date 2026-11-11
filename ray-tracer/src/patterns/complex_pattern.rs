use crate::patterns::{ColorProvider, Pattern};
use crate::primitives::{Color, Point, Transformation};
use crate::{impl_color_provider_for_pattern, impl_transform_for};
use core::fmt::{Display, Formatter, Result};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ComplexPattern {
    pattern_a: Arc<dyn Pattern>,
    pattern_b: Arc<dyn Pattern>,
    transformation_inverse: Transformation,
}

impl ComplexPattern {
    pub const fn new(pattern_a: Arc<dyn Pattern>, pattern_b: Arc<dyn Pattern>) -> Self {
        return Self {
            pattern_a,
            pattern_b,
            transformation_inverse: Transformation::IDENTITY,
        };
    }
}

impl_transform_for!(ComplexPattern);

impl_color_provider_for_pattern!(ComplexPattern);

impl Pattern for ComplexPattern {
    /// Alternates between the two patterns along x, evaluating each in its own pattern space
    /// so that sub-patterns can be transformed independently (the book's "nested patterns").
    fn color_at(&self, point: &Point) -> Color {
        let pattern = if (point.x.floor() as i64) % 2 == 0 {
            &self.pattern_a
        } else {
            &self.pattern_b
        };
        return pattern.color_at(&(pattern.transformation_inverse() * *point));
    }
}

impl PartialEq for ComplexPattern {
    fn eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || self.pattern_a.as_ref() as &dyn ColorProvider
                == other.pattern_a.as_ref() as &dyn ColorProvider
                && self.pattern_b.as_ref() as &dyn ColorProvider
                    == other.pattern_b.as_ref() as &dyn ColorProvider
                && self.transformation_inverse == other.transformation_inverse;
    }
}

impl Display for ComplexPattern {
    fn fmt(&self, f: &mut Formatter) -> Result {
        return f
            .debug_struct("ComplexPattern")
            .field("pattern_a", &self.pattern_a)
            .field("pattern_b", &self.pattern_b)
            .field("transformation", &self.transformation())
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::StripePattern;
    use crate::primitives::transformations;
    use crate::shapes::{Sphere, Transform};
    use rstest::rstest;

    const A: Color = Color::RED;
    const B: Color = Color::BLUE;

    fn red_or_blue() -> ComplexPattern {
        return ComplexPattern::new(
            Arc::new(StripePattern::new(A, A)),
            Arc::new(StripePattern::new(B, B)),
        );
    }

    #[rstest]
    #[case(0.0, A)]
    #[case(0.9, A)]
    #[case(1.0, B)]
    #[case(1.9, B)]
    #[case(2.0, A)]
    #[case(-0.1, B)]
    #[case(-1.0, B)]
    #[case(-1.1, A)]
    fn alternates_between_patterns_along_x(#[case] x: f64, #[case] expected: Color) {
        assert_eq!(red_or_blue().color_at(&Point::new(x, 0, 0)), expected);
    }

    #[test]
    fn is_constant_along_y_and_z() {
        let pattern = red_or_blue();
        assert_eq!(pattern.color_at(&Point::new(0.5, 7, -3)), A);
        assert_eq!(pattern.color_at(&Point::new(1.5, -7, 3)), B);
    }

    #[test]
    fn sub_pattern_transformations_apply() {
        let mut stripes = StripePattern::new(Color::WHITE, Color::BLACK);
        stripes.set_transformation(transformations::scaling(0.5, 1, 1));
        let pattern = ComplexPattern::new(Arc::new(stripes), Arc::new(StripePattern::new(B, B)));
        // x = 0.75 is in the first pattern; its half-width stripes put it in the second stripe
        assert_eq!(pattern.color_at(&Point::new(0.75, 0, 0)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(0.25, 0, 0)), Color::WHITE);
    }

    #[test]
    fn own_transformation_applies() {
        let mut pattern = red_or_blue();
        pattern.set_transformation(transformations::scaling(2, 1, 1));
        let color = pattern.color_at_shape(&Sphere::default(), &Point::new(1.5, 0, 0));
        assert_eq!(color, A);
    }
}
