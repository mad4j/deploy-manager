use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::runner::run_deploy;

// Include the generated protobuf/tonic code.
pub mod proto {
    tonic::include_proto!("deploy_manager");
}

use proto::deploy_manager_server::DeployManager;
pub use proto::deploy_manager_server::DeployManagerServer;
use proto::{ExecuteRequest, ExecuteResponse};

#[derive(Debug, Default)]
pub struct DeployManagerService;

#[tonic::async_trait]
impl DeployManager for DeployManagerService {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let req = request.into_inner();
        info!(file = %req.file_path, dry_run = req.dry_run, "gRPC Execute request received");

        match run_deploy(&req.file_path, req.dry_run).await {
            Ok(()) => {
                info!("gRPC Execute completed successfully");
                Ok(Response::new(ExecuteResponse {
                    success: true,
                    message: "All actions completed successfully".to_string(),
                }))
            }
            Err(e) => {
                error!(error = %e, "gRPC Execute failed");
                Ok(Response::new(ExecuteResponse {
                    success: false,
                    message: e.to_string(),
                }))
            }
        }
    }
}
