use super::Shape;
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::{BINCODE_CONFIG, EPSILON};
use crate::primitives::{transformations, Transformation};
use crate::primitives::{Point, Vector};
use bincode::Encode;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct Plane {
    pub material: Material,
    pub transformation: Transformation,
}

impl Plane {
    pub const fn new(material: Material, transformation: Transformation) -> Self {
        return Self {
            material,
            transformation,
        };
    }
}

impl Shape for Plane {
    fn local_normal_at(&self, _: Point) -> Vector {
        return Vector::UP;
    }

    fn material(&self) -> &Material {
        return &self.material;
    }

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn local_intersect(&self, ray: &Ray) -> Option<Intersections> {
        if ray.direction.y.abs() < EPSILON {
            return None;
        } else {
            let distance = -ray.origin.y / ray.direction.y;
            return Some(Intersections::from([Intersection::new(distance, self)]));
        }
    }

    fn encoded(&self) -> Vec<u8> {
        return bincode::encode_to_vec(self, BINCODE_CONFIG).expect("Failed to serialise Plane");
    }
}

impl Default for Plane {
    fn default() -> Plane {
        return Plane::new(Material::default(), transformations::IDENTITY);
    }
}

impl Display for Plane {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter
            .debug_struct("Plane")
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .finish();
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
        let intersections = boxed_shape.local_intersect(&ray);
        assert_eq!(intersections, None);
    }

    #[test]
    fn ray_intersects_plane_from_above() {
        let plane = Plane::default();
        let boxed_shape: Box<dyn Shape> = Box::new(plane);
        let ray = Ray::new(Point::new(0, 1, 0), Vector::DOWN);
        let intersections = boxed_shape.as_ref().local_intersect(&ray).unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 1.0);
        assert_eq!(intersections[0].shape, boxed_shape.as_ref());
    }

    #[test]
    fn ray_intersects_plane_from_below() {
        let plane = Plane::default();
        let boxed_shape: Box<dyn Shape> = Box::new(plane);
        let ray = Ray::new(Point::new(0, -1, 0), Vector::UP);
        let intersections = boxed_shape.as_ref().local_intersect(&ray).unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 1.0);
        assert_eq!(intersections[0].shape, boxed_shape.as_ref());
    }
}
