use std::fmt::{Debug, Display, Formatter};
use crate::{EPSILON, Intersection, Intersections, Material, Matrix, Ray, Shape, Tuple, TupleTrait};

#[derive(Clone, Debug, PartialEq)]
pub struct Triangle {
    p1: Tuple,
    p2: Tuple,
    p3: Tuple,
    e1: Tuple,
    e2: Tuple,
    normal: Tuple,
    material: Material,
    transformation: Matrix,
}

impl Triangle {
    pub fn new(p1: Tuple, p2: Tuple, p3: Tuple) -> Triangle {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = (e2.cross(&e1)).normalize();
        return Triangle { p1, p2, p3, e1, e2, normal, material: Material::default(), transformation: Matrix::identity() };
    }
}

impl Display for Triangle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Shape for Triangle {
    fn local_normal_at(&self, point: Tuple) -> Tuple {
        return self.normal;
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
        let direction_cross_e2 = ray.direction.cross(&self.e2);
        let determinant = self.e1.dot(&direction_cross_e2);
        if determinant.abs() < EPSILON {
            return intersections;
        }
        let f = 1.0 / determinant;
        let p1_to_origin = ray.origin - self.p1;
        let u = f * p1_to_origin.dot(&direction_cross_e2);
        if u < 0.0 || u > 1.0 {
            return intersections;
        }
        let origin_cross_e1 = p1_to_origin.cross(&self.e1);
        let v = f * ray.direction.dot(&origin_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return intersections;
        }
        let t = f * self.e2.dot(&origin_cross_e1);
        intersections.add(Intersection::new(t, self.box_clone()));
        return intersections;
    }

    fn box_clone(&self) -> Box<dyn Shape> {
        return Box::new(self.clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::{Ray, Shape, Tuple, TupleTrait};
    use crate::triangle::Triangle;

    #[test]
    fn creating_triangle() {
        let p1 = Tuple::point(0.0, 1.0, 0.0);
        let p2 = Tuple::point(-1.0, 0.0, 0.0);
        let p3 = Tuple::point(1.0, 0.0, 0.0);
        let triangle = Triangle::new(p1, p2, p3);
        assert_eq!(triangle.p1, p1);
        assert_eq!(triangle.p2, p2);
        assert_eq!(triangle.p3, p3);
        assert_eq!(triangle.e1, Tuple::vector(-1.0, -1.0, 0.0));
        assert_eq!(triangle.e2, Tuple::vector(1.0, -1.0, 0.0));
        assert_eq!(triangle.normal, Tuple::vector(0.0, 0.0, -1.0));
    }

    #[test]
    fn normal_on_triangle() {
        let triangle = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0));
        let n1 = triangle.local_normal_at(Tuple::point(0.0, 0.5, 0.0));
        let n2 = triangle.local_normal_at(Tuple::point(-0.5, 0.75, 0.0));
        let n3 = triangle.local_normal_at(Tuple::point(0.5, 0.25, 0.0));
        assert_eq!(n1, triangle.normal);
        assert_eq!(n2, triangle.normal);
        assert_eq!(n3, triangle.normal);
    }

    #[test]
    fn ray_parallel_to_triangle() {
        let triangle = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0));
        let ray = Ray::new(Tuple::point(0.0, -1.0, -2.0), Tuple::vector(0.0, 1.0, 0.0));
        assert_eq!(triangle.local_intersect(&ray).len(), 0);
    }

    #[test]
    fn ray_misses_p1_p3_edge() {
        let triangle = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0));
        let ray = Ray::new(Tuple::point(1.0, 1.0, -2.0), Tuple::vector(0.0, 0.0, 1.0));
        assert_eq!(triangle.local_intersect(&ray).len(), 0);
    }

    #[test]
    fn ray_misses_p1_p2_edge() {
        let triangle = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0));
        let ray = Ray::new(Tuple::point(-1.0, 1.0, -2.0), Tuple::vector(0.0, 0.0, 1.0));
        assert_eq!(triangle.local_intersect(&ray).len(), 0);
    }

    #[test]
    fn ray_misses_p2_p3_edge() {
        let triangle = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0));
        let ray = Ray::new(Tuple::point(0.0, -1.0, -2.0), Tuple::vector(0.0, 0.0, 1.0));
        assert_eq!(triangle.local_intersect(&ray).len(), 0);
    }

    #[test]
    fn ray_strikes_triangle() {
        let triangle = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0));
        let ray = Ray::new(Tuple::point(0.0, 0.5, -2.0), Tuple::vector(0.0, 0.0, 1.0));
        let intersections = triangle.local_intersect(&ray);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].t, 2.0);
    }
}
