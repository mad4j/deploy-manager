use std::net::SocketAddr;
use tokio::sync::{oneshot, Mutex};
use tonic::{Request, Response, Status};
use tracing::{info, warn};

pub mod managed_application_proto {
    tonic::include_proto!("managed_application");
}

use managed_application_proto::managed_application_server::ManagedApplication;
pub use managed_application_proto::managed_application_server::ManagedApplicationServer;
use managed_application_proto::{
    InfoRequest, InfoResponse, ListeningAddress, TerminateRequest, TerminateResponse,
};

pub struct ManagedApplicationService {
    app_name: String,
    listening_addr: SocketAddr,
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
}

impl ManagedApplicationService {
    pub fn new(app_name: impl Into<String>, listening_addr: SocketAddr, shutdown_tx: oneshot::Sender<()>) -> Self {
        Self {
            app_name: app_name.into(),
            listening_addr,
            shutdown_tx: Mutex::new(Some(shutdown_tx)),
        }
    }
}

#[tonic::async_trait]
impl ManagedApplication for ManagedApplicationService {
    async fn info(
        &self,
        _request: Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {
        info!("gRPC Info request received");

        let response = InfoResponse {
            app_name: self.app_name.clone(),
            listening_addresses: vec![ListeningAddress {
                address: self.listening_addr.to_string(),
                services: vec![
                    "deploy_manager.DeployManager".to_string(),
                    "managed_application.ManagedApplication".to_string(),
                ],
            }],
        };

        Ok(Response::new(response))
    }

    async fn terminate(
        &self,
        request: Request<TerminateRequest>,
    ) -> Result<Response<TerminateResponse>, Status> {
        let reason = &request.into_inner().reason;

        if reason.is_empty() {
            info!("gRPC Terminate request received");
        } else {
            info!(reason = %reason, "gRPC Terminate request received");
        }

        let sender = self.shutdown_tx.lock().await.take();

        match sender {
            Some(tx) => {
                if tx.send(()).is_err() {
                    warn!("Shutdown signal could not be delivered: receiver already dropped");
                }
                Ok(Response::new(TerminateResponse {
                    success: true,
                    message: "Shutdown initiated".to_string(),
                }))
            }
            None => {
                warn!("Terminate called but shutdown already in progress");
                Ok(Response::new(TerminateResponse {
                    success: false,
                    message: "Shutdown already in progress".to_string(),
                }))
            }
        }
    }
}
