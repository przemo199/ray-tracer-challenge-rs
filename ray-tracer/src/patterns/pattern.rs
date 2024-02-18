use crate::consts::BINCODE_CONFIG;
use crate::primitives::{transformations, Transformation};
use crate::primitives::{Color, Point};
use crate::shapes::Shape;
use bincode::enc::write::Writer;
use bincode::enc::Encoder;
use bincode::error::EncodeError;
use bincode::Encode;
use core::fmt::{Debug, Display, Formatter};

pub trait Pattern: Debug + Display + Send + Sync {
    fn color_at(&self, point: &Point) -> Color;

    #[inline]
    fn color_at_shape(&self, shape: &dyn Shape, point: &Point) -> Color {
        let object_point = shape.transformation_inverse() * *point;
        let pattern_point = self.transformation_inverse() * object_point;
        return self.color_at(&pattern_point);
    }

    fn set_transformation(&mut self, transformation: Transformation);

    fn transformation(&self) -> Transformation;

    fn transformation_inverse(&self) -> Transformation;

    fn encoded(&self) -> Vec<u8>;
}

impl PartialEq for dyn Pattern {
    fn eq(&self, rhs: &dyn Pattern) -> bool {
        return self.encoded() == rhs.encoded();
    }
}

impl Encode for dyn Pattern {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        return encoder.writer().write(&self.encoded());
    }
}

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct TestPattern {
    transformation_inverse: Transformation,
}

impl TestPattern {
    const PATTERN_IDENTIFIER: &'static [u8] = b"TestPattern";

    pub const fn new() -> TestPattern {
        return Self {
            transformation_inverse: transformations::IDENTITY,
        };
    }
}

impl Pattern for TestPattern {
    fn color_at(&self, point: &Point) -> Color {
        return Color::new(point.x, point.y, point.z);
    }

    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation_inverse = transformation.inverse();
    }

    fn transformation(&self) -> Transformation {
        return self.transformation_inverse.inverse();
    }

    fn transformation_inverse(&self) -> Transformation {
        return self.transformation_inverse;
    }

    fn encoded(&self) -> Vec<u8> {
        let mut encoded = Self::PATTERN_IDENTIFIER.to_vec();
        encoded.extend(bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap());
        return encoded;
    }
}

impl Display for TestPattern {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("TestPattern")
            .field("transformation", &self.transformation_inverse)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{Color, Point};
    use crate::shapes::Sphere;

    #[test]
    fn default_test_pattern_transformation() {
        let pattern = TestPattern::new();
        assert_eq!(pattern.transformation_inverse, transformations::IDENTITY);
    }

    #[test]
    fn assigning_test_pattern_transformation() {
        let mut pattern = TestPattern::new();
        pattern.transformation_inverse = transformations::translation(1, 2, 3);
        assert_eq!(
            pattern.transformation_inverse,
            transformations::translation(1, 2, 3)
        );
    }

    #[test]
    fn test_pattern_with_object_transformation() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::scaling(2, 2, 2));
        let pattern = TestPattern::new();
        let color = pattern.color_at_shape(&sphere, &Point::new(2, 3, 4));
        assert_eq!(color, Color::new(1, 1.5, 2));
    }

    #[test]
    fn test_pattern_with_pattern_transformation() {
        let sphere = Sphere::default();
        let mut pattern = TestPattern::new();
        pattern.set_transformation(transformations::scaling(2, 2, 2));
        let color = pattern.color_at_shape(&sphere, &Point::new(2, 3, 4));
        assert_eq!(color, Color::new(1, 1.5, 2));
    }

    #[test]
    fn test_pattern_with_pattern_and_object_transformations() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::scaling(2, 2, 2));
        let mut pattern = TestPattern::new();
        pattern.set_transformation(transformations::translation(0.5, 1, 1.5));
        let color = pattern.color_at_shape(&sphere, &Point::new(2.5, 3, 3.5));
        assert_eq!(color, Color::new(0.75, 0.5, 0.25));
    }
}
