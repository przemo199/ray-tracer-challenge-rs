use crate::consts::BINCODE_CONFIG;
use crate::patterns::Pattern;
use crate::primitives::{Color, Point, Transformation};
use crate::shapes::Transform;
use bincode::Encode;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct StripePattern {
    color_a: Color,
    color_b: Color,
    transformation_inverse: Transformation,
}

impl StripePattern {
    const PATTERN_IDENTIFIER: &'static [u8] = b"StripePattern";

    pub const fn new(color_a: Color, color_b: Color) -> Self {
        return Self {
            color_a,
            color_b,
            transformation_inverse: Transformation::IDENTITY,
        };
    }
}

impl Pattern for StripePattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = point.x.floor() as i64;
        return if distance % 2 == 0 {
            self.color_a
        } else {
            self.color_b
        };
    }

    fn encoded(&self) -> Vec<u8> {
        let mut encoded = Self::PATTERN_IDENTIFIER.to_vec();
        encoded.extend(bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap());
        return encoded;
    }
}

impl Transform for StripePattern {
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

impl Display for StripePattern {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("StripePattern")
            .field("color_a", &self.color_a)
            .field("color_b", &self.color_b)
            .field("transformation", &self.transformation())
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::Material;
    use crate::primitives::{transformations, Light, Vector};
    use crate::shapes::{Sphere, Transform};
    use std::sync::Arc;

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let pattern = StripePattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0.9, 0, 0)), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(1, 0, 0)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(-0.1, 0, 0)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(-1, 0, 0)), Color::BLACK);
        assert_eq!(pattern.color_at(&Point::new(-1.1, 0, 0)), Color::WHITE);
    }

    #[test]
    fn stripe_pattern_is_constant_in_y() {
        let pattern = StripePattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0, 1, 0)), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0, 2, 0)), Color::WHITE);
    }

    #[test]
    fn stripe_pattern_is_constant_in_z() {
        let pattern = StripePattern::new(Color::WHITE, Color::BLACK);
        assert_eq!(pattern.color_at(&Point::ORIGIN), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0, 0, 1)), Color::WHITE);
        assert_eq!(pattern.color_at(&Point::new(0, 0, 2)), Color::WHITE);
    }

    #[test]
    fn lighting_with_stripe_pattern_applied() {
        let shape = Sphere::default();
        let mut material = Material::default();
        material.pattern = Option::from(
            Arc::new(StripePattern::new(Color::WHITE, Color::BLACK)) as Arc<dyn Pattern>
        );
        material.ambient = 1.0;
        material.diffuse = 0.0;
        material.specular = 0.0;
        let camera = Vector::new(0, 0, -1);
        let normal = Vector::new(0, 0, -1);
        let light = Light::new(Point::new(0, 10, -10), Color::WHITE);
        let color1 = material.lighting(
            &shape,
            &light,
            &Point::new(0.9, 0, 0),
            &camera,
            &normal,
            false,
        );
        let color2 = material.lighting(
            &shape,
            &light,
            &Point::new(1.1, 0, 0),
            &camera,
            &normal,
            false,
        );
        assert_eq!(color1, Color::WHITE);
        assert_eq!(color2, Color::BLACK);
    }

    #[test]
    fn stripe_pattern_with_object_transformation() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::scaling(2, 2, 2));
        let pattern = StripePattern::new(Color::WHITE, Color::BLACK);
        let color = pattern.color_at_shape(&sphere, &Point::new(1.5, 0, 0));
        assert_eq!(color, Color::WHITE);
    }

    #[test]
    fn stripe_pattern_with_pattern_transformation() {
        let sphere = Sphere::default();
        let mut pattern = StripePattern::new(Color::WHITE, Color::BLACK);
        pattern.set_transformation(transformations::scaling(2, 2, 2));
        let color = pattern.color_at_shape(&sphere, &Point::new(1.5, 0, 0));
        assert_eq!(color, Color::WHITE);
    }

    #[test]
    fn stripe_pattern_with_pattern_and_object_transformations() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::scaling(2, 2, 2));
        let mut pattern = StripePattern::new(Color::WHITE, Color::BLACK);
        pattern.set_transformation(transformations::translation(0.5, 0, 0));
        let color = pattern.color_at_shape(&sphere, &Point::new(2.5, 0, 0));
        assert_eq!(color, Color::WHITE);
    }
}
