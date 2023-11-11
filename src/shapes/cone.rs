use super::Shape;
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::{BINCODE_CONFIG, EPSILON};
use crate::primitives::Transformation;
use crate::primitives::{Matrix, Point, Vector};
use crate::utils::{solve_quadratic, CoarseEq, Squared};
use bincode::Encode;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, Encode)]
pub struct Cone {
    pub transformation: Transformation,
    pub material: Material,
    pub min: f64,
    pub max: f64,
    pub closed: bool,
}

impl Cone {
    pub fn new(
        material: Material,
        transformation: Transformation,
        min: impl Into<f64>,
        max: impl Into<f64>,
        closed: bool,
    ) -> Self {
        return Self {
            transformation,
            material,
            min: min.into(),
            max: max.into(),
            closed,
        };
    }

    fn check_caps(ray: &Ray, distance: impl Into<f64>, radius: impl Into<f64>) -> bool {
        let distance = distance.into();
        let x = ray.direction.x.mul_add(distance, ray.origin.x);
        let z = ray.direction.z.mul_add(distance, ray.origin.z);
        return x.squared() + z.squared() <= radius.into().squared();
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
        if Self::check_caps(ray, distance, self.min) {
            intersections.push(Intersection::new(distance, self));
        }

        let distance = (self.max - ray.origin.y) / ray.direction.y;
        if Self::check_caps(ray, distance, self.max) {
            intersections.push(Intersection::new(distance, self));
        }
    }
}

impl Shape for Cone {
    fn local_normal_at(&self, point: Point) -> Vector {
        let distance = point.x.squared() + point.z.squared();

        if distance < self.max.squared() && point.y >= (self.max - EPSILON) {
            return Vector::UP;
        }

        if distance < self.min.squared() && point.y <= (self.min + EPSILON) {
            return Vector::DOWN;
        }

        let mut y = distance.sqrt();
        if point.y > 0.0 {
            y = -y;
        }

        return Vector::new(point.x, y, point.z);
    }

    fn material(&self) -> &Material {
        return &self.material;
    }

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn local_intersect(&self, ray: &Ray) -> Option<Intersections> {
        let a = ray.direction.x.squared() - ray.direction.y.squared() + ray.direction.z.squared();
        let b = 2.0
            * ray.origin.z.mul_add(
                ray.direction.z,
                ray.origin
                    .x
                    .mul_add(ray.direction.x, -ray.origin.y * ray.direction.y),
            );
        let c = ray.origin.x.squared() - ray.origin.y.squared() + ray.origin.z.squared();

        let mut intersections = Intersections::new();
        if a.abs() < EPSILON && b.abs() > EPSILON {
            let distance = -c / (2.0 * b);
            intersections.push(Intersection::new(distance, self));
        } else if let Some((mut distance_1, mut distance_2)) = solve_quadratic(a, b, c) {
            if distance_1 > distance_2 {
                core::mem::swap(&mut distance_1, &mut distance_2);
            }

            let y1 = ray.direction.y.mul_add(distance_1, ray.origin.y);
            if self.min < y1 && y1 < self.max {
                intersections.push(Intersection::new(distance_1, self));
            }

            let y2 = ray.direction.y.mul_add(distance_2, ray.origin.y);
            if self.min < y2 && y2 < self.max {
                intersections.push(Intersection::new(distance_2, self));
            }
        }

        self.intersect_caps(ray, &mut intersections);
        return intersections.into_option();
    }

    fn encoded(&self) -> Vec<u8> {
        return bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap();
    }
}

impl Default for Cone {
    fn default() -> Cone {
        return Cone::new(
            Material::default(),
            Matrix::default(),
            f64::NEG_INFINITY,
            f64::INFINITY,
            false,
        );
    }
}

impl Display for Cone {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("Cone")
            .field("min", &self.min)
            .field("max", &self.max)
            .field("closed", &self.closed)
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .finish();
    }
}

impl PartialEq for Cone {
    fn eq(&self, rhs: &Self) -> bool {
        return self.material == rhs.material
            && self.transformation == rhs.transformation
            && self.closed == rhs.closed
            && self.min.coarse_eq(rhs.min)
            && self.max.coarse_eq(rhs.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn default_cone() {
        let cone = Cone::default();
        assert_eq!(cone.min, f64::NEG_INFINITY);
        assert_eq!(cone.max, f64::INFINITY);
        assert!(!cone.closed);
    }

    #[rstest]
    #[case(Point::new(0, 0, -5), Vector::FORWARD, 5.0, 5.0)]
    #[case(Point::new(0, 0, -5), Vector::new(1, 1, 1), 8.660254015492644, 8.660254060196127)]
    #[case(Point::new(1, 1, -5), Vector::new(-0.5, -1, 1), 4.550055679356354, 49.44994432064365)]
    fn intersecting_ray_with_cone(
        #[case] origin: Point,
        #[case] direction: Vector,
        #[case] distance_1: f64,
        #[case] distance_2: f64,
    ) {
        let cone = Cone::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cone);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = boxed_shape.local_intersect(&ray).unwrap();
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, distance_1);
        assert_eq!(intersections[1].distance, distance_2);
    }

    #[test]
    fn intersecting_ray_with_cone_parallel_to_one_of_cone_halves() {
        let cone = Cone::default();
        let boxed_shape: Box<dyn Shape> = Box::new(cone);
        let ray = Ray::new(Point::new(0, 0, -1), Vector::new(0, 1, 1).normalized());
        let intersections = boxed_shape.local_intersect(&ray).unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 0.3535533905932738);
    }

    #[rstest]
    #[case(Point::new(0, 0, -5), Vector::UP, 0)]
    #[case(Point::new(0, 0, -0.25), Vector::new(0, 1, 1), 2)]
    #[case(Point::new(0, 0, -0.25), Vector::UP, 4)]
    fn intersecting_ray_with_cone_caps(
        #[case] origin: Point,
        #[case] direction: Vector,
        #[case] count: usize,
    ) {
        let cone = Cone {
            min: -0.5,
            max: 0.5,
            closed: true,
            ..Default::default()
        };
        let boxed_shape: Box<dyn Shape> = Box::new(cone);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = boxed_shape.local_intersect(&ray);
        if count > 0 {
            assert_eq!(intersections.unwrap().len(), count);
        } else {
            assert_eq!(intersections, None);
        }
    }

    #[rstest]
    #[case(Point::ORIGIN, Vector::ZERO)]
    #[case(Point::new(1, 1, 1), Vector::new(1, -(2.0_f64.sqrt()), 1))]
    #[case(Point::new(-1, -1, 0), Vector::new(-1, 1, 0))]
    fn computing_normal_vector_on_cone(#[case] point: Point, #[case] expected_normal: Vector) {
        let cone = Cone::default();
        let normal = cone.local_normal_at(point);
        assert_eq!(normal, expected_normal);
    }
}
