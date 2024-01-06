use std::sync::{mpsc, Arc, Mutex};

use crate::{
    components::Msg,
    grpc::server::{grpc_client, MyGrpc, PingRequest},
};

use tuirealm::listener::Poll;

#[derive(PartialEq, Eq, PartialOrd, Clone)]
pub enum UserEvent {
    Pong,
}

pub enum Request {
    Ping,
}

#[derive(Clone)]
pub struct NetworkClient {
    messsages: Arc<Mutex<Vec<UserEvent>>>,
}

impl NetworkClient {
    #[tokio::main(flavor = "current_thread")]
    pub async fn start_network_client(self, message_receiver: mpsc::Receiver<Request>) {
        let mut client = grpc_client::GrpcClient::connect("http://127.0.0.1:6969")
            .await
            .unwrap();

        while let Ok(message) = message_receiver.recv() {
            let ping_request = PingRequest { client_id: None };
            client.ping(ping_request).await;
            self.messsages.lock().unwrap().push(UserEvent::Pong);
        }
    }

    pub fn new() -> Self {
        Self {
            messsages: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl Poll<UserEvent> for NetworkClient {
    fn poll(&mut self) -> tuirealm::listener::ListenerResult<Option<tuirealm::Event<UserEvent>>> {
        Ok(self
            .messsages
            .lock()
            .unwrap()
            .first()
            .cloned()
            .map(tuirealm::Event::User))
    }
}
