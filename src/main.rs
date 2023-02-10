use std::sync::Arc;
use std::time::Instant;
use rayon::prelude::*;
use raytracer::camera::Camera;
use raytracer::canvas::Canvas;
use raytracer::color::Color;
use raytracer::cone::Cone;
use raytracer::consts::PI;
use raytracer::light::Light;
use raytracer::material::Material;
use raytracer::plane::Plane;
use raytracer::ray::Ray;
use raytracer::shape::Shape;
use raytracer::sphere::Sphere;
use raytracer::transformations::Transformations;
use raytracer::tuple::{Tuple, TupleTrait};
use raytracer::world::World;

fn main() {
    // use std::time::Instant;
    // let now = Instant::now();
    // render_scene_parallel();
    // let elapsed = now.elapsed();
    // println!("Elapsed: {:.2?}", elapsed.as_millis());

    let now2 = Instant::now();
    render_scene_parallel(3840, 2560);
    let elapsed2 = now2.elapsed();
    println!("Elapsed: {:.2?}", elapsed2.as_millis());
}

fn raytrace_red_sphere() {
    let ray_origin = Tuple::point(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_side_length = 1000;
    let pixel_size = wall_size / (canvas_side_length as f64);
    let half_wall_size = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_side_length, canvas_side_length);
    let color = Color::red();
    let shape = Sphere::default();
    let arc_shape: Arc<dyn Shape> = Arc::new(shape);

    for y in 0..canvas_side_length {
        let world_y = half_wall_size - pixel_size * (y as f64);
        for x in 0..canvas_side_length {
            let world_x = -half_wall_size + pixel_size * (x as f64);
            let position = Tuple::point(world_x, world_y, wall_z);
            let ray = Ray::new(ray_origin, (position - ray_origin).normalize());
            let intersections = ray.intersect(&arc_shape);
            let hit = intersections.hit();
            if hit.is_some() {
                canvas.set_pixel(x, y, color);
            }
        }
    }

    canvas.to_ppm_file("red_sphere.ppm");
}

fn raytrace_red_sphere_parallel() {
    let ray_origin = Tuple::point(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_side_length = 1000;
    let pixel_size = wall_size / (canvas_side_length as f64);
    let half_wall_size = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_side_length, canvas_side_length);
    let color = Color::red();
    let shape = Sphere::default();
    let shape_box: Arc<dyn Shape> = Arc::new(shape);

    canvas.pixels.par_iter_mut().enumerate().for_each(|(index, pixel)| {
        let x: u32 = index as u32 % canvas_side_length;
        let y: u32 = index as u32 / canvas_side_length;
        let world_x = -half_wall_size + pixel_size * (x as f64);
        let world_y = half_wall_size - pixel_size * (y as f64);
        let position = Tuple::point(world_x, world_y, wall_z);
        let ray = Ray::new(ray_origin, (position - ray_origin).normalize());
        let intersections = ray.intersect(&shape_box);
        let hit = intersections.hit();
        if hit.is_some() {
            *pixel = color;
        }
    });

    canvas.to_ppm_file("red_sphere.ppm");
}

fn raytrace_shaded_sphere_parallel() {
    let ray_origin = Tuple::point(0.0, 0.0, -5.0);
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
    let shape_box: Arc<dyn Shape> = Arc::new(shape);
    let light = Light::new(Tuple::point(-10.0, 10.0, -10.0), Color::white());

    canvas.pixels.par_iter_mut().enumerate().for_each(|(index, pixel)| {
        let x: u32 = index as u32 % canvas.width;
        let y: u32 = index as u32 / canvas.width;
        let world_x = -half_wall_size + pixel_size * (x as f64);
        let world_y = half_wall_size - pixel_size * (y as f64);
        let position = Tuple::point(world_x, world_y, wall_z);
        let ray = Ray::new(ray_origin, (position - ray_origin).normalize());
        let intersections = ray.intersect(&shape_box);
        let maybe_hit = intersections.hit();
        if let Some(hit) = maybe_hit {
            let point = ray.position(hit.t);
            let normal = hit.object.normal_at(point);
            let camera = -ray.direction;
            *pixel = hit.object.material().lighting(
                &*hit.object,
                &light,
                &point,
                &camera,
                &normal,
                false,
            );
        }
    });

    canvas.to_png_file("shaded_sphere.png");
}

fn render_scene_parallel(x: u32, y: u32) {
    let mut floor = Sphere::default();
    floor.transformation = Transformations::scaling(10.0, 0.01, 10.0);
    floor.material.color = Color::new(1.0, 0.9, 0.9);
    floor.material.specular = 0.0;

    let mut left_wall = Sphere::default();
    left_wall.transformation = Transformations::translation(0.0, 0.0, 5.0) *
        Transformations::rotation_y(-PI / 4.0) *
        Transformations::rotation_x(PI / 2.0) *
        Transformations::scaling(10.0, 0.01, 10.0);
    left_wall.material = floor.material.clone();

    let mut right_wall = Sphere::default();
    right_wall.transformation = Transformations::translation(0.0, 0.0, 5.0) *
        Transformations::rotation_y(PI / 4.0) *
        Transformations::rotation_x(PI / 2.0) *
        Transformations::scaling(10.0, 0.01, 10.0);
    right_wall.material = floor.material.clone();

    let mut middle_sphere = Sphere::default();
    middle_sphere.transformation = Transformations::translation(-0.5, 1.0, 0.5);
    middle_sphere.material.color = Color::new(0.1, 1.0, 0.5);
    middle_sphere.material.diffuse = 0.7;
    middle_sphere.material.specular = 0.3;

    let mut right_sphere = Sphere::default();
    right_sphere.transformation = Transformations::translation(1.5, 0.5, -0.5) *
        Transformations::scaling(0.5, 0.5, 0.5);
    right_sphere.material.color = Color::new(0.5, 1.0, 0.1);
    right_sphere.material.diffuse = 0.7;
    right_sphere.material.specular = 0.3;

    let mut left_sphere = Sphere::default();
    left_sphere.transformation = Transformations::translation(-1.5, 0.33, -0.75) *
        Transformations::scaling(0.33, 0.33, 0.33);
    left_sphere.material.color = Color::new(1.0, 0.8, 0.1);
    left_sphere.material.diffuse = 0.7;
    left_sphere.material.specular = 0.3;

    let mut world = World::default();
    world.objects = vec![
        Arc::new(floor),
        Arc::new(left_wall),
        Arc::new(right_wall),
        Arc::new(middle_sphere),
        Arc::new(right_sphere),
        Arc::new(left_sphere),
    ];

    let mut camera = Camera::new(x, y, PI / 3.0);
    camera.transformation = Transformations::view_transform(
        Tuple::point(0.0, 1.5, -5.0),
        Tuple::point(0.0, 1.0, 0.0),
        Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render_parallel(&world);
    canvas.to_ppm_file("test.ppm");
    canvas.to_png_file("test.png");
}

fn render_scene_parallel2(x: u32, y: u32) {
    let mut floor = Plane::default();
    floor.material.color = Color::new(1.0, 0.9, 0.9);
    floor.material.specular = 0.0;

    let mut left_wall = Sphere::default();
    left_wall.transformation = Transformations::translation(0.0, 0.0, 5.0) *
        Transformations::rotation_y(-PI / 4.0) *
        Transformations::rotation_x(PI / 2.0) *
        Transformations::scaling(10.0, 0.01, 10.0);
    left_wall.material = floor.material.clone();

    let mut right_wall = Sphere::default();
    right_wall.transformation = Transformations::translation(0.0, 0.0, 5.0) *
        Transformations::rotation_y(PI / 4.0) *
        Transformations::rotation_x(PI / 2.0) *
        Transformations::scaling(10.0, 0.01, 10.0);
    right_wall.material = floor.material.clone();

    let mut middle_sphere = Sphere::default();
    middle_sphere.transformation = Transformations::translation(-0.5, 1.0, 0.5);
    middle_sphere.material.color = Color::new(0.1, 1.0, 0.5);
    middle_sphere.material.diffuse = 0.7;
    middle_sphere.material.specular = 0.3;

    let mut right_sphere = Sphere::default();
    right_sphere.transformation = Transformations::translation(1.5, 0.5, -0.5) *
        Transformations::scaling(0.5, 0.5, 0.5);
    right_sphere.material.color = Color::new(0.5, 1.0, 0.1);
    right_sphere.material.diffuse = 0.7;
    right_sphere.material.specular = 0.3;

    let mut left_sphere = Sphere::default();
    left_sphere.transformation = Transformations::translation(-1.5, 0.33, -0.75) *
        Transformations::scaling(0.33, 0.33, 0.33);
    left_sphere.material.color = Color::new(1.0, 0.8, 0.1);
    left_sphere.material.diffuse = 0.7;
    left_sphere.material.specular = 0.3;

    let mut world = World::default();
    world.objects = vec![
        Arc::new(floor),
        Arc::new(middle_sphere),
        Arc::new(right_sphere),
        Arc::new(left_sphere),
    ];

    world.light = Light::new(Tuple::point(-10.0, 10.0, -10.0), Color::white());

    let mut camera = Camera::new(x, y, PI / 3.0);
    camera.transformation = Transformations::view_transform(
        Tuple::point(0.0, 1.5, -5.0),
        Tuple::point(0.0, 1.0, 0.0),
        Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render_parallel(&world);
    canvas.to_png_file("3_sphere_scene.png");
}

fn render_scene_parallel3(x: u32, y: u32) {
    let mut floor = Plane::default();
    floor.material.color = Color::red();
    floor.material.specular = 0.0;

    let mut left_wall = Sphere::default();
    left_wall.transformation = Transformations::translation(0.0, 0.0, 5.0) *
        Transformations::rotation_y(-PI / 4.0) *
        Transformations::rotation_x(PI / 2.0) *
        Transformations::scaling(10.0, 0.01, 10.0);
    left_wall.material = floor.material.clone();

    let mut right_wall = Sphere::default();
    right_wall.transformation = Transformations::translation(0.0, 0.0, 5.0) *
        Transformations::rotation_y(PI / 4.0) *
        Transformations::rotation_x(PI / 2.0) *
        Transformations::scaling(10.0, 0.01, 10.0);
    right_wall.material = floor.material.clone();

    let mut middle_sphere = Sphere::default();
    middle_sphere.transformation = Transformations::translation(-0.5, 1.0, 0.5);
    middle_sphere.material.color = Color::new(0.1, 1.0, 0.5);
    middle_sphere.material.diffuse = 0.7;
    middle_sphere.material.specular = 0.3;
    middle_sphere.material.shininess = 150.0;

    let mut right_sphere = Sphere::default();
    right_sphere.transformation = Transformations::translation(1.5, 0.5, -0.5) *
        Transformations::scaling(0.5, 0.5, 0.5);
    right_sphere.material.color = Color::new(0.5, 1.0, 0.1);
    right_sphere.material.diffuse = 0.7;
    right_sphere.material.specular = 0.3;

    let mut left_sphere = Sphere::default();
    left_sphere.transformation = Transformations::translation(-1.5, 0.5, -0.75) *
        Transformations::scaling(0.33, 0.33, 0.33);
    left_sphere.material.color = Color::new(1.0, 0.8, 0.1);
    left_sphere.material.diffuse = 0.7;
    left_sphere.material.specular = 0.3;

    let mut cone = Cone::default();
    cone.closed = true;
    cone.maximum = 1.0;
    cone.minimum = 0.0;
    cone.set_transformation(Transformations::translation(0.0, 0.2, -1.5) *
        Transformations::scaling(0.5, 0.5, 0.5));
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
    cone2.set_transformation(Transformations::translation(0.0, 0.25, -1.5) *
        Transformations::scaling(0.4, 0.4, 0.4));

    let mut world = World::default();
    world.objects = vec![
        Arc::new(floor),
        Arc::new(middle_sphere),
        Arc::new(right_sphere),
        Arc::new(left_sphere),
        Arc::new(cone),
        Arc::new(cone2),
    ];

    world.light = Light::new(Tuple::point(-100.0, 100.0, -100.0), Color::white());

    let mut camera = Camera::new(x, y, PI / 3.0);
    camera.transformation = Transformations::view_transform(
        Tuple::point(0.0, 1.5, -5.0),
        Tuple::point(0.0, 1.0, 0.0),
        Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render_parallel(&world);
    canvas.to_png_file("3_sphere_scene.png");
}
