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
use crate::vector::Vector;

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

    fn check_cap(ray: &Ray, t: f64) -> bool {
        let x = ray.origin.x + ray.direction.x * t;
        let z = ray.origin.z + ray.direction.z * t;
        return x * x + z * z <= 1.0;
    }

    fn intersect_caps(self: Arc<Self>, ray: &Ray, intersections: &mut Intersections) {
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        let t0 = (self.minimum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, t0) {
            intersections.add(Intersection::new(t0, self.clone()));
        }

        let t1 = (self.maximum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, t1) {
            intersections.add(Intersection::new(t1, self));
        }
    }
}

impl Shape for Cylinder {
    fn local_normal_at(&self, point: Point) -> Vector {
        let distance = point.x * point.x + point.z * point.z;

        if distance < 1.0 && point.y >= self.maximum - EPSILON {
            return Vector::new(0.0, 1.0, 0.0);
        }

        if distance < 1.0 && point.y <= self.minimum + EPSILON {
            return Vector::new(0.0, -1.0, 0.0);
        }

        return Vector::new(point.x, 0.0, point.z);
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

        self.intersect_caps(ray, &mut intersections);
        return intersections;
    }
}

impl Default for Cylinder {
    fn default() -> Cylinder {
        return Cylinder::new(
            f64::NEG_INFINITY,
            f64::INFINITY,
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
    use super::*;

    #[test]
    fn default_cylinder() {
        let cylinder = Cylinder::default();
        assert_eq!(cylinder.minimum, f64::NEG_INFINITY);
        assert_eq!(cylinder.maximum, f64::INFINITY);
        assert!(!cylinder.closed);
    }

    #[test]
    fn ray_misses_cylinder() {
        let origins = [
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
            Point::new(0.0, 0.0, -5.0)];
        let directions = [
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(1.0, 1.0, 1.0)];

        for (origin, direction) in origins.iter().zip(directions) {
            let cylinder = Cylinder::default();
            let arc_cylinder: Arc<dyn Shape> = Arc::new(cylinder);
            let ray = Ray::new(*origin, direction.normalize());
            let intersections = arc_cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), 0);
        }
    }

    #[test]
    fn ray_intersects_cylinder() {
        let origins = [
            Point::new(1.0, 0.0, -5.0),
            Point::new(0.0, 0.0, -5.0),
            Point::new(0.5, 0.0, -5.0)];
        let directions = [
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(0.1, 1.0, 1.0)];
        let t0s = [5.0, 4.0, 6.80798191702732];
        let t1s = [5.0, 6.0, 7.088723439378861];

        for i in 0..origins.len() {
            let cylinder = Cylinder::default();
            let arc_cylinder: Arc<dyn Shape> = Arc::new(cylinder);
            let ray = Ray::new(origins[i], directions[i].normalize());
            let intersections = arc_cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), 2);
            assert_eq!(intersections[0].t, t0s[i]);
            assert_eq!(intersections[1].t, t1s[i]);
        }
    }

    #[test]
    fn normal_vector_on_cylinder() {
        let points = [
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 5.0, -1.0),
            Point::new(0.0, -2.0, 1.0),
            Point::new(-1.0, 1.0, 0.0)];
        let normals = [
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(0.0, 0.0, -1.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(-1.0, 0.0, 0.0)];

        for (point, normal) in points.iter().zip(normals.iter()) {
            let cylinder = Cylinder::default();
            let local_normal = cylinder.local_normal_at(*point);
            assert_eq!(local_normal, *normal);
        }
    }

    #[test]
    fn intersecting_constraint_cylinder() {
        let points = [
            Point::new(0.0, 1.5, 0.0),
            Point::new(0.0, 3.0, -5.0),
            Point::new(0.0, 0.0, -5.0),
            Point::new(0.0, 2.0, -5.0),
            Point::new(0.0, 1.0, -5.0),
            Point::new(0.0, 1.5, -2.0)];
        let directions = [
            Vector::new(0.1, 1.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(0.0, 0.0, 1.0)];
        let counts = [0, 0, 0, 0, 0, 2];

        for ((point, direction), count) in points.iter().zip(directions.iter()).zip(counts.iter()) {
            let cylinder = Cylinder {
                minimum: 1.0,
                maximum: 2.0,
                ..Default::default()
            };
            let arc_cylinder: Arc<dyn Shape> = Arc::new(cylinder);
            let ray = Ray::new(*point, direction.normalize());
            let intersections = arc_cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), *count);
        }
    }

    #[test]
    fn intersecting_caps_of_closed_cylinder() {
        let points = [
            Point::new(0.0, 3.0, 0.0),
            Point::new(0.0, 3.0, -2.0),
            Point::new(0.0, 4.0, -2.0),
            Point::new(0.0, 0.0, -2.0),
            Point::new(0.0, -1.0, -2.0)];
        let directions = [
            Vector::new(0.0, -1.0, 0.0),
            Vector::new(0.0, -1.0, 2.0),
            Vector::new(0.0, -1.0, 1.0),
            Vector::new(0.0, 1.0, 2.0),
            Vector::new(0.0, 1.0, 1.0)];
        let counts = [2, 2, 2, 2, 2];

        for ((point, direction), count) in points.iter().zip(directions.iter()).zip(counts.iter()) {
            let mut cylinder = Cylinder::default();
            cylinder.minimum = 1.0;
            cylinder.maximum = 2.0;
            cylinder.closed = true;
            let arc_cylinder: Arc<dyn Shape> = Arc::new(cylinder);
            let ray = Ray::new(*point, direction.normalize());
            let intersections = arc_cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), *count);
        }
    }

    #[test]
    fn normal_vector_on_cylinder_caps() {
        let points = [
            Point::new(0.0, 1.0, 0.0),
            Point::new(0.5, 1.0, 0.0),
            Point::new(0.0, 1.0, 0.5),
            Point::new(0.0, 2.0, 0.0),
            Point::new(0.5, 2.0, 0.0),
            Point::new(0.0, 2.0, 0.5)];
        let normals = [
            Vector::new(0.0, -1.0, 0.0),
            Vector::new(0.0, -1.0, 0.0),
            Vector::new(0.0, -1.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 1.0, 0.0)];

        for (point, normal) in points.iter().zip(normals.iter()) {
            let mut cylinder = Cylinder::default();
            cylinder.minimum = 1.0;
            cylinder.maximum = 2.0;
            cylinder.closed = true;
            assert_eq!(cylinder.local_normal_at(*point), *normal);
        }
    }
}
