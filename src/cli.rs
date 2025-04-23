use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the config file
    #[arg(short, long, value_name = "FILE")]
    pub config_path: PathBuf,

    /// Path to the allowlist file
    #[arg(short, long, value_name = "FILE")]
    pub allow_list: Option<PathBuf>,

    /// Path to the blocklist file
    #[arg(short, long, value_name = "FILE")]
    pub block_list: Option<PathBuf>,
}
