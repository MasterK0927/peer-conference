use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{
        protocol::Message,
        Error as WsError
    }
};
use futures_util::{
    StreamExt,
    SinkExt
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// Enhanced message structures
#[derive(Debug, Serialize, Deserialize, Clone)]
struct SignalMessage {
    signal_type: String,
    payload: String,
    sender_ip: Option<String>,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedPayload {
    encrypted: Vec<u8>,
    iv: Vec<u8>,
    sender_ip: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    text: String,
    sender: String,
    timestamp: String,
}

// Client state tracking
#[derive(Debug, Clone)]
struct ClientState {
    sender: mpsc::Sender<Message>,
    connected_peers: Vec<SocketAddr>,
    is_screen_sharing: bool,
}

type Clients = Arc<Mutex<HashMap<SocketAddr, ClientState>>>;

// IP verification utility
fn verify_ip_signature(data: &[u8], signature: &[u8], ip: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    hasher.update(data);
    let expected = hasher.finalize();
    signature == expected.as_slice()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030);
    let listener = TcpListener::bind(&addr).await?;
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    println!("Enhanced WebRTC signaling server listening on: {}", addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        let clients = Arc::clone(&clients);
        
        tokio::spawn(async move {
            match handle_connection(stream, client_addr, clients).await {
                Ok(_) => println!("Connection closed for {}", client_addr),
                Err(e) => eprintln!("Error handling connection for {}: {}", client_addr, e),
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    clients: Clients
) -> Result<(), WsError> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let (tx, mut rx) = mpsc::channel(100);
    
    // Initialize client state
    {
        let mut clients_map = clients.lock().await;
        clients_map.insert(addr, ClientState {
            sender: tx,
            connected_peers: Vec::new(),
            is_screen_sharing: false,
        });
    }

    // Handle outgoing messages
    let clients_clone = Arc::clone(&clients);
    let forward_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                eprintln!("Error forwarding message: {}", e);
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(message) = ws_receiver.next().await {
        let message = message?;
        
        if message.is_text() {
            let text = message.to_text()?;
            
            match serde_json::from_str::<SignalMessage>(text) {
                Ok(mut signal) => {
                    // Add sender's IP to the message
                    signal.sender_ip = Some(addr.ip().to_string());
                    signal.timestamp = chrono::Utc::now().timestamp();

                    match signal.signal_type.as_str() {
                        "offer" => handle_offer(&signal, addr, Arc::clone(&clients_clone)).await,
                        "answer" => handle_answer(&signal, addr, Arc::clone(&clients_clone)).await,
                        "ice-candidate" => broadcast_to_peers(&signal, addr, Arc::clone(&clients_clone)).await,
                        "chat" => handle_chat(&signal, addr, Arc::clone(&clients_clone)).await,
                        "screen-share-start" | "screen-share-stop" => handle_screen_share(&signal, addr, Arc::clone(&clients_clone)).await,
                        _ => eprintln!("Unknown signal type: {}", signal.signal_type),
                    }
                }
                Err(e) => eprintln!("Error parsing signal message: {}", e),
            }
        }
    }

    // Cleanup on disconnect
    forward_task.abort();
    cleanup_client(addr, clients).await;

    Ok(())
}

async fn handle_offer(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) {
    {
        let mut clients_map = clients.lock().await;
        
        if let Ok(payload) = serde_json::from_str::<EncryptedPayload>(&signal.payload) {
            // Verify the sender's IP signature
            if !verify_ip_signature(&payload.encrypted, &payload.iv, &payload.sender_ip) {
                eprintln!("Invalid IP signature in offer from {}", sender_addr);
                return;
            }
        }
    }
    broadcast_to_peers(signal, sender_addr, clients).await;
}

async fn handle_answer(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) {
    let clients_map = clients.lock().await;
    
    if let Some(offer_sender) = extract_offer_sender(&signal.payload) {
        if let Some(client_state) = clients_map.get(&offer_sender) {
            let _ = client_state.sender.send(Message::Text(
                serde_json::to_string(signal).unwrap()
            )).await;
        }
    }
}

async fn handle_chat(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) {
    if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&signal.payload) {
        broadcast_to_peers(signal, sender_addr, clients).await;
    }
}

async fn handle_screen_share(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) {
    let clients_clone = Arc::clone(&clients);
    let mut clients_map = clients.lock().await;
    
    if let Some(client_state) = clients_map.get_mut(&sender_addr) {
        client_state.is_screen_sharing = signal.signal_type == "screen-share-start";
        broadcast_to_peers(signal, sender_addr, clients_clone).await;
    }
}

async fn broadcast_to_peers(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) {
    let clients_map = clients.lock().await;
    
    let signal_json = match serde_json::to_string(signal) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error serializing signal: {}", e);
            return;
        }
    };

    for (addr, client_state) in clients_map.iter() {
        if *addr != sender_addr {
            if let Err(e) = client_state.sender.send(Message::Text(signal_json.clone())).await {
                eprintln!("Error broadcasting to {}: {}", addr, e);
            }
        }
    }
}

async fn cleanup_client(addr: SocketAddr, clients: Clients) {
    let mut clients_map = clients.lock().await;
    
    if let Some(client_state) = clients_map.remove(&addr) {
        // Notify peers about disconnection
        let disconnect_signal = SignalMessage {
            signal_type: "peer-disconnected".to_string(),
            payload: addr.to_string(),
            sender_ip: Some(addr.ip().to_string()),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let signal_json = serde_json::to_string(&disconnect_signal).unwrap();
        
        for peer_addr in client_state.connected_peers {
            if let Some(peer_state) = clients_map.get(&peer_addr) {
                let _ = peer_state.sender.send(Message::Text(signal_json.clone())).await;
            }
        }
    }
}

fn extract_offer_sender(payload: &str) -> Option<SocketAddr> {
    None // Placeholder
}
