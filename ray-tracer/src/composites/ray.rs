use crate::composites::Intersections;
use crate::primitives::{Point, Transformation, Vector};
use crate::shapes::{Intersect, Transform};
use crate::utils::CoarseEq;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

impl Ray {
    /// Creates new instance of struct [Ray]
    /// # Examples
    /// ```
    /// use ray_tracer::composites::Ray;
    /// use ray_tracer::primitives::{Point, Vector};
    ///
    /// let ray = Ray::new(Point::ORIGIN, Vector::FORWARD);
    ///
    /// assert_eq!(ray.origin, Point::ORIGIN);
    /// assert_eq!(ray.direction, Vector::FORWARD);
    /// ```
    pub const fn new(origin: Point, direction: Vector) -> Self {
        return Self { origin, direction };
    }

    #[inline]
    pub fn position(&self, distance: impl Into<f64>) -> Point {
        return self.origin + self.direction * distance.into();
    }

    #[inline]
    pub fn intersect<'shape, T: Transform + Intersect + ?Sized>(
        &self,
        shape: &'shape T,
        intersections: &mut Intersections<'shape>,
    ) {
        let local_ray = self.transform(&shape.transformation_inverse());
        shape.local_intersect(&local_ray, intersections);
    }

    #[inline]
    pub fn transform(&self, transformation: &Transformation) -> Self {
        let new_origin = *transformation * self.origin;
        let new_direction = *transformation * self.direction;
        return Self::new(new_origin, new_direction);
    }
}

impl Display for Ray {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        return formatter
            .debug_struct("Ray")
            .field("origin", &self.origin)
            .field("direction", &self.direction)
            .finish();
    }
}

impl CoarseEq for Ray {
    fn coarse_eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs)
            || self.origin.coarse_eq(&rhs.origin) && self.direction.coarse_eq(&rhs.direction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::transformations;
    use crate::shapes::{Shape, Sphere, Transform};

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
        let ray = Ray::new(Point::new(2, 3, 4), Vector::RIGHT);
        assert_eq!(ray.position(0), Point::new(2, 3, 4));
        assert_eq!(ray.position(1), Point::new(3, 3, 4));
        assert_eq!(ray.position(-1), Point::new(1, 3, 4));
        assert_eq!(ray.position(2.5), Point::new(4.5, 3, 4));
    }

    #[test]
    fn intersections_in_the_middle_of_sphere() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        ray.intersect(boxed_shape.as_ref(), &mut intersections);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 4.0);
        assert_eq!(intersections[1].distance, 6.0);
        assert_eq!(intersections[0].shape, boxed_shape.as_ref());
        assert_eq!(intersections[1].shape, boxed_shape.as_ref());
    }

    #[test]
    fn intersections_on_the_edge_of_sphere() {
        let ray = Ray::new(Point::new(0, 1, -5), Vector::FORWARD);
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        ray.intersect(boxed_shape.as_ref(), &mut intersections);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 5.0);
        assert_eq!(intersections[1].distance, 5.0);
        assert_eq!(intersections[0].shape, boxed_shape.as_ref());
        assert_eq!(intersections[1].shape, boxed_shape.as_ref());
    }

    #[test]
    fn no_intersections() {
        let ray = Ray::new(Point::new(0, 2, -5), Vector::FORWARD);
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        ray.intersect(boxed_shape.as_ref(), &mut intersections);
        assert!(intersections.is_empty());
    }

    #[test]
    fn ray_origin_inside_sphere() {
        let ray = Ray::new(Point::ORIGIN, Vector::FORWARD);
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        ray.intersect(boxed_shape.as_ref(), &mut intersections);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, -1.0);
        assert_eq!(intersections[1].distance, 1.0);
        assert_eq!(intersections[0].shape, boxed_shape.as_ref());
        assert_eq!(intersections[1].shape, boxed_shape.as_ref());
    }

    #[test]
    fn ray_origin_behind_sphere() {
        let ray = Ray::new(Point::new(0, 0, 5), Vector::FORWARD);
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        ray.intersect(boxed_shape.as_ref(), &mut intersections);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, -6.0);
        assert_eq!(intersections[1].distance, -4.0);
        assert_eq!(intersections[0].shape, boxed_shape.as_ref());
        assert_eq!(intersections[1].shape, boxed_shape.as_ref());
    }

    #[test]
    fn ray_translation() {
        let ray = Ray::new(Point::new(1, 2, 3), Vector::UP);
        let matrix = transformations::translation(3, 4, 5);
        let transformed_ray = ray.transform(&matrix);
        assert_eq!(transformed_ray.origin, Point::new(4, 6, 8));
        assert_eq!(transformed_ray.direction, Vector::UP);
    }

    #[test]
    fn ray_scaling() {
        let ray = Ray::new(Point::new(1, 2, 3), Vector::UP);
        let matrix = transformations::scaling(2, 3, 4);
        let transformed_ray = ray.transform(&matrix);
        assert_eq!(transformed_ray.origin, Point::new(2, 6, 12));
        assert_eq!(transformed_ray.direction, Vector::new(0, 3, 0));
    }

    #[test]
    fn intersecting_scaled_sphere() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::scaling(2, 2, 2));
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        ray.intersect(boxed_shape.as_ref(), &mut intersections);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].distance, 3.0);
        assert_eq!(intersections[1].distance, 7.0);
    }

    #[test]
    fn intersecting_translated_sphere() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::translation(5, 0, 0));
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        ray.intersect(boxed_shape.as_ref(), &mut intersections);
        assert!(intersections.is_empty());
    }
}
