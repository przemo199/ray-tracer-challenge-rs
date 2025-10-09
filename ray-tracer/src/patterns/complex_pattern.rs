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
    fn color_at(&self, point: &Point) -> Color {
        let distance = point.x.floor() as i64;
        return if distance % 2 == 0 {
            self.pattern_a.color_at(point)
        } else {
            self.pattern_b.color_at(point)
        };
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
