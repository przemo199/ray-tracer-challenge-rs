use crate::impl_transform_for;
use crate::patterns::ColorProvider;
use crate::primitives::{Color, Point, Transformation};
use core::fmt::{Debug, Display, Formatter};

pub trait Pattern: Transform + ColorProvider + ClonePattern {
    fn color_at(&self, point: &Point) -> Color;
}

/// Makes `Box<dyn Pattern>` cloneable (for nested patterns): `Clone` itself can't be called
/// through `dyn`. Implemented automatically for every `Clone` pattern.
pub trait ClonePattern {
    fn clone_box(&self) -> Box<dyn Pattern>;
}

impl<T: Pattern + Clone + 'static> ClonePattern for T {
    fn clone_box(&self) -> Box<dyn Pattern> {
        return Box::new(self.clone());
    }
}

impl Clone for Box<dyn Pattern> {
    fn clone(&self) -> Self {
        // `dyn Pattern` also has `CloneColorProvider::clone_box`; this one returns a `dyn Pattern`
        return ClonePattern::clone_box(self.as_ref());
    }
}

#[macro_export]
macro_rules! impl_color_provider_for_pattern {
    ($type:ty) => {
        use $crate::shapes::Transform;

        impl $crate::patterns::color_provider::ColorProvider for $type {
            fn color_at_shape(
                &self,
                shape: &dyn $crate::shapes::Shape,
                point: &$crate::primitives::Point,
            ) -> $crate::primitives::Color {
                let object_point = shape.transformation_inverse() * *point;
                let pattern_point = self.transformation_inverse() * object_point;
                return self.color_at(&pattern_point);
            }
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestPattern {
    transformation_inverse: Transformation,
}

impl TestPattern {
    pub const fn new() -> TestPattern {
        return Self {
            transformation_inverse: Transformation::IDENTITY,
        };
    }
}

impl_transform_for!(TestPattern);

impl_color_provider_for_pattern!(TestPattern);

impl Pattern for TestPattern {
    fn color_at(&self, point: &Point) -> Color {
        return Color::new(point.x, point.y, point.z);
    }
}

impl Display for TestPattern {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        return f
            .debug_struct("TestPattern")
            .field("transformation", &self.transformation())
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{Color, Point, transformations};
    use crate::shapes::{Sphere, Transform};

    #[test]
    fn default_test_pattern_transformation() {
        let pattern = TestPattern::new();
        assert_eq!(pattern.transformation_inverse, Transformation::IDENTITY);
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
        let color =
            (Box::new(pattern) as Box<dyn Pattern>).color_at_shape(&sphere, &Point::new(2, 3, 4));
        assert_eq!(color, Color::new(1, 1.5, 2));
    }

    #[test]
    fn test_pattern_with_pattern_transformation() {
        let sphere = Sphere::default();
        let mut pattern = TestPattern::new();
        pattern.set_transformation(transformations::scaling(2, 2, 2));
        let color =
            (Box::new(pattern) as Box<dyn Pattern>).color_at_shape(&sphere, &Point::new(2, 3, 4));
        assert_eq!(color, Color::new(1, 1.5, 2));
    }

    #[test]
    fn test_pattern_with_pattern_and_object_transformations() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::scaling(2, 2, 2));
        let mut pattern = TestPattern::new();
        pattern.set_transformation(transformations::translation(0.5, 1, 1.5));
        let color = (Box::new(pattern) as Box<dyn Pattern>)
            .color_at_shape(&sphere, &Point::new(2.5, 3, 3.5));
        assert_eq!(color, Color::new(0.75, 0.5, 0.25));
    }
}
