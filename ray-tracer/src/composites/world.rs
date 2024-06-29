use crate::composites::{ComputedHit, Intersections, Ray};
use crate::primitives::{Color, Light, Point};
use crate::shapes::Shape;
use crate::utils::{world_default_sphere_1, world_default_sphere_2, Squared};
use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct World {
    pub lights: Vec<Light>,
    pub shapes: Vec<Box<dyn Shape>>,
}

impl World {
    pub const MAX_REFLECTION_ITERATIONS: u8 = 6;
    pub const DEFAULT_COLOR: Color = Color::BLACK;

    pub const fn new(lights: Vec<Light>, shapes: Vec<Box<dyn Shape>>) -> Self {
        return Self { lights, shapes };
    }

    /// Clears intersection buffer and fills it with result of intersecting [Ray] with shapes in [World]
    pub fn collect_intersections<'shapes>(&'shapes self, ray: &Ray, intersections: &mut Intersections<'shapes>) {
        intersections.clear();
        for shape in &self.shapes {
            ray.intersect(shape.as_ref(), intersections);
        }
        intersections.sort_by_distance();
    }

    fn shade_hit<'shapes>(&'shapes self, computed_hit: &ComputedHit, intersections: &mut Intersections<'shapes>, remaining_iterations: u8) -> Color {
        let material = computed_hit.shape.material();
        let surface_color = self
            .lights
            .iter()
            .map(|light| {
                let in_shadow = self.is_in_shadow(light, &computed_hit.over_point, intersections);
                return material.lighting_from_computed_hit(computed_hit, light, in_shadow);
            })
            .fold(Color::new(0, 0, 0), |acc, color| acc + color);

        let reflected_color = self.reflected_color(computed_hit, intersections, remaining_iterations);
        let refracted_color = self.refracted_color(computed_hit, intersections, remaining_iterations);

        if material.reflectiveness > 0.0 && material.transparency > 0.0 {
            let reflectance = computed_hit.schlick();
            return surface_color
                + reflected_color * reflectance
                + refracted_color * (1.0 - reflectance);
        } else {
            return surface_color + reflected_color + refracted_color;
        }
    }

    fn internal_color_at<'shapes>(&'shapes self, ray: &Ray, intersections: &mut Intersections<'shapes>, remaining_iterations: u8) -> Color {
        self.collect_intersections(ray, intersections);
        let maybe_hit = intersections.hit();

        if let Some(hit) = maybe_hit {
            let computed_hit = hit.prepare_computations(ray, intersections);
            let mut shade_hit_intersections = Intersections::new();
            return self.shade_hit(&computed_hit, &mut shade_hit_intersections, remaining_iterations);
        } else {
            return Self::DEFAULT_COLOR;
        }
    }

    pub fn color_at<'shapes>(&'shapes self, ray: &Ray, intersections: &mut Intersections<'shapes>) -> Color {
        return self.internal_color_at(ray, intersections, Self::MAX_REFLECTION_ITERATIONS);
    }

    /// Returns whether between the [Light] and [Point] is shape casting shadow
    fn is_in_shadow<'shapes>(&'shapes self, light: &Light, point: &Point, intersections: &mut Intersections<'shapes>) -> bool {
        let point_to_light_vector = light.position - *point;
        let distance_to_light = point_to_light_vector.magnitude();
        let shadow_ray = Ray::new(*point, point_to_light_vector.normalized());
        self.collect_intersections(&shadow_ray, intersections);
        return intersections.into_iter()
            .any(|intersection| {
                intersection.shape.material().casts_shadow &&
                intersection.is_within_distance(distance_to_light)
            });
    }

    fn reflected_color<'shapes>(&'shapes self, computed_hit: &ComputedHit, intersections: &mut Intersections<'shapes>, remaining_iterations: u8) -> Color {
        if remaining_iterations == 0 || computed_hit.shape.material().reflectiveness == 0.0 {
            return Self::DEFAULT_COLOR;
        }

        let reflected_ray = Ray::new(computed_hit.over_point, computed_hit.reflection_vector);
        let reflected_color = self.internal_color_at(&reflected_ray, intersections, remaining_iterations - 1);
        return reflected_color * computed_hit.shape.material().reflectiveness;
    }

    fn refracted_color<'shapes>(&'shapes self, computed_hit: &ComputedHit, intersections: &mut Intersections<'shapes>, remaining_iterations: u8) -> Color {
        if remaining_iterations == 0 || computed_hit.shape.material().transparency == 0.0 {
            return Self::DEFAULT_COLOR;
        }

        let n_ratio = computed_hit.refractive_index_1 / computed_hit.refractive_index_2;
        let cos_i = computed_hit.camera_vector.dot(&computed_hit.normal);
        let sin2_t = n_ratio.squared() * (1.0 - cos_i.squared());
        let is_total_internal_reflection = sin2_t > 1.0;

        if is_total_internal_reflection {
            return Self::DEFAULT_COLOR;
        }

        let cos_t = (1.0 - sin2_t).sqrt();
        let direction = computed_hit.normal * n_ratio.mul_add(cos_i, -cos_t)
            - (computed_hit.camera_vector * n_ratio);
        let refracted_ray = Ray::new(computed_hit.under_point, direction);
        let refracted_color = self.internal_color_at(&refracted_ray, intersections, remaining_iterations - 1);

        return refracted_color * computed_hit.shape.material().transparency;
    }
}

impl Default for World {
    fn default() -> Self {
        let light = Light::default();
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(world_default_sphere_1()),
            Box::new(world_default_sphere_2()),
        ];
        return Self::new(vec![light], shapes);
    }
}

impl PartialEq for World {
    fn eq(&self, rhs: &Self) -> bool {
        return self.lights.len() == rhs.lights.len()
            && self.shapes.len() == rhs.shapes.len()
            && self.lights.iter().all(|light| rhs.lights.contains(light))
            && self.shapes.iter().all(|shape| {
                return rhs.shapes.iter().any(|entry| entry == shape);
            });
    }
}

impl Display for World {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("World")
            .field("light", &self.lights)
            .field("shapes", &self.shapes)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composites::{Intersection, Intersections, Material, Ray};
    use crate::consts::PI;
    use crate::patterns::TestPattern;
    use crate::primitives::transformations;
    use crate::primitives::Vector;
    use crate::shapes::{Plane, Sphere, Transform};
    use std::sync::Arc;

    #[test]
    fn default_world() {
        let world = World::default();
        let light = Light::new(Point::new(-10, 10, -10), Color::new(1, 1, 1));
        let mut sphere_1 = Sphere::default();
        sphere_1.material.color = Color::new(0.8, 1, 0.6);
        sphere_1.material.diffuse = 0.7;
        sphere_1.material.specular = 0.2;
        let mut sphere_2 = Sphere::default();
        sphere_2.set_transformation(transformations::scaling(0.5, 0.5, 0.5));
        assert_eq!(world.lights[0], light);
        assert_eq!(world.shapes.len(), 2);
        assert!(world
            .shapes
            .iter()
            .any(|element| element.as_ref() == &sphere_1));
        assert!(world
            .shapes
            .iter()
            .any(|element| element.as_ref() == &sphere_2));
    }

    #[test]
    fn compare_worlds() {
        let mut world_1 = World::default();
        let mut world_2 = World::default();
        assert_eq!(world_1, world_2);
        let sphere_1 = Sphere::default();
        let mut sphere_2 = Sphere::default();
        sphere_2.set_transformation(transformations::rotation_z(PI));
        world_2.shapes.push(Box::new(sphere_2.clone()));
        assert_ne!(world_1, world_2);
        world_1.shapes.push(Box::new(sphere_1.clone()));
        assert_ne!(world_1, world_2);
        world_2.shapes.push(Box::new(sphere_1));
        world_1.shapes.push(Box::new(sphere_2));
        assert_eq!(world_1, world_2);
    }

    #[test]
    fn intersect_world_with_ray() {
        let world = World::default();
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let mut intersections = Intersections::new();
        world.collect_intersections(&ray, &mut intersections);
        assert_eq!(intersections.len(), 4);
        assert_eq!(intersections[0].distance, 4.0);
        assert_eq!(intersections[1].distance, 4.5);
        assert_eq!(intersections[2].distance, 5.5);
        assert_eq!(intersections[3].distance, 6.0);
    }

    #[test]
    fn shading_intersection() {
        let world = World::default();
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let shape = &world.shapes[0];
        let intersection = Intersection::new(4.0, shape.as_ref());
        let intersections = Intersections::new();
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        let mut intersections = Intersections::new();
        let color = world.shade_hit(&computed_hit, &mut intersections, 1);
        assert_eq!(
            color,
            Color::new(
                0.38066119308103435,
                0.47582649135129296,
                0.28549589481077575
            )
        );
    }

    #[test]
    fn shading_intersection_from_inside() {
        let mut world = World::default();
        world.lights = vec![Light::new(Point::new(0, 0.25, 0), Color::WHITE)];
        let ray = Ray::new(Point::ORIGIN, Vector::FORWARD);
        let intersection = Intersection::new(0.5, world.shapes[1].as_ref());
        let intersections = Intersections::new();
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        let mut intersections = Intersections::new();
        let color = world.shade_hit(&computed_hit, &mut intersections, 1);
        assert_eq!(
            color,
            Color::new(0.9049844720832575, 0.9049844720832575, 0.9049844720832575)
        );
    }

    #[test]
    fn color_when_ray_misses() {
        let world = World::default();
        let ray = Ray::new(Point::new(0, 0, -5), Vector::UP);
        let color = world.color_at(&ray, &mut Intersections::new());
        assert_eq!(color, World::DEFAULT_COLOR);
    }

    #[test]
    fn color_when_ray_hits() {
        let world = World::default();
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let color = world.color_at(&ray, &mut Intersections::new());
        assert_eq!(
            color,
            Color::new(
                0.38066119308103435,
                0.47582649135129296,
                0.28549589481077575
            )
        );
    }

    #[test]
    fn color_with_intersection_behind_ray() {
        let mut world = World::default();

        let mut sphere1 = world_default_sphere_1();
        let mut material1 = sphere1.material().clone();
        material1.ambient = 1.0;
        sphere1.material = material1;
        world.shapes[0] = Box::new(sphere1);

        let mut sphere2 = world_default_sphere_2();
        let mut material2 = sphere2.material().clone();
        material2.ambient = 1.0;
        sphere2.material = material2;
        world.shapes[1] = Box::new(sphere2);

        let ray = Ray::new(Point::new(0, 0, 0.75), Vector::BACKWARD);
        let color = world.color_at(&ray, &mut Intersections::new());
        assert_eq!(color, world.shapes[1].material().color);
    }

    #[test]
    fn no_shadow_when_nothing_obscures_light() {
        let world = World::default();
        let point = Point::new(0, 10, 0);
        let mut intersections = Intersections::new();
        assert!(!world.is_in_shadow(&world.lights[0], &point, &mut intersections));
    }

    #[test]
    fn no_shadow_when_light_is_behind_point() {
        let world = World::default();
        let point = Point::new(-20, 20, -20);
        let mut intersections = Intersections::new();
        assert!(!world.is_in_shadow(&world.lights[0], &point, &mut intersections));
    }

    #[test]
    fn no_shadow_when_object_is_behind_point() {
        let world = World::default();
        let point = Point::new(-2, 2, -2);
        let mut intersections = Intersections::new();
        assert!(!world.is_in_shadow(&world.lights[0], &point, &mut intersections));
    }

    #[test]
    fn shadow_when_object_is_between_hit_and_light() {
        let world = World::default();
        let point = Point::new(10, -10, 10);
        let mut intersections = Intersections::new();
        assert!(world.is_in_shadow(&world.lights[0], &point, &mut intersections));
    }

    #[test]
    fn shade_hit_is_given_intersection_in_shadow() {
        let mut world = World::default();
        world.lights = vec![Light::new(Point::new(0, 0, -10), Color::WHITE)];
        world.shapes.push(Box::<Sphere>::default());
        let mut sphere = Sphere::default();
        sphere.set_transformation(transformations::translation(0, 0, 10));
        world.shapes.push(Box::new(sphere.clone()));
        let ray = Ray::new(Point::new(0, 0, 5), Vector::FORWARD);
        let boxed_sphere = Box::new(sphere);
        let intersection = Intersection::new(4, boxed_sphere.as_ref());
        let intersections = Intersections::new();
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        let color = world.shade_hit(&computed_hit, &mut Intersections::new(), 1);
        assert_eq!(color, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn reflected_color_for_nonreflective_material() {
        let mut world = World::default();
        let ray = Ray::new(Point::ORIGIN, Vector::FORWARD);
        let mut sphere1 = world_default_sphere_2();
        let mut material = sphere1.material().clone();
        material.ambient = 1.0;
        sphere1.material = material;
        world.shapes[0] = Box::new(sphere1);
        let intersection = Intersection::new(1.0, world.shapes[1].as_ref());
        let intersections = Intersections::new();
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        let color = world.reflected_color(&computed_hit, &mut Intersections::new(), 1);
        assert_eq!(color, World::DEFAULT_COLOR);
    }

    #[test]
    fn reflected_color_for_reflective_material() {
        let mut world = World::default();
        let mut shape = Plane::default();
        let mut material = shape.material().clone();
        material.reflectiveness = 0.5;
        shape.material = material;
        shape.set_transformation(transformations::translation(0, -1, 0));
        world.shapes.push(Box::new(shape.clone()));
        let ray = Ray::new(
            Point::new(0, 0, -3),
            Vector::new(0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let boxed_shape = Box::new(shape);
        let intersection = Intersection::new(2.0_f64.sqrt(), boxed_shape.as_ref());
        let intersections = Intersections::new();
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        let color = world.reflected_color(&computed_hit, &mut Intersections::new(), 1);
        assert_eq!(
            color,
            Color::new(
                0.19033061377890123,
                0.23791326722362655,
                0.14274796033417592
            )
        );
    }

    #[test]
    fn shade_hit_with_reflective_material() {
        let mut world = World::default();
        let mut shape = Plane::default();
        let mut material = shape.material().clone();
        material.reflectiveness = 0.5;
        shape.material = material;
        shape.set_transformation(transformations::translation(0, -1, 0));
        world.shapes.push(Box::new(shape.clone()));
        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector::new(0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let boxed_shape = Box::new(shape);
        let intersection = Intersection::new(2.0_f64.sqrt(), boxed_shape.as_ref());
        let intersections = Intersections::new();
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        let color = world.shade_hit(&computed_hit, &mut Intersections::new(), 1);
        assert_eq!(
            color,
            Color::new(0.8767560027604027, 0.9243386562051279, 0.8291733493156773)
        );
    }

    #[test]
    fn no_infinite_recursion_in_reflections() {
        let mut world = World::default();
        world.shapes = Vec::new();
        world.lights = vec![Light::new(Point::ORIGIN, Color::new(1, 1, 1))];
        let mut lower = Plane::default();
        lower.material.reflectiveness = 1.0;
        lower.set_transformation(transformations::translation(0, -1, 0));
        let arc_lower = Box::new(lower);
        world.shapes.push(arc_lower);
        let mut upper = Plane::default();
        upper.material.reflectiveness = 1.0;
        upper.set_transformation(transformations::translation(0, 1, 0));
        let arc_upper = Box::new(upper);
        world.shapes.push(arc_upper);
        let ray = Ray::new(Point::ORIGIN, Vector::UP);
        world.color_at(&ray, &mut Intersections::new());
    }

    #[test]
    fn reflected_color_at_max_recursion_depth() {
        let mut world = World::default();
        let mut shape = Plane::default();
        let mut material = shape.material().clone();
        material.reflectiveness = 0.5;
        shape.material = material;
        shape.set_transformation(transformations::translation(0, -1, 0));
        world.shapes.push(Box::new(shape.clone()));
        let ray = Ray::new(
            Point::new(0, 0, -3),
            Vector::new(0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let boxed_shape = Box::new(shape);
        let intersection = Intersection::new(2.0_f64.sqrt(), boxed_shape.as_ref());
        let intersections = Intersections::new();
        let computed_hit = intersection.prepare_computations(&ray, &intersections);
        let color = world.reflected_color(&computed_hit, &mut Intersections::new(), 0);
        assert_eq!(color, World::DEFAULT_COLOR);
    }

    #[test]
    fn refracted_color_with_opaque_material() {
        let world = World::default();
        let shape = world.shapes[0].as_ref();
        let mut intersections = Intersections::new();
        intersections.push(Intersection::new(4, shape));
        intersections.push(Intersection::new(6, shape));
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, &mut Intersections::new(), 5);
        assert_eq!(color, World::DEFAULT_COLOR);
    }

    #[test]
    fn refracted_color_at_max_recursion_depth() {
        let mut world = World::default();
        let mut sphere1 = world_default_sphere_1();
        let mut material = sphere1.material().clone();
        material.transparency = 1.0;
        material.refractive_index = 1.5;
        sphere1.material = material;
        world.shapes[0] = Box::new(sphere1.clone());
        let ray = Ray::new(Point::new(0, 0, -5), Vector::FORWARD);
        let mut intersections = Intersections::new();
        let shape = Box::new(sphere1);
        intersections.push(Intersection::new(4, shape.as_ref()));
        intersections.push(Intersection::new(6, shape.as_ref()));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, &mut Intersections::new(), 0);
        assert_eq!(color, World::DEFAULT_COLOR);
    }

    #[test]
    fn refracted_color_under_total_internal_reflection() {
        let mut world = World::default();
        let mut sphere1 = world_default_sphere_1();
        let mut material = sphere1.material().clone();
        material.transparency = 1.0;
        material.refractive_index = 1.5;
        sphere1.material = material;
        world.shapes[0] = Box::new(sphere1.clone());
        let ray = Ray::new(Point::new(0, 0, 2.0_f64.sqrt() / 2.0), Vector::UP);
        let mut intersections = Intersections::new();
        let boxed_shape = Box::new(sphere1);
        intersections.push(Intersection::new(
            -(2.0_f64.sqrt()) / 2.0,
            boxed_shape.as_ref(),
        ));
        intersections.push(Intersection::new(
            2.0_f64.sqrt() / 2.0,
            boxed_shape.as_ref(),
        ));
        let computed_hit = intersections[1].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, &mut Intersections::new(),5);
        assert_eq!(color, World::DEFAULT_COLOR);
    }

    #[test]
    fn refracted_color_with_refracted_ray() {
        let mut world = World::default();
        let mut sphere1 = world_default_sphere_1();
        let mut material1 = sphere1.material().clone();
        material1.ambient = 1.0;
        material1.pattern = Some(Arc::new(TestPattern::new()));
        sphere1.material = material1;
        world.shapes[0] = Box::new(sphere1);
        let mut sphere2 = world_default_sphere_2();
        let mut material2 = sphere2.material().clone();
        material2.transparency = 1.0;
        material2.refractive_index = 1.5;
        sphere2.material = material2;
        world.shapes[1] = Box::new(sphere2);
        let ray = Ray::new(Point::new(0, 0, 0.1), Vector::UP);
        let mut intersections = Intersections::new();
        intersections.push(Intersection::new(-0.9899, world.shapes[0].as_ref()));
        intersections.push(Intersection::new(-0.4899, world.shapes[1].as_ref()));
        intersections.push(Intersection::new(0.4899, world.shapes[1].as_ref()));
        intersections.push(Intersection::new(0.9899, world.shapes[0].as_ref()));
        let computed_hit = intersections[2].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, &mut Intersections::new(), 5);
        assert_eq!(
            color,
            Color::new(0, 0.9988846813665367, 0.04721645191320928)
        );
    }

    #[test]
    fn shade_hit_with_transparent_material() {
        let mut world = World::default();
        let mut floor_material = Material::default();
        floor_material.transparency = 0.5;
        floor_material.refractive_index = 1.5;
        let floor = Plane::new(floor_material, transformations::translation(0, -1, 0));
        world.shapes.push(Box::new(floor.clone()));
        let mut ball_material = Material::default();
        ball_material.color = Color::RED;
        ball_material.ambient = 0.5;
        let mut ball = Sphere::default();
        ball.material = ball_material;
        ball.set_transformation(transformations::translation(0, -3.5, -0.5));
        world.shapes.push(Box::new(ball));
        let ray = Ray::new(
            Point::new(0, 0, -3),
            Vector::new(0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let mut intersections = Intersections::new();
        let arc_floor = Box::new(floor);
        intersections.push(Intersection::new(2.0_f64.sqrt(), arc_floor.as_ref()));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.shade_hit(&computed_hit, &mut Intersections::new(), 5);
        assert_eq!(
            color,
            Color::new(0.9364253889815014, 0.6864253889815014, 0.6864253889815014)
        );
    }

    #[test]
    fn shade_hit_with_reflective_and_transparent_material() {
        let mut world = World::default();
        let floor_transformation = transformations::translation(0, -1, 0);
        let mut floor_material = Material::default();
        floor_material.reflectiveness = 0.5;
        floor_material.transparency = 0.5;
        floor_material.refractive_index = 1.5;
        let floor = Plane::new(floor_material, floor_transformation);
        world.shapes.push(Box::new(floor.clone()));
        let mut ball_material = Material::default();
        ball_material.color = Color::RED;
        ball_material.ambient = 0.5;
        let mut ball = Sphere::default();
        ball.material = ball_material;
        ball.set_transformation(transformations::translation(0, -3.5, -0.5));
        world.shapes.push(Box::new(ball));
        let ray = Ray::new(
            Point::new(0, 0, -3),
            Vector::new(0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let mut intersections = Intersections::new();
        let boxed_floor = Box::new(floor);
        intersections.push(Intersection::new(2.0_f64.sqrt(), boxed_floor.as_ref()));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.shade_hit(&computed_hit, &mut Intersections::new(), 5);
        assert_eq!(
            color,
            Color::new(0.9339151412754023, 0.696434227200244, 0.692430691912747)
        );
    }
}
