use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub scene_path: String,
    pub image_output_path: String,

    #[arg(value_enum, short, long, default_value_t = RenderingMode::Parallel)]
    pub rendering_mode: RenderingMode,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum RenderingMode {
    Serial,
    Parallel,
}
