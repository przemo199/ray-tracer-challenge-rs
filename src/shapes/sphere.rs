use super::Shape;
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::BINCODE_CONFIG;
use crate::primitives::{transformations, Transformation};
use crate::primitives::{Point, Vector};
use crate::utils::solve_quadratic;
use bincode::Encode;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct Sphere {
    pub material: Material,
    pub transformation: Transformation,
}

impl Sphere {
    pub fn new(material: Material, transformation: Transformation) -> Sphere {
        return Sphere {
            material,
            transformation,
        };
    }

    pub fn glass() -> Sphere {
        let mut sphere = Sphere::default();
        let mut material = Material::default();
        material.transparency = 1.0;
        material.refractive_index = 1.5;
        sphere.set_material(material);
        return sphere;
    }

    fn mut_material(&mut self) -> &mut Material {
        return &mut self.material;
    }
}

impl Shape for Sphere {
    fn local_normal_at(&self, point: Point) -> Vector {
        return Vector::new(point.x, point.y, point.z);
    }

    fn material(&self) -> Material {
        return self.material.clone();
    }

    fn set_material(&mut self, material: Material) {
        self.material = material
    }

    fn transformation(&self) -> Transformation {
        return self.transformation;
    }

    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation = transformation;
    }

    fn local_intersect(&self, local_ray: &Ray) -> Option<Intersections> {
        let sphere_to_ray_distance =
            Vector::new(local_ray.origin.x, local_ray.origin.y, local_ray.origin.z);
        let a = local_ray.direction.dot(&local_ray.direction);
        let b = 2.0 * local_ray.direction.dot(&sphere_to_ray_distance);
        let c = sphere_to_ray_distance.dot(&sphere_to_ray_distance) - 1.0;

        return solve_quadratic(a, b, c).map(|(distance_1, distance_2)| {
            return Intersections::with([
                Intersection::new(distance_1, self),
                Intersection::new(distance_2, self),
            ]);
        });
    }

    fn encoded(&self) -> Vec<u8> {
        return bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap();
    }
}

impl Default for Sphere {
    fn default() -> Sphere {
        let material = Material::default();
        let transformation = transformations::IDENTITY;
        return Sphere {
            material,
            transformation,
        };
    }
}

impl Display for Sphere {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter
            .debug_struct("Sphere")
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use crate::composites::Material;
    use crate::consts::PI;

    use super::*;

    #[test]
    fn default_transformation() {
        let sphere = Sphere::default();
        assert_eq!(sphere.transformation, transformations::IDENTITY);
    }

    #[test]
    fn changing_transformation() {
        let mut sphere = Sphere::default();
        let transformation = transformations::translation(2, 3, 4);
        sphere.transformation = transformation;
        assert_eq!(sphere.transformation, transformation);
    }

    #[test]
    fn sphere_normal() {
        let sphere = Sphere::default();
        let normal = sphere.normal_at(Point::new(1, 0, 0));
        assert_eq!(normal, Vector::RIGHT);

        let normal = sphere.normal_at(Point::new(0, 1, 0));
        assert_eq!(normal, Vector::UP);

        let normal = sphere.normal_at(Point::new(0, 0, 1));
        assert_eq!(normal, Vector::FORWARD);

        let normal = sphere.normal_at(Point::new(
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
        ));
        assert_eq!(
            normal,
            Vector::new(
                3.0_f64.sqrt() / 3.0,
                3.0_f64.sqrt() / 3.0,
                3.0_f64.sqrt() / 3.0
            )
        );
    }

    #[test]
    fn sphere_normal_is_normalized() {
        let sphere = Sphere::default();
        let normal = sphere.normal_at(Point::new(
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
        ));
        assert_eq!(normal, normal.normalized());
    }

    #[test]
    fn normal_on_translated_sphere() {
        let mut sphere = Sphere::default();
        sphere.transformation = transformations::translation(0, 1, 0);
        let normal = sphere.normal_at(Point::new(
            0.0,
            1.0 + std::f64::consts::FRAC_1_SQRT_2,
            -std::f64::consts::FRAC_1_SQRT_2,
        ));
        assert_eq!(
            normal,
            Vector::new(
                0,
                std::f64::consts::FRAC_1_SQRT_2,
                -std::f64::consts::FRAC_1_SQRT_2
            )
        );
    }

    #[test]
    fn normal_on_transformed_sphere() {
        let mut sphere = Sphere::default();
        sphere.transformation =
            transformations::scaling(1, 0.5, 1) * transformations::rotation_z(PI / 5.0);
        let normal = sphere.normal_at(Point::new(0, 2.0_f64.sqrt() / 2.0, -(2.0_f64.sqrt()) / 2.0));
        assert_eq!(
            normal,
            Vector::new(0, 0.9701425001453319, -0.24253562503633294)
        );
    }

    #[test]
    fn sphere_has_default_material() {
        let sphere = Sphere::default();
        assert_eq!(sphere.material, Material::default());
    }

    #[test]
    fn sphere_may_be_assigned_material() {
        let mut sphere = Sphere::default();
        let mut material = Material::default();
        material.ambient = 1.0;
        sphere.material = material.clone();
        assert_eq!(sphere.material, material);
    }

    #[test]
    fn glass_sphere() {
        let glass_sphere = Sphere::glass();
        assert_eq!(glass_sphere.material.transparency, 1.0);
        assert_eq!(glass_sphere.material.refractive_index, 1.5);
    }
}
