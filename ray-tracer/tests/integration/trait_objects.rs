//! Equality of shapes and patterns stored as trait objects.

use ray_tracer::patterns::{ComplexPattern, GradientPattern, RingPattern};
use ray_tracer::primitives::Color;
use ray_tracer::shapes::{Shape, Sphere};
use std::sync::Arc;

fn sphere_with(pattern: ComplexPattern) -> Box<dyn Shape> {
    let mut sphere = Sphere::default();
    sphere.material.pattern = Some(Arc::new(pattern));
    return Box::new(sphere);
}

#[test]
fn compare_dyn_shapes() {
    let ring = |a, b| Arc::new(RingPattern::new(a, b));
    let gradient = |a, b| Arc::new(GradientPattern::new(a, b));
    let reference = sphere_with(ComplexPattern::new(
        ring(Color::WHITE, Color::BLACK),
        gradient(Color::WHITE, Color::BLACK),
    ));
    let same = sphere_with(ComplexPattern::new(
        ring(Color::WHITE, Color::BLACK),
        gradient(Color::WHITE, Color::BLACK),
    ));
    let different_color = sphere_with(ComplexPattern::new(
        ring(Color::BLACK, Color::BLACK),
        gradient(Color::WHITE, Color::BLACK),
    ));
    let different_type = sphere_with(ComplexPattern::new(
        ring(Color::WHITE, Color::BLACK),
        ring(Color::WHITE, Color::BLACK),
    ));
    assert_eq!(reference, same);
    assert_ne!(reference, different_color);
    assert_ne!(reference, different_type);
}
