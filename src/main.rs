use crate::args::{Args, RenderingMode};
use clap::Parser;
use raytracer::scene_loader::load_scene_description;
use std::time::Instant;

mod args;
mod scenes;

fn main() {
    let args = Args::parse();
    let (world, camera) = load_scene_description(args.scene_path);
    let now = Instant::now();
    let canvas = match args.rendering_mode {
        RenderingMode::SERIAL => camera.render(&world),
        RenderingMode::PARALLEL => camera.render_parallel(&world),
    };
    let elapsed = now.elapsed();
    println!("Rendered in: {:.3?}s", elapsed.as_secs_f64());
    canvas.to_png_file(args.image_output_path);
}
