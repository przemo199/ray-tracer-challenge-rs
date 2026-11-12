use super::{Intersect, Shape, Transform};
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::EPSILON;
use crate::impl_shape_transform_for;
use crate::primitives::{Point, Transformation, Vector};
use crate::utils::CoarseEq;
use core::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Triangle {
    transformation_inverse: Transformation,
    /// Transposed inverse, cached for `normal_at`.
    normal_transformation: Transformation,
    pub material: Material,
    // Private: the edges and normal are derived from the vertices, so none of them may change alone.
    vertex_1: Point,
    vertex_2: Point,
    vertex_3: Point,
    edge_1: Vector,
    edge_2: Vector,
    normal: Vector,
}

impl Triangle {
    pub const fn vertex_1(&self) -> Point {
        return self.vertex_1;
    }

    pub const fn vertex_2(&self) -> Point {
        return self.vertex_2;
    }

    pub const fn vertex_3(&self) -> Point {
        return self.vertex_3;
    }

    /// From `vertex_1` to `vertex_2`.
    pub const fn edge_1(&self) -> Vector {
        return self.edge_1;
    }

    /// From `vertex_1` to `vertex_3`.
    pub const fn edge_2(&self) -> Vector {
        return self.edge_2;
    }

    /// The object-space normal, the same at every point of the triangle.
    pub const fn normal(&self) -> Vector {
        return self.normal;
    }

    pub fn new(vertex_1: Point, vertex_2: Point, vertex_3: Point) -> Self {
        let edge_1 = vertex_2 - vertex_1;
        let edge_2 = vertex_3 - vertex_1;
        let normal = edge_2.cross(&edge_1).normalized();
        return Self {
            material: Material::default(),
            transformation_inverse: Transformation::IDENTITY,
            normal_transformation: Transformation::IDENTITY,
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
        if !(0.0..=1.0).contains(&u) {
            return;
        }
        let origin_cross_edge1 = vertex1_to_origin.cross(&self.edge_1);
        let v = ray.direction.dot(&origin_cross_edge1) / determinant;
        // edges are inclusive, like `u` above
        if v >= 0.0 && u + v <= 1.0 {
            let distance = self.edge_2.dot(&origin_cross_edge1) / determinant;
            intersections.push(Intersection::new(distance, self));
        }
    }
}

impl_shape_transform_for!(Triangle);

impl Shape for Triangle {
    fn local_normal_at(&self, _: Point) -> Vector {
        return self.normal;
    }

    fn material(&self) -> &Material {
        return &self.material;
    }
}

impl Display for Triangle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return f
            .debug_struct("Triangle")
            .field("p1", &self.vertex_1)
            .field("p2", &self.vertex_2)
            .field("p3", &self.vertex_3)
            .field("e1", &self.edge_1)
            .field("e2", &self.edge_2)
            .field("normal", &self.normal)
            .field("material", &self.material)
            .field("transformation", &self.transformation())
            .finish();
    }
}

impl CoarseEq for Triangle {
    fn coarse_eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || self.material == other.material
                && self
                    .transformation_inverse
                    .coarse_eq(&other.transformation_inverse)
                && self.vertex_1.coarse_eq(&other.vertex_1)
                && self.vertex_2.coarse_eq(&other.vertex_2)
                && self.vertex_3.coarse_eq(&other.vertex_3)
                && self.edge_1.coarse_eq(&other.edge_1)
                && self.edge_2.coarse_eq(&other.edge_2)
                && self.normal.coarse_eq(&other.normal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::Ray;
    use crate::primitives::Vector;
    use rstest::rstest;

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

    /// Edges belong to the triangle, like vertices do (`u` was already inclusive): a ray exactly on
    /// an edge shared by two triangles must hit at least one of them, or the mesh shows a crack.
    #[rstest]
    #[case::p1_p2_edge(Point::new(-0.5, 0.5, -2))]
    #[case::p2_p3_edge(Point::new(0, 0, -2))]
    #[case::p1_vertex(Point::new(0, 1, -2))]
    fn ray_on_edge_hits_triangle(#[case] origin: Point) {
        let triangle = Triangle::new(
            Point::new(0, 1, 0),
            Point::new(-1, 0, 0),
            Point::new(1, 0, 0),
        );
        let ray = Ray::new(origin, Vector::FORWARD);
        let mut intersections = Intersections::new();
        triangle.local_intersect(&ray, &mut intersections);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].distance, 2.0);
    }

    #[test]
    fn display_shows_each_edge() {
        let triangle = Triangle::new(
            Point::new(0, 1, 0),
            Point::new(-1, 0, 0),
            Point::new(1, 0, 0),
        );
        let shown = triangle.to_string();
        assert!(
            shown.contains(&format!("e1: {:?}", triangle.edge_1)),
            "{shown}"
        );
        assert!(
            shown.contains(&format!("e2: {:?}", triangle.edge_2)),
            "{shown}"
        );
    }
}
