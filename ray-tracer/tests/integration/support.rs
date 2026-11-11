//! Shared scenes and helpers for the integration tests.

use ray_tracer::composites::{Camera, Canvas, Material, World};
use ray_tracer::consts::PI;
use ray_tracer::patterns::{CheckerPattern, GradientPattern, RingPattern, StripePattern};
use ray_tracer::primitives::{Color, Light, Point, Vector, transformations};
use ray_tracer::shapes::{Cone, Cube, Cylinder, Plane, Shape, Sphere, Transform, Triangle};
use std::path::PathBuf;
use std::sync::Arc;

/// Camera the book uses for rendering the default world.
pub fn default_world_camera(width: u32, height: u32) -> Camera {
    let mut camera = Camera::new(width, height, PI / 2.0);
    camera.set_transformation(transformations::view_transform(
        Point::new(0, 0, -5),
        Point::ORIGIN,
        Vector::UP,
    ));
    return camera;
}

/// A small scene exercising every shape and pattern, reflection, refraction (a glass sphere
/// with an air bubble), two lights and a shape that casts no shadow.
pub fn showcase(width: u32, height: u32) -> (World, Camera) {
    let mut shapes: Vec<Box<dyn Shape>> = Vec::new();

    let mut floor = Plane::default();
    let mut checkers = CheckerPattern::new(Color::new(0.9, 0.9, 0.9), Color::new(0.3, 0.3, 0.35));
    checkers.set_transformation(transformations::scaling(0.5, 0.5, 0.5));
    floor.material.pattern = Some(Arc::new(checkers));
    floor.material.reflectiveness = 0.2;
    shapes.push(Box::new(floor));

    let mut wall = Plane::default();
    wall.set_transformation(
        transformations::translation(0, 0, 4) * transformations::rotation_x(PI / 2.0),
    );
    let mut stripes = StripePattern::new(Color::new(0.8, 0.5, 0.4), Color::new(0.6, 0.3, 0.3));
    stripes.set_transformation(transformations::rotation_y(PI / 4.0));
    wall.material.pattern = Some(Arc::new(stripes));
    shapes.push(Box::new(wall));

    let mut glass = Sphere::default();
    glass.material = Material::glass();
    glass.material.reflectiveness = 0.9;
    glass.material.diffuse = 0.1;
    glass.set_transformation(transformations::translation(0, 1, 0));
    shapes.push(Box::new(glass));

    let mut bubble = Sphere::default();
    bubble.material = Material::glass();
    bubble.material.refractive_index = 1.0;
    bubble.set_transformation(
        transformations::translation(0, 1, 0) * transformations::scaling(0.5, 0.5, 0.5),
    );
    shapes.push(Box::new(bubble));

    let mut cube = Cube::default();
    cube.set_transformation(
        transformations::translation(-2.4, 0.6, 0.5)
            * transformations::rotation_y(0.5)
            * transformations::scaling(0.6, 0.6, 0.6),
    );
    let mut gradient = GradientPattern::new(Color::RED, Color::BLUE);
    gradient.set_transformation(transformations::scaling(2, 2, 2));
    cube.material.pattern = Some(Arc::new(gradient));
    shapes.push(Box::new(cube));

    let cylinder_material = Material {
        pattern: Some(Arc::new(RingPattern::new(Color::GREEN, Color::WHITE))),
        reflectiveness: 0.3,
        ..Material::default()
    };
    let cylinder_transformation =
        transformations::translation(2.3, 0, 0.5) * transformations::scaling(0.5, 1, 0.5);
    shapes.push(Box::new(Cylinder::new(
        cylinder_material,
        cylinder_transformation,
        0,
        1.5,
        true,
    )));

    let cone_material = Material {
        color: Color::new(0.9, 0.8, 0.2),
        ..Material::default()
    };
    let cone_transformation =
        transformations::translation(1.4, 1, -1.6) * transformations::scaling(0.5, 1, 0.5);
    shapes.push(Box::new(Cone::new(
        cone_material,
        cone_transformation,
        -1,
        0,
        true,
    )));

    let mut triangle = Triangle::new(
        Point::new(-2, 0.01, -1.5),
        Point::new(-1.2, 1.4, -1.8),
        Point::new(-0.6, 0.01, -2.2),
    );
    triangle.material.color = Color::new(0.2, 0.6, 0.9);
    shapes.push(Box::new(triangle));

    let mut no_shadow = Sphere::default();
    no_shadow.material.color = Color::new(1, 0.4, 0.8);
    no_shadow.material.casts_shadow = false;
    no_shadow.set_transformation(
        transformations::translation(0.7, 0.3, -2.2) * transformations::scaling(0.3, 0.3, 0.3),
    );
    shapes.push(Box::new(no_shadow));

    let lights = vec![
        Light::new(Point::new(-10, 10, -10), Color::new(0.7, 0.7, 0.7)),
        Light::new(Point::new(5, 8, -6), Color::new(0.4, 0.4, 0.4)),
    ];

    let mut camera = Camera::new(width, height, PI / 3.0);
    camera.set_transformation(transformations::view_transform(
        Point::new(0, 2.5, -6),
        Point::new(0, 0.8, 0),
        Vector::UP,
    ));
    return (World::new(lights, shapes), camera);
}

/// The 8-bit RGB values `Canvas::to_png_file` writes for each pixel, row by row.
pub fn to_rgb8(canvas: &Canvas) -> Vec<[u8; 3]> {
    let mut pixels = Vec::new();
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let channels = canvas.get_pixel(x, y).clamped().channels();
            pixels.push(channels.map(|channel| (channel * 255.0).round() as u8));
        }
    }
    return pixels;
}

/// Path for a file written by a test, inside `target/` (see AGENTS.md).
pub fn output_path(name: &str) -> PathBuf {
    return PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("integration")
        .join(name);
}
