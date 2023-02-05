use crate::color::Color;
use crate::consts::EPSILON;
use crate::light::Light;
use crate::pattern::Pattern;
use crate::shape::Shape;
use crate::tuple::{Tuple, TupleTrait};

#[derive(Clone, Debug)]
pub struct Material {
    pub color: Color,
    pub pattern: Option<Box<dyn Pattern>>,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflectiveness: f64,
    pub refractive_index: f64,
    pub transparency: f64,
}

impl Material {
    pub fn new(color: Color, pattern: Option<Box<dyn Pattern>>, ambient: f64, diffuse: f64, specular: f64, shininess: f64, reflectiveness: f64, transparency: f64, refractive_index: f64) -> Material {
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

    pub fn lighting(&self, object: &dyn Shape, light: &Light, point: &Tuple, camera_vector: &Tuple, normal_vector: &Tuple, in_shadow: bool) -> Color {
        let diffuse: Color;
        let specular: Color;
        let color = match &self.pattern {
            Some(pattern) => pattern.color_at_shape(object, point),
            None => self.color
        };
        let effective_color = color * light.intensity;
        let ambient = effective_color * self.ambient;

        if in_shadow {
            return ambient;
        }

        let light_vector = (light.position - *point).normalize();
        let light_dot_normal = light_vector.dot(normal_vector);
        if light_dot_normal < 0.0 {
            diffuse = Color::black();
            specular = Color::black();
        } else {
            diffuse = effective_color * self.diffuse * light_dot_normal;
            let reflect_vector = (-light_vector).reflect(normal_vector);
            let reflect_dot_camera = reflect_vector.dot(camera_vector);

            if reflect_dot_camera <= 0.0 {
                specular = Color::black();
            } else {
                let factor = reflect_dot_camera.powf(self.shininess);
                specular = light.intensity * self.specular * factor;
            }
        }

        return ambient + diffuse + specular;
    }
}

impl Default for Material {
    fn default() -> Material {
        return Material::new(Color::new(1.0, 1.0, 1.0), None, 0.1, 0.9, 0.9, 200.0, 0.0, 0.0, 1.0);
    }
}

impl PartialEq for Material {
    fn eq(&self, rhs: &Self) -> bool {
        return self.color == rhs.color &&
            self.pattern == rhs.pattern &&
            (self.ambient - rhs.ambient).abs() < EPSILON &&
            (self.diffuse - rhs.diffuse).abs() < EPSILON &&
            (self.specular - rhs.specular).abs() < EPSILON &&
            (self.shininess - rhs.shininess).abs() < EPSILON &&
            (self.reflectiveness - rhs.reflectiveness).abs() < EPSILON &&
            (self.refractive_index - rhs.refractive_index).abs() < EPSILON &&
            (self.transparency - rhs.transparency).abs() < EPSILON;
    }
}

#[cfg(test)]
mod tests {
    use crate::light::Light;
    use crate::sphere::Sphere;
    use crate::tuple::Tuple;
    use super::*;

    #[test]
    fn default_material() {
        let material = Material::default();
        assert_eq!(material.color, Color::new(1.0, 1.0, 1.0));
        assert_eq!(material.ambient, 0.1);
        assert_eq!(material.diffuse, 0.9);
        assert_eq!(material.specular, 0.9);
        assert_eq!(material.shininess, 200.0);
        assert_eq!(material.reflectiveness, 0.0);
        assert_eq!(material.transparency, 0.0);
        assert_eq!(material.refractive_index, 1.0);
    }

    #[test]
    fn lighting_with_camera_between_sun_and_surface() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let camera = Tuple::point(0.0, 0.0, -1.0);
        let normal = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = material.lighting(&object, &light, &position, &camera, &normal, false);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn lighting_with_camera_between_sun_and_surface_eye_offset_45_degree() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let camera = Tuple::point(0.0, 2.0_f64.sqrt() / 2.0, -(2.0_f64.sqrt()) / 2.0);
        let normal = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = material.lighting(&object, &light, &position, &camera, &normal, false);
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn lighting_with_camera_opposite_surface_light_offset_45_degree() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let camera = Tuple::point(0.0, 0.0, -1.0);
        let normal = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = material.lighting(&object, &light, &position, &camera, &normal, false);
        assert_eq!(result, Color::new(0.7363961030678927, 0.7363961030678927, 0.7363961030678927));
    }

    #[test]
    fn lighting_with_camera_in_path_of_reflection_vector() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let camera = Tuple::point(0.0, -(2.0_f64.sqrt()) / 2.0, -(2.0_f64.sqrt()) / 2.0);
        let normal = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = material.lighting(&object, &light, &position, &camera, &normal, false);
        assert_eq!(result, Color::new(1.6363961030678928, 1.6363961030678928, 1.6363961030678928));
    }

    #[test]
    fn lighting_with_light_behind_surface() {
        let object = Sphere::default();
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let camera = Tuple::point(0.0, 0.0, -1.0);
        let normal = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new(Tuple::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));
        let result = material.lighting(&object, &light, &position, &camera, &normal, false);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_surface_in_shadow() {
        let object = Sphere::default();
        let material = Material::default();
        let camera_vector = Tuple::vector(0.0, 0.0, -1.0);
        let normal_vector = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let in_shadow = true;
        let position = Tuple::point(0.0, 0.0, 0.0);
        let result = material.lighting(&object, &light, &position, &camera_vector, &normal_vector, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }
}
