use std::fmt::Debug;
use std::sync::Arc;

use crate::computed_hit::ComputedHit;
use crate::intersections::Intersections;
use crate::ray::Ray;
use crate::shapes::Shape;
use crate::utils::CloseEnough;

#[derive(Clone, Debug)]
pub struct Intersection {
    pub distance: f64,
    pub object: Arc<dyn Shape>,
}

impl Intersection {
    pub fn new(distance: impl Into<f64>, object: Arc<dyn Shape>) -> Intersection {
        return Intersection { distance: distance.into(), object };
    }

    pub fn prepare_computations(&self, ray: &Ray, intersections: &Intersections) -> ComputedHit {
        let point = ray.position(self.distance);
        let mut normal_vector = self.object.normal_at(point);
        let camera_vector = -ray.direction;
        let is_inside = normal_vector.dot(&camera_vector) < 0.0;

        if is_inside {
            normal_vector = -normal_vector;
        }

        let reflection_vector = ray.direction.reflect(&normal_vector);

        let mut containers: Vec<Arc<dyn Shape>> = Vec::new();
        let mut n1: f64 = 1.0;
        let mut n2: f64 = 1.0;
        for intersection in intersections.intersections.iter() {
            if intersection == self {
                if containers.is_empty() {
                    n1 = 1.0;
                } else {
                    n1 = containers.last().unwrap().material().refractive_index;
                }
            }

            if containers.contains(&intersection.object) {
                containers.retain(|object| object != &intersection.object);
            } else {
                containers.push(intersection.object.clone());
            }

            if intersection == self {
                if containers.is_empty() {
                    n2 = 1.0;
                } else {
                    n2 = containers.last().unwrap().material().refractive_index;
                }
                break;
            }
        }

        return ComputedHit::new(
            self.distance,
            self.object.clone(),
            point,
            camera_vector,
            normal_vector,
            reflection_vector,
            is_inside,
            n1,
            n2,
        );
    }
}

impl PartialEq for Intersection {
    fn eq(&self, rhs: &Intersection) -> bool {
        return self.distance.close_enough(rhs.distance) && *self.object == *rhs.object;
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::EPSILON;
    use crate::primitives::{Point, Vector};
    use crate::primitives::transformations;
    use crate::shapes::{Plane, Shape, Sphere};

    use super::*;

    #[test]
    fn new_intersection() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersection = Intersection::new(3.5, arc_sphere.clone());
        assert_eq!(intersection.distance, 3.5);
        assert_eq!(&intersection.object, &arc_sphere);
    }

    #[test]
    fn precomputing_intersection_state() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::new(0, 0, 1));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersection = Intersection::new(4, arc_sphere.clone());
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        assert_eq!(computed_hit.distance, intersection.distance);
        assert_eq!(&computed_hit.object, &arc_sphere);
        assert_eq!(computed_hit.point, Point::new(0, 0, -1));
        assert_eq!(computed_hit.camera_vector, Vector::new(0, 0, -1));
        assert_eq!(computed_hit.normal_vector, Vector::new(0, 0, -1));
    }

    #[test]
    fn hit_when_intersection_is_outside() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::new(0, 0, 1));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersection1 = Intersection::new(4, arc_sphere);
        let computations = intersection1.prepare_computations(&ray, &Intersections::new());
        assert!(!computations.is_inside);
    }

    #[test]
    fn hit_when_intersection_is_inside() {
        let ray = Ray::new(Point::new(0, 0, 0), Vector::new(0, 0, 1));
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersection1 = Intersection::new(1, arc_sphere);
        let computations = intersection1.prepare_computations(&ray, &Intersections::new());
        assert!(computations.is_inside);
        assert_eq!(computations.point, Point::new(0, 0, 1));
        assert_eq!(computations.camera_vector, Vector::new(0, 0, -1));
    }

    #[test]
    fn hit_offsets_point() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::new(0, 0, 1));
        let sphere = Sphere { transformation: transformations::translation(0, 0, 1), ..Default::default() };
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersection = Intersection::new(5, arc_sphere);
        let computations = intersection.prepare_computations(&ray, &Intersections::new());
        assert!(computations.over_point.z < -EPSILON / 2.0);
        assert!(computations.point.z > computations.over_point.z);
    }

    #[test]
    fn precomputing_reflection_vector() {
        let plane = Plane::default();
        let arc_plane: Arc<dyn Shape> = Arc::new(plane);
        let ray = Ray::new(Point::new(0, 1, -1), Vector::new(0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let intersection = Intersection::new(2.0_f64.sqrt(), arc_plane);
        let prepared_computations = intersection.prepare_computations(&ray, &Intersections::new());
        assert_eq!(prepared_computations.reflection_vector, Vector::new(0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0));
    }

    #[test]
    fn finding_n1_and_n2() {
        let mut sphere1 = Sphere::glass();
        sphere1.set_transformation(transformations::scaling(2, 2, 2));
        let mut material1 = sphere1.material.clone();
        material1.refractive_index = 1.5;
        sphere1.set_material(material1);
        let arc_sphere1: Arc<dyn Shape> = Arc::new(sphere1);
        let mut sphere2 = Sphere::glass();
        sphere2.set_transformation(transformations::translation(0, 0, -0.25));
        let mut material2 = sphere2.material.clone();
        material2.refractive_index = 2.0;
        sphere2.set_material(material2);
        let arc_sphere2: Arc<dyn Shape> = Arc::new(sphere2);
        let mut sphere3 = Sphere::glass();
        sphere3.set_transformation(transformations::translation(0, 0, 0.25));
        let mut material3 = sphere3.material.clone();
        material3.refractive_index = 2.5;
        sphere3.set_material(material3);
        let arc_sphere3: Arc<dyn Shape> = Arc::new(sphere3);
        let ray = Ray::new(Point::new(0, 0, -4), Vector::new(0, 0, 1));

        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(2, arc_sphere1.clone()));
        intersections.add(Intersection::new(2.75, arc_sphere2.clone()));
        intersections.add(Intersection::new(3.25, arc_sphere3.clone()));
        intersections.add(Intersection::new(4.75, arc_sphere2));
        intersections.add(Intersection::new(5.25, arc_sphere3));
        intersections.add(Intersection::new(6, arc_sphere1));

        let n_1s = [1.0, 1.5, 2.0, 2.5, 2.5, 1.5];
        let n_2s = [1.5, 2.0, 2.5, 2.5, 1.5, 1.0];
        for (intersection, (n_1, n_2)) in intersections.intersections.iter().zip(n_1s.iter().zip(n_2s.iter())) {
            let prepared_computations = intersection.prepare_computations(&ray, &intersections);
            assert_eq!(prepared_computations.n1, *n_1);
            assert_eq!(prepared_computations.n2, *n_2);
        }
    }

    #[test]
    fn under_point_is_below_surface() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::new(0, 0, 1));
        let mut sphere = Sphere::glass();
        sphere.set_transformation(transformations::translation(0, 0, 1));
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let intersection = Intersection::new(5, arc_sphere);
        let mut intersections = Intersections::new();
        intersections.add(intersection.clone());
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        assert!(computed_hit.under_point.z > EPSILON / 2.0);
        assert!(computed_hit.point.z < computed_hit.under_point.z);
    }
}
