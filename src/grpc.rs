use std::collections::HashMap;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info};

use crate::core::config::parse_deploy_file_with_env;
use crate::core::executor::execute;
use crate::frontend::progress::ProgressTracker;

pub mod proto {
    tonic::include_proto!("deploy_manager");
}

use proto::{
    deploy_manager_server::{DeployManager, DeployManagerServer},
    DeployRequest, DeployResponse,
};

#[derive(Debug, Default)]
pub struct DeployManagerService;

#[tonic::async_trait]
impl DeployManager for DeployManagerService {
    async fn deploy(
        &self,
        request: Request<DeployRequest>,
    ) -> Result<Response<DeployResponse>, Status> {
        let req = request.into_inner();

        if req.yaml_content.is_empty() {
            return Err(Status::invalid_argument("yaml_content must not be empty"));
        }

        let env_overrides: HashMap<String, String> = req
            .env_vars
            .into_iter()
            .map(|e| (e.key, e.value))
            .collect();

        let deploy = parse_deploy_file_with_env(&req.yaml_content, &env_overrides)
            .map_err(|e| Status::invalid_argument(format!("Invalid deploy file: {e}")))?;

        info!(actions = deploy.actions.len(), "gRPC deploy request received");

        let tracker = ProgressTracker::new(deploy.actions.len());

        match execute(&deploy, false, &tracker).await {
            Ok(()) => {
                info!("gRPC deploy request completed successfully");
                Ok(Response::new(DeployResponse {
                    success: true,
                    report: vec!["Deployment completed successfully.".to_string()],
                }))
            }
            Err(e) => {
                error!(error = %e, "gRPC deploy request failed");
                Ok(Response::new(DeployResponse {
                    success: false,
                    report: vec![e.to_string()],
                }))
            }
        }
    }
}

/// Start the gRPC server listening on `addr`.
pub async fn serve(addr: &str) -> anyhow::Result<()> {
    let socket_addr: std::net::SocketAddr = addr
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid listen address '{}': {}", addr, e))?;

    let service = DeployManagerService;

    info!(address = %socket_addr, "DeployManager gRPC server listening");

    Server::builder()
        .add_service(DeployManagerServer::new(service))
        .serve(socket_addr)
        .await
        .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
}
