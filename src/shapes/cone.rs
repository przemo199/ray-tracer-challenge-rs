use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::consts::EPSILON;
use crate::intersection::Intersection;
use crate::intersections::Intersections;
use crate::material::Material;
use crate::primitives::{Matrix, Point, Vector};
use crate::primitives::Transformation;
use crate::ray::Ray;
use crate::utils::CloseEnough;

use super::Shape;

#[derive(Clone, Debug)]
pub struct Cone {
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub transformation: Transformation,
    pub material: Material,
}

impl Cone {
    pub fn new(
        minimum: impl Into<f64>,
        maximum: impl Into<f64>,
        closed: bool, transformation: Transformation,
        material: Material
    ) -> Cone {
        return Cone { minimum: minimum.into(), maximum: maximum.into(), closed, transformation, material };
    }

    fn check_cap(ray: &Ray, distance: impl Into<f64>) -> bool {
        let distance = distance.into();
        let x = ray.origin.x + ray.direction.x * distance;
        let z = ray.origin.z + ray.direction.z * distance;
        return x * x + z * z <= (ray.origin.y + distance * ray.direction.y).abs();
    }

    fn intersect_caps(self: Arc<Self>, ray: &Ray, intersections: &mut Intersections) {
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        let distance = (self.minimum - ray.origin.y) / ray.direction.y;
        if Cone::check_cap(ray, distance) {
            intersections.add(Intersection::new(distance, self.clone()));
        }

        let distance = (self.maximum - ray.origin.y) / ray.direction.y;
        if Cone::check_cap(ray, distance) {
            intersections.add(Intersection::new(distance, self));
        }
    }
}

impl Shape for Cone {
    fn local_normal_at(&self, point: Point) -> Vector {
        let distance = point.x * point.x + point.z * point.z;

        if distance < 1.0 && point.y >= self.maximum - EPSILON {
            return Vector::new(0, 1, 0);
        }

        if distance < 1.0 && point.y <= self.minimum + EPSILON {
            return Vector::new(0, -1, 0);
        }

        let mut y = distance.sqrt();
        if point.y > 0.0 {
            y *= -1.0;
        }

        return Vector::new(point.x, y, point.z);
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
        let a = ray.direction.x.powi(2) -
            ray.direction.y.powi(2) +
            ray.direction.z.powi(2);

        let b = 2.0 * (ray.origin.x * ray.direction.x -
            ray.origin.y * ray.direction.y +
            ray.origin.z * ray.direction.z);

        let c = ray.origin.x.powi(2) -
            ray.origin.y.powi(2) +
            ray.origin.z.powi(2);

        if a.abs() < EPSILON && b.abs() > EPSILON {
            let distance = -c / (2.0 * b);
            intersections.add(Intersection::new(distance, self.clone()));
        } else {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
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
            }
        }

        self.intersect_caps(ray, &mut intersections);
        return intersections;
    }
}

impl Default for Cone {
    fn default() -> Cone {
        return Cone::new(f64::MIN, f64::MAX, false, Matrix::default(), Material::default());
    }
}

impl Display for Cone {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("Cone")
            .field("minimum", &self.minimum)
            .field("maximum", &self.maximum)
            .field("closed", &self.closed)
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .finish();
    }
}

impl PartialEq for Cone {
    fn eq(&self, rhs: &Self) -> bool {
        return self.material == rhs.material &&
            self.transformation == rhs.transformation &&
            self.closed == rhs.closed &&
            self.minimum.close_enough(rhs.minimum) &&
            self.maximum.close_enough(rhs.maximum);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use rstest::rstest;

    use crate::primitives::{Point, Vector};
    use crate::shapes::Shape;

    use super::*;

    #[rstest]
    #[case(Point::new(0, 0, -5), Vector::new(0, 0, 1), 5.0, 5.0)]
    #[case(Point::new(0, 0, -5), Vector::new(1, 1, 1), 8.660254037844386, 8.660254037844386)]
    #[case(Point::new(1, 1, -5), Vector::new(-0.5, -1, 1), 4.550055679356349, 49.449944320643645)]
    fn intersecting_ray_with_cone(
        #[case] origin: Point,
        #[case] direction: Vector,
        #[case] distance_1: f64,
        #[case] distance_2: f64
    ) {
        let cone = Cone::default();
        let arc_cone: Arc<dyn Shape> = Arc::new(cone);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = arc_cone.local_intersect(&ray);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, distance_1);
        assert_eq!(intersections[1].distance, distance_2);
    }

    #[test]
    fn intersecting_ray_with_cone_parallel_to_one_of_cone_halves() {
        let cone = Cone::default();
        let arc_cone: Arc<dyn Shape> = Arc::new(cone);
        let ray = Ray::new(Point::new(0, 0, -1), Vector::new(0, 1, 1).normalized());
        let intersections = arc_cone.local_intersect(&ray);
        assert_eq!(intersections.intersections.len(), 1);
        assert_eq!(intersections[0].distance, 0.3535533905932738);
    }

    #[rstest]
    #[case(Point::new(0, 0, -5), Vector::new(0, 1, 0), 0)]
    #[case(Point::new(0, 0, -0.25), Vector::new(0, 1, 1), 2)]
    #[case(Point::new(0, 0, -0.25), Vector::new(0, 1, 0), 4)]
    fn intersecting_ray_with_cone_caps(#[case] origin: Point, #[case] direction: Vector, #[case] count: usize) {
        let cone = Cone { minimum: -0.5, maximum: 0.5, closed: true, ..Default::default() };
        let arc_cone: Arc<dyn Shape> = Arc::new(cone);
        let ray = Ray::new(origin, direction.normalized());
        let intersections = arc_cone.local_intersect(&ray);
        assert_eq!(intersections.len(), count);
    }

    #[rstest]
    #[case(Point::new(0, 0, 0), Vector::new(0, 0, 0))]
    #[case(Point::new(1, 1, 1), Vector::new(1, -(2.0_f64.sqrt()), 1))]
    #[case(Point::new(-1, -1, 0), Vector::new(-1, 1, 0))]
    fn computing_normal_vector_on_cone(#[case] point: Point, #[case] expected_normal: Vector) {
        let cone = Cone::default();
        let normal = cone.local_normal_at(point);
        assert_eq!(normal, expected_normal);
    }
}