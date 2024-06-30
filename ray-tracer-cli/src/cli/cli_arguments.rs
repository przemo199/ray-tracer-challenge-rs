use crate::cli::RenderingMode;
use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArguments {
    pub scene_path: String,
    pub image_output_path: String,
    #[arg(value_enum, short, long, default_value_t = RenderingMode::Parallel)]
    pub rendering_mode: RenderingMode,
    #[arg(long, short, action)]
    pub quiet: bool,
}
