//! `World::color_at`: the whole shading pipeline for a single ray.

use ray_tracer::composites::{Intersections, Ray, World};
use ray_tracer::primitives::{Color, Light, Point, Vector, transformations};
use ray_tracer::shapes::{Plane, Sphere, Transform};
use ray_tracer::utils::{CoarseEq, world_default_sphere_1, world_default_sphere_2};

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
    let expected = Color::new(
        0.38066119308103435,
        0.47582649135129296,
        0.28549589481077575,
    );
    assert!(color.coarse_eq(&expected));
}

#[test]
fn color_with_intersection_behind_ray() {
    let mut world = World::default();
    let mut outer = world_default_sphere_1();
    outer.material.ambient = 1.0;
    let mut inner = world_default_sphere_2();
    inner.material.ambient = 1.0;
    let inner_color = inner.material.color;
    world.shapes = vec![Box::new(outer), Box::new(inner)];
    let ray = Ray::new(Point::new(0, 0, 0.75), Vector::BACKWARD);
    let color = world.color_at(&ray, &mut Intersections::new());
    assert_eq!(color, inner_color);
}

#[test]
fn no_infinite_recursion_in_reflections() {
    let mut lower = Plane::default();
    lower.material.reflectiveness = 1.0;
    lower.set_transformation(transformations::translation(0, -1, 0));
    let mut upper = Plane::default();
    upper.material.reflectiveness = 1.0;
    upper.set_transformation(transformations::translation(0, 1, 0));
    let world = World::new(
        vec![Light::new(Point::ORIGIN, Color::WHITE)],
        vec![Box::new(lower), Box::new(upper)],
    );
    let ray = Ray::new(Point::ORIGIN, Vector::UP);
    // terminates thanks to the recursion limit
    world.color_at(&ray, &mut Intersections::new());
}

#[test]
fn shapes_that_do_not_cast_shadows_let_light_through() {
    let light = Light::new(Point::new(0, 10, 0), Color::WHITE);
    let mut blocker = Sphere::default();
    blocker.set_transformation(transformations::translation(0, 3, 0));
    let floor_only = World::new(vec![light], vec![Box::new(Plane::default())]);
    let shadowed = World::new(
        vec![light],
        vec![Box::new(Plane::default()), Box::new(blocker.clone())],
    );
    blocker.material.casts_shadow = false;
    let unshadowed = World::new(
        vec![light],
        vec![Box::new(Plane::default()), Box::new(blocker)],
    );

    // looks at the floor right below the sphere, without hitting the sphere itself
    let ray = Ray::new(
        Point::new(0, 1, -5),
        (Point::ORIGIN - Point::new(0, 1, -5)).normalized(),
    );
    let color_at = |world: &World| world.color_at(&ray, &mut Intersections::new());
    assert!(color_at(&shadowed) != color_at(&floor_only));
    assert_eq!(color_at(&unshadowed), color_at(&floor_only));
}
