use anyhow::{Context, Result};
use clap::Parser;
use tracing::info;

mod cli;
mod core;
mod frontend;
mod grpc;
mod runner;

use cli::{Cli, Command};
use frontend::logger;
use grpc::{DeployManagerService, DeployManagerServer};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            file,
            verbose,
            dry_run,
        } => {
            logger::init(verbose, None);
            runner::run_deploy(file.to_str().unwrap_or_default(), dry_run).await?;
            info!("All actions completed successfully");
        }

        Command::Serve { addr } => {
            logger::init(false, None);
            info!(%addr, "Starting DeployManager gRPC server");

            tonic::transport::Server::builder()
                .add_service(DeployManagerServer::new(DeployManagerService::default()))
                .serve(addr)
                .await
                .with_context(|| format!("gRPC server failed on {addr}"))?;
        }
    }

    Ok(())
}
