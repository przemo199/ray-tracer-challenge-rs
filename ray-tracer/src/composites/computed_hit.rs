use crate::consts::EPSILON;
use crate::primitives::{Point, Vector};
use crate::shapes::Shape;
use crate::utils::Squared;

#[derive(Clone, Debug)]
pub struct ComputedHit<'shape> {
    pub distance: f64,
    pub shape: &'shape dyn Shape,
    pub point: Point,
    pub over_point: Point,
    pub under_point: Point,
    pub camera_direction: Vector,
    pub normal: Vector,
    pub reflect_direction: Vector,
    pub is_inside: bool,
    pub refractive_index_1: f64,
    pub refractive_index_2: f64,
}

impl ComputedHit<'_> {
    pub fn new(
        distance: impl Into<f64>,
        shape: &dyn Shape,
        point: Point,
        camera_direction: Vector,
        normal: Vector,
        reflect_direction: Vector,
        is_inside: bool,
        refractive_index_1: impl Into<f64>,
        refractive_index_2: impl Into<f64>,
    ) -> ComputedHit {
        let over_point = point + (normal * EPSILON);
        let under_point = point - (normal * EPSILON);
        return ComputedHit {
            distance: distance.into(),
            shape,
            point,
            over_point,
            under_point,
            camera_direction,
            normal,
            reflect_direction,
            is_inside,
            refractive_index_1: refractive_index_1.into(),
            refractive_index_2: refractive_index_2.into(),
        };
    }

    pub fn schlick(&self) -> f64 {
        let mut cos = self.camera_direction.dot(&self.normal);

        if self.refractive_index_1 > self.refractive_index_2 {
            let refraction_ratio = self.refractive_index_1 / self.refractive_index_2;
            let sin2_t = refraction_ratio.squared() * (1.0 - cos.squared());

            if sin2_t > 1.0 {
                return 1.0;
            }

            cos = (1.0 - sin2_t).sqrt();
        }

        let r0 = ((self.refractive_index_1 - self.refractive_index_2)
            / (self.refractive_index_1 + self.refractive_index_2))
            .squared();
        return (1.0 - r0).mul_add((1.0 - cos).powi(5), r0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::{Intersection, Intersections, Material, Ray};
    use crate::shapes::Sphere;
    use crate::utils::CoarseEq;

    #[test]
    fn schlick_approximation_under_total_internal_reflection() {
        let mut sphere = Sphere::default();
        sphere.material = Material::glass();
        let ray = Ray::new(Point::new(0, 0, 2.0_f64.sqrt() / 2.0), Vector::UP);
        let mut intersections = Intersections::new();
        let boxed_shape = Box::new(sphere);
        intersections.push(Intersection::new(
            -(2.0_f64.sqrt()) / 2.0,
            boxed_shape.as_ref(),
        ));
        intersections.push(Intersection::new(
            2.0_f64.sqrt() / 2.0,
            boxed_shape.as_ref(),
        ));
        let computed_hit = intersections[1].prepare_computations(&ray, &intersections);
        assert_eq!(computed_hit.schlick(), 1.0);
    }

    #[test]
    fn schlick_approximation_perpendicular_to_viewing_angle() {
        let mut sphere = Sphere::default();
        sphere.material = Material::glass();
        let ray = Ray::new(Point::ORIGIN, Vector::UP);
        let mut intersections = Intersections::new();
        let boxed_shape = Box::new(sphere);
        intersections.push(Intersection::new(-1, boxed_shape.as_ref()));
        intersections.push(Intersection::new(1, boxed_shape.as_ref()));
        let computed_hit = intersections[1].prepare_computations(&ray, &intersections);
        assert_eq!(computed_hit.schlick(), 0.04000000000000001);
    }

    #[test]
    fn schlick_approximation_with_small_angle() {
        let mut sphere = Sphere::default();
        sphere.material = Material::glass();
        let ray = Ray::new(Point::new(0, 0.99, -2), Vector::FORWARD);
        let mut intersections = Intersections::new();
        let boxed_shape = Box::new(sphere);
        intersections.push(Intersection::new(1.8589, boxed_shape.as_ref()));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        assert!(computed_hit.schlick().coarse_eq(0.4887308101221217));
    }
}
