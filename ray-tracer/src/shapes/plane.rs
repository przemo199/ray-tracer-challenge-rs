use super::{Intersect, Shape, Transform};
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::EPSILON;
use crate::primitives::{Point, Transformation, Vector};
use crate::utils::CoarseEq;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Plane {
    pub material: Material,
    transformation_inverse: Transformation,
}

impl Plane {
    pub fn new(material: Material, transformation: Transformation) -> Self {
        return Self {
            material,
            transformation_inverse: transformation.inverse(),
        };
    }
}

impl Transform for Plane {
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

impl Intersect for Plane {
    fn local_intersect<'shape>(&'shape self, ray: &Ray, intersections: &mut Intersections<'shape>) {
        if ray.direction.y.abs() < EPSILON {
            return;
        }
        let distance = -ray.origin.y / ray.direction.y;
        intersections.push(Intersection::new(distance, self));
    }
}

impl Shape for Plane {
    fn local_normal_at(&self, _: Point) -> Vector {
        return Vector::UP;
    }

    fn material(&self) -> &Material {
        return &self.material;
    }
}

impl Default for Plane {
    fn default() -> Plane {
        return Plane::new(Material::default(), Transformation::IDENTITY);
    }
}

impl Display for Plane {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        return formatter
            .debug_struct("Plane")
            .field("material", &self.material)
            .field("transformation", &self.transformation())
            .finish();
    }
}

impl CoarseEq for Plane {
    fn coarse_eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs)
            || self.material == rhs.material
                && self
                    .transformation_inverse
                    .coarse_eq(&rhs.transformation_inverse);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::Ray;

    #[test]
    fn normal_is_constant() {
        let plane = Plane::default();
        let normal1 = plane.normal_at(Point::ORIGIN);
        let normal2 = plane.normal_at(Point::new(10, 0, -10));
        let normal3 = plane.normal_at(Point::new(-5, 0, 150));
        let normal = Vector::UP;
        assert_eq!(normal1, normal);
        assert_eq!(normal2, normal);
        assert_eq!(normal3, normal);
    }

    #[test]
    fn ray_intersects_plane_in_parallel() {
        let plane = Plane::default();
        let boxed_shape: Box<dyn Shape> = Box::new(plane);
        let ray = Ray::new(Point::new(0, 10, 0), Vector::FORWARD);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert!(intersections.is_empty());
    }

    #[test]
    fn ray_intersects_plane_from_above() {
        let plane = Plane::default();
        let boxed_shape: Box<dyn Shape> = Box::new(plane);
        let ray = Ray::new(Point::new(0, 1, 0), Vector::DOWN);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 1.0);
        assert_eq!(intersections[0].shape, boxed_shape.as_ref());
    }

    #[test]
    fn ray_intersects_plane_from_below() {
        let plane = Plane::default();
        let boxed_shape: Box<dyn Shape> = Box::new(plane);
        let ray = Ray::new(Point::new(0, -1, 0), Vector::UP);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 1.0);
        assert_eq!(intersections[0].shape, boxed_shape.as_ref());
    }
}
