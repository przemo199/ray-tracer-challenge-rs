use std::fmt::{Debug, Display};

use crate::intersections::Intersections;
use crate::material::Material;
use crate::primitives::{Point, Transformation, Vector};
use crate::ray::Ray;

pub trait Shape: Debug + Display + Send + Sync {
    fn normal_at(&self, point: Point) -> Vector {
        let transform_inverse = self.transformation().inverse();
        let local_point = transform_inverse * point;
        let local_normal = self.local_normal_at(local_point);
        let world_normal = transform_inverse.transpose() * local_normal;
        return world_normal.normalized();
    }

    fn local_normal_at(&self, point: Point) -> Vector;

    fn material(&self) -> Material;

    fn set_material(&mut self, material: Material);

    fn transformation(&self) -> Transformation;

    fn set_transformation(&mut self, transformation: Transformation);

    fn local_intersect(&self, ray: &Ray) -> Option<Intersections>;

    fn local_ray(&self, ray: &Ray) -> Ray {
        return ray.transform(self.transformation().inverse());
    }

    fn encoded(&self) -> Vec<u8>;
}

impl<'a> PartialEq for &'a dyn Shape {
    fn eq(&self, rhs: &Self) -> bool {
        return self.encoded() == rhs.encoded();
    }
}

impl PartialEq for Box<dyn Shape> {
    fn eq(&self, rhs: &Self) -> bool {
        return self.encoded() == rhs.encoded();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::patterns::{ComplexPattern, GradientPattern, RingPattern};
    use crate::primitives::Color;
    use crate::shapes::Sphere;

    use super::*;

    // #[derive(Clone, Debug)]
    // pub struct TestShape {
    //     pub transformation: Matrix,
    //     pub material: Material,
    // }
    //
    // impl Shape for TestShape {
    //     fn local_normal_at(&self, point: Point) -> Vector {
    //         todo!()
    //     }
    //
    //     fn set_material(&mut self, material: Material) {
    //         todo!()
    //     }
    //
    //     fn set_transformation(&mut self, transformation: Matrix) {
    //         todo!()
    //     }
    //
    //     fn local_intersect(&self, ray: &Ray) -> Intersections {
    //         todo!()
    //     }
    // }

    #[test]
    fn default_material() {
        let shape = Sphere::default();
        assert_eq!(shape.material(), Material::default());
    }

    #[test]
    fn assigning_material() {
        let mut shape = Sphere::default();
        let mut material = Material::default();
        material.color = Color::new(0.8, 1, 0.6);
        shape.set_material(material.clone());
        assert_eq!(shape.material(), material);
    }

    #[test]
    fn compare_dyn_shapes() {
        let mut sphere_1 = Sphere::default();
        let mut sphere_2 = Sphere::default();
        sphere_1.material.pattern = Some(Arc::new(ComplexPattern::new(Arc::new(RingPattern::new(Color::WHITE, Color::BLACK)), Arc::new(GradientPattern::new(Color::WHITE, Color::BLACK)))));
        sphere_2.material.pattern = Some(Arc::new(ComplexPattern::new(Arc::new(RingPattern::new(Color::WHITE, Color::BLACK)), Arc::new(GradientPattern::new(Color::WHITE, Color::BLACK)))));
        let arc_sphere_1: Box<dyn Shape> = Box::new(sphere_1);
        let arc_sphere_2: Box<dyn Shape> = Box::new(sphere_2);
        assert_eq!(arc_sphere_1.as_ref(), arc_sphere_2.as_ref());
        let mut sphere_1 = Sphere::default();
        sphere_1.material.pattern = Some(Arc::new(ComplexPattern::new(Arc::new(RingPattern::new(Color::BLACK, Color::BLACK)), Arc::new(GradientPattern::new(Color::WHITE, Color::BLACK)))));
        let arc_sphere_1: Box<dyn Shape> = Box::new(sphere_1);
        assert_ne!(arc_sphere_1.as_ref(), arc_sphere_2.as_ref());
        let mut sphere_1 = Sphere::default();
        sphere_1.material.pattern = Some(Arc::new(ComplexPattern::new(Arc::new(RingPattern::new(Color::WHITE, Color::BLACK)), Arc::new(RingPattern::new(Color::WHITE, Color::BLACK)))));
        let arc_sphere_1: Box<dyn Shape> = Box::new(sphere_1);
        assert_ne!(arc_sphere_1.as_ref(), arc_sphere_2.as_ref());
    }
}
