use std::fmt::{Display, Formatter};
use std::sync::Arc;
use crate::{Intersection, Intersections, Material, Matrix, Ray, Shape, Transformations, Tuple, TupleTrait};

#[derive(Clone, Debug)]
pub struct Sphere {
    pub material: Material,
    pub transformation: Matrix,
}

impl Sphere {
    pub fn new() -> Sphere {
        let material = Material::default();
        let transformation = Transformations::identity();
        return Sphere { material, transformation };
    }

    pub fn glass() -> Sphere {
        let mut sphere = Sphere::default();
        let mut material = Material::default();
        material.transparency = 1.0;
        material.refractive_index = 1.5;
        sphere.set_material(material);
        return sphere;
    }

    // pub fn normal_at(&self, world_point: Tuple) -> Tuple {
    //     let object_point = self.transformation.clone().inverse() * world_point;
    //     let object_normal = object_point - self.center.clone();
    //     let mut world_normal = self.transformation.inverse().transpose() * object_normal;
    //     world_normal.w = 0.0;
    //     return world_normal.normalize();
    // }

    fn mut_material(&mut self) -> &mut Material {
        return &mut self.material;
    }
}

impl Shape for Sphere {
    fn local_normal_at(&self, point: Tuple) -> Tuple {
        return point;
    }

    fn material(&self) -> Material {
        return self.material.clone();
    }

    fn set_material(&mut self, material: Material) {
        self.material = material
    }

    fn transformation(&self) -> Matrix {
        return self.transformation.clone();
    }

    fn set_transformation(&mut self, transformation: Matrix) {
        self.transformation = transformation;
    }

    fn local_intersect(self: Arc<Self>, local_ray: &Ray) -> Intersections {
        let sphere_to_ray_distance = Tuple::vector(local_ray.origin.x, local_ray.origin.y, local_ray.origin.z);
        let a = local_ray.direction.dot(&local_ray.direction);
        let b = 2.0 * local_ray.direction.dot(&sphere_to_ray_distance);
        let c = sphere_to_ray_distance.dot(&sphere_to_ray_distance) - 1.0;
        let discriminant = b * b - 4.0 * a * c;
        let mut intersections = Intersections::new();
        if discriminant < 0.0 {
            return intersections;
        }
        let discriminant_root = discriminant.sqrt();
        let t_1 = (-b - discriminant_root) / (2.0 * a);
        let t_2 = (-b + discriminant_root) / (2.0 * a);
        intersections.add(Intersection::new(t_1, self.clone()));
        intersections.add(Intersection::new(t_2, self));
        return intersections;
    }
}

impl Default for Sphere {
    fn default() -> Sphere {
        return Sphere::new();
    }
}

impl Display for Sphere {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("Sphere")
            .field("material", &self.material)
            .field("transformation", &self.transformation)
            .finish();
    }
}

impl From<Sphere> for Box<dyn Shape> {
    fn from(sphere: Sphere) -> Box<dyn Shape> {
        return Box::new(sphere);
    }
}

impl PartialEq for Sphere {
    fn eq(&self, rhs: &Sphere) -> bool {
        return self.transformation == rhs.transformation && self.material == rhs.material;
    }
}

#[cfg(test)]
mod tests {
    use crate::Transformations;
    use super::*;

    #[test]
    fn default_transformation() {
        let sphere = Sphere::default();
        // assert_eq!(sphere.center, Tuple::point(0.0, 0.0, 0.0));
        // assert_eq!(sphere.radius, 1.0);
        assert_eq!(sphere.transformation, Transformations::identity());
    }

    #[test]
    fn changing_transformation() {
        let mut sphere = Sphere::default();
        let transformation = Transformations::translation(2.0, 3.0, 4.0);
        sphere.transformation = transformation.clone();
        assert_eq!(sphere.transformation, transformation);
    }

    #[test]
    fn sphere_normal() {
        let sphere = Sphere::default();
        let normal = sphere.normal_at(Tuple::point(1.0, 0.0, 0.0));
        assert_eq!(normal, Tuple::vector(1.0, 0.0, 0.0));

        let normal = sphere.normal_at(Tuple::point(0.0, 1.0, 0.0));
        assert_eq!(normal, Tuple::vector(0.0, 1.0, 0.0));

        let normal = sphere.normal_at(Tuple::point(0.0, 0.0, 1.0));
        assert_eq!(normal, Tuple::vector(0.0, 0.0, 1.0));

        let normal = sphere.normal_at(Tuple::point(3.0_f64.sqrt() / 3.0, 3.0_f64.sqrt() / 3.0, 3.0_f64.sqrt() / 3.0));
        assert_eq!(normal, Tuple::vector(3.0_f64.sqrt() / 3.0, 3.0_f64.sqrt() / 3.0, 3.0_f64.sqrt() / 3.0));
    }

    #[test]
    fn sphere_normal_is_normalized() {
        let sphere = Sphere::default();
        let normal = sphere.normal_at(Tuple::point(3.0_f64.sqrt() / 3.0, 3.0_f64.sqrt() / 3.0, 3.0_f64.sqrt() / 3.0));
        assert_eq!(normal, normal.normalize());
    }

    #[test]
    fn normal_on_translated_sphere() {
        let mut sphere = Sphere::default();
        sphere.transformation = Transformations::translation(0.0, 1.0, 0.0);
        let normal = sphere.normal_at(Tuple::point(0.0, 1.70711, -0.70711));
        assert_eq!(normal, Tuple::vector(0.0, 0.7071067811865475, -0.7071067811865476));
    }

    #[test]
    fn normal_on_transformed_sphere() {
        let mut sphere = Sphere::default();
        let transformation = Transformations::scaling(1.0, 0.5, 1.0) * Transformations::rotation_z(crate::PI / 5.0);
        sphere.transformation = transformation;
        let normal = sphere.normal_at(Tuple::point(0.0, 2.0_f64.sqrt() / 2.0, -(2.0_f64.sqrt()) / 2.0));
        assert_eq!(normal, Tuple::vector(0.0, 0.9701425001453319, -0.24253562503633294));
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
