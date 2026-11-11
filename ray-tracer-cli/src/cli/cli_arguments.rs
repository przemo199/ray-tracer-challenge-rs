use crate::cli::RenderingMode;
use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArguments {
    /// YAML scene file to render
    pub scene_path: String,
    /// Where to write the rendered PNG image
    pub image_output_path: String,
    #[arg(value_enum, short, long, default_value_t = RenderingMode::Parallel)]
    pub rendering_mode: RenderingMode,
    /// Print nothing, not even the progress bar
    #[arg(long, short, action)]
    pub quiet: bool,
}
