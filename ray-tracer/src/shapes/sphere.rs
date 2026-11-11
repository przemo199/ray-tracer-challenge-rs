use super::{Intersect, Shape, Transform};
use crate::composites::{Intersection, Intersections, Material, Ray};
use crate::impl_shape_transform_for;
use crate::primitives::{Point, Transformation, Vector};
use crate::utils::{CoarseEq, solve_quadratic};
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Sphere {
    transformation_inverse: Transformation,
    /// Transposed inverse, cached for `normal_at`.
    normal_transformation: Transformation,
    pub material: Material,
}

impl Sphere {
    pub fn new(material: Material, transformation: Transformation) -> Self {
        let transformation_inverse = transformation.inverse();
        return Self {
            material,
            transformation_inverse,
            normal_transformation: transformation_inverse.transpose(),
        };
    }
}

impl_shape_transform_for!(Sphere);

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
}

impl Default for Sphere {
    fn default() -> Self {
        return Self::new(Material::default(), Transformation::IDENTITY);
    }
}

impl Display for Sphere {
    fn fmt(&self, f: &mut Formatter) -> Result {
        return f
            .debug_struct("Sphere")
            .field("material", &self.material)
            .field("transformation", &self.transformation())
            .finish();
    }
}

impl CoarseEq for Sphere {
    fn coarse_eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || (self.material == other.material
                && self
                    .transformation_inverse
                    .coarse_eq(&other.transformation_inverse));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::PI;
    use crate::primitives::{Vector, transformations};
    use crate::shapes::Transform;
    use crate::utils::CoarseEq;
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
        sphere.set_transformation(transformation);
        assert_eq!(sphere.transformation(), transformation);
        assert_eq!(sphere.transformation_inverse, transformation.inverse());
        // the cached normal matrix follows every change
        assert_eq!(
            sphere.normal_transformation,
            transformation.inverse().transpose()
        );
        let constructed = Sphere::new(Material::default(), transformation);
        assert_eq!(
            constructed.normal_transformation,
            sphere.normal_transformation
        );
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
        let expected = Vector::new(
            0,
            core::f64::consts::FRAC_1_SQRT_2,
            -core::f64::consts::FRAC_1_SQRT_2,
        );
        assert!(normal.coarse_eq(&expected));
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
