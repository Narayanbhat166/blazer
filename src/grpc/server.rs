pub use blazer_grpc::{grpc_client, grpc_server, PingRequest, PingResponse};

mod blazer_grpc {
    // The string specified here must match the proto package name
    tonic::include_proto!("server");
}

pub struct MyGrpc {}

impl MyGrpc {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl grpc_server::Grpc for MyGrpc {
    async fn ping(
        &self,
        request: tonic::Request<PingRequest>,
    ) -> Result<tonic::Response<PingResponse>, tonic::Status> {
        log::info!("{request:?}");
        let request = request.into_inner();

        let response = PingResponse {
            client_id: request.client_id.unwrap_or("hola_client".to_string()),
            client_name: Some("Hola".to_string()),
        };

        Ok(tonic::Response::new(response))
    }
}
