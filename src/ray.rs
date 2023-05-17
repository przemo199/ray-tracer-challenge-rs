use std::fmt::{Display, Formatter};

use crate::intersections::Intersections;
use crate::primitives::{Point, Transformation, Vector};
use crate::shapes::Shape;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

impl Ray {
    pub fn new(origin: Point, direction: Vector) -> Ray {
        return Ray { origin, direction };
    }

    pub fn position(&self, distance: impl Into<f64>) -> Point {
        return self.origin + self.direction * distance.into();
    }

    pub fn intersect<'a>(&self, shape: &'a dyn Shape) -> Intersections<'a> {
        let local_ray = self.transform(shape.transformation().inverse());
        return shape.local_intersect(&local_ray);
    }

    pub fn transform(&self, transformation: Transformation) -> Ray {
        let new_origin = transformation * self.origin;
        let new_direction = transformation * self.direction;
        return Ray::new(new_origin, new_direction);
    }
}

impl Display for Ray {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("Ray")
            .field("origin", &self.origin)
            .field("direction", &self.direction)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::transformations;
    use crate::shapes::Sphere;

    use super::*;

    #[test]
    fn creating_and_inspecting_ray() {
        let origin = Point::new(1, 2, 3);
        let direction = Vector::new(4, 5, 6);
        let ray = Ray::new(origin, direction);
        assert_eq!(ray.origin, origin);
        assert_eq!(ray.direction, direction);
    }

    #[test]
    fn compute_point_from_distance() {
        let ray = Ray::new(Point::new(2, 3, 4), Vector::new(1, 0, 0));
        assert_eq!(ray.position(0), Point::new(2, 3, 4));
        assert_eq!(ray.position(1), Point::new(3, 3, 4));
        assert_eq!(ray.position(-1), Point::new(1, 3, 4));
        assert_eq!(ray.position(2.5), Point::new(4.5, 3, 4));
    }

    #[test]
    fn intersections_in_the_middle_of_sphere() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::new(0, 0, 1));
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersections = ray.intersect(boxed_shape.as_ref());
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 4.0);
        assert_eq!(intersections[1].distance, 6.0);
        assert_eq!(intersections[0].object, boxed_shape.as_ref());
        assert_eq!(intersections[1].object, boxed_shape.as_ref());
    }

    #[test]
    fn intersections_on_the_edge_of_sphere() {
        let ray = Ray::new(Point::new(0, 1, -5), Vector::new(0, 0, 1));
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersections = ray.intersect(boxed_shape.as_ref());
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 5.0);
        assert_eq!(intersections[1].distance, 5.0);
        assert_eq!(intersections[0].object, boxed_shape.as_ref());
        assert_eq!(intersections[1].object, boxed_shape.as_ref());
    }

    #[test]
    fn no_intersections() {
        let ray = Ray::new(Point::new(0, 2, -5), Vector::new(0, 0, 1));
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersections = ray.intersect(boxed_shape.as_ref());
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn ray_origin_inside_sphere() {
        let ray = Ray::new(Point::new(0, 0, 0), Vector::new(0, 0, 1));
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersections = ray.intersect(boxed_shape.as_ref());
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, -1.0);
        assert_eq!(intersections[1].distance, 1.0);
        assert_eq!(intersections[0].object, boxed_shape.as_ref());
        assert_eq!(intersections[1].object, boxed_shape.as_ref());
    }

    #[test]
    fn ray_origin_behind_sphere() {
        let ray = Ray::new(Point::new(0, 0, 5), Vector::new(0, 0, 1));
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersections = ray.intersect(boxed_shape.as_ref());
        intersections[0].object.encoded();
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, -6.0);
        assert_eq!(intersections[1].distance, -4.0);
        assert_eq!(intersections[0].object, boxed_shape.as_ref());
        assert_eq!(intersections[1].object, boxed_shape.as_ref());
    }

    #[test]
    fn ray_translation() {
        let ray = Ray::new(Point::new(1, 2, 3), Vector::new(0, 1, 0));
        let matrix = transformations::translation(3, 4, 5);
        let transformed_ray = ray.transform(matrix);
        assert_eq!(transformed_ray.origin, Point::new(4, 6, 8));
        assert_eq!(transformed_ray.direction, Vector::new(0, 1, 0));
    }

    #[test]
    fn ray_scaling() {
        let ray = Ray::new(Point::new(1, 2, 3), Vector::new(0, 1, 0));
        let matrix = transformations::scaling(2, 3, 4);
        let transformed_ray = ray.transform(matrix);
        assert_eq!(transformed_ray.origin, Point::new(2, 6, 12));
        assert_eq!(transformed_ray.direction, Vector::new(0, 3, 0));
    }

    #[test]
    fn intersecting_scaled_sphere() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::new(0, 0, 1));
        let mut sphere = Sphere::default();
        sphere.transformation = transformations::scaling(2, 2, 2);
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersections = ray.intersect(boxed_shape.as_ref());
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 3.0);
        assert_eq!(intersections[1].distance, 7.0);
    }

    #[test]
    fn intersecting_translated_sphere() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::new(0, 0, 1));
        let mut sphere = Sphere::default();
        sphere.transformation = transformations::translation(5, 0, 0);
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersections = ray.intersect(boxed_shape.as_ref());
        assert_eq!(intersections.len(), 0);
    }
}
