use crate::composites::ComputedHit;
use crate::patterns::Pattern;
use crate::primitives::{Color, Light, Point, Vector};
use crate::shapes::Shape;
use crate::utils::CoarseEq;
use bincode::Encode;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Clone, Debug, Encode)]
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
    pub fn new(
        color: Color,
        pattern: Option<Arc<dyn Pattern>>,
        ambient: impl Into<f64>,
        diffuse: impl Into<f64>,
        specular: impl Into<f64>,
        shininess: impl Into<f64>,
        reflectiveness: impl Into<f64>,
        transparency: impl Into<f64>,
        refractive_index: impl Into<f64>,
    ) -> Material {
        return Material {
            color,
            pattern,
            ambient: ambient.into(),
            diffuse: diffuse.into(),
            specular: specular.into(),
            shininess: shininess.into(),
            reflectiveness: reflectiveness.into(),
            transparency: transparency.into(),
            refractive_index: refractive_index.into(),
        };
    }

    pub fn empty() -> Self {
        return Material::new(Color::BLACK, None, 0, 0, 0, 0, 0, 0, 1);
    }

    pub fn lighting(
        &self,
        shape: &dyn Shape,
        light: &Light,
        point: &Point,
        camera_vector: &Vector,
        normal_vector: &Vector,
        is_shadowed: &bool,
    ) -> Color {
        let diffuse: Color;
        let specular: Color;
        let color = match &self.pattern {
            Some(pattern) => pattern.color_at_shape(shape, point),
            None => self.color,
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

    pub fn lighting_from_computed_hit(
        &self,
        computed_hit: &ComputedHit,
        light: &Light,
        is_shadowed: &bool,
    ) -> Color {
        return self.lighting(
            computed_hit.object,
            light,
            &computed_hit.point,
            &computed_hit.camera_vector,
            &computed_hit.normal_vector,
            is_shadowed,
        );
    }
}

impl Default for Material {
    fn default() -> Material {
        return Material::new(Color::WHITE, None, 0.1, 0.9, 0.9, 200, 0, 0, 1);
    }
}

impl Display for Material {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter
            .debug_struct("Material")
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
        return self.color == rhs.color
            && self.pattern == rhs.pattern
            && self.ambient.coarse_eq(rhs.ambient)
            && self.diffuse.coarse_eq(rhs.diffuse)
            && self.specular.coarse_eq(rhs.specular)
            && self.shininess.coarse_eq(rhs.shininess)
            && self.reflectiveness.coarse_eq(rhs.reflectiveness)
            && self.refractive_index.coarse_eq(rhs.refractive_index)
            && self.transparency.coarse_eq(rhs.transparency);
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
        let shape = Sphere::default();
        let position = Point::ORIGIN;
        let camera = Vector::BACKWARD;
        let normal = Vector::BACKWARD;
        let light = Light::new(Point::new(0, 0, -10), Color::WHITE);
        let result = shape
            .material
            .lighting(&shape, &light, &position, &camera, &normal, &false);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn lighting_with_camera_between_light_and_surface_eye_offset_45_degree() {
        let shape = Sphere::default();
        let position = Point::ORIGIN;
        let camera = Vector::new(0, 2.0_f64.sqrt() / 2.0, -(2.0_f64.sqrt()) / 2.0);
        let normal = Vector::BACKWARD;
        let light = Light::new(Point::new(0, 0, -10), Color::WHITE);
        let result = shape
            .material
            .lighting(&shape, &light, &position, &camera, &normal, &false);
        assert_eq!(result, Color::new(1, 1, 1));
    }

    #[test]
    fn lighting_with_camera_opposite_surface_light_offset_45_degree() {
        let shape = Sphere::default();
        let position = Point::ORIGIN;
        let camera = Vector::BACKWARD;
        let normal = Vector::BACKWARD;
        let light = Light::new(Point::new(0, 10, -10), Color::WHITE);
        let result = shape
            .material
            .lighting(&shape, &light, &position, &camera, &normal, &false);
        assert_eq!(
            result,
            Color::new(0.7363961030678927, 0.7363961030678927, 0.7363961030678927)
        );
    }

    #[test]
    fn lighting_with_camera_in_path_of_reflection_vector() {
        let shape = Sphere::default();
        let position = Point::ORIGIN;
        let camera = Vector::new(0, -(2.0_f64.sqrt()) / 2.0, -(2.0_f64.sqrt()) / 2.0);
        let normal = Vector::BACKWARD;
        let light = Light::new(Point::new(0, 10, -10), Color::WHITE);
        let result = shape
            .material
            .lighting(&shape, &light, &position, &camera, &normal, &false);
        assert_eq!(
            result,
            Color::new(1.6363961030678928, 1.6363961030678928, 1.6363961030678928)
        );
    }

    #[test]
    fn lighting_with_light_behind_surface() {
        let shape = Sphere::default();
        let position = Point::ORIGIN;
        let camera = Vector::BACKWARD;
        let normal = Vector::BACKWARD;
        let light = Light::new(Point::new(0, 0, 10), Color::WHITE);
        let result = shape
            .material
            .lighting(&shape, &light, &position, &camera, &normal, &false);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_surface_in_shadow() {
        let shape = Sphere::default();
        let position = Point::ORIGIN;
        let camera = Vector::BACKWARD;
        let normal = Vector::BACKWARD;
        let light = Light::new(Point::new(0, 0, -10), Color::WHITE);
        let result = shape
            .material
            .lighting(&shape, &light, &position, &camera, &normal, &true);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }
}
