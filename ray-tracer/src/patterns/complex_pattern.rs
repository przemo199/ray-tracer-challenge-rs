use crate::patterns::Pattern;
use crate::primitives::{Color, Point, Transformation};
use crate::shapes::Transform;
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

impl Transform for ComplexPattern {
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

impl PartialEq for ComplexPattern {
    fn eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs)
            || self.pattern_a.as_ref() == rhs.pattern_a.as_ref()
                && self.pattern_b.as_ref() == rhs.pattern_b.as_ref()
                && self.transformation_inverse == rhs.transformation_inverse;
    }
}

impl Display for ComplexPattern {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        return formatter
            .debug_struct("ComplexPattern")
            .field("pattern_a", &self.pattern_a)
            .field("pattern_b", &self.pattern_b)
            .field("transformation", &self.transformation())
            .finish();
    }
}
