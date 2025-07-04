use super::{Intersect, Shape, Transform};
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::{EPSILON, MAX, MIN};
use crate::primitives::{Point, Transformation, Vector};
use crate::utils::CoarseEq;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Cube {
    pub material: Material,
    transformation_inverse: Transformation,
}

impl Cube {
    pub fn new(material: Material, transformation: Transformation) -> Self {
        return Self {
            material,
            transformation_inverse: transformation.inverse(),
        };
    }

    pub fn check_axis(origin: impl Into<f64>, direction: impl Into<f64>) -> (f64, f64) {
        let origin = origin.into();
        let direction = direction.into();
        let distance_min_numerator = -1.0 - origin;
        let distance_max_numerator = 1.0 - origin;
        let mut distance_max: f64;
        let mut distance_min: f64;

        if direction.abs() >= EPSILON {
            distance_min = distance_min_numerator / direction;
            distance_max = distance_max_numerator / direction;
        } else {
            distance_min = distance_min_numerator * MAX;
            distance_max = distance_max_numerator * MAX;
        }

        if distance_min > distance_max {
            core::mem::swap(&mut distance_min, &mut distance_max);
        }

        return (distance_min, distance_max);
    }
}

impl Transform for Cube {
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

impl Intersect for Cube {
    fn local_intersect<'shape>(&'shape self, ray: &Ray, intersections: &mut Intersections<'shape>) {
        let (x_distance_min, x_distance_max) = Self::check_axis(ray.origin.x, ray.direction.x);
        let (y_distance_min, y_distance_max) = Self::check_axis(ray.origin.y, ray.direction.y);
        let (z_distance_min, z_distance_max) = Self::check_axis(ray.origin.z, ray.direction.z);

        let distance_min = [x_distance_min, y_distance_min, z_distance_min]
            .iter()
            .copied()
            .fold(MIN, f64::max);
        let distance_max = [x_distance_max, y_distance_max, z_distance_max]
            .iter()
            .copied()
            .fold(MAX, f64::min);

        if distance_min < distance_max && distance_max > 0.0 {
            intersections.extend([
                Intersection::new(distance_min, self),
                Intersection::new(distance_max, self),
            ]);
        }
    }
}

impl Shape for Cube {
    fn local_normal_at(&self, point: Point) -> Vector {
        let abs_point = point.abs();
        let max_value = Into::<[f64; 3]>::into(abs_point)
            .into_iter()
            .fold(MIN, f64::max);

        if max_value.coarse_eq(&abs_point.x) {
            return Vector::new(point.x, 0, 0);
        } else if max_value.coarse_eq(&abs_point.y) {
            return Vector::new(0, point.y, 0);
        }
        return Vector::new(0, 0, point.z);
    }

    fn material(&self) -> &Material {
        return &self.material;
    }
}

impl Default for Cube {
    fn default() -> Cube {
        return Cube::new(Material::default(), Transformation::IDENTITY);
    }
}

impl Display for Cube {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        return formatter
            .debug_struct("Cube")
            .field("material", &self.material)
            .field("transformation", &self.transformation())
            .finish();
    }
}

impl CoarseEq for Cube {
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
    use crate::primitives::Vector;
    use rstest::rstest;

    #[rstest]
    #[case(Point::new(5, 0.5, 0), Vector::LEFT, 4, 6)]
    #[case(Point::new(-5, 0.5, 0), Vector::RIGHT, 4, 6)]
    #[case(Point::new(0.5, 5, 0), Vector::DOWN, 4, 6)]
    #[case(Point::new(0.5, -5, 0), Vector::UP, 4, 6)]
    #[case(Point::new(0.5, 0, 5), Vector::BACKWARD, 4, 6)]
    #[case(Point::new(0.5, 0, -5), Vector::FORWARD, 4, 6)]
    #[case(Point::new(0, 0.5, 0), Vector::FORWARD, -1, 1)]
    fn ray_intersects_cube(
        #[case] origin: Point,
        #[case] direction: Vector,
        #[case] distance_1: impl Into<f64>,
        #[case] distance_2: impl Into<f64>,
    ) {
        let cube = Cube::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cube);
        let ray = Ray::new(origin, direction);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, distance_1.into());
        assert_eq!(intersections[1].distance, distance_2.into());
    }

    #[rstest]
    #[case(Point::new(-2, 0, 0), Vector::new(0.2673, 0.5345, 0.8018))]
    #[case(Point::new(0, -2, 0), Vector::new(0.8018, 0.2673, 0.5345))]
    #[case(Point::new(0, 0, -2), Vector::new(0.5345, 0.8018, 0.2673))]
    #[case(Point::new(2, 0, 2), Vector::BACKWARD)]
    #[case(Point::new(0, 2, 2), Vector::DOWN)]
    #[case(Point::new(2, 2, 0), Vector::LEFT)]
    #[case(Point::new(0, 0, 2), Vector::new(0, 0, 1))]
    fn ray_misses_cube(#[case] origin: Point, #[case] direction: Vector) {
        let cube = Cube::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cube);
        let ray = Ray::new(origin, direction);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert!(intersections.is_empty());
    }

    #[rstest]
    #[case(Point::new(1, 0.5, -0.8), Vector::RIGHT)]
    #[case(Point::new(-1, -0.2, 0.9), Vector::LEFT)]
    #[case(Point::new(-0.4, 1, -0.1), Vector::UP)]
    #[case(Point::new(0.3, -1, -0.7), Vector::DOWN)]
    #[case(Point::new(-0.6, 0.3, 1), Vector::FORWARD)]
    #[case(Point::new(0.4, 0.4, -1), Vector::BACKWARD)]
    #[case(Point::new(1, 1, 1), Vector::RIGHT)]
    #[case(Point::new(-1, -1, -1), Vector::LEFT)]
    fn normal_on_surface_of_cube(#[case] point: Point, #[case] normal: Vector) {
        let cube = Cube::default();
        let cube_normal = cube.local_normal_at(point);
        assert_eq!(cube_normal, normal);
    }
}
