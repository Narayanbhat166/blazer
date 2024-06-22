use ::blazer::app::{
    client::types::ClientConfig,
    server::grpc::server::{grpc_client, PingRequest, RoomServiceRequest},
    utils,
};
use blazer::app::server::start_server;

#[tokio::test]
async fn connect() {
    tokio::spawn(async { start_server(true).await });

    // Can this sleep time be avoided, only if I knew once the server starts
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

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
}
