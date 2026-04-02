use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::runner::run_deploy_from_content;

// Include the generated protobuf/tonic code from external/roe.
pub mod deploy_manager {
    tonic::include_proto!("deploy_manager");
}

use deploy_manager::deploy_manager_server::DeployManager;
pub use deploy_manager::deploy_manager_server::DeployManagerServer;
use deploy_manager::{DeployRequest, DeployResponse};

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

        info!(
            env_vars = req.env_vars.len(),
            "gRPC Deploy request received"
        );

        // Apply supplied environment variables so that ${VAR} interpolation
        // inside the YAML content resolves correctly.
        for env_var in &req.env_vars {
            // SAFETY: single-threaded async context per request; no other
            // threads modify the environment concurrently.
            #[allow(unused_unsafe)]
            unsafe {
                std::env::set_var(&env_var.key, &env_var.value);
            }
        }

        match run_deploy_from_content(&req.yaml_content, false).await {
            Ok(report) => {
                info!("gRPC Deploy completed successfully");
                Ok(Response::new(DeployResponse {
                    success: true,
                    report,
                }))
            }
            Err(e) => {
                error!(error = %e, "gRPC Deploy failed");
                Ok(Response::new(DeployResponse {
                    success: false,
                    report: vec![e.to_string()],
                }))
            }
        }
    }
}
