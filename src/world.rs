use crate::{Color, ComputedHit, Intersections, Light, Ray, Shape, Sphere, Transformations, Tuple};
use crate::tuple::TupleTrait;

#[derive(Clone, Debug, PartialEq)]
pub struct World {
    pub light: Light,
    pub objects: Vec<Box<dyn Shape>>,
}

impl World {
    pub fn new(light: Light, objects: Vec<Box<dyn Shape>>) -> World {
        return World { light, objects };
    }

    pub fn intersections(&self, ray: &Ray) -> Intersections {
        let mut result = Intersections::new();
        for object in &self.objects {
            result.add_all(ray.intersect(object));
        }
        result.intersections.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        return result;
    }

    // TODO: add handling for multiple light sources
    pub fn shade_hit(&self, computed_hit: &ComputedHit, remaining_iterations: u8) -> Color {
        let in_shadow = self.is_shadowed(&computed_hit.over_point);
        let surface_color = computed_hit.object.material().lighting(
            &*computed_hit.object,
            &self.light,
            &computed_hit.point,
            &computed_hit.camera_vector,
            &computed_hit.normal_vector,
            in_shadow,
        );
        let reflected_color = self.reflected_color(computed_hit, remaining_iterations);
        let refracted_color = self.refracted_color(computed_hit, remaining_iterations);

        let material = computed_hit.object.material();
        if material.reflectiveness > 0.0 && material.transparency > 0.0 {
            let reflectance = computed_hit.schlick();
            return surface_color + reflected_color * reflectance + refracted_color * (1.0 - reflectance);
        } else {
            return surface_color + reflected_color + refracted_color;
        }
    }

    fn internal_color_at(&self, ray: &Ray, remaining_iterations: u8) -> Color {
        let intersections = self.intersections(ray);
        let maybe_hit = intersections.hit();

        if let Some(hit) = maybe_hit {
            let computed_hit = hit.prepare_computations(ray, &Intersections::new());
            return self.shade_hit(&computed_hit, remaining_iterations);
        } else {
            return Color::black();
        }
    }

    pub fn color_at(&self, ray: &Ray) -> Color {
        return self.internal_color_at(ray, crate::MAX_REFLECTION_ITERATIONS);
    }

    pub fn is_shadowed(&self, point: &Tuple) -> bool {
        let point_to_light_vector = self.light.position - *point;
        let distance_to_light = point_to_light_vector.magnitude();
        let shadow_ray = Ray::new(*point, point_to_light_vector.normalize());
        let intersections = self.intersections(&shadow_ray);
        let maybe_hit = intersections.hit();
        return maybe_hit.is_some() && maybe_hit.unwrap().t < distance_to_light;
    }

    fn reflected_color(&self, computed_hit: &ComputedHit, remaining_iterations: u8) -> Color {
        if remaining_iterations == 0 || computed_hit.object.material().reflectiveness == 0.0 {
            return Color::black();
        }

        let reflected_ray = Ray::new(computed_hit.over_point, computed_hit.reflection_vector);
        let reflected_color = self.internal_color_at(&reflected_ray, remaining_iterations - 1);
        return reflected_color * computed_hit.object.material().reflectiveness;
    }

    fn refracted_color(&self, computed_hit: &ComputedHit, remaining_iterations: u8) -> Color {
        if remaining_iterations == 0 || computed_hit.object.material().transparency == 0.0 {
            return Color::black();
        }

        let n_ratio = computed_hit.n1 / computed_hit.n2;
        let cos_i = computed_hit.camera_vector.dot(&computed_hit.normal_vector);
        let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);
        let is_total_internal_reflection = sin2_t > 1.0;

        if is_total_internal_reflection {
            return Color::black();
        }

        let cos_t = (1.0 - sin2_t).sqrt();
        let direction = computed_hit.normal_vector * (n_ratio * cos_i - cos_t) - computed_hit.camera_vector * n_ratio;
        let refracted_ray = Ray::new(computed_hit.under_point, direction);

        return self.internal_color_at(&refracted_ray, remaining_iterations - 1) * computed_hit.object.material().transparency;
    }
}

impl Default for World {
    fn default() -> World {
        let light = Light::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut objects: Vec<Box<dyn Shape>> = Vec::new();
        let mut sphere1 = Sphere::default();
        sphere1.material.color = Color::new(0.8, 1.0, 0.6);
        sphere1.material.diffuse = 0.7;
        sphere1.material.specular = 0.2;
        objects.push(sphere1.box_clone());
        let mut sphere2 = Sphere::default();
        sphere2.transformation = Transformations::scaling(0.5, 0.5, 0.5);
        objects.push(sphere2.box_clone());
        return World::new(light, objects);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::IndexMut;
    use crate::intersection::Intersection;
    use crate::pattern::{TestPattern};
    use crate::{Material, Plane, Transformations};

    #[test]
    fn default_world() {
        let world = World::default();
        let light = Light::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut sphere1 = Sphere::default();
        sphere1.material.color = Color::new(0.8, 1.0, 0.6);
        sphere1.material.diffuse = 0.7;
        sphere1.material.specular = 0.2;
        let mut sphere2 = Sphere::default();
        sphere2.transformation = Transformations::scaling(0.5, 0.5, 0.5);
        assert_eq!(world.light, light);
        assert_eq!(world.objects.len(), 2);
        assert!(world.objects.contains(&sphere1.box_clone()));
        assert!(world.objects.contains(&sphere2.box_clone()));
    }

    #[test]
    fn intersect_world_with_ray() {
        let world = World::default();
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let intersections = world.intersections(&ray);
        assert_eq!(intersections.len(), 4);
        assert_eq!(intersections[0].t, 4.0);
        assert_eq!(intersections[1].t, 4.5);
        assert_eq!(intersections[2].t, 5.5);
        assert_eq!(intersections[3].t, 6.0);
    }

    #[test]
    fn shading_intersection() {
        let world = World::default();
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = world.objects[0].clone();
        let intersection = Intersection::new(4.0, shape);
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 1);
        assert_eq!(color, Color::new(0.38066119308103435, 0.47582649135129296, 0.28549589481077575));
    }

    #[test]
    fn shading_intersection_from_inside() {
        let world = World {
            light: Light::new(Tuple::point(0.0, 0.25, 0.0),
                              Color::new(1.0, 1.0, 1.0)),
            ..Default::default()
        };
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = world.objects[1].clone();
        let intersection = Intersection::new(0.5, shape);
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 1);
        assert_eq!(color, Color::new(0.9049844720832575, 0.9049844720832575, 0.9049844720832575));
    }

    #[test]
    fn color_when_ray_misses() {
        let world = World::default();
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 1.0, 0.0));
        let color = world.color_at(&ray);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn color_when_ray_hits() {
        let world = World::default();
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let color = world.color_at(&ray);
        assert_eq!(color, Color::new(0.38066119308103435, 0.47582649135129296, 0.28549589481077575));
    }

    #[test]
    fn color_with_intersection_behind_ray() {
        let mut world = World::default();
        let mut material1 = world.objects[0].material();
        material1.ambient = 1.0;
        world.objects[0].set_material(material1);
        let mut material1 = world.objects[1].material();
        material1.ambient = 1.0;
        world.objects[1].set_material(material1);
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.75), Tuple::vector(0.0, 0.0, -1.0));
        let color = world.color_at(&ray);
        assert_eq!(color, world.objects[1].material().color);
    }

    #[test]
    fn no_shadow_when_nothing_obscures_light() {
        let world = World::default();
        let point = Tuple::point(0.0, 10.0, 0.0);
        assert!(!world.is_shadowed(&point));
    }

    #[test]
    fn shadow_when_point_between_point_and_light() {
        let world = World::default();
        let point = Tuple::point(10.0, -10.0, 10.0);
        assert!(world.is_shadowed(&point));
    }

    #[test]
    fn no_shadow_when_light_is_behind_point() {
        let world = World::default();
        let point = Tuple::point(-20.0, 20.0, -20.0);
        assert!(!world.is_shadowed(&point));
    }

    #[test]
    fn no_shadow_when_object_is_behind_point() {
        let world = World::default();
        let point = Tuple::point(-2.0, 2.0, -2.0);
        assert!(!world.is_shadowed(&point));
    }

    #[test]
    fn shade_hit_is_given_intersection_in_shadow() {
        let mut world = World::default();
        world.light = Light::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        world.objects.push(Box::new(Sphere::default()));
        let sphere = Sphere { transformation: Transformations::translation(0.0, 0.0, 10.0), ..Default::default() };
        world.objects.push(sphere.box_clone());
        let ray = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let intersection = Intersection::new(4.0, sphere.box_clone());
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 1);
        assert_eq!(color, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn reflected_color_for_nonreflective_material() {
        let mut world = World::default();
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut material = world.objects[1].material();
        material.ambient = 1.0;
        world.objects[1].set_material(material);
        let intersection = Intersection::new(1.0, world.objects[1].box_clone());
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.reflected_color(&computed_hit, 0);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn reflected_color_for_reflective_material() {
        let mut world = World::default();
        let mut shape = Plane::default();
        let mut material = shape.material();
        material.reflectiveness = 0.5;
        shape.set_material(material);
        shape.set_transformation(Transformations::translation(0.0, -1.0, 0.0));
        world.objects.push(Box::new(shape.clone()));
        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let intersection = Intersection::new(2.0_f64.sqrt(), shape.box_clone());
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.reflected_color(&computed_hit, 1);
        assert_eq!(color, Color::new(0.19033061377890123, 0.23791326722362655, 0.14274796033417592));
    }

    #[test]
    fn shade_hit_with_reflective_material() {
        let mut world = World::default();
        let mut shape = Plane::default();
        let mut material = shape.material();
        material.reflectiveness = 0.5;
        shape.set_material(material);
        shape.set_transformation(Transformations::translation(0.0, -1.0, 0.0));
        world.objects.push(Box::new(shape.clone()));
        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let intersection = Intersection::new(2.0_f64.sqrt(), shape.box_clone());
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 1);
        assert_eq!(color, Color::new(0.8767560027604027, 0.9243386562051279, 0.8291733493156773));
    }

    #[test]
    fn avoid_infinite_recursion_in_reflections() {
        let mut world = World::default();
        world.objects = Vec::new();
        world.light = Light::new(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0));
        let mut lower = Plane::default();
        lower.material.reflectiveness = 1.0;
        lower.transformation = Transformations::translation(0.0, -1.0, 0.0);
        world.objects.push(Box::new(lower));
        let mut upper = Plane::default();
        upper.material.reflectiveness = 1.0;
        upper.transformation = Transformations::translation(0.0, 1.0, 0.0);
        world.objects.push(Box::new(upper));
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 1.0, 0.0));
        world.color_at(&ray);
    }

    #[test]
    fn reflected_color_at_maximum_recursion_depth() {
        let mut world = World::default();
        let mut shape = Plane::default();
        let mut material = shape.material();
        material.reflectiveness = 0.5;
        shape.set_material(material);
        shape.set_transformation(Transformations::translation(0.0, -1.0, 0.0));
        world.objects.push(Box::new(shape.clone()));
        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let intersection = Intersection::new(2.0_f64.sqrt(), shape.box_clone());
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 0);
        assert_eq!(color, Color::new( 0.6864253889815014, 0.6864253889815014, 0.6864253889815014));
    }

    #[test]
    fn refracted_color_with_opaque_material() {
        let mut world = World::default();
        let shape = world.objects.index_mut(0);
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(4.0, shape.box_clone()));
        intersections.add(Intersection::new(6.0, shape.box_clone()));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, 5);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn refracted_color_at_maximum_recursion_depth() {
        let mut world = World::default();
        let shape = world.objects.index_mut(0);
        let mut material = shape.material();
        material.transparency = 1.0;
        material.refractive_index = 1.5;
        shape.set_material(material);
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(4.0, shape.box_clone()));
        intersections.add(Intersection::new(6.0, shape.box_clone()));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, 0);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn refracted_color_under_total_internal_reflection() {
        let mut world = World::default();
        let shape = world.objects.index_mut(0);
        let mut material = shape.material();
        material.transparency = 1.0;
        material.refractive_index = 1.5;
        shape.set_material(material);
        let ray = Ray::new(Tuple::point(0.0, 0.0, 2.0_f64.sqrt() / 2.0), Tuple::vector(0.0, 1.0, 0.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(-(2.0_f64.sqrt()) / 2.0, shape.box_clone()));
        intersections.add(Intersection::new(2.0_f64.sqrt() / 2.0, shape.box_clone()));
        let computed_hit = intersections[1].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, 5);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn refracted_color_with_refracted_ray() {
        let mut world = World::default();
        let shape = world.objects.index_mut(0);
        let mut material = shape.material();
        material.ambient = 1.0;
        material.pattern = Some(Box::new(TestPattern::new()));
        shape.set_material(material);
        let shape2 = world.objects.index_mut(1);
        let mut material2 = shape2.material();
        material2.transparency = 1.0;
        material2.refractive_index = 1.5;
        shape2.set_material(material2);
        let ray = Ray::new(Tuple::point(0.0, 0.0, 0.1), Tuple::vector(0.0, 1.0, 0.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(-0.9899, world.objects[0].clone()));
        intersections.add(Intersection::new(-0.4899, world.objects[1].clone()));
        intersections.add(Intersection::new(0.4899, world.objects[1].clone()));
        intersections.add(Intersection::new(0.9899, world.objects[0].clone()));
        let computed_hit = intersections[2].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, 5);
        assert_eq!(color, Color::new(0.0, 0.9988846813665367, 0.04721645191320928));
    }

    #[test]
    fn shade_hit_with_transparent_material() {
        let mut world = World::default();
        let floor_transformation = Transformations::translation(0.0, -1.0, 0.0);
        let mut floor_material = Material::default();
        floor_material.transparency = 0.5;
        floor_material.refractive_index = 1.5;
        let floor = Plane::new(floor_material, floor_transformation);
        world.objects.push(Box::new(floor.clone()));
        let mut ball_material = Material::default();
        ball_material.color = Color::new(1.0, 0.0, 0.0);
        ball_material.ambient = 0.5;
        let mut ball = Sphere::default();
        ball.set_material(ball_material);
        ball.set_transformation(Transformations::translation(0.0, -3.5, -0.5));
        world.objects.push(Box::new(ball));
        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(2.0_f64.sqrt(), floor));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.shade_hit(&computed_hit, 5);
        assert_eq!(color, Color::new(0.9364253889815014, 0.6864253889815014, 0.6864253889815014));
    }

    #[test]
    fn shade_hit_with_reflective_and_transparent_material() {
        let mut world = World::default();
        let floor_transformation = Transformations::translation(0.0, -1.0, 0.0);
        let mut floor_material = Material::default();
        floor_material.reflectiveness = 0.5;
        floor_material.transparency = 0.5;
        floor_material.refractive_index = 1.5;
        let floor = Plane::new(floor_material, floor_transformation);
        world.objects.push(Box::new(floor.clone()));
        let mut ball_material = Material::default();
        ball_material.color = Color::new(1.0, 0.0, 0.0);
        ball_material.ambient = 0.5;
        let mut ball = Sphere::default();
        ball.set_material(ball_material);
        ball.set_transformation(Transformations::translation(0.0, -3.5, -0.5));
        world.objects.push(Box::new(ball));
        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(2.0_f64.sqrt(), floor));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.shade_hit(&computed_hit, 5);
        assert_eq!(color, Color::new(0.9339151412754023, 0.696434227200244, 0.692430691912747));
    }
}
