use super::{Intersect, Shape, Transform};
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::consts::BINCODE_CONFIG;
use crate::primitives::{Point, Transformation, Vector};
use crate::utils::solve_quadratic;
use bincode::Encode;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Encode)]
pub struct Sphere {
    pub material: Material,
    transformation_inverse: Transformation,
}

impl Sphere {
    pub fn new(material: Material, transformation: Transformation) -> Self {
        return Self {
            material,
            transformation_inverse: transformation.inverse(),
        };
    }
}

impl Transform for Sphere {
    fn set_transformation(&mut self, transformation: Transformation) {
        self.transformation_inverse = transformation.inverse();
    }

    fn transformation(&self) -> Transformation {
        return self.transformation_inverse.inverse();
    }

    fn transformation_inverse(&self) -> Transformation {
        return self.transformation_inverse;
    }
}

impl Intersect for Sphere {
    fn local_intersect<'shape>(&'shape self, ray: &Ray, intersections: &mut Intersections<'shape>) {
        let sphere_to_ray_distance: Vector = ray.origin.into();
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot(&sphere_to_ray_distance);
        let c = sphere_to_ray_distance.dot(&sphere_to_ray_distance) - 1.0;

        solve_quadratic(a, b, c).map(|(distance_1, distance_2)| {
            intersections.extend([
                Intersection::new(distance_1, self),
                Intersection::new(distance_2, self),
            ]);
        });
    }
}

impl Shape for Sphere {
    fn local_normal_at(&self, point: Point) -> Vector {
        return Vector::new(point.x, point.y, point.z);
    }

    fn material(&self) -> &Material {
        return &self.material;
    }

    fn encoded(&self) -> Vec<u8> {
        return bincode::encode_to_vec(self, BINCODE_CONFIG).expect("Failed to serialise Sphere");
    }
}

impl Default for Sphere {
    fn default() -> Self {
        return Self::new(Material::default(), Transformation::IDENTITY);
    }
}

impl Display for Sphere {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("Sphere")
            .field("material", &self.material)
            .field("transformation", &self.transformation_inverse)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::PI;
    use crate::primitives::{transformations, Vector};
    use core::default::Default;

    #[test]
    fn default_transformation() {
        let sphere = Sphere::default();
        assert_eq!(sphere.transformation_inverse, Transformation::IDENTITY);
    }

    #[test]
    fn changing_transformation() {
        let mut sphere = Sphere::default();
        let transformation = transformations::translation(2, 3, 4);
        sphere.transformation_inverse = transformation;
        assert_eq!(sphere.transformation_inverse, transformation);
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

        let third_of_sqrt_3 = 3.0_f64.sqrt() / 3.0;
        let normal = sphere.normal_at(Point::new(
            third_of_sqrt_3,
            third_of_sqrt_3,
            third_of_sqrt_3,
        ));
        assert_eq!(
            normal,
            Vector::new(third_of_sqrt_3, third_of_sqrt_3, third_of_sqrt_3)
        );
    }

    #[test]
    fn sphere_normal_is_normalized() {
        let sphere = Sphere::default();
        let third_of_sqrt_3 = 3.0_f64.sqrt() / 3.0;
        let normal = sphere.normal_at(Point::new(
            third_of_sqrt_3,
            third_of_sqrt_3,
            third_of_sqrt_3,
        ));
        assert_eq!(normal, normal.normalized());
    }

    #[test]
    fn normal_on_translated_sphere() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::translation(0, 1, 0));
        let normal = sphere.normal_at(Point::new(
            0.0,
            1.0 + core::f64::consts::FRAC_1_SQRT_2,
            -core::f64::consts::FRAC_1_SQRT_2,
        ));
        assert_eq!(
            normal,
            Vector::new(
                0,
                core::f64::consts::FRAC_1_SQRT_2,
                -core::f64::consts::FRAC_1_SQRT_2
            )
        );
    }

    #[test]
    fn normal_on_transformed_sphere() {
        let mut sphere = Sphere::default();
        sphere.set_transformation(
            transformations::scaling(1, 0.5, 1) * transformations::rotation_z(PI / 5.0),
        );
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
}
