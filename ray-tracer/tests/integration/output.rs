//! Writing canvases to image files.

use crate::support;
use ray_tracer::composites::Canvas;
use ray_tracer::primitives::Color;
use std::fs;

fn sample_canvas() -> Canvas {
    let mut canvas = Canvas::new(4, 2);
    canvas.set_pixel(0, 0, Color::new(1.5, 0, 0));
    canvas.set_pixel(1, 0, Color::new(0, 0.5, 0));
    canvas.set_pixel(2, 0, Color::new(-0.5, 0, 1));
    canvas.set_pixel(3, 1, Color::WHITE);
    return canvas;
}

#[test]
fn png_round_trip_clamps_and_rounds_channels() {
    let path = support::output_path("round_trip.png");
    sample_canvas().to_png_file(&path).unwrap();
    let image = image::open(&path).unwrap().to_rgb8();
    assert_eq!((image.width(), image.height()), (4, 2));
    assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0]);
    assert_eq!(image.get_pixel(1, 0).0, [0, 128, 0]);
    assert_eq!(image.get_pixel(2, 0).0, [0, 0, 255]);
    assert_eq!(image.get_pixel(3, 1).0, [255, 255, 255]);
    assert_eq!(image.get_pixel(0, 1).0, [0, 0, 0]);
}

#[test]
fn ppm_file_has_header_and_trailing_newline() {
    let path = support::output_path("sample.ppm");
    sample_canvas().to_ppm_file(&path).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("P3\n4 2\n255\n"), "{content}");
    assert!(content.ends_with('\n'));
}

#[test]
fn writing_creates_missing_directories() {
    let path = support::output_path("nested/directory/image.png");
    let _ = fs::remove_dir_all(support::output_path("nested"));
    Canvas::new(1, 1).to_png_file(&path).unwrap();
    assert!(path.is_file());
}
