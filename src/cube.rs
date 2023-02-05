use std::fmt::{Display, Formatter};
use std::sync::Arc;
use crate::consts::EPSILON;
use crate::intersection::Intersection;
use crate::intersections::Intersections;
use crate::material::Material;
use crate::matrix::Matrix;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::tuple::Tuple;

#[derive(Clone, Debug, PartialEq)]
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

    pub fn check_axis(origin: f64, direction: f64) -> (f64, f64) {
        let mut t_max: f64;
        let mut t_min: f64;
        let t_min_numerator = -1.0 - origin;
        let t_max_numerator = 1.0 - origin;

        if direction.abs() >= EPSILON {
            t_min = t_min_numerator / direction;
            t_max = t_max_numerator / direction;
        } else {
            t_min = t_min_numerator * f64::INFINITY;
            t_max = t_max_numerator * f64::INFINITY;
        }

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        return (t_min, t_max);
    }
}

impl Shape for Cube {
    fn local_normal_at(&self, point: Tuple) -> Tuple {
        let max_value = [point.x.abs(), point.y.abs(), point.z.abs()].iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if max_value == point.x.abs() {
            return Tuple::vector(point.x, 0.0, 0.0)
        } else if max_value == point.y.abs() {
            return Tuple::vector(0.0, point.y, 0.0)
        }
        return Tuple::vector(0.0, 0.0, point.z)
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
        let (x_tmin, x_tmax) = Cube::check_axis(ray.origin.x, ray.direction.x);
        let (y_tmin, y_tmax) = Cube::check_axis(ray.origin.y, ray.direction.y);
        let (z_tmin, z_tmax) = Cube::check_axis(ray.origin.z, ray.direction.z);

        let t_min = [x_tmin, y_tmin, z_tmin].iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let t_max = [x_tmax, y_tmax, z_tmax].iter().copied().fold(f64::INFINITY, f64::min);

        let mut result = Intersections::new();
        if t_min > t_max {
            return result;
        }
        result.add(Intersection::new(t_min, self.clone()));
        result.add(Intersection::new(t_max, self));
        return result;
    }
}

impl Default for Cube {
    fn default() -> Cube {
        return Cube::new(Material::default(), Matrix::identity());
    }
}

impl Display for Cube {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("Cube")
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .finish();
    }
}

impl From<Cube> for Box<dyn Shape> {
    fn from(cube: Cube) -> Box<dyn Shape> {
        return Box::new(cube);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuple::Tuple;

    #[test]
    fn ray_intersects_cube() {
        let origins = [
            Tuple::point(5.0, 0.5, 0.0),
            Tuple::point(-5.0, 0.5, 0.0),
            Tuple::point(0.5, 5.0, 0.0),
            Tuple::point(0.5, -5.0, 0.0),
            Tuple::point(0.5, 0.0, 5.0),
            Tuple::point(0.5, 0.0, -5.0),
            Tuple::point(0.0, 0.5, 0.0)];
        let directions = [
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, 1.0)];
        let t1s = [4.0, 4.0, 4.0, 4.0, 4.0, 4.0, -1.0];
        let t2s = [6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 1.0];

        for i in 0..origins.len() {
            let cube = Cube::default();
            let arc_cube: Arc<dyn Shape> = Arc::new(cube);
            let ray = Ray::new(origins[i], directions[i]);
            let intersections = arc_cube.local_intersect(&ray);
            assert_eq!(intersections.len(), 2);
            assert_eq!(intersections[0].t, t1s[i]);
            assert_eq!(intersections[1].t, t2s[i]);
        }
    }

    #[test]
    fn ray_misses_cube() {
        let origins = [
            Tuple::point(-2.0, 0.0, 0.0),
            Tuple::point(0.0, -2.0, 0.0),
            Tuple::point(0.0, 0.0, -2.0),
            Tuple::point(2.0, 0.0, 2.0),
            Tuple::point(0.0, 2.0, 2.0),
            Tuple::point(2.0, 2.0, 0.0)];
        let directions = [
            Tuple::vector(0.2673, 0.5345, 0.8018),
            Tuple::vector(0.8018, 0.2673, 0.5345),
            Tuple::vector(0.5345, 0.8018, 0.2673),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0)];
        for i in 0..origins.len() {
            let cube = Cube::default();
            let arc_cube: Arc<dyn Shape> = Arc::new(cube);
            let ray = Ray::new(origins[i], directions[i]);
            let intersections = arc_cube.local_intersect(&ray);
            assert_eq!(intersections.len(), 0);
        }
    }

    #[test]
    fn normal_on_surface_of_cube() {
        let points = [
            Tuple::point(1.0, 0.5, -0.8),
            Tuple::point(-1.0, -0.2, 0.9),
            Tuple::point(-0.4, 1.0, -0.1),
            Tuple::point(0.3, -1.0, -0.7),
            Tuple::point(-0.6, 0.3, 1.0),
            Tuple::point(0.4, 0.4, -1.0),
            Tuple::point(1.0, 1.0, 1.0),
            Tuple::point(-1.0, -1.0, -1.0),
        ];
        let normals = [
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
        ];

        for i in 0..points.len() {
            let cube = Cube::default();
            let point = points[i];
            let normal = cube.local_normal_at(point);
            assert_eq!(normal, normals[i]);
        }
    }
}
