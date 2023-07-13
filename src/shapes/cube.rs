use super::Shape;
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::{BINCODE_CONFIG, EPSILON};
use crate::primitives::Transformation;
use crate::primitives::{Matrix, Point, Vector};
use bincode::Encode;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct Cube {
    pub material: Material,
    pub transformation: Matrix<4>,
}

impl Cube {
    pub fn new(material: Material, transformation: Matrix<4>) -> Cube {
        return Cube {
            material,
            transformation,
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
            distance_min = distance_min_numerator * f64::INFINITY;
            distance_max = distance_max_numerator * f64::INFINITY;
        }

        if distance_min > distance_max {
            std::mem::swap(&mut distance_min, &mut distance_max);
        }

        return (distance_min, distance_max);
    }
}

impl Shape for Cube {
    fn local_normal_at(&self, point: Point) -> Vector {
        let max_value = [point.x.abs(), point.y.abs(), point.z.abs()]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        if max_value == point.x.abs() {
            return Vector::new(point.x, 0, 0);
        } else if max_value == point.y.abs() {
            return Vector::new(0, point.y, 0);
        }
        return Vector::new(0, 0, point.z);
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
        let (x_distance_min, x_distance_max) = Cube::check_axis(ray.origin.x, ray.direction.x);
        let (y_distance_min, y_distance_max) = Cube::check_axis(ray.origin.y, ray.direction.y);
        let (z_distance_min, z_distance_max) = Cube::check_axis(ray.origin.z, ray.direction.z);

        let distance_min = [x_distance_min, y_distance_min, z_distance_min]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let distance_max = [x_distance_max, y_distance_max, z_distance_max]
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);

        if distance_min > distance_max {
            return None;
        } else {
            return Some(Intersections::with([
                Intersection::new(distance_min, self),
                Intersection::new(distance_max, self),
            ]));
        }
    }

    fn encoded(&self) -> Vec<u8> {
        return bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap();
    }
}

impl Default for Cube {
    fn default() -> Cube {
        return Cube::new(Material::default(), Matrix::IDENTITY);
    }
}

impl Display for Cube {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter
            .debug_struct("Cube")
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        #[case] t_1: impl Into<f64>,
        #[case] t_2: impl Into<f64>,
    ) {
        let cube = Cube::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cube);
        let ray = Ray::new(origin, direction);
        let intersections = boxed_shape.local_intersect(&ray).unwrap();
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, t_1.into());
        assert_eq!(intersections[1].distance, t_2.into());
    }

    #[rstest]
    #[case(Point::new(-2, 0, 0), Vector::new(0.2673, 0.5345, 0.8018))]
    #[case(Point::new(0, -2, 0), Vector::new(0.8018, 0.2673, 0.5345))]
    #[case(Point::new(0, 0, -2), Vector::new(0.5345, 0.8018, 0.2673))]
    #[case(Point::new(2, 0, 2), Vector::BACKWARD)]
    #[case(Point::new(0, 2, 2), Vector::DOWN)]
    #[case(Point::new(2, 2, 0), Vector::LEFT)]
    fn ray_misses_cube(#[case] origin: Point, #[case] direction: Vector) {
        let cube = Cube::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cube);
        let ray = Ray::new(origin, direction);
        let intersections = boxed_shape.local_intersect(&ray);
        assert_eq!(intersections, None);
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
