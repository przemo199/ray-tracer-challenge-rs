use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::computed_hit::ComputedHit;
use crate::patterns::Pattern;
use crate::primitives::{Color, Light, Point, Vector};
use crate::shapes::Shape;
use crate::utils::CloseEnough;

#[derive(Clone, Debug)]
pub struct Material {
    pub color: Color,
    pub pattern: Option<Arc<dyn Pattern>>,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflectiveness: f64,
    pub refractive_index: f64,
    pub transparency: f64,
}

impl Material {
    pub fn new(color: Color, pattern: Option<Arc<dyn Pattern>>, ambient: f64, diffuse: f64, specular: f64, shininess: f64, reflectiveness: f64, transparency: f64, refractive_index: f64) -> Material {
        return Material {
            color,
            pattern,
            ambient,
            diffuse,
            specular,
            shininess,
            reflectiveness,
            transparency,
            refractive_index,
        };
    }

    pub fn empty() -> Self {
        return Material::new(Color::BLACK, None, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    }

    pub fn lighting(&self, object: &dyn Shape, light: &Light, point: &Point, camera_vector: &Vector, normal_vector: &Vector, is_shadowed: &bool) -> Color {
        let diffuse: Color;
        let specular: Color;
        let color = match &self.pattern {
            Some(pattern) => pattern.color_at_shape(object, point),
            None => self.color
        };
        let effective_color = color * light.intensity;
        let ambient = effective_color * self.ambient;

        if *is_shadowed {
            return ambient;
        }

        let light_vector = (light.position - *point).normalized();
        let light_dot_normal = light_vector.dot(normal_vector);
        if light_dot_normal < 0.0 {
            diffuse = Color::BLACK;
            specular = Color::BLACK;
        } else {
            diffuse = effective_color * self.diffuse * light_dot_normal;
            let reflect_vector = (-light_vector).reflect(normal_vector);
            let reflect_dot_camera = reflect_vector.dot(camera_vector);

            if reflect_dot_camera <= 0.0 {
                specular = Color::BLACK;
            } else {
                let factor = reflect_dot_camera.powf(self.shininess);
                specular = light.intensity * self.specular * factor;
            }
        }

        return ambient + diffuse + specular;
    }

    pub fn lighting_from_computed_hit(&self, computed_hit: &ComputedHit, light: &Light, is_shadowed: &bool) -> Color {
        return self.lighting(&*computed_hit.object, light, &computed_hit.point, &computed_hit.camera_vector, &computed_hit.normal_vector, is_shadowed);
    }
}

impl Default for Material {
    fn default() -> Material {
        return Material::new(
            Color::new(1.0, 1.0, 1.0),
            None,
            0.1,
            0.9,
            0.9,
            200.0,
            0.0,
            0.0,
            1.0
        );
    }
}

impl Display for Material {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("Material")
            .field("color", &self.color)
            .field("pattern", &self.pattern)
            .field("ambient", &self.ambient)
            .field("diffuse", &self.diffuse)
            .field("specular", &self.specular)
            .field("shininess", &self.shininess)
            .field("reflectiveness", &self.reflectiveness)
            .field("refractive_index", &self.refractive_index)
            .field("transparency", &self.transparency)
            .finish();
    }
}

impl PartialEq for Material {
    fn eq(&self, rhs: &Self) -> bool {
        return self.color == rhs.color &&
            self.pattern == rhs.pattern &&
            self.ambient.close_enough(rhs.ambient) &&
            self.diffuse.close_enough(rhs.diffuse) &&
            self.specular.close_enough(rhs.specular) &&
            self.shininess.close_enough(rhs.shininess) &&
            self.reflectiveness.close_enough(rhs.reflectiveness) &&
            self.refractive_index.close_enough(rhs.refractive_index) &&
            self.transparency.close_enough(rhs.transparency);
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::Light;
    use crate::shapes::Sphere;

    use super::*;

    #[test]
    fn default_material() {
        let material = Material::default();
        assert_eq!(material.color, Color::WHITE);
        assert_eq!(material.ambient, 0.1);
        assert_eq!(material.diffuse, 0.9);
        assert_eq!(material.specular, 0.9);
        assert_eq!(material.shininess, 200.0);
        assert_eq!(material.reflectiveness, 0.0);
        assert_eq!(material.transparency, 0.0);
        assert_eq!(material.refractive_index, 1.0);
    }

    #[test]
    fn lighting_with_camera_between_light_and_surface() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let camera = Vector::new(0.0, 0.0, -1.0);
        let normal = Vector::new(0.0, 0.0, -1.0);
        let light = Light::new(Point::new(0.0, 0.0, -10.0), Color::WHITE);
        let result = material.lighting(&object, &light, &position, &camera, &normal, &false);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn lighting_with_camera_between_light_and_surface_eye_offset_45_degree() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let camera = Vector::new(0.0, 2.0_f64.sqrt() / 2.0, -(2.0_f64.sqrt()) / 2.0);
        let normal = Vector::new(0.0, 0.0, -1.0);
        let light = Light::new(Point::new(0.0, 0.0, -10.0), Color::WHITE);
        let result = material.lighting(&object, &light, &position, &camera, &normal, &false);
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn lighting_with_camera_opposite_surface_light_offset_45_degree() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let camera = Vector::new(0.0, 0.0, -1.0);
        let normal = Vector::new(0.0, 0.0, -1.0);
        let light = Light::new(Point::new(0.0, 10.0, -10.0), Color::WHITE);
        let result = material.lighting(&object, &light, &position, &camera, &normal, &false);
        assert_eq!(result, Color::new(0.7363961030678927, 0.7363961030678927, 0.7363961030678927));
    }

    #[test]
    fn lighting_with_camera_in_path_of_reflection_vector() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let camera = Vector::new(0.0, -(2.0_f64.sqrt()) / 2.0, -(2.0_f64.sqrt()) / 2.0);
        let normal = Vector::new(0.0, 0.0, -1.0);
        let light = Light::new(Point::new(0.0, 10.0, -10.0), Color::WHITE);
        let result = material.lighting(&object, &light, &position, &camera, &normal, &false);
        assert_eq!(result, Color::new(1.6363961030678928, 1.6363961030678928, 1.6363961030678928));
    }

    #[test]
    fn lighting_with_light_behind_surface() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Point::new(0.0, 0.0, 0.0);
        let camera = Vector::new(0.0, 0.0, -1.0);
        let normal = Vector::new(0.0, 0.0, -1.0);
        let light = Light::new(Point::new(0.0, 0.0, 10.0), Color::WHITE);
        let result = material.lighting(&object, &light, &position, &camera, &normal, &false);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_surface_in_shadow() {
        let object = Sphere::default();
        let material = Material::default();
        let camera_vector = Vector::new(0.0, 0.0, -1.0);
        let normal_vector = Vector::new(0.0, 0.0, -1.0);
        let light = Light::new(Point::new(0.0, 0.0, -10.0), Color::WHITE);
        let in_shadow = true;
        let position = Point::new(0.0, 0.0, 0.0);
        let result = material.lighting(&object, &light, &position, &camera_vector, &normal_vector, &in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }
}
