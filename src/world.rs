use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::computed_hit::ComputedHit;
use crate::consts::{EPSILON, MAX_REFLECTION_ITERATIONS};
use crate::intersections::Intersections;
use crate::primitives::{Color, Light, Point};
use crate::ray::Ray;
use crate::shapes::Shape;
use crate::utils::{world_default_sphere_1, world_default_sphere_2};

#[derive(Clone, Debug)]
pub struct World {
    pub lights: Vec<Light>,
    pub objects: Vec<Arc<dyn Shape>>,
}

impl World {
    pub fn new(lights: Vec<Light>, objects: Vec<Arc<dyn Shape>>) -> World {
        return World { lights, objects };
    }

    pub fn intersections(&self, ray: &Ray) -> Intersections {
        let mut intersections = Intersections::new();
        for object in &self.objects {
            intersections.add_all(ray.intersect(object));
        }
        if !intersections.intersections.is_empty() {
            intersections.intersections.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        }
        return intersections;
    }

    pub fn shade_hit(&self, computed_hit: &ComputedHit, remaining_iterations: u8) -> Color {
        let material = computed_hit.object.material();
        let surface_color = self.lights.iter().map(|light| {
            let is_shadowed = &self.is_shadowed(light, &computed_hit.over_point);
            return material.lighting_from_computed_hit(computed_hit, light, is_shadowed);
        }).fold(Color::BLACK, |acc, color| acc + color);

        let reflected_color = self.reflected_color(computed_hit, remaining_iterations);
        let refracted_color = self.refracted_color(computed_hit, remaining_iterations);

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

        return match maybe_hit {
            Some(hit) => {
                let computed_hit = hit.prepare_computations(ray, &intersections);
                self.shade_hit(&computed_hit, remaining_iterations)
            }
            None => Color::BLACK
        }
    }

    pub fn color_at(&self, ray: &Ray) -> Color {
        return self.internal_color_at(ray, MAX_REFLECTION_ITERATIONS);
    }

    pub fn is_shadowed(&self, light: &Light, point: &Point) -> bool {
        let point_to_light_vector = light.position - *point;
        let distance_to_light = point_to_light_vector.magnitude();
        let shadow_ray = Ray::new(*point, point_to_light_vector.normalized());
        let intersections = self.intersections(&shadow_ray);
        let maybe_hit = intersections.hit();
        return match maybe_hit {
            Some(hit) => hit.distance < distance_to_light,
            None => false,
        }
    }

    fn reflected_color(&self, computed_hit: &ComputedHit, remaining_iterations: u8) -> Color {
        if remaining_iterations == 0 || computed_hit.object.material().reflectiveness == 0.0 {
            return Color::BLACK;
        }

        let reflected_ray = Ray::new(computed_hit.over_point, computed_hit.reflection_vector);
        let reflected_color = self.internal_color_at(&reflected_ray, remaining_iterations - 1);
        return reflected_color * computed_hit.object.material().reflectiveness;
    }

    fn refracted_color(&self, computed_hit: &ComputedHit, remaining_iterations: u8) -> Color {
        if remaining_iterations == 0 || computed_hit.object.material().transparency == 0.0 {
            return Color::BLACK;
        }

        let n_ratio = computed_hit.n1 / computed_hit.n2;
        let cos_i = computed_hit.camera_vector.dot(&computed_hit.normal_vector);
        let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);
        let is_total_internal_reflection = sin2_t + EPSILON > 1.0;

        if is_total_internal_reflection {
            return Color::BLACK;
        }

        let cos_t = (1.0 - sin2_t).sqrt();
        let direction = computed_hit.normal_vector * (n_ratio * cos_i - cos_t) - (computed_hit.camera_vector * n_ratio);
        let refracted_ray = Ray::new(computed_hit.under_point, direction);
        let refracted_color = self.internal_color_at(&refracted_ray, remaining_iterations - 1);

        return refracted_color * computed_hit.object.material().transparency;
    }
}

impl Default for World {
    fn default() -> World {
        let light = Light::default();
        let objects: Vec<Arc<dyn Shape>> =
            vec![Arc::new(world_default_sphere_1()), Arc::new(world_default_sphere_2())];
        return World::new(vec![light], objects);
    }
}

impl PartialEq for World {
    fn eq(&self, rhs: &Self) -> bool {
        return self.lights.len() == rhs.lights.len() &&
            self.lights.iter().all(|light| rhs.lights.contains(light)) &&
            self.objects.len() == rhs.objects.len() &&
            self.objects.iter().all(|object| rhs.objects.contains(object));
    }
}

impl Display for World {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter.debug_struct("World")
            .field("light", &self.lights)
            .field("objects", &self.objects)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::PI;
    use crate::intersection::Intersection;
    use crate::material::Material;
    use crate::patterns::TestPattern;
    use crate::primitives::transformations;
    use crate::primitives::Vector;
    use crate::shapes::{Plane, Sphere};

    use super::*;

    #[test]
    fn default_world() {
        let world = World::default();
        let light = Light::new(Point::new(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut sphere_1 = Sphere::default();
        sphere_1.material.color = Color::new(0.8, 1.0, 0.6);
        sphere_1.material.diffuse = 0.7;
        sphere_1.material.specular = 0.2;
        let mut sphere_2 = Sphere::default();
        sphere_2.transformation = transformations::scaling(0.5, 0.5, 0.5);
        assert_eq!(world.lights[0], light);
        assert_eq!(world.objects.len(), 2);
        assert!(world.objects.contains(&(Arc::new(sphere_1) as Arc<dyn Shape>)));
        assert!(world.objects.contains(&(Arc::new(sphere_2) as Arc<dyn Shape>)));
    }

    #[test]
    fn compare_worlds() {
        let mut world_1 = World::default();
        let mut world_2 = World::default();
        assert_eq!(world_1, world_2);
        let sphere_1 = Sphere::default();
        let mut sphere_2 = Sphere::default();
        sphere_2.set_transformation(transformations::rotation_z(PI));
        world_2.objects.push(Arc::new(sphere_2.clone()) as Arc<dyn Shape>);
        assert_ne!(world_1, world_2);
        world_1.objects.push(Arc::new(sphere_1.clone()) as Arc<dyn Shape>);
        assert_ne!(world_1, world_2);
        world_2.objects.push(Arc::new(sphere_1) as Arc<dyn Shape>);
        world_1.objects.push(Arc::new(sphere_2) as Arc<dyn Shape>);
        assert_eq!(world_1, world_2);
    }

    #[test]
    fn intersect_world_with_ray() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let intersections = world.intersections(&ray);
        assert_eq!(intersections.len(), 4);
        assert_eq!(intersections[0].distance, 4.0);
        assert_eq!(intersections[1].distance, 4.5);
        assert_eq!(intersections[2].distance, 5.5);
        assert_eq!(intersections[3].distance, 6.0);
    }

    #[test]
    fn shading_intersection() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let shape = world.objects[0].clone();
        let intersection = Intersection::new(4.0, shape);
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 1);
        assert_eq!(color, Color::new(0.38066119308103435, 0.47582649135129296, 0.28549589481077575));
    }

    #[test]
    fn shading_intersection_from_inside() {
        let world = World {
            lights: vec![Light::new(Point::new(0.0, 0.25, 0.0), Color::WHITE)],
            ..Default::default()
        };
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let shape = world.objects[1].clone();
        let intersection = Intersection::new(0.5, shape);
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 1);
        assert_eq!(color, Color::new(0.9049844720832575, 0.9049844720832575, 0.9049844720832575));
    }

    #[test]
    fn color_when_ray_misses() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 1.0, 0.0));
        let color = world.color_at(&ray);
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn color_when_ray_hits() {
        let world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let color = world.color_at(&ray);
        assert_eq!(color, Color::new(0.38066119308103435, 0.47582649135129296, 0.28549589481077575));
    }

    #[test]
    fn color_with_intersection_behind_ray() {
        let mut world = World::default();

        let mut sphere1 = world_default_sphere_1();
        let mut material1 = sphere1.material();
        material1.ambient = 1.0;
        sphere1.set_material(material1);
        world.objects[0] = Arc::new(sphere1) as Arc<dyn Shape>;

        let mut sphere2 = world_default_sphere_2();
        let mut material2 = sphere2.material();
        material2.ambient = 1.0;
        sphere2.set_material(material2);
        world.objects[1] = Arc::new(sphere2) as Arc<dyn Shape>;

        let ray = Ray::new(Point::new(0.0, 0.0, 0.75), Vector::new(0.0, 0.0, -1.0));
        let color = world.color_at(&ray);
        assert_eq!(color, world.objects[1].material().color);
    }

    #[test]
    fn no_shadow_when_nothing_obscures_light() {
        let world = World::default();
        let point = Point::new(0.0, 10.0, 0.0);
        assert!(!world.is_shadowed(&world.lights[0], &point));
    }

    #[test]
    fn no_shadow_when_light_is_behind_point() {
        let world = World::default();
        let point = Point::new(-20.0, 20.0, -20.0);
        assert!(!world.is_shadowed(&world.lights[0], &point));
    }

    #[test]
    fn no_shadow_when_object_is_behind_point() {
        let world = World::default();
        let point = Point::new(-2.0, 2.0, -2.0);
        assert!(!world.is_shadowed(&world.lights[0], &point));
    }

    #[test]
    fn shadow_when_object_between_hit_and_light() {
        let world = World::default();
        let point = Point::new(10.0, -10.0, 10.0);
        assert!(world.is_shadowed(&world.lights[0], &point));
    }

    #[test]
    fn shade_hit_is_given_intersection_in_shadow() {
        let mut world = World::default();
        world.lights = vec![Light::new(Point::new(0.0, 0.0, -10.0), Color::WHITE)];
        world.objects.push(Arc::new(Sphere::default()));
        let sphere = Sphere { transformation: transformations::translation(0.0, 0.0, 10.0), ..Default::default() };
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        world.objects.push(arc_sphere.clone());
        let ray = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0));
        let intersection = Intersection::new(4.0, arc_sphere);
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 1);
        assert_eq!(color, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn reflected_color_for_nonreflective_material() {
        let mut world = World::default();
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let mut sphere1 = world_default_sphere_2();
        let mut material = sphere1.material();
        material.ambient = 1.0;
        sphere1.set_material(material);
        world.objects[0] = Arc::new(sphere1) as Arc<dyn Shape>;
        let intersection = Intersection::new(1.0, world.objects[1].clone());
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.reflected_color(&computed_hit, 0);
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn reflected_color_for_reflective_material() {
        let mut world = World::default();
        let mut shape = Plane::default();
        let mut material = shape.material();
        material.reflectiveness = 0.5;
        shape.set_material(material);
        shape.set_transformation(transformations::translation(0.0, -1.0, 0.0));
        let arc_shape: Arc<dyn Shape> = Arc::new(shape);
        world.objects.push(arc_shape.clone());
        let ray = Ray::new(Point::new(0.0, 0.0, -3.0), Vector::new(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let intersection = Intersection::new(2.0_f64.sqrt(), arc_shape);
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
        shape.set_transformation(transformations::translation(0.0, -1.0, 0.0));
        let arc_shape: Arc<dyn Shape> = Arc::new(shape);
        world.objects.push(arc_shape.clone());
        let ray = Ray::new(Point::new(0.0, 0.0, -3.0), Vector::new(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let intersection = Intersection::new(2.0_f64.sqrt(), arc_shape);
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 1);
        assert_eq!(color, Color::new(0.8767560027604027, 0.9243386562051279, 0.8291733493156773));
    }

    #[test]
    fn avoid_infinite_recursion_in_reflections() {
        let mut world = World::default();
        world.objects = Vec::new();
        world.lights = vec![Light::new(Point::new(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0))];
        let mut lower = Plane::default();
        lower.material.reflectiveness = 1.0;
        lower.transformation = transformations::translation(0.0, -1.0, 0.0);
        let arc_lower: Arc<dyn Shape> = Arc::new(lower);
        world.objects.push(arc_lower);
        let mut upper = Plane::default();
        upper.material.reflectiveness = 1.0;
        upper.transformation = transformations::translation(0.0, 1.0, 0.0);
        let arc_upper: Arc<dyn Shape> = Arc::new(upper);
        world.objects.push(arc_upper);
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0));
        world.color_at(&ray);
    }

    #[test]
    fn reflected_color_at_maximum_recursion_depth() {
        let mut world = World::default();
        let mut shape = Plane::default();
        let mut material = shape.material();
        material.reflectiveness = 0.5;
        shape.set_material(material);
        shape.set_transformation(transformations::translation(0.0, -1.0, 0.0));
        let arc_shape: Arc<dyn Shape> = Arc::new(shape);
        world.objects.push(arc_shape.clone());
        let ray = Ray::new(Point::new(0.0, 0.0, -3.0), Vector::new(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let intersection = Intersection::new(2.0_f64.sqrt(), arc_shape);
        let computed_hit = intersection.prepare_computations(&ray, &Intersections::new());
        let color = world.shade_hit(&computed_hit, 0);
        assert_eq!(color, Color::new( 0.6864253889815014, 0.6864253889815014, 0.6864253889815014));
    }

    #[test]
    fn refracted_color_with_opaque_material() {
        let world = World::default();
        let shape = &world.objects[0];
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(4.0, shape.clone()));
        intersections.add(Intersection::new(6.0, shape.clone()));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, 5);
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn refracted_color_at_maximum_recursion_depth() {
        let mut world = World::default();
        let mut sphere1 = world_default_sphere_1();
        let mut material = sphere1.material();
        material.transparency = 1.0;
        material.refractive_index = 1.5;
        sphere1.set_material(material);
        let shape = Arc::new(sphere1) as Arc<dyn Shape>;
        world.objects[0] = shape.clone();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(4.0, shape.clone()));
        intersections.add(Intersection::new(6.0, shape));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, 0);
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn refracted_color_under_total_internal_reflection() {
        let mut world = World::default();
        let mut sphere1 = world_default_sphere_1();
        let mut material = sphere1.material();
        material.transparency = 1.0;
        material.refractive_index = 1.5;
        sphere1.set_material(material);
        let shape = Arc::new(sphere1) as Arc<dyn Shape>;
        world.objects[0] = shape.clone();
        let ray = Ray::new(Point::new(0.0, 0.0, 2.0_f64.sqrt() / 2.0), Vector::new(0.0, 1.0, 0.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(-(2.0_f64.sqrt()) / 2.0, shape.clone()));
        intersections.add(Intersection::new(2.0_f64.sqrt() / 2.0, shape.clone()));
        let computed_hit = intersections[1].prepare_computations(&ray, &intersections);
        let color = world.refracted_color(&computed_hit, 5);
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn refracted_color_with_refracted_ray() {
        let mut world = World::default();
        let mut sphere1 = world_default_sphere_1();
        let mut material1 = sphere1.material();
        material1.ambient = 1.0;
        material1.pattern = Some(Arc::new(TestPattern::new()));
        sphere1.set_material(material1);
        world.objects[0] = Arc::new(sphere1) as Arc<dyn Shape>;
        let mut sphere2 = world_default_sphere_2();
        let mut material2 = sphere2.material();
        material2.transparency = 1.0;
        material2.refractive_index = 1.5;
        sphere2.set_material(material2);
        world.objects[1] = Arc::new(sphere2) as Arc<dyn Shape>;
        let ray = Ray::new(Point::new(0.0, 0.0, 0.1), Vector::new(0.0, 1.0, 0.0));
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
        let floor_transformation = transformations::translation(0.0, -1.0, 0.0);
        let mut floor_material = Material::default();
        floor_material.transparency = 0.5;
        floor_material.refractive_index = 1.5;
        let floor = Plane::new(floor_material, floor_transformation);
        let arc_floor: Arc<dyn Shape> = Arc::new(floor);
        world.objects.push(arc_floor.clone());
        let mut ball_material = Material::default();
        ball_material.color = Color::RED;
        ball_material.ambient = 0.5;
        let mut ball = Sphere::default();
        ball.set_material(ball_material);
        ball.set_transformation(transformations::translation(0.0, -3.5, -0.5));
        let arc_ball: Arc<dyn Shape> = Arc::new(ball);
        world.objects.push(arc_ball);
        let ray = Ray::new(Point::new(0.0, 0.0, -3.0), Vector::new(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(2.0_f64.sqrt(), arc_floor));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.shade_hit(&computed_hit, 5);
        assert_eq!(color, Color::new(0.9364253889815014, 0.6864253889815014, 0.6864253889815014));
    }

    #[test]
    fn shade_hit_with_reflective_and_transparent_material() {
        let mut world = World::default();
        let floor_transformation = transformations::translation(0.0, -1.0, 0.0);
        let mut floor_material = Material::default();
        floor_material.reflectiveness = 0.5;
        floor_material.transparency = 0.5;
        floor_material.refractive_index = 1.5;
        let floor = Plane::new(floor_material, floor_transformation);
        let arc_floor: Arc<dyn Shape> = Arc::new(floor);
        world.objects.push(arc_floor.clone());
        let mut ball_material = Material::default();
        ball_material.color = Color::RED;
        ball_material.ambient = 0.5;
        let mut ball = Sphere::default();
        ball.set_material(ball_material);
        ball.set_transformation(transformations::translation(0.0, -3.5, -0.5));
        world.objects.push(Arc::new(ball));
        let ray = Ray::new(Point::new(0.0, 0.0, -3.0), Vector::new(0.0, -(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0));
        let mut intersections = Intersections::new();
        intersections.add(Intersection::new(2.0_f64.sqrt(), arc_floor));
        let computed_hit = intersections[0].prepare_computations(&ray, &intersections);
        let color = world.shade_hit(&computed_hit, 5);
        assert_eq!(color, Color::new(0.9339151412754023, 0.696434227200244, 0.692430691912747));
    }
}
