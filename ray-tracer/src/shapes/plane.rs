use super::{Intersect, Shape, Transform};
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::EPSILON;
use crate::impl_shape_transform_for;
use crate::primitives::{Point, Transformation, Vector};
use crate::utils::CoarseEq;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plane {
    transformation_inverse: Transformation,
    /// Transposed inverse, cached for `normal_at`.
    normal_transformation: Transformation,
    pub material: Material,
}

impl Plane {
    pub fn new(material: Material, transformation: Transformation) -> Self {
        let transformation_inverse = transformation.inverse();
        return Self {
            material,
            transformation_inverse,
            normal_transformation: transformation_inverse.transpose(),
        };
    }
}

impl_shape_transform_for!(Plane);

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

impl Display for Plane {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return f
            .debug_struct("Plane")
            .field("material", &self.material)
            .field("transformation", &self.transformation())
            .finish();
    }
}

impl CoarseEq for Plane {
    fn coarse_eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || self.material == other.material
                && self
                    .transformation_inverse
                    .coarse_eq(&other.transformation_inverse);
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
