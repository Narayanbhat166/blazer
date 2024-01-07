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
        _request: tonic::Request<PingRequest>,
    ) -> Result<tonic::Response<PingResponse>, tonic::Status> {
        log::info!("Received Request");
        Err(tonic::Status::ok("All good"))
    }
}
