use blazer::grpc::{
    self,
    server::{self, grpc_server, MyGrpc},
};

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(true)
        .init();

    let addr = "127.0.0.1:6969".parse().unwrap();

    let server = MyGrpc::new();
    log::info!("Running the server on {:?}", addr);

    tonic::transport::Server::builder()
        .add_service(grpc_server::GrpcServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
