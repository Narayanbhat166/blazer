pub mod errors;
pub mod grpc;

use app::server::grpc::{
    server::{grpc_server, MyGrpc, FILE_DESCRIPTOR_SET},
    utils::create_redis_client,
};
use tokio_stream::StreamExt;

fn get_signals() -> signal_hook_tokio::Signals {
    signal_hook_tokio::Signals::new(&[signal_hook::consts::SIGINT])
        .expect("Unable to register the signals for graceful shutdown")
}

async fn handle_signal(
    mut signals: signal_hook_tokio::Signals,
    shutdown_informer: tokio::sync::oneshot::Sender<()>,
) {
    while let Some(signal) = signals.next().await {
        match signal {
            signal_hook::consts::SIGINT => {
                tracing::warn!("Received signal to shutdown");
                shutdown_informer
                    .send(())
                    .expect("Unable to inform shutdown signal");
                break;
            }
            _ => unreachable!(),
        }
    }
}

use crate::app::{self, types};

pub async fn start_server(
    server_config: types::ServerConfig,
    tcp_listener: tokio::net::TcpListener,
) {
    let redis_client = create_redis_client(server_config.redis.unwrap_or_default())
        .await
        .unwrap();

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let shutdown_signals = get_signals();
    tokio::spawn(handle_signal(shutdown_signals, sender));

    let service = MyGrpc::new(redis_client).await;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    tracing::info!("Server successfully running");

    tonic::transport::Server::builder()
        .add_service(reflection_service)
        .add_service(grpc_server::GrpcServer::new(service))
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(tcp_listener),
            async {
                receiver.await.unwrap();
                tracing::info!("Initiating graceful shutdown, waiting for tasks to finish");
            },
        )
        .await
        .expect("Could not start the server");

    tracing::info!("Server shutdown");
}
