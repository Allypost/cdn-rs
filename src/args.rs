use std::path::PathBuf;

use clap::{Parser, ValueHint};
use once_cell::sync::Lazy;

#[derive(Debug, Parser)]
pub struct CliArgs {
    /// Which directory to use as the root for the CDN
    #[arg(short='d', long, value_name = "DIRECTORY_TO_SERVE", env = "CDN_SERVE_DIRECTORY", value_hint = ValueHint::DirPath)]
    pub serve_directory: PathBuf,

    /// Temp directory path to use. Defaults to OS temp directory.
    ///
    /// Will be used as a temporary directory to store files while they're being compressed.
    /// It is required to not polute the CDN directory with partially processed files.
    ///
    /// If path doesn't exist, it will be created.
    #[arg(long, value_name = "TEMP_DIRECTORY", env = "CDN_TEMP_DIRECTORY", value_hint = ValueHint::DirPath)]
    pub temp_directory: Option<PathBuf>,

    /// Whether to append index.html to directories
    ///
    /// Whether to serve the index.html file if folder is requested
    #[arg(
        long,
        env = "CDN_APPEND_INDEX_HTML_ON_DIRECTORIES",
        default_value_t = false
    )]
    pub append_index_html_on_directories: bool,

    /// Whether to generate compressed files dynamically
    ///
    /// If enabled, the application will generate gzipped and deflated versions of the files.
    /// Compression happens automatically and asynchronously.
    ///
    /// Warning!
    /// Compressed files will be saved alongside already existing files.
    #[arg(short, long, env = "CDN_COMPRESS_FILES", default_value_t = true)]
    pub compress_files: bool,

    /// Tells the CDN whether to cache files or set cache-busting headers
    #[arg(long, env = "CDN_NO_CACHE_FILES", default_value_t = false)]
    pub no_cache_files: bool,

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

impl CliArgs {
    pub fn temp_dir(&self) -> PathBuf {
        self.temp_directory
            .clone()
            .unwrap_or_else(std::env::temp_dir)
    }
}

pub static ARGS: Lazy<CliArgs> = Lazy::new(CliArgs::parse);
