use std::time::Instant;

use clap::Parser;

use raytracer::scene_loader::load_scene_description;

use crate::args::Args;

mod args;
mod scenes;

fn main() {
    let args = Args::parse();
    let world_camera = load_scene_description(args.scene_path);
    let now = Instant::now();
    let canvas = world_camera.1.render_parallel(&world_camera.0);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed.as_millis());
    canvas.to_png_file(args.image_output_path);
}
