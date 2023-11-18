use clap::ValueEnum;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum RenderingMode {
    Serial,
    Parallel,
}
