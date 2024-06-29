pub mod errors;
pub mod grpc;

use app::server::grpc::{
    server::{grpc_server, MyGrpc, FILE_DESCRIPTOR_SET},
    utils::create_redis_client,
};

use crate::app::{self, types};

pub async fn start_server(
    _test_mode: bool,
    server_config: types::ServerConfig,
    tcp_listener: tokio::net::TcpListener,
) {
    let redis_client = create_redis_client(server_config.redis.unwrap_or_default())
        .await
        .unwrap();

    let service = MyGrpc::new(redis_client).await;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    tracing::info!("Server successfully running");

    tonic::transport::Server::builder()
        .add_service(reflection_service)
        .add_service(grpc_server::GrpcServer::new(service))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(tcp_listener))
        .await
        .expect("Could not start the server");
}
