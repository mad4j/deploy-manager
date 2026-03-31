use clap::Parser;
use std::path::PathBuf;

/// deploy-manager – platform-independent CLI deployment tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the YAML deploy file
    #[arg(short, long, value_name = "FILE")]
    pub file: PathBuf,

    /// Enable verbose / debug output
    #[arg(short, long)]
    pub verbose: bool,

    /// Dry-run: parse and validate the deploy file but do not execute actions
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}
