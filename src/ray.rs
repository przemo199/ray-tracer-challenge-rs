use std::fmt::{Display, Formatter};
use std::sync::Arc;

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

    pub fn position(&self, distance: f64) -> Point {
        return self.origin + self.direction * distance;
    }

    pub fn intersect(&self, shape: &Arc<dyn Shape>) -> Intersections {
        let local_ray = self.transform(shape.transformation().inverse());
        return shape.clone().local_intersect(&local_ray);
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
    fn new_ray() {
        let origin = Point::new(1.0, 2.0, 3.0);
        let direction = Vector::new(4.0, 5.0, 6.0);
        let ray = Ray::new(origin, direction);
        assert_eq!(ray.origin, origin);
        assert_eq!(ray.direction, direction);
    }

    #[test]
    fn compute_point_from_distance() {
        let ray = Ray::new(Point::new(2.0, 3.0, 4.0), Vector::new(1.0, 0.0, 0.0));
        assert_eq!(ray.position(0.0), Point::new(2.0, 3.0, 4.0));
        assert_eq!(ray.position(1.0), Point::new(3.0, 3.0, 4.0));
        assert_eq!(ray.position(-1.0), Point::new(1.0, 3.0, 4.0));
        assert_eq!(ray.position(2.5), Point::new(4.5, 3.0, 4.0));
    }

    #[test]
    fn intersections_in_the_middle_of_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 4.0);
        assert_eq!(intersections[1].distance, 6.0);
        assert_eq!(&intersections[0].object, &arc_sphere);
        assert_eq!(&intersections[1].object, &arc_sphere);
    }

    #[test]
    fn intersections_on_the_edge_of_sphere() {
        let ray = Ray::new(Point::new(0.0, 1.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 5.0);
        assert_eq!(intersections[1].distance, 5.0);
        assert_eq!(&intersections[0].object, &arc_sphere);
        assert_eq!(&intersections[1].object, &arc_sphere);
    }

    #[test]
    fn no_intersections() {
        let ray = Ray::new(Point::new(0.0, 2.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn ray_begins_inside_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, -1.0);
        assert_eq!(intersections[1].distance, 1.0);
        assert_eq!(&intersections[0].object, &arc_sphere);
        assert_eq!(&intersections[1].object, &arc_sphere);
    }

    #[test]
    fn ray_begins_behind_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, -6.0);
        assert_eq!(intersections[1].distance, -4.0);
        assert_eq!(&intersections[0].object, &arc_sphere);
        assert_eq!(&intersections[1].object, &arc_sphere);
    }

    #[test]
    fn ray_translation() {
        let ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 1.0, 0.0));
        let matrix = transformations::translation(3.0, 4.0, 5.0);
        let transformed_ray = ray.transform(matrix);
        assert_eq!(transformed_ray.origin, Point::new(4.0, 6.0, 8.0));
        assert_eq!(transformed_ray.direction, Vector::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn ray_scaling() {
        let ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 1.0, 0.0));
        let matrix = transformations::scaling(2.0, 3.0, 4.0);
        let transformed_ray = ray.transform(matrix);
        assert_eq!(transformed_ray.origin, Point::new(2.0, 6.0, 12.0));
        assert_eq!(transformed_ray.direction, Vector::new(0.0, 3.0, 0.0));
    }

    #[test]
    fn intersecting_scaled_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut sphere = Sphere::default();
        sphere.transformation = transformations::scaling(2.0, 2.0, 2.0);
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 3.0);
        assert_eq!(intersections[1].distance, 7.0);
    }

    #[test]
    fn intersecting_translated_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut sphere = Sphere::default();
        sphere.transformation = transformations::translation(5.0, 0.0, 0.0);
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 0);
    }
}
