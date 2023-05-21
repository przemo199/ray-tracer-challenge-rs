use std::sync::Arc;
use rayon::iter::ParallelIterator;
use rayon::prelude::*;
use raytracer::camera::Camera;
use raytracer::canvas::Canvas;
use raytracer::consts::PI;
use raytracer::material::Material;
use raytracer::patterns::{CheckerPattern, RingPattern};
use raytracer::primitives::{Color, Light, Point, transformations, Vector};
use raytracer::ray::Ray;
use raytracer::shapes::{Cone, Plane, Shape, Sphere};
use raytracer::world::World;

pub fn raytrace_red_sphere() {
    let ray_origin = Point::new(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_side_length = 1000;
    let pixel_size = wall_size / (canvas_side_length as f64);
    let half_wall_size = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_side_length, canvas_side_length);
    let color = Color::RED;
    let shape = Sphere::default();
    let boxed_shape: Box<dyn Shape> = Box::new(shape);

    for y in 0..canvas_side_length {
        let world_y = half_wall_size - pixel_size * (y as f64);
        for x in 0..canvas_side_length {
            let world_x = -half_wall_size + pixel_size * (x as f64);
            let position = Point::new(world_x, world_y, wall_z);
            let ray = Ray::new(ray_origin, (position - ray_origin).normalized());
            let intersections = ray.intersect(boxed_shape.as_ref());
            let hit = intersections.hit();
            if hit.is_some() {
                canvas.set_pixel(x, y, color);
            }
        }
    }

    canvas.to_ppm_file("red_sphere.ppm");
}

pub fn raytrace_red_sphere_parallel() {
    let ray_origin = Point::new(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_side_length = 1000;
    let pixel_size = wall_size / (canvas_side_length as f64);
    let half_wall_size = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_side_length, canvas_side_length);
    let color = Color::RED;
    let shape = Sphere::default();
    let boxed_shape: Box<dyn Shape> = Box::new(shape);

    canvas.pixels.par_iter_mut().enumerate().for_each(|(index, pixel)| {
        let x: u32 = index as u32 % canvas_side_length;
        let y: u32 = index as u32 / canvas_side_length;
        let world_x = -half_wall_size + pixel_size * (x as f64);
        let world_y = half_wall_size - pixel_size * (y as f64);
        let position = Point::new(world_x, world_y, wall_z);
        let ray = Ray::new(ray_origin, (position - ray_origin).normalized());
        let intersections = ray.intersect(boxed_shape.as_ref());
        let hit = intersections.hit();
        if hit.is_some() {
            *pixel = color;
        }
    });

    canvas.to_ppm_file("rendered_images/red_sphere.ppm");
}

pub fn raytrace_shaded_sphere_parallel() {
    let ray_origin = Point::new(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_side_length = 1000;
    let pixel_size = wall_size / (canvas_side_length as f64);
    let half_wall_size = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_side_length, canvas_side_length);
    let mut shape = Sphere::default();
    shape.material = Material::default();
    shape.material.color = Color::new(0.5, 0.5, 1.0);
    // shape.material.color = Color::green();
    let boxed_shape: Box<dyn Shape> = Box::new(shape);
    let light = Light::new(Point::new(-10.0, 10.0, -10.0), Color::WHITE);

    canvas.pixels.par_iter_mut().enumerate().for_each(|(index, pixel)| {
        let x: u32 = index as u32 % canvas.width;
        let y: u32 = index as u32 / canvas.width;
        let world_x = -half_wall_size + pixel_size * (x as f64);
        let world_y = half_wall_size - pixel_size * (y as f64);
        let position = Point::new(world_x, world_y, wall_z);
        let ray = Ray::new(ray_origin, (position - ray_origin).normalized());
        let intersections = ray.intersect(boxed_shape.as_ref());
        let maybe_hit = intersections.hit();
        if let Some(hit) = maybe_hit {
            let point = ray.position(hit.distance);
            let normal = hit.object.normal_at(point);
            let camera = -ray.direction;
            *pixel = hit.object.material().lighting(
                &*hit.object,
                &light,
                &point,
                &camera,
                &normal,
                &false,
            );
        }
    });

    canvas.to_png_file("rendered_images/shaded_sphere.png");
}

pub fn render_scene_parallel(x: u32, y: u32) {
    let mut floor = Sphere::default();
    floor.transformation = transformations::scaling(10.0, 0.01, 10.0);
    floor.material.color = Color::new(1.0, 0.9, 0.9);
    floor.material.specular = 0.0;

    let mut left_wall = Sphere::default();
    left_wall.transformation = transformations::translation(0.0, 0.0, 5.0) *
        transformations::rotation_y(-PI / 4.0) *
        transformations::rotation_x(PI / 2.0) *
        transformations::scaling(10.0, 0.01, 10.0);
    left_wall.material = floor.material.clone();

    let mut right_wall = Sphere::default();
    right_wall.transformation = transformations::translation(0.0, 0.0, 5.0) *
        transformations::rotation_y(PI / 4.0) *
        transformations::rotation_x(PI / 2.0) *
        transformations::scaling(10.0, 0.01, 10.0);
    right_wall.material = floor.material.clone();

    let mut middle_sphere = Sphere::default();
    middle_sphere.transformation = transformations::translation(-0.5, 1.0, 0.5);
    middle_sphere.material.color = Color::new(0.1, 1.0, 0.5);
    middle_sphere.material.diffuse = 0.7;
    middle_sphere.material.specular = 0.3;

    let mut right_sphere = Sphere::default();
    right_sphere.transformation = transformations::translation(1.5, 0.5, -0.5) *
        transformations::scaling(0.5, 0.5, 0.5);
    right_sphere.material.color = Color::new(0.5, 1.0, 0.1);
    right_sphere.material.diffuse = 0.7;
    right_sphere.material.specular = 0.3;

    let mut left_sphere = Sphere::default();
    left_sphere.transformation = transformations::translation(-1.5, 0.33, -0.75) *
        transformations::scaling(0.33, 0.33, 0.33);
    left_sphere.material.color = Color::new(1.0, 0.8, 0.1);
    left_sphere.material.diffuse = 0.7;
    left_sphere.material.specular = 0.3;

    let mut world = World::default();
    world.shapes = vec![
        Box::new(floor),
        Box::new(left_wall),
        Box::new(right_wall),
        Box::new(middle_sphere),
        Box::new(right_sphere),
        Box::new(left_sphere),
    ];

    let mut camera = Camera::new(x, y, PI / 3.0);
    camera.transformation = transformations::view_transform(
        Point::new(0.0, 1.5, -5.0),
        Point::new(0.0, 1.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
    );

    let canvas = camera.render_parallel(&world);
    canvas.to_png_file("rendered_images/test.png");
}

pub fn render_scene_parallel2(x: u32, y: u32) {
    let mut floor = Plane::default();
    floor.material.color = Color::new(1.0, 0.9, 0.9);
    floor.material.specular = 0.0;
    floor.material.pattern = Some(Arc::new(RingPattern::new(Color::new(0.15, 0.15, 0.15), Color::new(0.85, 0.85, 0.85))));

    let mut left_wall = Sphere::default();
    left_wall.transformation = transformations::translation(0.0, 0.0, 5.0) *
        transformations::rotation_y(-PI / 4.0) *
        transformations::rotation_x(PI / 2.0) *
        transformations::scaling(10.0, 0.01, 10.0);
    left_wall.material = floor.material.clone();

    let mut right_wall = Sphere::default();
    right_wall.transformation = transformations::translation(0.0, 0.0, 5.0) *
        transformations::rotation_y(PI / 4.0) *
        transformations::rotation_x(PI / 2.0) *
        transformations::scaling(10.0, 0.01, 10.0);
    right_wall.material = floor.material.clone();

    let mut middle_sphere = Sphere::default();
    middle_sphere.transformation = transformations::translation(-0.5, 1.0, 0.5);
    middle_sphere.material.color = Color::new(0.1, 1.0, 0.5);
    middle_sphere.material.diffuse = 0.7;
    middle_sphere.material.specular = 0.3;

    let mut right_sphere = Sphere::default();
    right_sphere.transformation = transformations::translation(1.5, 0.5, -0.5) *
        transformations::scaling(0.5, 0.5, 0.5);
    right_sphere.material.color = Color::new(0.5, 1.0, 0.1);
    right_sphere.material.diffuse = 0.7;
    right_sphere.material.specular = 0.3;

    let mut left_sphere = Sphere::default();
    left_sphere.transformation = transformations::translation(-1.5, 0.33, -0.75) *
        transformations::scaling(0.33, 0.33, 0.33);
    left_sphere.material.color = Color::new(1.0, 0.8, 0.1);
    left_sphere.material.diffuse = 0.7;
    left_sphere.material.specular = 0.3;

    let mut world = World::default();
    world.shapes = vec![
        Box::new(floor),
        Box::new(middle_sphere),
        Box::new(right_sphere),
        Box::new(left_sphere),
    ];

    world.lights = vec![Light::new(Point::new(-10.0, 10.0, -10.0), Color::WHITE)];

    let mut camera = Camera::new(x, y, PI / 3.0);
    camera.transformation = transformations::view_transform(
        Point::new(0.0, 1.5, -5.0),
        Point::new(0.0, 1.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
    );

    let canvas = camera.render_parallel(&world);
    canvas.to_png_file("rendered_images/3_sphere_scene.png");
}

pub fn render_scene_parallel3(x: u32, y: u32) {
    let mut floor = Plane::default();
    floor.material.color = Color::RED;
    floor.material.specular = 0.0;
    floor.material.pattern = Some(Arc::new(CheckerPattern::new(Color::new(0.15, 0.15, 0.15), Color::new(0.85, 0.85, 0.85))));

    let mut left_wall = Sphere::default();
    left_wall.transformation = transformations::translation(0.0, 0.0, 5.0) *
        transformations::rotation_y(-PI / 4.0) *
        transformations::rotation_x(PI / 2.0) *
        transformations::scaling(10.0, 0.01, 10.0);
    left_wall.material = floor.material.clone();

    let mut right_wall = Sphere::default();
    right_wall.transformation = transformations::translation(0.0, 0.0, 5.0) *
        transformations::rotation_y(PI / 4.0) *
        transformations::rotation_x(PI / 2.0) *
        transformations::scaling(10.0, 0.01, 10.0);
    right_wall.material = floor.material.clone();

    let mut middle_sphere = Sphere::default();
    middle_sphere.transformation = transformations::translation(-0.5, 1.0, 0.5);
    middle_sphere.material.color = Color::new(0.1, 1.0, 0.5);
    middle_sphere.material.diffuse = 0.7;
    middle_sphere.material.specular = 0.3;
    middle_sphere.material.shininess = 150.0;

    let mut right_sphere = Sphere::default();
    right_sphere.transformation = transformations::translation(1.5, 0.5, -0.5) *
        transformations::scaling(0.5, 0.5, 0.5);
    right_sphere.material.color = Color::new(0.5, 1.0, 0.1);
    right_sphere.material.diffuse = 0.7;
    right_sphere.material.specular = 0.3;

    let mut left_sphere = Sphere::default();
    left_sphere.transformation = transformations::translation(-1.5, 0.5, -0.75) *
        transformations::scaling(0.33, 0.33, 0.33);
    left_sphere.material.color = Color::new(1.0, 0.8, 0.1);
    left_sphere.material.diffuse = 0.7;
    left_sphere.material.specular = 0.3;

    let mut cone = Cone::default();
    cone.closed = true;
    cone.maximum = 1.0;
    cone.minimum = 0.0;
    cone.set_transformation(transformations::translation(0.0, 0.2, -1.5) *
        transformations::scaling(0.5, 0.5, 0.5));
    let mut material = Material::default();
    material.diffuse = 0.2;
    material.ambient = 0.0;
    material.specular = 1.0;
    material.shininess = 200.0;
    material.reflectiveness = 0.7;
    material.transparency = 0.7;
    material.refractive_index = 2.5;
    material.color = Color::new(1.0, 1.0, 1.0);
    // material.color = Color::new(0.373, 0.404, 0.550);
    cone.set_material(material);

    let mut cone2 = cone.clone();
    cone2.set_transformation(transformations::translation(0.0, 0.25, -1.5) *
        transformations::scaling(0.4, 0.4, 0.4));

    let mut world = World::default();
    world.shapes = vec![
        Box::new(floor),
        Box::new(middle_sphere),
        Box::new(right_sphere),
        Box::new(left_sphere),
        Box::new(cone),
        Box::new(cone2),
    ];

    world.lights = vec![Light::new(Point::new(-100.0, 100.0, -100.0), Color::WHITE)];

    let mut camera = Camera::new(x, y, PI / 3.0);
    camera.transformation = transformations::view_transform(
        Point::new(0.0, 1.5, -5.0),
        Point::new(0.0, 1.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
    );

    let canvas = camera.render_parallel(&world);
    canvas.to_png_file("rendered_images/3_sphere_scene.png");
}

pub fn render_refraction_test() {
    let light = Light::new(Point::new(2.0, 10.0, -5.0), Color::new(0.9, 0.9, 0.9));
    let material = Material::new(
        Color::new(0.0, 0.0, 0.0),
        Some(Arc::new(CheckerPattern::new(Color::new(0.15, 0.15, 0.15), Color::new(0.85, 0.85, 0.85)))),
        0.8,
        0.2,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0
    );
    let transformation = transformations::translation(0.0, 0.0, 10.0) * transformations::rotation_x(1.5708);
    let wall = Plane::new(material, transformation);
    let material = Material::new(Color::new(1.0, 1.0, 1.0), None, 0.0, 0.0, 0.9, 300.0, 0.9, 0.9, 1.5);
    let ball = Sphere::new(material, transformations::IDENTITY);
    let material = Material::new(Color::new(1.0, 1.0, 1.0), None, 0.0, 0.0, 0.9, 300.0, 0.9, 0.9, 1.0000034);
    let transformation = transformations::scaling(0.5, 0.5, 0.5);
    let ball2 = Sphere::new(material, transformation);
    let mut objects: Vec<Box<dyn Shape>> = Vec::new();
    objects.push(Box::new(wall));
    objects.push(Box::new(ball));
    objects.push(Box::new(ball2));
    let world = World::new(vec![light], objects);
    let mut camera = Camera::new(3840, 2650, 0.65);
    camera.transformation = transformations::view_transform(
        Point::new(0.0, 0.0, -5.0),
        Point::new(0.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
    );
    let canvas = camera.render_parallel(&world);
    canvas.to_png_file("rendered_images/refraction_test.png");
}

pub fn render_refraction_test2() {
    let light = Light::new(Point::new(2.0, 10.0, -5.0), Color::new(0.9, 0.9, 0.9));
    let material = Material::new(
        Color::new(0.0, 0.0, 0.0),
        Some(Arc::new(CheckerPattern::new(Color::new(0.15, 0.15, 0.15), Color::new(0.85, 0.85, 0.85)))),
        0.8, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0);
    let transformation = transformations::translation(0.0, 0.0, 10.0) * transformations::rotation_x(1.5708);
    let wall = Plane::new(material, transformation);
    let material = Material::new( Color::new(1.0, 1.0, 1.0), None, 0.0, 0.0, 0.9, 300.0, 0.9, 0.9, 1.5);
    let ball = Sphere::new(material, transformations::IDENTITY);
    let material = Material::new(Color::new(1.0, 1.0, 1.0), None, 0.0, 0.0, 0.9, 300.0, 0.9, 0.9, 1.0000034);
    let transformation = transformations::scaling(0.5, 0.5, 0.5);
    let ball2 = Sphere::new(material, transformation);
    let mut objects: Vec<Box<dyn Shape>> = Vec::new();
    objects.push(Box::new(wall));
    objects.push(Box::new(ball));
    objects.push(Box::new(ball2));
    let world = World::new(vec![light], objects);
    let mut camera = Camera::new(500, 500, 0.65);
    camera.transformation = transformations::view_transform(
        Point::new(0.0, 0.0, -5.0),
        Point::new(0.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
    );
    let canvas = camera.render_parallel(&world);
    canvas.to_png_file("rendered_images/refraction_test.png");
}