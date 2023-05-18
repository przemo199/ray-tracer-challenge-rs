use std::time::Instant;

use clap::Parser;

use raytracer::scene_loader::load_scene_description;

use crate::args::Args;

mod args;
mod scenes;

fn main() {
    let args = Args::parse();
    let (world, camera) = load_scene_description(args.scene_path);
    let now = Instant::now();
    let canvas = camera.render_parallel(&world);
    let elapsed = now.elapsed();
    println!("Rendered in: {:.3?}s", elapsed.as_secs_f64());
    canvas.to_png_file(args.image_output_path);
}
