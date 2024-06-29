use ::blazer::app::{
    client::types::ClientConfig,
    server::grpc::server::{grpc_client, PingRequest, RoomServiceRequest},
    utils,
};
use blazer::app::{server::start_server, types};

async fn configure_server(config: &types::ServerConfig) -> tokio::net::TcpListener {
    let formatter = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true);

    let server_config = config.server.clone().unwrap_or_default();

    tracing_subscriber::fmt().event_format(formatter).init();

    let server_address = format!("{}:{}", server_config.host, server_config.port);
    let tcp_listener = tokio::net::TcpListener::bind(server_address)
        .await
        .expect("Could not bind to server address {server_address}");

    tcp_listener
}

#[tokio::test]
async fn connect() {
    let config =
        utils::read_config::<types::ServerConfig>("config/server.toml", Some("BLAZER_SERVER"));
    let tcp_listener = configure_server(&config).await;
    tokio::spawn(async { start_server(true, config, tcp_listener).await });

    let config = utils::read_config::<ClientConfig>("config/client.toml", Some("BLAZER"));

    let mut client = grpc_client::GrpcClient::connect(config.server_url.clone())
        .await
        .unwrap();

    let ping_response = client.ping(PingRequest { user_id: None }).await.unwrap();
    let user_id = ping_response.into_inner().user_id;

    let room_service_response = client
        .room_service(RoomServiceRequest {
            client_id: user_id,
            room_id: None,
            request_type: 2,
        })
        .await
        .unwrap();

    drop(room_service_response);

    // This sleep is necessary for the server to do some cleanup activities as soon as the function ends
    // If some sort of signaling mechanism is implemented for the server, this can be avoided
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
