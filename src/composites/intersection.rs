use crate::composites::{ComputedHit, Intersections, Ray};
use crate::shapes::Shape;
use crate::utils::CoarseEq;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Intersection<'a> {
    pub distance: f64,
    pub object: &'a dyn Shape,
}

impl<'a> Intersection<'a> {
    pub fn new(distance: impl Into<f64>, object: &dyn Shape) -> Intersection {
        return Intersection {
            distance: distance.into(),
            object,
        };
    }

    pub fn prepare_computations(
        &'a self,
        ray: &Ray,
        intersections: &'a Intersections<'a>,
    ) -> ComputedHit<'a> {
        let point = ray.position(self.distance);
        let mut normal = self.object.normal_at(point);
        let camera_vector = -ray.direction;
        let is_inside = normal.dot(&camera_vector) < 0.0;

        if is_inside {
            normal = -normal;
        }

        let reflection_vector = ray.direction.reflect(&normal);

        let mut containers: Vec<usize> = Vec::new();
        let mut refractive_index_1: f64 = 1.0;
        let mut refractive_index_2: f64 = 1.0;

        let encoded_objects: Vec<_> = intersections
            .into_iter()
            .map(|intersection| intersection.object.encoded())
            .collect();

        for (index, intersection) in intersections.intersections.iter().enumerate() {
            if intersection == self {
                if containers.is_empty() {
                    refractive_index_1 = 1.0;
                } else {
                    refractive_index_1 = intersections[*containers.last().unwrap()]
                        .object
                        .material()
                        .refractive_index;
                }
            }

            let len = containers.len();
            containers.retain(|entry| encoded_objects[*entry] != encoded_objects[index]);
            if len == containers.len() {
                containers.push(index);
            }

            if intersection == self {
                if containers.is_empty() {
                    refractive_index_2 = 1.0;
                } else {
                    refractive_index_2 = intersections[*containers.last().unwrap()]
                        .object
                        .material()
                        .refractive_index;
                }
                break;
            }
        }

        return ComputedHit::new(
            self.distance,
            self.object,
            point,
            camera_vector,
            normal,
            reflection_vector,
            is_inside,
            refractive_index_1,
            refractive_index_2,
        );
    }

    pub fn is_within_distance(&self, distance: impl Into<f64>) -> bool {
        return self.distance >= 0.0 && self.distance < distance.into();
    }
}

impl<'a> PartialEq<Intersection<'a>> for Intersection<'a> {
    fn eq(&self, rhs: &Intersection) -> bool {
        return self.distance.coarse_eq(rhs.distance)
            && self.object.encoded() == rhs.object.encoded();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::{Intersections, Material, Ray};
    use crate::consts::EPSILON;
    use crate::primitives::transformations;
    use crate::primitives::{Point, Vector};
    use crate::shapes::{Plane, Shape, Sphere};

    #[test]
    fn new_intersection() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersection = Intersection::new(3.5, boxed_shape.as_ref());
        assert_eq!(intersection.distance, 3.5);
        assert_eq!(intersection.object, boxed_shape.as_ref());
    }

    #[test]
    fn precomputing_intersection_state() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersection = Intersection::new(4, boxed_shape.as_ref());
        let intersections = Intersections::new();
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        assert_eq!(computed_hit.distance, intersection.distance);
        assert_eq!(computed_hit.object, boxed_shape.as_ref());
        assert_eq!(computed_hit.point, Point::new(0, 0, -1));
        assert_eq!(computed_hit.camera_vector, Vector::BACKWARD);
        assert_eq!(computed_hit.normal, Vector::BACKWARD);
    }

    #[test]
    fn hit_when_intersection_is_outside() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersection = Intersection::new(4, boxed_shape.as_ref());
        let intersections = Intersections::new();
        let computations = intersection.prepare_computations(&ray, &intersections);
        assert!(!computations.is_inside);
    }

    #[test]
    fn hit_when_intersection_is_inside() {
        let ray = Ray::new(Point::ORIGIN, Vector::FORWARD);
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersection = Intersection::new(1, boxed_shape.as_ref());
        let intersections = Intersections::new();
        let computations = intersection.prepare_computations(&ray, &intersections);
        assert!(computations.is_inside);
        assert_eq!(computations.point, Point::new(0, 0, 1));
        assert_eq!(computations.camera_vector, Vector::BACKWARD);
    }

    #[test]
    fn hit_offsets_point() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let sphere = Sphere {
            transformation: transformations::translation(0, 0, 1),
            ..Default::default()
        };
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersection = Intersection::new(5, boxed_shape.as_ref());
        let intersections = Intersections::new();
        let computations = intersection.prepare_computations(&ray, &intersections);
        assert!(computations.over_point.z < -EPSILON / 2.0);
        assert!(computations.point.z > computations.over_point.z);
    }

    #[test]
    fn precomputing_reflection_vector() {
        let plane = Plane::default();
        let boxed_shape: Box<dyn Shape> = Box::new(plane);
        let ray = Ray::new(
            Point::new(0, 1, -1),
            Vector::new(0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let intersection = Intersection::new(2.0_f64.sqrt(), boxed_shape.as_ref());
        let intersections = Intersections::new();
        let prepared_computations = intersection.prepare_computations(&ray, &intersections);
        assert_eq!(
            prepared_computations.reflection_vector,
            Vector::new(0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0)
        );
    }

    #[test]
    fn finding_refractive_indexes() {
        let mut sphere1 = Sphere {
            material: Material::glass(),
            ..Default::default()
        };
        sphere1.set_transformation(transformations::scaling(2, 2, 2));
        let mut material1 = sphere1.material.clone();
        material1.refractive_index = 1.5;
        sphere1.set_material(material1);
        let boxed_shape1: Box<dyn Shape> = Box::new(sphere1);

        let mut sphere2 = Sphere {
            material: Material::glass(),
            ..Default::default()
        };
        sphere2.set_transformation(transformations::translation(0, 0, -0.25));
        let mut material2 = sphere2.material.clone();
        material2.refractive_index = 2.0;
        sphere2.set_material(material2);
        let boxed_shape2: Box<dyn Shape> = Box::new(sphere2);

        let mut sphere3 = Sphere {
            material: Material::glass(),
            ..Default::default()
        };
        sphere3.set_transformation(transformations::translation(0, 0, 0.25));
        let mut material3 = sphere3.material.clone();
        material3.refractive_index = 2.5;
        sphere3.set_material(material3);
        let boxed_shape3: Box<dyn Shape> = Box::new(sphere3);

        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(2, boxed_shape1.as_ref()));
        intersections.add(Intersection::new(2.75, boxed_shape2.as_ref()));
        intersections.add(Intersection::new(3.25, boxed_shape3.as_ref()));
        intersections.add(Intersection::new(4.75, boxed_shape2.as_ref()));
        intersections.add(Intersection::new(5.25, boxed_shape3.as_ref()));
        intersections.add(Intersection::new(6, boxed_shape1.as_ref()));

        let refractive_indexes_1 = [1.0, 1.5, 2.0, 2.5, 2.5, 1.5];
        let refractive_indexes_2 = [1.5, 2.0, 2.5, 2.5, 1.5, 1.0];
        let ray = Ray::new(Point::new(0, 0, -4), Vector::FORWARD);

        for (intersection, (refractive_index_1, refractive_index_2)) in intersections
            .intersections
            .iter()
            .zip(refractive_indexes_1.iter().zip(refractive_indexes_2.iter()))
        {
            let computed_hit = intersection.prepare_computations(&ray, &intersections);
            assert_eq!(computed_hit.refractive_index_1, *refractive_index_1);
            assert_eq!(computed_hit.refractive_index_2, *refractive_index_2);
        }
    }

    #[test]
    fn under_point_is_below_surface() {
        let mut sphere = Sphere {
            material: Material::glass(),
            ..Default::default()
        };
        sphere.set_transformation(transformations::translation(0, 0, 1));
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersection = Intersection::new(5, boxed_shape.as_ref());
        let mut intersections = Intersections::new();
        intersections.add(intersection.clone());
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        assert!(computed_hit.under_point.z > EPSILON / 2.0);
        assert!(computed_hit.point.z < computed_hit.under_point.z);
    }
}
