#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    blazer::app::server::start_server(false).await;
    Ok(())
}
