use crate::cli::{CliArguments, RenderingMode};
use crate::scene_loader::load_scene_description;
use clap::Parser;
use std::error::Error;
use std::time::Instant;

mod cli;
mod scene_loader;
mod scenes;

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArguments::parse();
    let (world, camera) = load_scene_description(&args.scene_path)?;
    if !args.quiet {
        println!("Rendering image using scene at {}", args.scene_path);
    }
    let now = Instant::now();
    let canvas = match args.rendering_mode {
        RenderingMode::Serial => camera.render(&world),
        RenderingMode::Parallel => camera.render_parallel(&world),
    };
    let seconds_elapsed = now.elapsed().as_secs_f64();
    if !args.quiet {
        println!("Image rendered in: {seconds_elapsed:.3?}s");
    }
    canvas.to_png_file(&args.image_output_path)?;
    if !args.quiet {
        println!("Image saved at {}", args.image_output_path);
    }
    return Ok(());
}
