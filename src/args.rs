use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub scene_path: String,
    pub image_output_path: String,
}
