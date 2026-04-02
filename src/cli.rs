use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;

/// deploy-manager – platform-independent CLI deployment tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Parse and execute a deploy file
    Run {
        /// Path to the YAML deploy file
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,

        /// Enable verbose / debug output
        #[arg(short, long)]
        verbose: bool,

        /// Dry-run: parse and validate the deploy file but do not execute actions
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Start a gRPC server that listens for DeployManager commands
    Serve {
        /// Address to listen on (e.g. 0.0.0.0:50051)
        #[arg(short, long, default_value = "[::1]:50051")]
        addr: SocketAddr,
    },
}
