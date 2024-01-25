use crate::cli::{CliArguments, RenderingMode};
use clap::Parser;
use raytracer::scene_loader::load_scene_description;
use std::error::Error;
use std::time::Instant;

mod cli;
mod scenes;

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArguments::parse();
    let (world, camera) = load_scene_description(args.scene_path)?;
    let now = Instant::now();
    let canvas = match args.rendering_mode {
        RenderingMode::Serial => camera.render(&world),
        RenderingMode::Parallel => camera.render_parallel(&world),
    };
    let elapsed = now.elapsed();
    println!("Rendered in: {:.3?}s", elapsed.as_secs_f64());
    return canvas.to_png_file(args.image_output_path);
}
