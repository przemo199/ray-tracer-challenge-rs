//! Properties that must hold for every ray, checked over a grid of camera rays through a scene
//! containing every shape type.

use crate::support;
use ray_tracer::composites::{Intersections, Ray};
use ray_tracer::utils::CoarseEq;

fn camera_rays() -> (ray_tracer::composites::World, Vec<Ray>) {
    let (world, camera) = support::showcase(40, 30);
    let rays = (0..30)
        .flat_map(|y| (0..40).map(move |x| (x, y)))
        .map(|(x, y)| camera.ray_for_pixel(x, y))
        .collect();
    return (world, rays);
}

#[test]
fn world_intersections_are_sorted_by_distance() {
    let (world, rays) = camera_rays();
    let mut intersections = Intersections::new();
    for ray in &rays {
        world.collect_intersections(ray, &mut intersections);
        let distances: Vec<f64> = intersections.iter().map(|hit| hit.distance).collect();
        assert!(distances.is_sorted(), "{ray:?}: {distances:?}");
    }
}

#[test]
fn normals_are_unit_vectors_on_every_shape() {
    let (world, rays) = camera_rays();
    let mut intersections = Intersections::new();
    let mut checked = 0;
    for ray in &rays {
        world.collect_intersections(ray, &mut intersections);
        for hit in intersections.iter() {
            let point = ray.position(hit.distance);
            let normal = hit.shape.normal_at(point);
            assert!(
                normal.magnitude().coarse_eq(&1.0),
                "{:?} at {point:?}: normal {normal:?}",
                hit.shape
            );
            checked += 1;
        }
    }
    assert!(checked > 1000, "only {checked} normals checked");
}
