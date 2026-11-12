//! Golden-image tests: small renders compared with reference PNGs in `golden/`.
//!
//! After an intentional change to rendered output, regenerate the references with
//! `UPDATE_GOLDEN=1 cargo test -p ray-tracer --test integration golden`, look at the new images,
//! and commit them together with the change.

use crate::support;
use ray_tracer::composites::{Canvas, World};
use std::path::PathBuf;

/// Per-channel difference (out of 255) tolerated for floating-point differences between platforms.
const TOLERANCE: u8 = 2;

fn check(name: &str, canvas: &Canvas) {
    let reference_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/integration/golden")
        .join(format!("{name}.png"));
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        canvas.to_png_file(&reference_path).unwrap();
        return;
    }
    let reference = image::open(&reference_path)
        .unwrap_or_else(|error| panic!("{}: {error}", reference_path.display()))
        .to_rgb8();
    assert_eq!(
        (reference.width(), reference.height()),
        (canvas.width, canvas.height),
        "{name}: size differs from the reference"
    );

    // largest channel difference of each pixel
    let actual = canvas.to_rgb8();
    let (actual_pixels, _) = actual.as_chunks::<3>();
    let (expected_pixels, _) = reference.as_raw().as_chunks::<3>();
    let differences: Vec<u8> = actual_pixels
        .iter()
        .zip(expected_pixels)
        .map(|([r, g, b], [expected_r, expected_g, expected_b])| {
            return r
                .abs_diff(*expected_r)
                .max(g.abs_diff(*expected_g))
                .max(b.abs_diff(*expected_b));
        })
        .collect();
    // Rays grazing an edge may flip to a different surface on another platform; allow a few.
    let allowed_outliers = (differences.len() / 1000).max(1);
    let outliers = differences
        .iter()
        .filter(|&&difference| difference > TOLERANCE)
        .count();
    if outliers > allowed_outliers {
        let actual_path = support::output_path(&format!("{name}-actual.png"));
        canvas.to_png_file(&actual_path).unwrap();
        panic!(
            "{name}: {outliers} pixels differ by more than {TOLERANCE} (allowed {allowed_outliers}, \
             max difference {}). Rendered image: {}",
            differences.iter().max().unwrap_or(&0),
            actual_path.display()
        );
    }
}

#[test]
fn default_world() {
    let camera = support::default_world_camera(500, 500);
    check("default_world", &camera.render(&World::default(), false));
}

#[test]
fn showcase() {
    let (world, camera) = support::showcase(750, 500);
    check("showcase", &camera.render_parallel(&world, false));
}

#[test]
fn textures() {
    let (world, camera) = support::textures(750, 500);
    check("textures", &camera.render_parallel(&world, false));
}
