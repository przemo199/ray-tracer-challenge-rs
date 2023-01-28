use std::fmt::Debug;
use crate::{ComputedHit, Intersections, Ray, Shape, TupleTrait};

#[derive(Clone, Debug)]
pub struct Intersection {
    pub t: f64,
    pub object: Box<dyn Shape>,
}

impl Intersection {
    pub fn new<T: Into<Box<dyn Shape>>>(t: f64, object: T) -> Intersection {
        return Intersection { t, object: object.into() };
    }

    pub fn prepare_computations(&self, ray: &Ray, intersections: &Intersections) -> ComputedHit {
        let point = ray.position(self.t);
        let mut normal_vector = self.object.normal_at(point);
        let camera_vector = -ray.direction;
        let is_inside = normal_vector.dot(&camera_vector) < 0.0;

        if is_inside {
            normal_vector = -normal_vector;
        }

        let reflection_vector = ray.direction.reflect(&normal_vector);

        let mut containers: Vec<Box<dyn Shape>> = Vec::new();
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
                containers.retain(|object| *object != intersection.object.box_clone());
            } else {
                containers.push(intersection.object.box_clone());
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
            self.t,
            self.object.box_clone(),
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
        return (self.t - rhs.t).abs() < crate::EPSILON && &self.object == &rhs.object;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Plane, Sphere, Transformations, Tuple};

    #[test]
    fn new_intersection() {
        let sphere = Sphere::default();
        let intersection = Intersection::new(3.5, sphere.box_clone());
        assert_eq!(intersection.t, 3.5);
        assert_eq!(&intersection.object, &sphere.box_clone());
    }

    #[test]
    fn precomputing_intersection_state() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let intersection1 = Intersection::new(4.0, sphere.clone());
        let computed_hit = intersection1.prepare_computations(&ray, &Intersections::new());
        assert_eq!(computed_hit.t, intersection1.t);
        assert_eq!(&computed_hit.object, &sphere.box_clone());
        assert_eq!(computed_hit.point, Tuple::point(0.0, 0.0, -1.0));
        assert_eq!(computed_hit.camera_vector, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(computed_hit.normal_vector, Tuple::vector(0.0, 0.0, -1.0));
    }

    #[test]
    fn hit_when_intersection_is_outside() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let intersection1 = Intersection::new(4.0, sphere.box_clone());
        let computations = intersection1.prepare_computations(&ray, &Intersections::new());
        assert!(!computations.is_inside);
    }

    #[test]
    fn hit_when_intersection_is_inside() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let intersection1 = Intersection::new(1.0, sphere.box_clone());
        let computations = intersection1.prepare_computations(&ray, &Intersections::new());
        assert_eq!(computations.point, Tuple::point(0.0, 0.0, 1.0));
        assert_eq!(computations.camera_vector, Tuple::vector(0.0, 0.0, -1.0));
        assert!(computations.is_inside);
        assert_eq!(computations.point, Tuple::point(0.0, 0.0, 1.0));
    }

    #[test]
    fn hit_offsets_point() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let sphere = Sphere { transformation: Transformations::translation(0.0, 0.0, 1.0), ..Default::default() };
        let intersection = Intersection::new(5.0, sphere.box_clone());
        let computations = intersection.prepare_computations(&ray, &Intersections::new());
        assert!(computations.over_point.z < -crate::EPSILON / 2.0);
        assert!(computations.point.z > computations.over_point.z);
    }

    #[test]
    fn precomputing_reflection_vector() {
        let shape = Plane::default();
        let ray = Ray::new(Tuple::point(0.0, 1.0, -1.0), Tuple::vector(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let intersection = Intersection::new(2.0_f64.sqrt(), shape.box_clone());
        let prepared_computations = intersection.prepare_computations(&ray, &Intersections::new());
        assert_eq!(prepared_computations.reflection_vector, Tuple::vector(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0));
    }

    #[test]
    fn finding_n1_and_n2() {
        let mut sphere1 = Sphere::glass();
        sphere1.set_transformation(Transformations::scaling(2.0, 2.0, 2.0));
        let mut material1 = sphere1.material.clone();
        material1.refractive_index = 1.5;
        sphere1.set_material(material1);
        let mut sphere2 = Sphere::glass();
        sphere2.set_transformation(Transformations::translation(0.0, 0.0, -0.25));
        let mut material2 = sphere2.material.clone();
        material2.refractive_index = 2.0;
        sphere2.set_material(material2);
        let mut sphere3 = Sphere::glass();
        sphere3.set_transformation(Transformations::translation(0.0, 0.0, 0.25));
        let mut material3 = sphere3.material.clone();
        material3.refractive_index = 2.5;
        sphere3.set_material(material3);
        let ray = Ray::new(Tuple::point(0.0, 0.0, -4.0), Tuple::vector(0.0, 0.0, 1.0));

        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(2.0, sphere1.box_clone()));
        intersections.add(Intersection::new(2.75, sphere2.box_clone()));
        intersections.add(Intersection::new(3.25, sphere3.box_clone()));
        intersections.add(Intersection::new(4.75, sphere2.box_clone()));
        intersections.add(Intersection::new(5.25, sphere3.box_clone()));
        intersections.add(Intersection::new(6.00, sphere1.box_clone()));

        let n1s = [1.0, 1.5, 2.0, 2.5, 2.5, 1.5];
        let n2s = [1.5, 2.0, 2.5, 2.5, 1.5, 1.0];
        for (intersection, (n1, n2)) in intersections.intersections.iter().zip(n1s.iter().zip(n2s.iter())) {
            let prepared_computations = intersection.prepare_computations(&ray, &intersections);
            assert_eq!(prepared_computations.n1, *n1);
            assert_eq!(prepared_computations.n2, *n2);
        }
    }

    #[test]
    fn under_point_is_below_surface() {
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut sphere = Sphere::glass();
        sphere.set_transformation(Transformations::translation(0.0, 0.0, 1.0));
        let intersection = Intersection::new(5.0, sphere.box_clone());
        let mut intersections = Intersections::new();
        intersections.add(intersection.clone());
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        assert!(computed_hit.under_point.z > crate::EPSILON / 2.0);
        assert!(computed_hit.point.z < computed_hit.under_point.z);
    }
}
