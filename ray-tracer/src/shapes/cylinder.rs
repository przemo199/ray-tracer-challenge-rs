use super::{Intersect, Shape, Transform};
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::{EPSILON, MAX, MIN};
use crate::primitives::{Point, Transformation, Vector};
use crate::utils::{CoarseEq, Squared, solve_quadratic};
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Cylinder {
    pub material: Material,
    transformation_inverse: Transformation,
    pub min: f64,
    pub max: f64,
    pub closed: bool,
}

impl Cylinder {
    pub fn new(
        material: Material,
        transformation: Transformation,
        min: impl Into<f64>,
        max: impl Into<f64>,
        closed: bool,
    ) -> Self {
        return Self {
            material,
            transformation_inverse: transformation.inverse(),
            min: min.into(),
            max: max.into(),
            closed,
        };
    }

    fn check_cap(ray: &Ray, distance: impl Into<f64>) -> bool {
        let distance = distance.into();
        let x = ray.direction.x.mul_add(distance, ray.origin.x);
        let z = ray.direction.z.mul_add(distance, ray.origin.z);
        return (x.squared() + z.squared()) <= 1.0;
    }

    fn intersect_caps<'intersections>(
        &'intersections self,
        ray: &Ray,
        intersections: &mut Intersections<'intersections>,
    ) {
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        let distance = (self.min - ray.origin.y) / ray.direction.y;
        if Self::check_cap(ray, distance) {
            intersections.push(Intersection::new(distance, self));
        }

        let distance = (self.max - ray.origin.y) / ray.direction.y;
        if Self::check_cap(ray, distance) {
            intersections.push(Intersection::new(distance, self));
        }
    }
}

impl Transform for Cylinder {
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

impl Intersect for Cylinder {
    fn local_intersect<'shape>(&'shape self, ray: &Ray, intersections: &mut Intersections<'shape>) {
        let a = ray.direction.x.squared() + ray.direction.z.squared();

        if a.abs() > 0.0 {
            let b = 2.0
                * ray
                    .origin
                    .x
                    .mul_add(ray.direction.x, ray.origin.z * ray.direction.z);
            let c = ray.origin.x.squared() + ray.origin.z.squared() - 1.0;

            if let Some((mut distance_1, mut distance_2)) = solve_quadratic(a, b, c) {
                if distance_1 > distance_2 {
                    core::mem::swap(&mut distance_1, &mut distance_2);
                }

                let y1 = distance_1.mul_add(ray.direction.y, ray.origin.y);
                if self.min < y1 && y1 < self.max {
                    intersections.push(Intersection::new(distance_1, self));
                }

                let y2 = distance_2.mul_add(ray.direction.y, ray.origin.y);
                if self.min < y2 && y2 < self.max {
                    intersections.push(Intersection::new(distance_2, self));
                }
            }
        }

        self.intersect_caps(ray, intersections);
    }
}

impl Shape for Cylinder {
    fn local_normal_at(&self, point: Point) -> Vector {
        let distance = point.x.squared() + point.z.squared();

        if distance < 1.0 && point.y >= (self.max - EPSILON) {
            return Vector::UP;
        }

        if distance < 1.0 && point.y <= (self.min + EPSILON) {
            return Vector::DOWN;
        }

        return Vector::new(point.x, 0, point.z);
    }

    fn material(&self) -> &Material {
        return &self.material;
    }
}

impl Default for Cylinder {
    fn default() -> Cylinder {
        return Cylinder::new(
            Material::default(),
            Transformation::IDENTITY,
            MIN,
            MAX,
            false,
        );
    }
}

impl Display for Cylinder {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        return formatter
            .debug_struct("Cylinder")
            .field("min", &self.min)
            .field("max", &self.max)
            .field("closed", &self.closed)
            .field("material", &self.material)
            .field("transformation", &self.transformation())
            .finish();
    }
}

impl CoarseEq for Cylinder {
    fn coarse_eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs)
            || self.material == rhs.material
                && self.closed == rhs.closed
                && self
                    .transformation_inverse
                    .coarse_eq(&rhs.transformation_inverse)
                && self.min.coarse_eq(&rhs.min)
                && self.max.coarse_eq(&rhs.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn default_cylinder() {
        let cylinder = Cylinder::default();
        assert_eq!(cylinder.min, MIN);
        assert_eq!(cylinder.max, MAX);
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
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert!(intersections.is_empty());
    }

    #[rstest]
    #[case(Point::new(1, 0, -5), Vector::FORWARD, 5.0, 5.0)]
    #[case(Point::new(0, 0, -5), Vector::FORWARD, 4.0, 6.0)]
    #[case(Point::new(0.5, 0, -5), Vector::new(0.1, 1, 1), 6.807981917027314, 7.088723439378867)]
    fn ray_intersects_cylinder(
        #[case] origin: Point,
        #[case] direction: Vector,
        #[case] distance_1: f64,
        #[case] distance_2: f64,
    ) {
        let cylinder = Cylinder::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, distance_1);
        assert_eq!(intersections[1].distance, distance_2);
    }

    #[rstest]
    #[case(Point::new(1, 0, 0), Vector::RIGHT)]
    #[case(Point::new(0, 5, -1), Vector::BACKWARD)]
    #[case(Point::new(0, -2, 1), Vector::FORWARD)]
    #[case(Point::new(-1, 1, 0), Vector::LEFT)]
    fn normal_vector_on_cylinder(#[case] point: Point, #[case] expected_normal: Vector) {
        let cylinder = Cylinder::default();
        let normal = cylinder.local_normal_at(point);
        assert_eq!(normal, expected_normal);
    }

    #[rstest]
    #[case(Point::new(0, 1.5, 0), Vector::new(0.1, 1, 0), 0)]
    #[case(Point::new(0, 3, -5), Vector::FORWARD, 0)]
    #[case(Point::new(0, 0, -5), Vector::FORWARD, 0)]
    #[case(Point::new(0, 2, -5), Vector::FORWARD, 0)]
    #[case(Point::new(0, 1, -5), Vector::FORWARD, 0)]
    #[case(Point::new(0, 1.5, -2), Vector::FORWARD, 2)]
    fn intersecting_constrained_cylinder(
        #[case] origin: Point,
        #[case] direction: Vector,
        #[case] count: usize,
    ) {
        let cylinder = Cylinder {
            min: 1.0,
            max: 2.0,
            ..Default::default()
        };
        let boxed_shape: Box<dyn Shape> = Box::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert_eq!(intersections.len(), count);
    }

    #[rstest]
    #[case(Point::new(0, 3, 0), Vector::DOWN, 2)]
    #[case(Point::new(0, 3, -2), Vector::new(0, -1, 2), 2)]
    #[case(Point::new(0, 4, -2), Vector::new(0, -1, 1), 2)]
    #[case(Point::new(0, 0, -2), Vector::new(0, 1, 2), 2)]
    #[case(Point::new(0, -1, -2), Vector::new(0, 1, 1), 2)]
    fn intersecting_caps_of_closed_cylinder(
        #[case] origin: Point,
        #[case] direction: Vector,
        #[case] count: usize,
    ) {
        let mut cylinder = Cylinder::default();
        cylinder.min = 1.0;
        cylinder.max = 2.0;
        cylinder.closed = true;
        let boxed_shape: Box<dyn Shape> = Box::new(cylinder);
        let ray = Ray::new(origin, direction.normalized());
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
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
        cylinder.min = 1.0;
        cylinder.max = 2.0;
        cylinder.closed = true;
        assert_eq!(cylinder.local_normal_at(point), normal);
    }
}
