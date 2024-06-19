use std::path::PathBuf;

use clap::{Parser, ValueHint};
use once_cell::sync::Lazy;

#[derive(Debug, Parser)]
pub struct CliArgs {
    /// Which directory to use as the root for the CDN
    #[arg(short='d', long, value_name = "DIRECTORY_TO_SERVE", env = "CDN_SERVE_DIRECTORY", value_hint = ValueHint::DirPath)]
    pub serve_directory: PathBuf,

    /// Whether to generate compressed files dynamically
    ///
    /// If enabled, the application will generate gzipped and deflated versions of the files.
    /// Compression happens automatically and asynchronously.
    ///
    /// Warning!
    /// Compressed files will be saved alongside already existing files.
    #[arg(short, long, env = "CDN_COMPRESS_FILES", default_value_t = true)]
    pub compress_files: bool,

    /// Host to bind to.
    ///
    /// Usually either localohst if you don't want the service visible to the outside or 0.0.0.0
    /// otherwise.
    #[arg(short = 'H', long, default_value = "0.0.0.0", env = "HOST")]
    pub host: String,

    /// Which port to use
    #[arg(short = 'P', long, default_value = "8000", env = "PORT")]
    pub port: u16,
}

pub static ARGS: Lazy<CliArgs> = Lazy::new(CliArgs::parse);
