use blazer::app::{types, utils::read_config};

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let formatter = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::fmt().event_format(formatter).init();

    let config = read_config::<types::ServerConfig>("config/server.toml", Some("BLAZER_SERVER"));
    let server_config = config.server.clone().unwrap_or_default();

    let server_address = format!("{}:{}", server_config.host, server_config.port);

    tracing::info!("Attempting to run server on {server_address}");
    let tcp_listener = tokio::net::TcpListener::bind(server_address)
        .await
        .expect("Could not bind to server address {server_address}");

    blazer::app::server::start_server(false, config, tcp_listener).await;
    Ok(())
}
