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
use grpc::{
    DeployManagerService, DeployManagerServer, ManagedApplicationServer, ManagedApplicationService,
};

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

            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

            let managed_app_svc =
                ManagedApplicationService::new("deploy-manager", addr, shutdown_tx);

            tonic::transport::Server::builder()
                .add_service(DeployManagerServer::new(DeployManagerService::default()))
                .add_service(ManagedApplicationServer::new(managed_app_svc))
                .serve_with_shutdown(addr, async {
                    shutdown_rx.await.ok();
                    info!("Graceful shutdown signal received");
                })
                .await
                .with_context(|| format!("gRPC server failed on {addr}"))?;
        }
    }

    Ok(())
}
