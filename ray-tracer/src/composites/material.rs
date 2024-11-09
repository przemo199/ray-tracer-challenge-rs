use crate::composites::ComputedHit;
use crate::patterns::Pattern;
use crate::primitives::{Color, Light, Point, Vector};
use crate::shapes::Shape;
use crate::utils::CoarseEq;
use bincode::Encode;
use core::fmt::{Display, Formatter, Result};
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
    pub casts_shadow: bool,
}

impl Material {
    /// Refractive index of air
    pub const DEFAULT_REFRACTIVE_INDEX: f64 = 1.0;

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
        casts_shadow: bool,
    ) -> Self {
        return Self {
            color,
            pattern,
            ambient: ambient.into(),
            diffuse: diffuse.into(),
            specular: specular.into(),
            shininess: shininess.into(),
            reflectiveness: reflectiveness.into(),
            transparency: transparency.into(),
            refractive_index: refractive_index.into(),
            casts_shadow,
        };
    }

    #[inline]
    pub fn lighting(
        &self,
        shape: &dyn Shape,
        light: &Light,
        point: &Point,
        camera_direction: &Vector,
        normal: &Vector,
        in_shadow: bool,
    ) -> Color {
        let effective_color = self.resolve_color(shape, point) * light.intensity;

        return self.calculate_lighting(
            &effective_color,
            light,
            point,
            camera_direction,
            normal,
            in_shadow,
        );
    }

    #[inline]
    fn resolve_color(&self, shape: &dyn Shape, point: &Point) -> Color {
        return self
            .pattern
            .as_ref()
            .map_or(self.color, |pattern| pattern.color_at_shape(shape, point));
    }

    #[inline]
    fn calculate_lighting(
        &self,
        effective_color: &Color,
        light: &Light,
        point: &Point,
        camera_direction: &Vector,
        normal: &Vector,
        in_shadow: bool,
    ) -> Color {
        let ambient = *effective_color * self.ambient;

        if in_shadow {
            return ambient;
        }

        let light_direction = (light.position - *point).normalized();
        let light_dot_normal = light_direction.dot(normal);
        if light_dot_normal < 0.0 {
            return ambient;
        }
        let diffuse = *effective_color * self.diffuse * light_dot_normal;
        let reflect_direction = (-light_direction).reflect(normal);
        let reflect_dot_camera = reflect_direction.dot(camera_direction);

        if reflect_dot_camera <= 0.0 {
            return ambient + diffuse;
        }
        let factor = reflect_dot_camera.powf(self.shininess);
        let specular = light.intensity * self.specular * factor;

        return ambient + diffuse + specular;
    }

    pub fn lighting_from_computed_hit(
        &self,
        computed_hit: &ComputedHit,
        light: &Light,
        in_shadow: bool,
    ) -> Color {
        return self.lighting(
            computed_hit.shape,
            light,
            &computed_hit.over_point,
            &computed_hit.camera_direction,
            &computed_hit.normal,
            in_shadow,
        );
    }

    pub fn glass() -> Self {
        return Self {
            transparency: 1.0,
            refractive_index: 1.5,
            ..Default::default()
        };
    }
}

impl Default for Material {
    fn default() -> Self {
        return Self::new(Color::WHITE, None, 0.1, 0.9, 0.9, 200, 0, 0, 1, true);
    }
}

impl Display for Material {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
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
        return std::ptr::eq(self, rhs)
            || self.color == rhs.color
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
    use super::*;
    use crate::primitives::Light;
    use crate::shapes::Sphere;
    use core::default::Default;

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
            .lighting(&shape, &light, &position, &camera, &normal, false);
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
            .lighting(&shape, &light, &position, &camera, &normal, false);
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
            .lighting(&shape, &light, &position, &camera, &normal, false);
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
            .lighting(&shape, &light, &position, &camera, &normal, false);
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
            .lighting(&shape, &light, &position, &camera, &normal, false);
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
            .lighting(&shape, &light, &position, &camera, &normal, true);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }
}
