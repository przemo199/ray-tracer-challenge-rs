use std::fmt::{Debug, Display};
use std::sync::Arc;
use crate::intersections::Intersections;
use crate::material::Material;
use crate::matrix::Matrix;
use crate::ray::Ray;
use crate::tuple::{Tuple, TupleTrait};

pub trait Shape: Debug + Display + Send + Sync {
    fn normal_at(&self, point: Tuple) -> Tuple {
        let transform_inverse = self.transformation().inverse();
        let local_point = transform_inverse * point;
        let local_normal = self.local_normal_at(local_point);
        let mut world_normal = transform_inverse.transpose() * local_normal;
        world_normal.w = 0.0;
        return world_normal.normalize();
    }

    fn local_normal_at(&self, point: Tuple) -> Tuple;

    fn material(&self) -> Material;

    fn set_material(&mut self, material: Material);

    fn transformation(&self) -> Matrix<4>;

    fn set_transformation(&mut self, transformation: Matrix<4>);

    fn local_intersect(self: Arc<Self>, ray: &Ray) -> Intersections;

    fn local_ray(&self, ray: &Ray) -> Ray {
        return ray.transform(self.transformation().inverse());
    }
}

impl PartialEq for dyn Shape {
    fn eq(&self, rhs: &Self) -> bool {
        return self.to_string() == rhs.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::sphere::Sphere;

    // #[derive(Clone, Debug)]
    // pub struct TestShape {
    //     pub transformation: Matrix,
    //     pub material: Material,
    // }
    //
    // impl Shape for TestShape {
    //     fn local_normal_at(&self, point: Tuple) -> Tuple {
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
        material.color = Color::new(0.8, 1.0, 0.6);
        shape.set_material(material.clone());
        assert_eq!(shape.material(), material);
    }
}
