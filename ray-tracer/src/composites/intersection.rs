use crate::composites::{ComputedHit, Intersections, Material, Ray};
use crate::shapes::Shape;
use crate::utils::CoarseEq;
use core::fmt::Debug;

/// A ray hitting `shape` at `distance` along the ray.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Intersection<'shape> {
    pub distance: f64,
    pub shape: &'shape dyn Shape,
}

impl<'shape> Intersection<'shape> {
    pub fn new(distance: impl Into<f64>, shape: &'shape dyn Shape) -> Self {
        return Intersection {
            distance: distance.into(),
            shape,
        };
    }

    pub fn prepare_computations(
        &self,
        ray: &Ray,
        intersections: &Intersections<'_>,
    ) -> ComputedHit<'shape> {
        let point = ray.position(self.distance);
        let mut normal = self.shape.normal_at(point);
        let camera_direction = -ray.direction;
        let is_inside = normal.dot(&camera_direction) < 0.0;

        if is_inside {
            normal = -normal;
        }

        // Only refraction and the Fresnel term use the refractive indices, and both skip opaque
        // surfaces, so most hits don't need the (comparatively expensive) containers walk.
        let (refractive_index_1, refractive_index_2) = if self.shape.material().is_opaque() {
            (
                Material::DEFAULT_REFRACTIVE_INDEX,
                Material::DEFAULT_REFRACTIVE_INDEX,
            )
        } else {
            self.refractive_indices(intersections)
        };

        return ComputedHit::new(
            self.distance,
            self.shape,
            point,
            camera_direction,
            normal,
            refractive_index_1,
            refractive_index_2,
            is_inside,
        );
    }

    /// Refractive indices on both sides of this hit (the book's "containers" algorithm): walks
    /// `intersections` in order, tracking which shapes the ray is inside.
    fn refractive_indices(&self, intersections: &Intersections<'_>) -> (f64, f64) {
        let innermost = |containers: &[&dyn Shape]| {
            return containers
                .last()
                .map_or(Material::DEFAULT_REFRACTIVE_INDEX, |shape| {
                    shape.material().refractive_index
                });
        };
        let mut containers = intersections.containers();
        containers.clear();
        for intersection in intersections {
            // for this hit: the medium the ray comes from, before entering/leaving the shape
            let refractive_index_1 = (self == intersection).then(|| innermost(&containers));
            match containers
                .iter()
                .position(|shape| *shape == intersection.shape)
            {
                Some(index) => {
                    containers.remove(index);
                }
                None => containers.push(intersection.shape),
            }
            if let Some(refractive_index_1) = refractive_index_1 {
                return (refractive_index_1, innermost(&containers));
            }
        }
        return (
            Material::DEFAULT_REFRACTIVE_INDEX,
            Material::DEFAULT_REFRACTIVE_INDEX,
        );
    }

    pub fn is_within_distance(&self, distance: f64) -> bool {
        return self.distance >= 0.0 && self.distance < distance;
    }
}

impl CoarseEq<Intersection<'_>> for Intersection<'_> {
    fn coarse_eq(&self, other: &Intersection<'_>) -> bool {
        return std::ptr::eq(self, other)
            || self.distance.coarse_eq(&other.distance) && self.shape == other.shape;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::{Intersections, Material, Ray};
    use crate::consts::EPSILON;
    use crate::primitives::transformations;
    use crate::primitives::{Point, Vector};
    use crate::shapes::{Cube, Plane, Shape, Sphere, Transform};

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
            prepared_computations.reflect_direction(),
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
        intersections.push(intersection);
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        assert!(computed_hit.under_point().z > EPSILON / 2.0);
        assert!(computed_hit.point.z < computed_hit.under_point().z);
    }

    #[test]
    fn intersections_compare_distance_and_shape() {
        let sphere: Box<dyn Shape> = Box::new(Sphere::default());
        let equal_copy: Box<dyn Shape> = Box::new(Sphere::default());
        let cube: Box<dyn Shape> = Box::new(Cube::default());
        let intersection = Intersection::new(1, sphere.as_ref());
        assert_eq!(intersection, Intersection::new(1, sphere.as_ref()));
        assert_eq!(intersection, Intersection::new(1, equal_copy.as_ref()));
        assert_ne!(intersection, Intersection::new(1, cube.as_ref()));
        assert_ne!(intersection, Intersection::new(2, sphere.as_ref()));
    }

    #[test]
    fn refractive_indices_are_only_computed_for_non_opaque_surfaces() {
        let mut glass = Sphere::default();
        glass.material = Material::glass();
        glass.set_transformation(transformations::scaling(2, 2, 2));
        let glass: Box<dyn Shape> = Box::new(glass);
        let mut opaque = Sphere::default();
        opaque.material.refractive_index = 2.0;
        let opaque: Box<dyn Shape> = Box::new(opaque);
        let mut intersections = Intersections::new();
        intersections.push(Intersection::new(3, glass.as_ref()));
        intersections.push(Intersection::new(4, opaque.as_ref()));
        intersections.push(Intersection::new(6, opaque.as_ref()));
        intersections.push(Intersection::new(7, glass.as_ref()));
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);

        // the glass surface sees the real media on both sides
        let glass_hit = intersections[0].prepare_computations(&ray, &intersections);
        let indices = (glass_hit.refractive_index_1, glass_hit.refractive_index_2);
        assert_eq!(indices, (1.0, 1.5));
        // nothing reads them for an opaque surface, so they are left at the default
        let opaque_hit = intersections[1].prepare_computations(&ray, &intersections);
        let indices = (opaque_hit.refractive_index_1, opaque_hit.refractive_index_2);
        assert_eq!(indices, (1.0, 1.0));
    }
}
