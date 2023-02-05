use std::fmt::{Display, Formatter};
use std::sync::Arc;
use crate::consts::EPSILON;
use crate::intersection::Intersection;
use crate::intersections::Intersections;
use crate::material::Material;
use crate::matrix::Matrix;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::tuple::Tuple;

#[derive(Clone, Debug, PartialEq)]
pub struct Plane {
    pub material: Material,
    pub transformation: Matrix<4>,
    pub normal: Tuple,
}

impl Plane {
    pub fn new(material: Material, transformation: Matrix<4>) -> Plane {
        let normal = Tuple::vector(0.0, 1.0, 0.0);
        return Plane {
            material,
            transformation,
            normal,
        };
    }
}

impl Default for Plane {
    fn default() -> Plane {
        return Plane::new(Material::default(), Matrix::identity());
    }
}

impl Shape for Plane {
    fn local_normal_at(&self, _: Tuple) -> Tuple {
        return self.normal;
    }

    fn material(&self) -> Material {
        return self.material.clone();
    }

    fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    fn transformation(&self) -> Matrix<4> {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Matrix<4>) {
        self.transformation = transformation;
    }

    fn local_intersect(self: Arc<Self>, ray: &Ray) -> Intersections {
        if ray.direction.y.abs() < EPSILON {
            return Intersections::new();
        }

        let t = -ray.origin.y / ray.direction.y;
        let mut result = Intersections::new();
        result.add(Intersection::new(t, self));
        return result;
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

impl From<Plane> for Box<dyn Shape> {
    fn from(plane: Plane) -> Box<dyn Shape> {
        return Box::new(plane);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_is_constant() {
        let plane = Plane::default();
        let normal1 = plane.normal_at(Tuple::point(0.0, 0.0, 0.0));
        let normal2 = plane.normal_at(Tuple::point(10.0, 0.0, -10.0));
        let normal3 = plane.normal_at(Tuple::point(-5.0, 0.0, 150.0));
        let normal = Tuple::vector(0.0, 1.0, 0.0);
        assert_eq!(normal1, normal);
        assert_eq!(normal2, normal);
        assert_eq!(normal3, normal);
    }

    #[test]
    fn ray_intersects_plane_in_parallel() {
        let plane = Plane::default();
        let arc_plane: Arc<dyn Shape> = Arc::new(plane);
        let ray = Ray::new(Tuple::point(0.0, 10.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let intersections = arc_plane.local_intersect(&ray);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn ray_intersects_plane_from_above() {
        let plane = Plane::default();
        let arc_plane: Arc<dyn Shape> = Arc::new(plane);
        let ray = Ray::new(Tuple::point(0.0, 1.0, 0.0), Tuple::vector(0.0, -1.0, 0.0));
        let intersections = arc_plane.clone().local_intersect(&ray);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].t, 1.0);
        assert_eq!(&intersections[0].object, &arc_plane);
    }

    #[test]
    fn ray_intersects_plane_from_below() {
        let plane = Plane::default();
        let arc_plane: Arc<dyn Shape> = Arc::new(plane);
        let ray = Ray::new(Tuple::point(0.0, -1.0, 0.0), Tuple::vector(0.0, 1.0, 0.0));
        let intersections = arc_plane.clone().local_intersect(&ray);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].t, 1.0);
        assert_eq!(&intersections[0].object, &arc_plane);
    }
}
