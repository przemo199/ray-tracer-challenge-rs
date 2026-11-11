//! Rendering a world through a camera.

use crate::support;
use ray_tracer::composites::World;
use ray_tracer::primitives::Color;
use ray_tracer::utils::CoarseEq;
use rstest::rstest;

#[rstest]
#[case::serial(false)]
#[case::parallel(true)]
fn rendering_world_with_camera(#[case] parallel: bool) {
    let world = World::default();
    let camera = support::default_world_camera(11, 11);
    let canvas = if parallel {
        camera.render_parallel(&world, false)
    } else {
        camera.render(&world, false)
    };
    let expected = Color::new(
        0.38066119308103435,
        0.47582649135129296,
        0.28549589481077575,
    );
    assert!(canvas.get_pixel(5, 5).coarse_eq(&expected));
}

#[test]
fn serial_and_parallel_renders_are_identical() {
    let (world, camera) = support::showcase(32, 24);
    let serial = camera.render(&world, false);
    let parallel = camera.render_parallel(&world, false);
    assert_eq!(serial.to_rgb8(), parallel.to_rgb8());
}
