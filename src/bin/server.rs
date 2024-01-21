use blazer::{
    app::{types, utils},
    grpc::server::{grpc_server, MyGrpc},
};

// Single threaded runtime
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(true)
        .init();

    let config =
        utils::read_config::<types::ServerConfig>("server_config.toml", Some("BLAZER_SERVER"));

    let redis_client = utils::create_redis_client(config.redis.unwrap_or_default())
        .await
        .unwrap();

    let server_config = config.server.unwrap_or_default();

    let server_address = format!("{}:{}", server_config.host, server_config.port);

    let addr = server_address.parse().unwrap();

    let server = MyGrpc::new(redis_client).await;
    log::info!("Server successfully running on {:?}", addr);

    tonic::transport::Server::builder()
        .add_service(grpc_server::GrpcServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
