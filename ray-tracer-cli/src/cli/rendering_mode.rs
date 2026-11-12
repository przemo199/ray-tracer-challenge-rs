use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ValueEnum)]
pub enum RenderingMode {
    Serial,
    Parallel,
}
