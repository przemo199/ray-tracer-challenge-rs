//! Shared scenes and helpers for the integration tests.

use image::{Rgb, RgbImage};
use ray_tracer::composites::{Camera, Material, World};
use ray_tracer::consts::PI;
use ray_tracer::patterns::{
    CheckerPattern, ColorProvider, CubeMap, CylindricalMapping, GradientPattern, PlanarMapping,
    RingPattern, SphericalMapping, StripePattern, TextureMap, UvAlignCheck, UvCheckers, UvImage,
    UvPattern,
};
use ray_tracer::primitives::{Color, Light, Point, Vector, transformations};
use ray_tracer::shapes::{Cone, Cube, Cylinder, Plane, Shape, Sphere, Transform, Triangle};
use std::path::PathBuf;

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
    floor.material.pattern = Some(Box::new(checkers));
    floor.material.reflectiveness = 0.2;
    shapes.push(Box::new(floor));

    let mut wall = Plane::default();
    wall.set_transformation(
        transformations::translation(0, 0, 4) * transformations::rotation_x(PI / 2.0),
    );
    let mut stripes = StripePattern::new(Color::new(0.8, 0.5, 0.4), Color::new(0.6, 0.3, 0.3));
    stripes.set_transformation(transformations::rotation_y(PI / 4.0));
    wall.material.pattern = Some(Box::new(stripes));
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
    cube.material.pattern = Some(Box::new(gradient));
    shapes.push(Box::new(cube));

    let cylinder_material = Material {
        pattern: Some(Box::new(RingPattern::new(Color::GREEN, Color::WHITE))),
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

/// Texture maps: planar checkers on the floor, spherical checkers on one sphere, and an image on
/// another. The image is generated so its orientation is easy to verify: red at the top fading to
/// blue at the bottom, a green stripe at u = 0.25 and a white marker in the top-left corner.
/// At the back, a cylinder shows an align check whose corners meet at the u = 0 seam facing the
/// camera: red (upper left) and green (bottom left) just right of it, yellow and cyan just left.
/// In front, a cube map shows a differently colored align check on each face; visible are `Back`
/// (blue, towards the camera), `Left` (orange) and `Up` (white). From outside, each face appears
/// mirrored (the book's convention is upright from inside), e.g. red, the upper-left corner, shows
/// at the top right of the blue face.
pub fn textures(width: u32, height: u32) -> (World, Camera) {
    fn texture_material(texture: impl ColorProvider) -> Material {
        return Material {
            pattern: Some(Box::new(texture)),
            specular: 0.3,
            ..Material::default()
        };
    }
    let floor_checkers =
        UvCheckers::new(2, 2, Color::new(0.9, 0.9, 0.9), Color::new(0.2, 0.3, 0.6));
    let floor = Plane::new(
        texture_material(TextureMap::new(
            Box::new(floor_checkers),
            Box::new(PlanarMapping),
        )),
        transformations::translation(0, -1, 0),
    );

    let globe_checkers = UvCheckers::new(16, 8, Color::new(0.1, 0.6, 0.2), Color::WHITE);
    let checkered_sphere = Sphere::new(
        texture_material(TextureMap::new(
            Box::new(globe_checkers),
            Box::new(SphericalMapping),
        )),
        transformations::translation(-1.2, 0, 0),
    );

    let image = RgbImage::from_fn(128, 64, |x, y| {
        let top = y < 8 && x < 16;
        let stripe = (28..36).contains(&x);
        return match (top, stripe) {
            (true, _) => Rgb([255, 255, 255]),
            (false, true) => Rgb([0, 200, 0]),
            (false, false) => Rgb([255 - (y * 4) as u8, 40, (y * 4) as u8]),
        };
    });
    let mut image_texture =
        TextureMap::new(Box::new(UvImage::new(image)), Box::new(SphericalMapping));
    // turn the stripe and marker towards the camera
    image_texture.set_transformation(transformations::rotation_y(PI / 2.0));
    let image_sphere = Sphere::new(
        texture_material(image_texture),
        transformations::translation(1.2, 0, 0),
    );

    let align_check = UvAlignCheck::new(
        Color::new(0.9, 0.9, 0.9),
        Color::RED,
        Color::new(1, 1, 0),
        Color::GREEN,
        Color::new(0, 1, 1),
    );
    let can = Cylinder::new(
        texture_material(TextureMap::new(
            Box::new(align_check),
            Box::new(CylindricalMapping),
        )),
        transformations::translation(0, 0.4, 4) * transformations::scaling(0.8, 1.8, 0.8),
        0,
        1,
        false,
    );

    let lights = vec![Light::new(Point::new(-5, 8, -10), Color::WHITE)];
    let mut camera = Camera::new(width, height, PI / 3.0);
    camera.set_transformation(transformations::view_transform(
        Point::new(0, 1.5, -5),
        Point::new(0, 0, 0),
        Vector::UP,
    ));
    let face = |main: Color| -> Box<dyn UvPattern> {
        return Box::new(UvAlignCheck::new(
            main,
            Color::RED,
            Color::new(1, 1, 0),
            Color::GREEN,
            Color::new(0, 1, 1),
        ));
    };
    let cube_map = CubeMap::new(
        face(Color::new(1, 0.5, 0)),
        face(Color::new(0.5, 0, 0.5)),
        face(Color::new(0.4, 0.4, 0.4)),
        face(Color::new(0.2, 0.3, 1)),
        face(Color::WHITE),
        face(Color::BLACK),
    );
    let cube = Cube::new(
        texture_material(cube_map),
        transformations::translation(0, -0.55, -1.2)
            * transformations::rotation_y(-0.6)
            * transformations::scaling(0.45, 0.45, 0.45),
    );

    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(floor),
        Box::new(checkered_sphere),
        Box::new(image_sphere),
        Box::new(can),
        Box::new(cube),
    ];
    return (World::new(lights, shapes), camera);
}

/// Path for a file written by a test, inside `target/` (see AGENTS.md).
pub fn output_path(name: &str) -> PathBuf {
    return PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("integration")
        .join(name);
}
