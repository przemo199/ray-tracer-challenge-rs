use std::sync::Arc;
use crate::consts::EPSILON;
use crate::shape::Shape;
use crate::tuple::{Tuple, TupleTrait};

#[derive(Clone, Debug)]
pub struct ComputedHit {
    pub t: f64,
    pub object: Arc<dyn Shape>,
    pub point: Tuple,
    pub over_point: Tuple,
    pub under_point: Tuple,
    pub camera_vector: Tuple,
    pub normal_vector: Tuple,
    pub reflection_vector: Tuple,
    pub is_inside: bool,
    pub n1: f64,
    pub n2: f64,
}

impl ComputedHit {
    pub fn new(
        t: f64,
        object: Arc<dyn Shape>,
        point: Tuple,
        camera_vector: Tuple,
        normal_vector: Tuple,
        reflection_vector: Tuple,
        is_inside: bool,
        n1: f64,
        n2: f64) -> ComputedHit {
        let over_point = point + normal_vector * EPSILON;
        let under_point = point - normal_vector * EPSILON;
        return ComputedHit {
            t,
            object,
            point,
            over_point,
            under_point,
            camera_vector,
            normal_vector,
            reflection_vector,
            is_inside,
            n1,
            n2,
        };
    }

    pub fn schlick(&self) -> f64 {
        let mut camera_dot_normal = self.camera_vector.dot(&self.normal_vector);

        if self.n1 > self.n2 {
            let n = self.n1 / self.n2;
            let sin2_t = n * n * (1.0 - camera_dot_normal * camera_dot_normal);

            if sin2_t > 1.0 {
                return 1.0;
            }

            camera_dot_normal = (1.0 - sin2_t).sqrt();
        }

        let r0 = ((self.n1 - self.n2) / (self.n1 + self.n2)).powi(2);
        return r0 + (1.0 - r0) * (1.0 - camera_dot_normal).powf(5.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intersection::Intersection;
    use crate::intersections::Intersections;
    use crate::ray::Ray;
    use crate::sphere::Sphere;

    #[test]
    fn schlick_approximation_under_total_internal_reflection() {
        let shape = Sphere::glass();
        let ray = Ray::new(Tuple::point(0.0, 0.0, 2.0_f64.sqrt() / 2.0), Tuple::vector(0.0, 1.0, 0.0));
        let mut intersections = Intersections::new();
        let arc_shape = Arc::new(shape);
        intersections.add(Intersection::new(-(2.0_f64.sqrt()) / 2.0, arc_shape.clone()));
        intersections.add(Intersection::new(2.0_f64.sqrt() / 2.0, arc_shape));
        let computed_hit = intersections[1].prepare_computations(&ray, &intersections);
        assert_eq!(computed_hit.schlick(), 1.0);
    }

    #[test]
    fn schlick_approximation_perpendicular_to_viewing_angle() {
        let shape = Sphere::glass();
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 1.0, 0.0));
        let mut intersections = Intersections::new();
        let arc_shape = Arc::new(shape);
        intersections.add(Intersection::new(-1.0, arc_shape.clone()));
        intersections.add(Intersection::new(1.0, arc_shape));
        let computed_hit = intersections[1].prepare_computations(&ray, &intersections);
        assert_eq!(computed_hit.schlick(), 0.04000000000000001);
    }

    #[test]
    fn schlick_approximation_with_small_angle() {
        let shape = Sphere::glass();
        let ray = Ray::new(Tuple::point(0.0, 0.99, -2.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(1.8589, Arc::new(shape)));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        assert_eq!(computed_hit.schlick(), 0.48873081012212183);
    }
}
