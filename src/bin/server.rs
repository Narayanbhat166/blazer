use blazer::{
    app::{types, utils},
    grpc::server::{grpc_server, MyGrpc},
};

// Single threaded runtime
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let formatter = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::fmt().event_format(formatter).init();

    let config =
        utils::read_config::<types::ServerConfig>("config/server.toml", Some("BLAZER_SERVER"));

    let redis_client = utils::create_redis_client(config.redis.unwrap_or_default())
        .await
        .unwrap();

    let server_config = config.server.unwrap_or_default();

    let server_address = format!("{}:{}", server_config.host, server_config.port);

    let addr = server_address.parse().unwrap();
    tracing::info!("Attempting to run server on {:?}", addr);

    let server = MyGrpc::new(redis_client).await;
    tracing::info!("Server successfully running on {:?}", addr);

    tonic::transport::Server::builder()
        .add_service(grpc_server::GrpcServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
