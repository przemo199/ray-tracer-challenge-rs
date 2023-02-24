use std::fmt::{Display, Formatter};
use std::sync::Arc;
use crate::consts::EPSILON;
use crate::intersection::Intersection;
use crate::intersections::Intersections;
use crate::material::Material;
use crate::matrix::Matrix;
use crate::point::Point;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::utils::CloseEnough;
use crate::vector::Vector;

#[derive(Clone, Debug)]
pub struct Cone {
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub transformation: Matrix<4>,
    pub material: Material,
}

impl Cone {
    pub fn new(
        minimum: f64,
        maximum: f64,
        closed: bool,
        transformation: Matrix<4>,
        material: Material,
    ) -> Cone {
        return Cone {
            minimum,
            maximum,
            closed,
            transformation,
            material,
        };
    }

    fn check_cap(ray: &Ray, t: f64) -> bool {
        let x = ray.origin.x + ray.direction.x * t;
        let z = ray.origin.z + ray.direction.z * t;
        return x * x + z * z <= (ray.origin.y + t * ray.direction.y).abs();
    }

    fn intersect_caps(self: Arc<Self>, ray: &Ray, intersections: &mut Intersections) {
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        let t0 = (self.minimum - ray.origin.y) / ray.direction.y;
        if Cone::check_cap(ray, t0) {
            intersections.add(Intersection::new(t0, self.clone()));
        }

        let t1 = (self.maximum - ray.origin.y) / ray.direction.y;
        if Cone::check_cap(ray, t1) {
            intersections.add(Intersection::new(t1, self));
        }
    }
}

impl Shape for Cone {
    fn local_normal_at(&self, point: Point) -> Vector {
        let distance = point.x * point.x + point.z * point.z;

        if distance < 1.0 && point.y >= self.maximum - EPSILON {
            return Vector::new(0.0, 1.0, 0.0);
        }

        if distance < 1.0 && point.y <= self.minimum + EPSILON {
            return Vector::new(0.0, -1.0, 0.0);
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

    fn transformation(&self) -> Matrix<4> {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Matrix<4>) {
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
            let t = -c / (2.0 * b);
            intersections.add(Intersection::new(t, self.clone()));
        } else {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
                let double_a = 2.0 * a;
                let discriminant_sqrt = discriminant.sqrt();
                let mut t0 = (-b - discriminant_sqrt) / double_a;
                let mut t1 = (-b + discriminant_sqrt) / double_a;
                if t0 > t1 {
                    std::mem::swap(&mut t0, &mut t1);
                }

                let y0 = ray.origin.y + t0 * ray.direction.y;
                if self.minimum < y0 && y0 < self.maximum {
                    intersections.add(Intersection::new(t0, self.clone()));
                }

                let y1 = ray.origin.y + t1 * ray.direction.y;
                if self.minimum < y1 && y1 < self.maximum {
                    intersections.add(Intersection::new(t1, self.clone()));
                }
            }
        }

        self.intersect_caps(ray, &mut intersections);
        return intersections;
    }
}

impl Default for Cone {
    fn default() -> Cone {
        return Cone::new(
            f64::NEG_INFINITY,
            f64::INFINITY,
            false,
            Matrix::default(),
            Material::default(),
        );
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
    use super::*;
    use std::sync::Arc;

    #[test]
    fn intersecting_ray_with_cone() {
        let origins = [
            Point::new(0.0, 0.0, -5.0),
            Point::new(0.0, 0.0, -5.0),
            Point::new(1.0, 1.0, -5.0)];
        let directions = [
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(1.0, 1.0, 1.0),
            Vector::new(-0.5, -1.0, 1.0)];
        let t0s = [5.0, 8.660254037844386, 4.550055679356349];
        let t1s = [5.0, 8.660254037844386, 49.449944320643645];

        for i in 0..origins.len() {
            let cone = Cone::default();
            let arc_cone: Arc<dyn Shape> = Arc::new(cone);
            let ray = Ray::new(origins[i], directions[i].normalize());
            let intersections = arc_cone.local_intersect(&ray);
            assert_eq!(intersections.len(), 2);
            assert_eq!(intersections[0].t, t0s[i]);
            assert_eq!(intersections[1].t, t1s[i]);
        }
    }

    #[test]
    fn intersecting_ray_with_cone_parallel_to_one_of_cone_halves() {
        let cone = Cone::default();
        let arc_cone: Arc<dyn Shape> = Arc::new(cone);
        let ray = Ray::new(Point::new(0.0, 0.0, -1.0), Vector::new(0.0, 1.0, 1.0).normalize());
        let intersections = arc_cone.local_intersect(&ray);
        assert_eq!(intersections.intersections.len(), 1);
        assert_eq!(intersections[0].t, 0.3535533905932738);
    }

    #[test]
    fn intersecting_ray_with_cone_caps() {
        let origins = [
            Point::new(0.0, 0.0, -5.0),
            Point::new(0.0, 0.0, -0.25),
            Point::new(0.0, 0.0, -0.25)];
        let directions = [
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 1.0, 1.0),
            Vector::new(0.0, 1.0, 0.0)];
        let count = [0, 2, 4];

        for i in 0..origins.len() {
            let cone = Cone { minimum: -0.5, maximum: 0.5, closed: true, ..Default::default() };
            let arc_cone: Arc<dyn Shape> = Arc::new(cone);
            let ray = Ray::new(origins[i], directions[i].normalize());
            let intersections = arc_cone.local_intersect(&ray);
            assert_eq!(intersections.len(), count[i]);
        }
    }

    #[test]
    fn computing_normal_vector_on_cone() {
        let points = [
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 1.0),
            Point::new(-1.0, -1.0, 0.0)];
        let normals = [
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(1.0, -(2.0_f64.sqrt()), 1.0),
            Vector::new(-1.0, 1.0, 0.0)];

        for i in 0..points.len() {
            let cone = Cone::default();
            let normal = cone.local_normal_at(points[i]);
            assert_eq!(normal, normals[i]);
        }
    }
}
