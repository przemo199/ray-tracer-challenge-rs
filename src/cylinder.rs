use std::fmt::{Display, Formatter};
use crate::{Intersection, Intersections, Material, Matrix, Ray, Shape, Tuple, EPSILON};

#[derive(Clone, Debug, PartialEq)]
pub struct Cylinder {
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub transformation: Matrix,
    pub material: Material,
}

impl Cylinder {
    pub fn new(
        minimum: f64,
        maximum: f64,
        closed: bool,
        transformation: Matrix,
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

    fn intersect_caps(&self, ray: &Ray, intersections: &mut Intersections) {
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        let t0 = (self.minimum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, t0) {
            intersections.add(Intersection::new(t0, self.clone()));
        }

        let t1 = (self.maximum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, t1) {
            intersections.add(Intersection::new(t1, self.clone()));
        }
    }
}

impl Shape for Cylinder {
    fn local_normal_at(&self, point: Tuple) -> Tuple {
        let distance = point.x * point.x + point.z * point.z;

        if distance < 1.0 && point.y >= self.maximum - EPSILON {
            return Tuple::vector(0.0, 1.0, 0.0);
        }

        if distance < 1.0 && point.y <= self.minimum + EPSILON {
            return Tuple::vector(0.0, -1.0, 0.0);
        }

        return Tuple::vector(point.x, 0.0, point.z);
    }

    fn material(&self) -> Material {
        return self.material.clone();
    }

    fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    fn transformation(&self) -> Matrix {
        return self.transformation.clone();
    }

    fn set_transformation(&mut self, transformation: Matrix) {
        self.transformation = transformation;
    }

    fn local_intersect(&self, ray: &Ray) -> Intersections {
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
            intersections.add(Intersection::new(t0, self.box_clone()));
        }

        let y1 = ray.origin.y + t1 * ray.direction.y;
        if self.minimum < y1 && y1 < self.maximum {
            intersections.add(Intersection::new(t1, self.box_clone()));
        }

        self.intersect_caps(ray, &mut intersections);
        return intersections;
    }

    fn box_clone(&self) -> Box<dyn Shape> {
        return Box::new(self.clone());
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

impl From<Cylinder> for Box<dyn Shape> {
    fn from(cylinder: Cylinder) -> Box<dyn Shape> {
        return Box::new(cylinder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TupleTrait;

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
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(0.0, 0.0, -5.0)];
        let directions = [
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(1.0, 1.0, 1.0)];

        for (origin, direction) in origins.iter().zip(directions.iter()) {
            let cylinder = Cylinder::default();
            let ray = Ray::new(*origin, direction.normalize());
            let intersections = cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), 0);
        }
    }

    #[test]
    fn ray_intersects_cylinder() {
        let origins = [
            Tuple::point(1.0, 0.0, -5.0),
            Tuple::point(0.0, 0.0, -5.0),
            Tuple::point(0.5, 0.0, -5.0)];
        let directions = [
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.1, 1.0, 1.0)];
        let t0s = [5.0, 4.0, 6.80798191702732];
        let t1s = [5.0, 6.0, 7.088723439378861];

        for i in 0..origins.len() {
            let cylinder = Cylinder::default();
            let ray = Ray::new(origins[i], directions[i].normalize());
            let intersections = cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), 2);
            assert_eq!(intersections[0].t, t0s[i]);
            assert_eq!(intersections[1].t, t1s[i]);
        }
    }

    #[test]
    fn normal_vector_on_cylinder() {
        let points = [
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::point(0.0, 5.0, -1.0),
            Tuple::point(0.0, -2.0, 1.0),
            Tuple::point(-1.0, 1.0, 0.0)];
        let normals = [
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(-1.0, 0.0, 0.0)];

        for (point, normal) in points.iter().zip(normals.iter()) {
            let cylinder = Cylinder::default();
            let local_normal = cylinder.local_normal_at(*point);
            assert_eq!(local_normal, *normal);
        }
    }

    #[test]
    fn intersecting_constraint_cylinder() {
        let points = [
            Tuple::point(0.0, 1.5, 0.0),
            Tuple::point(0.0, 3.0, -5.0),
            Tuple::point(0.0, 0.0, -5.0),
            Tuple::point(0.0, 2.0, -5.0),
            Tuple::point(0.0, 1.0, -5.0),
            Tuple::point(0.0, 1.5, -2.0)];
        let directions = [
            Tuple::vector(0.1, 1.0, 0.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, 1.0)];
        let counts = [0, 0, 0, 0, 0, 2];

        for ((point, direction), count) in points.iter().zip(directions.iter()).zip(counts.iter()) {
            let mut cylinder = Cylinder::default();
            cylinder.minimum = 1.0;
            cylinder.maximum = 2.0;
            let ray = Ray::new(*point, direction.normalize());
            let intersections = cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), *count);
        }
    }

    #[test]
    fn intersecting_caps_of_closed_cylinder() {
        let points = [
            Tuple::point(0.0, 3.0, 0.0),
            Tuple::point(0.0, 3.0, -2.0),
            Tuple::point(0.0, 4.0, -2.0),
            Tuple::point(0.0, 0.0, -2.0),
            Tuple::point(0.0, -1.0, -2.0)];
        let directions = [
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, -1.0, 2.0),
            Tuple::vector(0.0, -1.0, 1.0),
            Tuple::vector(0.0, 1.0, 2.0),
            Tuple::vector(0.0, 1.0, 1.0)];
        let counts = [2, 2, 2, 2, 2];

        for ((point, direction), count) in points.iter().zip(directions.iter()).zip(counts.iter()) {
            let mut cylinder = Cylinder::default();
            cylinder.minimum = 1.0;
            cylinder.maximum = 2.0;
            cylinder.closed = true;
            let ray = Ray::new(*point, direction.normalize());
            let intersections = cylinder.local_intersect(&ray);
            assert_eq!(intersections.len(), *count);
        }
    }

    #[test]
    fn normal_vector_on_cylinder_caps() {
        let points = [
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(0.5, 1.0, 0.0),
            Tuple::point(0.0, 1.0, 0.5),
            Tuple::point(0.0, 2.0, 0.0),
            Tuple::point(0.5, 2.0, 0.0),
            Tuple::point(0.0, 2.0, 0.5)];
        let normals = [
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0)];

        for (point, normal) in points.iter().zip(normals.iter()) {
            let mut cylinder = Cylinder::default();
            cylinder.minimum = 1.0;
            cylinder.maximum = 2.0;
            cylinder.closed = true;
            assert_eq!(cylinder.local_normal_at(*point), *normal);
        }
    }
}
