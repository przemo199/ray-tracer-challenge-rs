use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::consts::EPSILON;
use crate::intersection::Intersection;
use crate::intersections::Intersections;
use crate::material::Material;
use crate::primitives::{Matrix, Point, Vector};
use crate::primitives::Transformation;
use crate::ray::Ray;
use crate::shapes::Shape;

#[derive(Clone, Debug, PartialEq)]
pub struct Cylinder {
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub transformation: Matrix<4>,
    pub material: Material,
}

impl Cylinder {
    pub fn new(
        minimum: f64,
        maximum: f64,
        closed: bool,
        transformation: Matrix<4>,
        material: Material,
    ) -> Cylinder {
        return Cylinder {
            minimum,
            maximum,
            closed,
            transformation,
            material,
        };
    }

    fn check_cap(ray: &Ray, distance: f64) -> bool {
        let x = ray.origin.x + ray.direction.x * distance;
        let z = ray.origin.z + ray.direction.z * distance;
        return (x * x + z * z) <= 1.0;
    }

    fn intersect_caps(self: Arc<Self>, ray: &Ray, intersections: &mut Intersections) {
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        let distance = (self.minimum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, distance) {
            intersections.add(Intersection::new(distance, self.clone()));
        }

        let distance = (self.maximum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, distance) {
            intersections.add(Intersection::new(distance, self));
        }
    }
}

impl Shape for Cylinder {
    fn local_normal_at(&self, point: Point) -> Vector {
        let distance = point.x * point.x + point.z * point.z;

        if distance < 1.0 && point.y >= self.maximum - EPSILON {
            return Vector::new(0, 1, 0);
        }

        if distance < 1.0 && point.y <= self.minimum + EPSILON {
            return Vector::new(0, -1, 0);
        }

        return Vector::new(point.x, 0, point.z);
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
        let mut intersections = Intersections::new();
        let a = ray.direction.x.powi(2) + ray.direction.z.powi(2);

        if a.abs() < EPSILON {
            self.intersect_caps(ray, &mut intersections);
            return intersections;
        }

        let b = 2.0 * (ray.origin.x * ray.direction.x + ray.origin.z * ray.direction.z);
        let c = ray.origin.x.powi(2) + ray.origin.z.powi(2) - 1.0;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return intersections;
        }

        let double_a = 2.0 * a;
        let discriminant_sqrt = discriminant.sqrt();
        let mut distance_0 = (-b - discriminant_sqrt) / double_a;
        let mut distance_1 = (-b + discriminant_sqrt) / double_a;
        if distance_0 > distance_1 {
            std::mem::swap(&mut distance_0, &mut distance_1);
        }

        let y0 = ray.origin.y + distance_0 * ray.direction.y;
        if self.minimum < y0 && y0 < self.maximum {
            intersections.add(Intersection::new(distance_0, self.clone()));
        }

        let y1 = ray.origin.y + distance_1 * ray.direction.y;
        if self.minimum < y1 && y1 < self.maximum {
            intersections.add(Intersection::new(distance_1, self.clone()));
        }

        self.intersect_caps(ray, &mut intersections);
        return intersections;
    }
}

impl Default for Cylinder {
    fn default() -> Cylinder {
        return Cylinder::new(
            f64::MIN,
            f64::MAX,
            false,
            Matrix::default(),
            Material::default(),
        );
    }
}

impl Display for Cylinder {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("Cylinder")
            .field("minimum", &self.minimum)
            .field("maximum", &self.maximum)
            .field("closed", &self.closed)
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[test]
    fn default_cylinder() {
        let cylinder = Cylinder::default();
        assert_eq!(cylinder.minimum, f64::MIN);
        assert_eq!(cylinder.maximum, f64::MAX);
        assert!(!cylinder.closed);
    }

    #[rstest]
    #[case(Point::new(1, 0, 0), Vector::new(0, 1, 0))]
    #[case(Point::new(0, 1, 0), Vector::new(0, 1, 0))]
    #[case(Point::new(0, 0, -5), Vector::new(1, 1, 1))]
    fn ray_misses_cylinder(#[case] origin: Point, #[case] direction: Vector) {
        let cylinder = Cylinder::default();
        let arc_cylinder: Arc<dyn Shape> = Arc::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = arc_cylinder.local_intersect(&ray);
        assert_eq!(intersections.len(), 0);
    }

    #[rstest]
    #[case(Point::new(1, 0, -5), Vector::new(0, 0, 1), 5.0, 5.0)]
    #[case(Point::new(0, 0, -5), Vector::new(0, 0, 1), 4.0, 6.0)]
    #[case(Point::new(0.5, 0, -5), Vector::new(0.1, 1, 1), 6.80798191702732, 7.088723439378861)]
    fn ray_intersects_cylinder(#[case] origin: Point, #[case] direction: Vector, #[case] distance_1: f64, #[case] distance_2: f64) {
            let cylinder = Cylinder::default();
            let arc_cylinder: Arc<dyn Shape> = Arc::new(cylinder);
            let ray = Ray::new(origin, direction.normalized());
            let intersections = arc_cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), 2);
            assert_eq!(intersections[0].distance, distance_1);
            assert_eq!(intersections[1].distance, distance_2);
    }

    #[rstest]
    #[case(Point::new(1, 0, 0), Vector::new(1, 0, 0))]
    #[case(Point::new(0, 5, -1), Vector::new(0, 0, -1))]
    #[case(Point::new(0, -2, 1), Vector::new(0, 0, 1))]
    #[case(Point::new(-1, 1, 0), Vector::new(-1, 0, 0))]
    fn normal_vector_on_cylinder(#[case] point: Point, #[case] normal: Vector) {
        let cylinder = Cylinder::default();
        let local_normal = cylinder.local_normal_at(point);
        assert_eq!(local_normal, normal);
    }

    #[rstest]
    #[case(Point::new(0, 1.5, 0), Vector::new(0.1, 1, 0), 0)]
    #[case(Point::new(0, 3, -5), Vector::new(0, 0, 1), 0)]
    #[case(Point::new(0, 0, -5), Vector::new(0, 0, 1), 0)]
    #[case(Point::new(0, 2, -5), Vector::new(0, 0, 1), 0)]
    #[case(Point::new(0, 1, -5), Vector::new(0, 0, 1), 0)]
    #[case(Point::new(0, 1.5, -2), Vector::new(0, 0, 1), 2)]
    fn intersecting_constrained_cylinder(#[case] origin: Point, #[case] direction: Vector, #[case] count: usize) {
        let cylinder = Cylinder {
            minimum: 1.0,
            maximum: 2.0,
            ..Default::default()
        };
        let arc_cylinder: Arc<dyn Shape> = Arc::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = arc_cylinder.local_intersect(&ray);
        assert_eq!(intersections.len(), count);
    }

    #[rstest]
    #[case(Point::new(0, 3, 0), Vector::new(0, -1, 0), 2)]
    #[case(Point::new(0, 3, -2), Vector::new(0, -1, 2), 2)]
    #[case(Point::new(0, 4, -2), Vector::new(0, -1, 1), 2)]
    #[case(Point::new(0, 0, -2), Vector::new(0, 1, 2), 2)]
    #[case(Point::new(0, -1, -2), Vector::new(0, 1, 1), 2)]
    fn intersecting_caps_of_closed_cylinder(#[case] origin: Point, #[case] direction: Vector, #[case] count: usize) {
        let mut cylinder = Cylinder::default();
        cylinder.minimum = 1.0;
        cylinder.maximum = 2.0;
        cylinder.closed = true;
        let arc_cylinder: Arc<dyn Shape> = Arc::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = arc_cylinder.local_intersect(&ray);
        assert_eq!(intersections.len(), count);
    }

    #[rstest]
    #[case(Point::new(0, 1, 0), Vector::new(0, -1, 0))]
    #[case(Point::new(0.5, 1, 0), Vector::new(0, -1, 0))]
    #[case(Point::new(0, 1, 0.5), Vector::new(0, -1, 0))]
    #[case(Point::new(0, 2, 0), Vector::new(0, 1, 0))]
    #[case(Point::new(0.5, 2, 0), Vector::new(0, 1, 0))]
    #[case(Point::new(0, 2, 0.5), Vector::new(0, 1, 0))]
    fn normal_vector_on_cylinder_caps(#[case] point: Point, #[case] normal: Vector) {
        let mut cylinder = Cylinder::default();
        cylinder.minimum = 1.0;
        cylinder.maximum = 2.0;
        cylinder.closed = true;
        assert_eq!(cylinder.local_normal_at(point), normal);
    }
}
