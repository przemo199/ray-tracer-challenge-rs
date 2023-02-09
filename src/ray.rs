use std::sync::Arc;
use crate::intersections::Intersections;
use crate::matrix::Matrix;
use crate::shape::Shape;
use crate::tuple::Tuple;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Tuple,
    pub direction: Tuple,
}

impl Ray {
    pub fn new(origin: Tuple, direction: Tuple) -> Ray {
        return Ray { origin, direction };
    }

    pub fn position(&self, distance: f64) -> Tuple {
        return self.origin + self.direction * distance;
    }

    pub fn intersect(&self, shape: &Arc<dyn Shape>) -> Intersections {
        let local_ray = self.transform(shape.transformation().inverse());
        return shape.clone().local_intersect(&local_ray);
    }

    pub fn transform(&self, matrix: Matrix<4>) -> Ray {
        let new_origin = matrix * self.origin;
        let new_direction = matrix * self.direction;
        return Ray::new(new_origin, new_direction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sphere::Sphere;
    use crate::transformations::Transformations;
    use crate::tuple::Tuple;

    #[test]
    fn new_ray() {
        let origin = Tuple::point(1.0, 2.0, 3.0);
        let direction = Tuple::vector(4.0, 5.0, 6.0);
        let ray = Ray::new(origin, direction);
        assert_eq!(ray.origin, origin);
        assert_eq!(ray.direction, direction);
    }

    #[test]
    fn compute_point_from_distance() {
        let ray = Ray::new(Tuple::point(2.0, 3.0, 4.0), Tuple::vector(1.0, 0.0, 0.0));
        assert_eq!(ray.position(0.0), Tuple::point(2.0, 3.0, 4.0));
        assert_eq!(ray.position(1.0), Tuple::point(3.0, 3.0, 4.0));
        assert_eq!(ray.position(-1.0), Tuple::point(1.0, 3.0, 4.0));
        assert_eq!(ray.position(2.5), Tuple::point(4.5, 3.0, 4.0));
    }

    #[test]
    fn intersections_in_the_middle_of_sphere() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].t, 4.0);
        assert_eq!(intersections[1].t, 6.0);
        assert_eq!(&intersections[0].object, &arc_sphere);
        assert_eq!(&intersections[1].object, &arc_sphere);
    }

    #[test]
    fn intersections_on_the_edge_of_sphere() {
        let ray = Ray::new(Tuple::point(0.0, 1.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].t, 5.0);
        assert_eq!(intersections[1].t, 5.0);
        assert_eq!(&intersections[0].object, &arc_sphere);
        assert_eq!(&intersections[1].object, &arc_sphere);
    }

    #[test]
    fn no_intersections() {
        let ray = Ray::new(Tuple::point(0.0, 2.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn ray_begins_inside_sphere() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].t, -1.0);
        assert_eq!(intersections[1].t, 1.0);
        assert_eq!(&intersections[0].object, &arc_sphere);
        assert_eq!(&intersections[1].object, &arc_sphere);
    }

    #[test]
    fn ray_begins_behind_sphere() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].t, -6.0);
        assert_eq!(intersections[1].t, -4.0);
        assert_eq!(&intersections[0].object, &arc_sphere);
        assert_eq!(&intersections[1].object, &arc_sphere);
    }

    #[test]
    fn ray_translation() {
        let ray = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let matrix = Transformations::translation(3.0, 4.0, 5.0);
        let transformed_ray = ray.transform(matrix);
        assert_eq!(transformed_ray.origin, Tuple::point(4.0, 6.0, 8.0));
        assert_eq!(transformed_ray.direction, Tuple::vector(0.0, 1.0, 0.0));
    }

    #[test]
    fn ray_scaling() {
        let ray = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let matrix = Transformations::scaling(2.0, 3.0, 4.0);
        let transformed_ray = ray.transform(matrix);
        assert_eq!(transformed_ray.origin, Tuple::point(2.0, 6.0, 12.0));
        assert_eq!(transformed_ray.direction, Tuple::vector(0.0, 3.0, 0.0));
    }

    #[test]
    fn intersecting_scaled_sphere() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut sphere = Sphere::default();
        sphere.transformation = Transformations::scaling(2.0, 2.0, 2.0);
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].t, 3.0);
        assert_eq!(intersections[1].t, 7.0);
    }

    #[test]
    fn intersecting_translated_sphere() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut sphere = Sphere::default();
        sphere.transformation = Transformations::translation(5.0, 0.0, 0.0);
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersections = ray.intersect(&arc_sphere);
        assert_eq!(intersections.len(), 0);
    }
}
