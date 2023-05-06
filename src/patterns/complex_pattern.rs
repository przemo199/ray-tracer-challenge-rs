use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::patterns::Pattern;
use crate::primitives::{Color, Point};
use crate::primitives::{Transformation, transformations};

#[derive(Clone, Debug)]
pub struct ComplexPattern {
    pattern_a: Arc<dyn Pattern>,
    pattern_b: Arc<dyn Pattern>,
    transformation: Transformation,
}

impl ComplexPattern {
    pub fn new(pattern_a: Arc<dyn Pattern>, pattern_b: Arc<dyn Pattern>) -> ComplexPattern {
        return ComplexPattern { pattern_a, pattern_b, transformation: transformations::IDENTITY };
    }
}

impl Pattern for ComplexPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = point.x.floor() as i64;
        return if distance % 2 == 0 { self.pattern_a.color_at(point) } else { self.pattern_b.color_at(point) };
    }

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation = transformation;
    }
}

impl Display for ComplexPattern {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("ComplexPattern")
            .field("pattern_a", &self.pattern_a)
            .field("pattern_b", &self.pattern_b)
            .field("transformation", &self.transformation)
            .finish();
    }
}
