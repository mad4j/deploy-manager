use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// deploy-manager – platform-independent CLI deployment tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose / debug output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run actions from a deploy YAML file
    Run {
        /// Path to the YAML deploy file
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,

        /// Dry-run: parse and validate the deploy file but do not execute actions
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Start a gRPC server accepting DeployManager calls
    Serve {
        /// Address to listen on (e.g. [::1]:50051 or 0.0.0.0:50051)
        #[arg(short, long, default_value = "[::1]:50051")]
        listen: String,
    },
}

