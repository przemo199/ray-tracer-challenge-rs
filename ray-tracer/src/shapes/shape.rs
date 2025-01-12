use crate::composites::{Intersections, Material, Ray};
use crate::primitives::{Point, Transformation, Vector};
use core::fmt::Debug;

pub trait Shape: Debug + Send + Sync + Transform + Intersect {
    #[inline]
    fn normal_at(&self, point: Point) -> Vector {
        let local_point = self.transformation_inverse() * point;
        let local_normal = self.local_normal_at(local_point);
        let world_normal = self.transformation_inverse().transpose() * local_normal;
        return world_normal.normalized();
    }

    fn local_normal_at(&self, point: Point) -> Vector;

    fn material(&self) -> &Material;

    fn encoded(&self) -> Vec<u8>;
}

pub trait Transform {
    fn transformation(&self) -> Transformation;

    fn set_transformation(&mut self, transformation: Transformation);

    fn transformation_inverse(&self) -> Transformation;

    fn set_transformation_inverse(&mut self, transformation: Transformation);
}

pub trait Intersect {
    fn local_intersect<'shape>(&'shape self, ray: &Ray, intersections: &mut Intersections<'shape>);
}

impl PartialEq for &dyn Shape {
    fn eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs) || self.encoded() == rhs.encoded();
    }
}

impl PartialEq for Box<dyn Shape> {
    fn eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs) || self.encoded() == rhs.encoded();
    }
}

#[cfg(test)]
mod tests {
    use crate::composites::Material;
    use crate::patterns::{ComplexPattern, GradientPattern, RingPattern};
    use crate::primitives::Color;
    use crate::shapes::Sphere;
    use std::sync::Arc;

    use super::*;

    #[test]
    fn default_material() {
        let shape = Sphere::default();
        assert_eq!(shape.material(), &Material::default());
    }

    #[test]
    fn assigning_material() {
        let mut shape = Sphere::default();
        let mut material = Material::default();
        material.color = Color::new(0.8, 1, 0.6);
        shape.material = material.clone();
        assert_eq!(shape.material(), &material);
    }

    #[test]
    fn compare_dyn_shapes() {
        let mut sphere_1 = Sphere::default();
        let mut sphere_2 = Sphere::default();
        sphere_1.material.pattern = Some(Arc::new(ComplexPattern::new(
            Arc::new(RingPattern::new(Color::WHITE, Color::BLACK)),
            Arc::new(GradientPattern::new(Color::WHITE, Color::BLACK)),
        )));
        sphere_2.material.pattern = Some(Arc::new(ComplexPattern::new(
            Arc::new(RingPattern::new(Color::WHITE, Color::BLACK)),
            Arc::new(GradientPattern::new(Color::WHITE, Color::BLACK)),
        )));
        let arc_sphere_1: Box<dyn Shape> = Box::new(sphere_1);
        let arc_sphere_2: Box<dyn Shape> = Box::new(sphere_2);
        assert_eq!(arc_sphere_1.as_ref(), arc_sphere_2.as_ref());
        let mut sphere_1 = Sphere::default();
        sphere_1.material.pattern = Some(Arc::new(ComplexPattern::new(
            Arc::new(RingPattern::new(Color::BLACK, Color::BLACK)),
            Arc::new(GradientPattern::new(Color::WHITE, Color::BLACK)),
        )));
        let arc_sphere_1: Box<dyn Shape> = Box::new(sphere_1);
        assert_ne!(arc_sphere_1.as_ref(), arc_sphere_2.as_ref());
        let mut sphere_1 = Sphere::default();
        sphere_1.material.pattern = Some(Arc::new(ComplexPattern::new(
            Arc::new(RingPattern::new(Color::WHITE, Color::BLACK)),
            Arc::new(RingPattern::new(Color::WHITE, Color::BLACK)),
        )));
        let arc_sphere_1: Box<dyn Shape> = Box::new(sphere_1);
        assert_ne!(arc_sphere_1.as_ref(), arc_sphere_2.as_ref());
    }
}
