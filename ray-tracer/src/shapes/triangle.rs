use super::{Intersect, Shape, Transform};
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::{BINCODE_CONFIG, EPSILON};
use crate::primitives::{Point, Transformation, Vector};
use bincode::Encode;
use core::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct Triangle {
    pub material: Material,
    transformation_inverse: Transformation,
    pub vertex_1: Point,
    pub vertex_2: Point,
    pub vertex_3: Point,
    pub edge_1: Vector,
    pub edge_2: Vector,
    pub normal: Vector,
}

impl Triangle {
    pub fn new(vertex_1: Point, vertex_2: Point, vertex_3: Point) -> Self {
        let edge_1 = vertex_2 - vertex_1;
        let edge_2 = vertex_3 - vertex_1;
        let normal = edge_2.cross(&edge_1).normalized();
        return Self {
            material: Material::default(),
            transformation_inverse: Transformation::IDENTITY,
            vertex_1,
            vertex_2,
            vertex_3,
            edge_1,
            edge_2,
            normal,
        };
    }
}

impl Intersect for Triangle {
    fn local_intersect<'shape>(&'shape self, ray: &Ray, intersections: &mut Intersections<'shape>) {
        let direction_cross_edge2 = ray.direction.cross(&self.edge_2);
        let determinant = self.edge_1.dot(&direction_cross_edge2);
        if determinant.abs() < EPSILON {
            return;
        }
        let vertex1_to_origin = ray.origin - self.vertex_1;
        let u = vertex1_to_origin.dot(&direction_cross_edge2) / determinant;
        if !(0.0..1.0).contains(&u) {
            return;
        }
        let origin_cross_edge1 = vertex1_to_origin.cross(&self.edge_1);
        let v = ray.direction.dot(&origin_cross_edge1) / determinant;
        if v > 0.0 && u + v < 1.0 {
            let distance = self.edge_2.dot(&origin_cross_edge1) / determinant;
            intersections.push(Intersection::new(distance, self));
        }
    }
}

impl Transform for Triangle {
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

impl Shape for Triangle {
    fn local_normal_at(&self, _: Point) -> Vector {
        return self.normal;
    }

    fn material(&self) -> &Material {
        return &self.material;
    }

    fn encoded(&self) -> Vec<u8> {
        return bincode::encode_to_vec(self, BINCODE_CONFIG).expect("Failed to serialise Triangle");
    }
}

impl Display for Triangle {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        return formatter
            .debug_struct("Triangle")
            .field("p1", &self.vertex_1)
            .field("p2", &self.vertex_2)
            .field("p3", &self.vertex_3)
            .field("e1", &self.edge_1)
            .field("e2", &self.edge_1)
            .field("normal", &self.normal)
            .field("material", &self.material)
            .field("transformation", &self.transformation())
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::Ray;
    use crate::primitives::Vector;

    #[test]
    fn creating_triangle() {
        let p_1 = Point::new(0, 1, 0);
        let p_2 = Point::new(-1, 0, 0);
        let p_3 = Point::new(1, 0, 0);
        let triangle = Triangle::new(p_1, p_2, p_3);
        assert_eq!(triangle.vertex_1, p_1);
        assert_eq!(triangle.vertex_2, p_2);
        assert_eq!(triangle.vertex_3, p_3);
        assert_eq!(triangle.edge_1, Vector::new(-1, -1, 0));
        assert_eq!(triangle.edge_2, Vector::new(1, -1, 0));
        assert_eq!(triangle.normal, Vector::BACKWARD);
    }

    #[test]
    fn normal_on_triangle() {
        let triangle = Triangle::new(
            Point::new(0, 1, 0),
            Point::new(-1, 0, 0),
            Point::new(1, 0, 0),
        );
        let n_1 = triangle.local_normal_at(Point::new(0, 0.5, 0));
        let n_2 = triangle.local_normal_at(Point::new(-0.5, 0.75, 0));
        let n_3 = triangle.local_normal_at(Point::new(0.5, 0.25, 0));
        assert_eq!(n_1, triangle.normal);
        assert_eq!(n_2, triangle.normal);
        assert_eq!(n_3, triangle.normal);
    }

    #[test]
    fn ray_parallel_to_triangle() {
        let triangle = Triangle::new(
            Point::new(0, 1, 0),
            Point::new(-1, 0, 0),
            Point::new(1, 0, 0),
        );
        let ray = Ray::new(Point::new(0, -1, -2), Vector::UP);
        let boxed_shape: Box<dyn Shape> = Box::new(triangle);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert!(intersections.is_empty());
    }

    #[test]
    fn ray_misses_p1_p3_edge() {
        let triangle = Triangle::new(
            Point::new(0, 1, 0),
            Point::new(-1, 0, 0),
            Point::new(1, 0, 0),
        );
        let ray = Ray::new(Point::new(1, 1, -2), Vector::FORWARD);
        let boxed_shape: Box<dyn Shape> = Box::new(triangle);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert!(intersections.is_empty());
    }

    #[test]
    fn ray_misses_p1_p2_edge() {
        let triangle = Triangle::new(
            Point::new(0, 1, 0),
            Point::new(-1, 0, 0),
            Point::new(1, 0, 0),
        );
        let ray = Ray::new(Point::new(-1, 1, -2), Vector::FORWARD);
        let boxed_shape: Box<dyn Shape> = Box::new(triangle);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert!(intersections.is_empty());
    }

    #[test]
    fn ray_misses_p2_p3_edge() {
        let triangle = Triangle::new(
            Point::new(0, 1, 0),
            Point::new(-1, 0, 0),
            Point::new(1, 0, 0),
        );
        let ray = Ray::new(Point::new(0, -1, -2), Vector::FORWARD);
        let boxed_shape = Box::new(triangle);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert!(intersections.is_empty());
    }

    #[test]
    fn ray_intersects_triangle() {
        let triangle = Triangle::new(
            Point::new(0, 1, 0),
            Point::new(-1, 0, 0),
            Point::new(1, 0, 0),
        );
        let ray = Ray::new(Point::new(0, 0.5, -2), Vector::FORWARD);
        let boxed_shape: Box<dyn Shape> = Box::new(triangle);
        let mut intersections = Intersections::new();
        boxed_shape.local_intersect(&ray, &mut intersections);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 2.0);
    }
}
