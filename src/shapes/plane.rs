use std::fmt::{Display, Formatter};
use std::sync::Arc;

use bincode::Encode;

use crate::consts::{BINCODE_CONFIG, EPSILON};
use crate::intersection::Intersection;
use crate::intersections::Intersections;
use crate::material::Material;
use crate::primitives::{Matrix, Point, Vector};
use crate::primitives::{Transformation, transformations};
use crate::ray::Ray;

use super::Shape;

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct Plane {
    pub material: Material,
    pub transformation: Transformation,
    pub normal: Vector,
}

impl Plane {
    pub fn new(material: Material, transformation: Matrix<4>) -> Plane {
        let normal = Vector::new(0, 1, 0);
        return Plane {
            material,
            transformation,
            normal,
        };
    }
}

impl Default for Plane {
    fn default() -> Plane {
        return Plane::new(Material::default(), transformations::IDENTITY);
    }
}

impl Shape for Plane {
    fn local_normal_at(&self, _: Point) -> Vector {
        return self.normal;
    }

    fn material(&self) -> Material {
        return self.material.clone();
    }

    fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation = transformation;
    }

    fn local_intersect(self: Arc<Self>, ray: &Ray) -> Intersections {
        if ray.direction.y.abs() < EPSILON {
            return Intersections::new();
        }

        let distance = -ray.origin.y / ray.direction.y;
        let mut result = Intersections::new();
        result.add(Intersection::new(distance, self));
        return result;
    }

    fn encoded(&self) -> Vec<u8> {
        return bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap();
    }
}

impl Display for Plane {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        return formatter.debug_struct("Plane")
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .field("normal", &self.normal)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_is_constant() {
        let plane = Plane::default();
        let normal1 = plane.normal_at(Point::new(0, 0, 0));
        let normal2 = plane.normal_at(Point::new(10, 0, -10));
        let normal3 = plane.normal_at(Point::new(-5, 0, 150));
        let normal = Vector::new(0, 1, 0);
        assert_eq!(normal1, normal);
        assert_eq!(normal2, normal);
        assert_eq!(normal3, normal);
    }

    #[test]
    fn ray_intersects_plane_in_parallel() {
        let plane = Plane::default();
        let arc_plane: Arc<dyn Shape> = Arc::new(plane);
        let ray = Ray::new(Point::new(0, 10, 0), Vector::new(0, 0, 1));
        let intersections = arc_plane.local_intersect(&ray);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn ray_intersects_plane_from_above() {
        let plane = Plane::default();
        let arc_plane: Arc<dyn Shape> = Arc::new(plane);
        let ray = Ray::new(Point::new(0, 1, 0), Vector::new(0, -1, 0));
        let intersections = arc_plane.clone().local_intersect(&ray);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 1.0);
        assert_eq!(&intersections[0].object, &arc_plane);
    }

    #[test]
    fn ray_intersects_plane_from_below() {
        let plane = Plane::default();
        let arc_plane: Arc<dyn Shape> = Arc::new(plane);
        let ray = Ray::new(Point::new(0, -1, 0), Vector::new(0, 1, 0));
        let intersections = arc_plane.clone().local_intersect(&ray);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 1.0);
        assert_eq!(&intersections[0].object, &arc_plane);
    }
}
