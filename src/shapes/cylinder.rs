use std::fmt::{Display, Formatter};

use bincode::Encode;

use crate::consts::{BINCODE_CONFIG, EPSILON};
use crate::intersection::Intersection;
use crate::intersections::Intersections;
use crate::material::Material;
use crate::primitives::{Matrix, Point, Vector};
use crate::primitives::Transformation;
use crate::ray::Ray;
use crate::shapes::Shape;
use crate::utils::{solve_quadratic, Squared};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct Cylinder {
    pub material: Material,
    pub transformation: Matrix<4>,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
}

impl Cylinder {
    pub fn new(
        material: Material,
        transformation: Matrix<4>,
        minimum: f64,
        maximum: f64,
        closed: bool,
    ) -> Cylinder {
        return Cylinder {
            material,
            transformation,
            minimum,
            maximum,
            closed,
        };
    }

    fn check_cap(ray: &Ray, distance: impl Into<f64>) -> bool {
        let distance = distance.into();
        let x = ray.origin.x + ray.direction.x * distance;
        let z = ray.origin.z + ray.direction.z * distance;
        return (x.squared() + z.squared()) <= 1.0;
    }

    fn intersect_caps<'a>(&'a self, ray: &Ray, intersections: &mut Intersections<'a>) {
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        let distance = (self.minimum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, distance) {
            intersections.add(Intersection::new(distance, self));
        }

        let distance = (self.maximum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, distance) {
            intersections.add(Intersection::new(distance, self));
        }
    }
}

impl Shape for Cylinder {
    fn local_normal_at(&self, point: Point) -> Vector {
        let distance = point.x.squared() + point.z.squared();

        if distance < 1.0 && point.y >= self.maximum - EPSILON {
            return Vector::UP;
        }

        if distance < 1.0 && point.y <= self.minimum + EPSILON {
            return Vector::DOWN;
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

    fn local_intersect(&self, ray: &Ray) -> Option<Intersections> {
        let a = ray.direction.x.squared() + ray.direction.z.squared();

        let mut intersections = Intersections::new();
        if a.abs() > 0.0 {
            let b = 2.0 * (ray.origin.x * ray.direction.x + ray.origin.z * ray.direction.z);
            let c = ray.origin.x.squared() + ray.origin.z.squared() - 1.0;

            if let Some((mut distance_1, mut distance_2)) = solve_quadratic(a, b, c) {
                if distance_1 > distance_2 {
                    std::mem::swap(&mut distance_1, &mut distance_2);
                }

                let y1 = ray.origin.y + distance_1 * ray.direction.y;
                if self.minimum < y1 && y1 < self.maximum {
                    intersections.add(Intersection::new(distance_1, self));
                }

                let y2 = ray.origin.y + distance_2 * ray.direction.y;
                if self.minimum < y2 && y2 < self.maximum {
                    intersections.add(Intersection::new(distance_2, self));
                }
                self.intersect_caps(ray, &mut intersections);
                return intersections.into_option();
            }
        }

        self.intersect_caps(ray, &mut intersections);
        return intersections.into_option();
    }

    fn encoded(&self) -> Vec<u8> {
        return bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap();
    }
}

impl Default for Cylinder {
    fn default() -> Cylinder {
        return Cylinder::new(
            Material::default(),
            Matrix::default(),
            f64::NEG_INFINITY,
            f64::INFINITY,
            false,
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
        assert_eq!(cylinder.minimum, f64::NEG_INFINITY);
        assert_eq!(cylinder.maximum, f64::INFINITY);
        assert!(!cylinder.closed);
    }

    #[rstest]
    #[case(Point::new(1, 0, 0), Vector::UP)]
    #[case(Point::new(0, 1, 0), Vector::UP)]
    #[case(Point::new(0, 0, - 5), Vector::new(1, 1, 1))]
    fn ray_misses_cylinder(#[case] origin: Point, #[case] direction: Vector) {
        let cylinder = Cylinder::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = boxed_shape.local_intersect(&ray);
        assert_eq!(intersections, None);
    }

    #[rstest]
    #[case(Point::new(1, 0, -5), Vector::FORWARD, 5.0, 5.0)]
    #[case(Point::new(0, 0, -5), Vector::FORWARD, 4.0, 6.0)]
    #[case(Point::new(0.5, 0, -5), Vector::new(0.1, 1, 1), 6.80798191702732, 7.088723439378861)]
    fn ray_intersects_cylinder(#[case] origin: Point, #[case] direction: Vector, #[case] distance_1: f64, #[case] distance_2: f64) {
        let cylinder = Cylinder::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = boxed_shape.local_intersect(&ray).unwrap();
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, distance_1);
        assert_eq!(intersections[1].distance, distance_2);
    }

    #[rstest]
    #[case(Point::new(1, 0, 0), Vector::RIGHT)]
    #[case(Point::new(0, 5, -1), Vector::BACKWARD)]
    #[case(Point::new(0, -2, 1), Vector::FORWARD)]
    #[case(Point::new(-1, 1, 0), Vector::LEFT)]
    fn normal_vector_on_cylinder(#[case] point: Point, #[case] normal: Vector) {
        let cylinder = Cylinder::default();
        let local_normal = cylinder.local_normal_at(point);
        assert_eq!(local_normal, normal);
    }

    #[rstest]
    #[case(Point::new(0, 1.5, 0), Vector::new(0.1, 1, 0), 0)]
    #[case(Point::new(0, 3, -5), Vector::FORWARD, 0)]
    #[case(Point::new(0, 0, -5), Vector::FORWARD, 0)]
    #[case(Point::new(0, 2, -5), Vector::FORWARD, 0)]
    #[case(Point::new(0, 1, -5), Vector::FORWARD, 0)]
    #[case(Point::new(0, 1.5, -2), Vector::FORWARD, 2)]
    fn intersecting_constrained_cylinder(#[case] origin: Point, #[case] direction: Vector, #[case] count: usize) {
        let cylinder = Cylinder {
            minimum: 1.0,
            maximum: 2.0,
            ..Default::default()
        };
        let boxed_shape: Box<dyn Shape> = Box::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = boxed_shape.local_intersect(&ray);
        if count > 0 {
            assert_eq!(intersections.unwrap().len(), count);
        } else {
            assert_eq!(intersections, None);
        }
    }

    #[rstest]
    #[case(Point::new(0, 3, 0), Vector::DOWN, 2)]
    #[case(Point::new(0, 3, -2), Vector::new(0, -1, 2), 2)]
    #[case(Point::new(0, 4, -2), Vector::new(0, -1, 1), 2)]
    #[case(Point::new(0, 0, -2), Vector::new(0, 1, 2), 2)]
    #[case(Point::new(0, -1, -2), Vector::new(0, 1, 1), 2)]
    fn intersecting_caps_of_closed_cylinder(#[case] origin: Point, #[case] direction: Vector, #[case] count: usize) {
        let mut cylinder = Cylinder::default();
        cylinder.minimum = 1.0;
        cylinder.maximum = 2.0;
        cylinder.closed = true;
        let boxed_shape: Box<dyn Shape> = Box::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = boxed_shape.local_intersect(&ray).unwrap();
        assert_eq!(intersections.len(), count);
    }

    #[rstest]
    #[case(Point::new(0, 1, 0), Vector::DOWN)]
    #[case(Point::new(0.5, 1, 0), Vector::DOWN)]
    #[case(Point::new(0, 1, 0.5), Vector::DOWN)]
    #[case(Point::new(0, 2, 0), Vector::UP)]
    #[case(Point::new(0.5, 2, 0), Vector::UP)]
    #[case(Point::new(0, 2, 0.5), Vector::UP)]
    fn normal_vector_on_cylinder_caps(#[case] point: Point, #[case] normal: Vector) {
        let mut cylinder = Cylinder::default();
        cylinder.minimum = 1.0;
        cylinder.maximum = 2.0;
        cylinder.closed = true;
        assert_eq!(cylinder.local_normal_at(point), normal);
    }
}
