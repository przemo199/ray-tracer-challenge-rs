use crate::composites::{ComputedHit, Intersections, Material, Ray};
use crate::shapes::Shape;
use crate::utils::CoarseEq;
use core::cmp::Ordering;
use core::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub struct Intersection<'shape> {
    pub distance: f64,
    pub shape: &'shape dyn Shape,
}

impl Intersection<'_> {
    pub fn new(distance: impl Into<f64>, shape: &dyn Shape) -> Intersection {
        return Intersection {
            distance: distance.into(),
            shape,
        };
    }

    pub fn prepare_computations(&self, ray: &Ray, intersections: &Intersections) -> ComputedHit {
        let point = ray.position(self.distance);
        let mut normal = self.shape.normal_at(point);
        let camera_direction = -ray.direction;
        let is_inside = normal.dot(&camera_direction) < 0.0;

        if is_inside {
            normal = -normal;
        }

        let reflect_direction = ray.direction.reflect(&normal);

        let mut shapes: Vec<&dyn Shape> = Vec::new();
        let mut refractive_index_1: f64 = Material::DEFAULT_REFRACTIVE_INDEX;
        let mut refractive_index_2: f64 = Material::DEFAULT_REFRACTIVE_INDEX;

        for intersection in intersections {
            let is_self = self == intersection;
            if is_self {
                refractive_index_1 = shapes
                    .last()
                    .map_or(Material::DEFAULT_REFRACTIVE_INDEX, |shape| {
                        shape.material().refractive_index
                    });
            }

            let position = shapes.iter().position(|shape| *shape == intersection.shape);
            if let Some(value) = position {
                shapes.remove(value);
            } else {
                shapes.push(intersection.shape);
            }

            if is_self {
                refractive_index_2 = shapes
                    .last()
                    .map_or(Material::DEFAULT_REFRACTIVE_INDEX, |shape| {
                        shape.material().refractive_index
                    });
                break;
            }
        }

        return ComputedHit::new(
            self.distance,
            self.shape,
            point,
            camera_direction,
            normal,
            reflect_direction,
            refractive_index_1,
            refractive_index_2,
            is_inside,
        );
    }

    pub fn is_within_distance(&self, distance: f64) -> bool {
        return self.distance >= 0.0 && self.distance < distance;
    }
}

impl CoarseEq<Intersection<'_>> for Intersection<'_> {
    fn coarse_eq(&self, rhs: &Intersection) -> bool {
        return std::ptr::eq(self, rhs)
            || self.distance.coarse_eq(&rhs.distance) && self.shape == rhs.shape;
    }
}

impl Eq for Intersection<'_> {}

impl PartialOrd for Intersection<'_> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        return self.distance.partial_cmp(&rhs.distance);
    }
}

impl Ord for Intersection<'_> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        return self.partial_cmp(rhs).unwrap_or(Ordering::Equal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::{Intersections, Material, Ray};
    use crate::consts::EPSILON;
    use crate::primitives::transformations;
    use crate::primitives::{Point, Vector};
    use crate::shapes::{Plane, Shape, Sphere, Transform};

    #[test]
    fn new_intersection() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersection = Intersection::new(3.5, boxed_shape.as_ref());
        assert_eq!(intersection.distance, 3.5);
        assert_eq!(intersection.shape, boxed_shape.as_ref());
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
        assert_eq!(computed_hit.shape, boxed_shape.as_ref());
        assert_eq!(computed_hit.point, Point::new(0, 0, -1));
        assert_eq!(computed_hit.camera_direction, Vector::BACKWARD);
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
        assert_eq!(computations.camera_direction, Vector::BACKWARD);
    }

    #[test]
    fn hit_offsets_point() {
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::translation(0, 0, 1));
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
            prepared_computations.reflect_direction,
            Vector::new(0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0)
        );
    }

    #[test]
    fn finding_refractive_indexes() {
        let mut sphere1 = Sphere::default();
        sphere1.material = Material::glass();
        sphere1.set_transformation(transformations::scaling(2, 2, 2));

        let mut material1 = sphere1.material.clone();
        material1.refractive_index = 1.5;
        sphere1.material = material1;
        let boxed_shape1: Box<dyn Shape> = Box::new(sphere1);

        let mut sphere2 = Sphere::default();
        sphere2.material = Material::glass();
        sphere2.set_transformation(transformations::translation(0, 0, -0.25));

        let mut material2 = sphere2.material.clone();
        material2.refractive_index = 2.0;
        sphere2.material = material2;
        let boxed_shape2: Box<dyn Shape> = Box::new(sphere2);

        let mut sphere3 = Sphere::default();
        sphere3.material = Material::glass();
        sphere3.set_transformation(transformations::translation(0, 0, 0.25));

        let mut material3 = sphere3.material.clone();
        material3.refractive_index = 2.5;
        sphere3.material = material3;
        let boxed_shape3: Box<dyn Shape> = Box::new(sphere3);

        let mut intersections = Intersections::new();
        intersections.push(Intersection::new(2, boxed_shape1.as_ref()));
        intersections.push(Intersection::new(2.75, boxed_shape2.as_ref()));
        intersections.push(Intersection::new(3.25, boxed_shape3.as_ref()));
        intersections.push(Intersection::new(4.75, boxed_shape2.as_ref()));
        intersections.push(Intersection::new(5.25, boxed_shape3.as_ref()));
        intersections.push(Intersection::new(6, boxed_shape1.as_ref()));

        let refractive_indexes_1 = [1.0, 1.5, 2.0, 2.5, 2.5, 1.5];
        let refractive_indexes_2 = [1.5, 2.0, 2.5, 2.5, 1.5, 1.0];
        let ray = Ray::new(Point::new(0, 0, -4), Vector::FORWARD);

        for (intersection, (refractive_index_1, refractive_index_2)) in intersections
            .into_iter()
            .zip(refractive_indexes_1.iter().zip(refractive_indexes_2.iter()))
        {
            let computed_hit = intersection.prepare_computations(&ray, &intersections);
            assert_eq!(computed_hit.refractive_index_1, *refractive_index_1);
            assert_eq!(computed_hit.refractive_index_2, *refractive_index_2);
        }
    }

    #[test]
    fn under_point_is_below_surface() {
        let mut sphere = Sphere::default();
        sphere.material = Material::glass();
        sphere.set_transformation(transformations::translation(0, 0, 1));
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let intersection = Intersection::new(5, boxed_shape.as_ref());
        let mut intersections = Intersections::new();
        intersections.push(intersection.clone());
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        assert!(computed_hit.under_point.z > EPSILON / 2.0);
        assert!(computed_hit.point.z < computed_hit.under_point.z);
    }
}
