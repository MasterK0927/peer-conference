use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug, Clone)]
pub struct Client {
    pub sender: mpsc::Sender<Message>,
    pub client_id: String,
    pub address: SocketAddr,
    pub public_key: Option<Vec<u8>>,
    pub verified: bool,
}

impl Client {
    pub fn new(
        sender: mpsc::Sender<Message>, 
        client_id: String, 
        address: SocketAddr
    ) -> Self {
        Self {
            sender,
            client_id,
            address,
            public_key: None,
            verified: false,
        }
    }
}