use std::fmt::{Debug, Display, Formatter};
use crate::color::Color;
use crate::matrix::Matrix;
use crate::point::Point;
use crate::shape::Shape;
use crate::transformations::Transformations;

pub trait PatternClone {
    fn box_clone(&self) -> Box<dyn Pattern>;
}

pub trait Pattern: PatternClone + Debug + Display + Sync + Send {
    fn color_at(&self, point: &Point) -> Color;

    fn color_at_shape(&self, object: &dyn Shape, point: &Point) -> Color {
        let object_point = object.transformation().inverse() * *point;
        let pattern_point = self.transformation().inverse() * object_point;
        return self.color_at(&pattern_point);
    }

    fn transformation(&self) -> Matrix<4>;

    fn set_transformation(&mut self, transformation: Matrix<4>);
}

impl<T> PatternClone for T where T: 'static + Pattern + Clone {
    fn box_clone(&self) -> Box<dyn Pattern> {
        return Box::new(self.clone());
    }
}

impl PartialEq for Box<dyn Pattern> {
    fn eq(&self, rhs: &Box<dyn Pattern>) -> bool {
        return self.to_string() == rhs.to_string();
    }
}

impl Clone for Box<dyn Pattern> {
    fn clone(&self) -> Box<dyn Pattern> {
        return self.box_clone();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StripePattern {
    color_a: Color,
    color_b: Color,
    transformation: Matrix<4>,
}

impl StripePattern {
    pub fn new(color_a: Color, color_b: Color) -> StripePattern {
        return StripePattern { color_a, color_b, transformation: Transformations::identity() };
    }
}

impl Pattern for StripePattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = point.x.floor() as i32;
        return if distance % 2 == 0 { self.color_a } else { self.color_b };
    }

    fn transformation(&self) -> Matrix<4> {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Matrix<4>) {
        self.transformation = transformation;
    }
}

impl Display for StripePattern {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("StripePattern")
            .field("color_a", &self.color_a)
            .field("color_b", &self.color_b)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestPattern {
    transformation: Matrix<4>,
}

impl TestPattern {
    pub fn new() -> TestPattern {
        return TestPattern { transformation: Transformations::identity() };
    }
}

impl Pattern for TestPattern {
    fn color_at(&self, point: &Point) -> Color {
        return Color::new(point.x, point.y, point.z);
    }

    fn transformation(&self) -> Matrix<4> {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Matrix<4>) {
        self.transformation = transformation;
    }
}

impl Display for TestPattern {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("TestPattern")
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GradientPattern {
    color_a: Color,
    color_b: Color,
    transformation: Matrix<4>,
}

impl GradientPattern {
    pub fn new(color_a: Color, color_b: Color) -> GradientPattern {
        return GradientPattern { color_a, color_b, transformation: Transformations::identity() };
    }
}

impl Pattern for GradientPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = self.color_b - self.color_a;
        let mut fraction = point.x.fract();
        if point.x as i32 % 2 != 0 {
            fraction = 1.0 - fraction;
        }
        return self.color_a + (distance * fraction);
    }

    fn transformation(&self) -> Matrix<4> {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Matrix<4>) {
        self.transformation = transformation;
    }
}

impl Display for GradientPattern {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("GradientPattern")
            .field("color_a", &self.color_a)
            .field("color_b", &self.color_b)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RingPattern {
    color_a: Color,
    color_b: Color,
    transformation: Matrix<4>,
}

impl RingPattern {
    pub fn new(color_a: Color, color_b: Color) -> RingPattern {
        return RingPattern { color_a, color_b, transformation: Transformations::identity() };
    }
}

impl Pattern for RingPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = (point.x * point.x + point.z * point.z).sqrt().floor() as i32;
        return if distance % 2 == 0 { self.color_a } else { self.color_b };
    }

    fn transformation(&self) -> Matrix<4> {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Matrix<4>) {
        self.transformation = transformation;
    }
}

impl Display for RingPattern {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("CircularPattern")
            .field("color_a", &self.color_a)
            .field("color_b", &self.color_b)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckerPattern {
    color_a: Color,
    color_b: Color,
    transformation: Matrix<4>,
}

impl CheckerPattern {
    pub fn new(color_a: Color, color_b: Color) -> CheckerPattern {
        return CheckerPattern { color_a, color_b, transformation: Transformations::identity() };
    }
}

impl Pattern for CheckerPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = (point.x.floor() + point.y.floor() + point.z.floor()) as i32;
        return if distance % 2 == 0 { self.color_a } else { self.color_b };
    }

    fn transformation(&self) -> Matrix<4> {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Matrix<4>) {
        self.transformation = transformation;
    }
}

impl Display for CheckerPattern {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("CheckerPattern")
            .field("color_a", &self.color_a)
            .field("color_b", &self.color_b)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[derive(Clone, Debug)]
pub struct ComplexPattern {
    pattern_a: Box<dyn Pattern>,
    pattern_b: Box<dyn Pattern>,
    transformation: Matrix<4>,
}

impl ComplexPattern {
    pub fn new(pattern_a: Box<dyn Pattern>, pattern_b: Box<dyn Pattern>) -> ComplexPattern {
        return ComplexPattern { pattern_a, pattern_b, transformation: Transformations::identity() };
    }
}

impl Pattern for ComplexPattern {
    fn color_at(&self, point: &Point) -> Color {
        let distance = point.x.floor() as i32;
        return if distance % 2 == 0 { self.pattern_a.color_at(point) } else { self.pattern_b.color_at(point) };
    }

    fn transformation(&self) -> Matrix<4> {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Matrix<4>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::light::Light;
    use crate::material::Material;
    use crate::sphere::Sphere;
    use crate::vector::Vector;

    #[test]
    fn stripe_pattern_is_constant_in_y() {
        let pattern = StripePattern::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.0, 1.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.0, 2.0, 0.0)), Color::white());
    }

    #[test]
    fn stripe_pattern_is_constant_in_z() {
        let pattern = StripePattern::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 1.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 2.0)), Color::white());
    }

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let pattern = StripePattern::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.9, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(1.0, 0.0, 0.0)), Color::black());
        assert_eq!(pattern.color_at(&Point::new(-0.1, 0.0, 0.0)), Color::black());
        assert_eq!(pattern.color_at(&Point::new(-1.0, 0.0, 0.0)), Color::black());
        assert_eq!(pattern.color_at(&Point::new(-1.1, 0.0, 0.0)), Color::white());
    }

    #[test]
    fn lighting_with_pattern_applied() {
        let object = Sphere::default();
        let mut material = Material::default();
        material.pattern = Option::from(Box::new(StripePattern::new(Color::white(), Color::black())) as Box<dyn Pattern>);
        material.ambient = 1.0;
        material.diffuse = 0.0;
        material.specular = 0.0;
        let camera = Vector::new(0.0, 0.0, -1.0);
        let normal = Vector::new(0.0, 0.0, -1.0);
        let light = Light::new(Point::new(0.0, 10.0, -10.0), Color::white());
        let color1 = material.lighting(&object, &light, &Point::new(0.9, 0.0, 0.0), &camera, &normal, false);
        let color2 = material.lighting(&object, &light, &Point::new(1.1, 0.0, 0.0), &camera, &normal, false);
        assert_eq!(color1, Color::white());
        assert_eq!(color2, Color::black());
    }

    #[test]
    fn stripes_with_object_transformation() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(Transformations::scaling(2.0, 2.0, 2.0));
        let pattern = StripePattern::new(Color::white(), Color::black());
        let color = pattern.color_at_shape(&sphere, &Point::new(1.5, 0.0, 0.0));
        assert_eq!(color, Color::white());
    }

    #[test]
    fn stripes_with_pattern_transformation() {
        let sphere = Sphere::default();
        let mut pattern = StripePattern::new(Color::white(), Color::black());
        pattern.transformation = Transformations::scaling(2.0, 2.0, 2.0);
        let color = pattern.color_at_shape(&sphere, &Point::new(1.5, 0.0, 0.0));
        assert_eq!(color, Color::white());
    }

    #[test]
    fn stripes_with_pattern_and_object_transformations() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(Transformations::scaling(2.0, 2.0, 2.0));
        let mut pattern = StripePattern::new(Color::white(), Color::black());
        pattern.transformation = Transformations::translation(0.5, 0.0, 0.0);
        let color = pattern.color_at_shape(&sphere, &Point::new(2.5, 0.0, 0.0));
        assert_eq!(color, Color::white());
    }

    #[test]
    fn default_test_pattern_transformation() {
        let pattern = TestPattern::new();
        assert_eq!(pattern.transformation, Transformations::identity());
    }

    #[test]
    fn assigning_test_pattern_transformation() {
        let mut pattern = TestPattern::new();
        pattern.transformation = Transformations::translation(1.0, 2.0, 3.0);
        assert_eq!(pattern.transformation, Transformations::translation(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_pattern_with_object_transformation() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(Transformations::scaling(2.0, 2.0, 2.0));
        let pattern = TestPattern::new();
        let color = pattern.color_at_shape(&sphere, &Point::new(2.0, 3.0, 4.0));
        assert_eq!(color, Color::new(1.0, 1.5, 2.0));
    }

    #[test]
    fn test_pattern_with_pattern_transformation() {
        let sphere = Sphere::default();
        let mut pattern = TestPattern::new();
        pattern.transformation = Transformations::scaling(2.0, 2.0, 2.0);
        let color = pattern.color_at_shape(&sphere, &Point::new(2.0, 3.0, 4.0));
        assert_eq!(color, Color::new(1.0, 1.5, 2.0));
    }

    #[test]
    fn test_pattern_with_pattern_and_object_transformations() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(Transformations::scaling(2.0, 2.0, 2.0));
        let mut pattern = TestPattern::new();
        pattern.transformation = Transformations::translation(0.5, 1.0, 1.5);
        let color = pattern.color_at_shape(&sphere, &Point::new(2.5, 3.0, 3.5));
        assert_eq!(color, Color::new(0.75, 0.5, 0.25));
    }

    #[test]
    fn gradient_interpolates_between_colors() {
        let pattern = GradientPattern::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.25, 0.0, 0.0)), Color::new(0.75, 0.75, 0.75));
        assert_eq!(pattern.color_at(&Point::new(0.5, 0.0, 0.0)), Color::new(0.5, 0.5, 0.5));
        assert_eq!(pattern.color_at(&Point::new(0.75, 0.0, 0.0)), Color::new(0.25, 0.25, 0.25));
        assert_eq!(pattern.color_at(&Point::new(1.0, 0.0, 0.0)), Color::black());
    }

    #[test]
    fn ring_pattern_extends_in_x_and_z() {
        let pattern = RingPattern::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(1.0, 0.0, 0.0)), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 1.0)), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.708, 0.0, 0.708)), Color::black());
    }

    #[test]
    fn checker_pattern_repeats_in_x() {
        let pattern = CheckerPattern::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.99, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(1.01, 0.0, 0.0)), Color::black());
    }

    #[test]
    fn checker_pattern_repeats_in_y() {
        let pattern = CheckerPattern::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.99, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.0, 1.01, 0.0)), Color::black());
    }

    #[test]
    fn checker_pattern_repeats_in_z() {
        let pattern = CheckerPattern::new(Color::white(), Color::black());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 0.99)), Color::white());
        assert_eq!(pattern.color_at(&Point::new(0.0, 0.0, 1.01)), Color::black());
    }
}
