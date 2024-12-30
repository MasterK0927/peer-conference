use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use ed25519_dalek::{Verifier, VerifyingKey};

#[derive(Debug, Serialize, Deserialize)]
struct SignalMessage {
    signal_type: String,
    payload: String,
    sender_id: String,
    timestamp: i64,
    signature: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SecureConnectionPayload {
    offer: serde_json::Value,
    public_key: Vec<u8>,
    signature: Vec<u8>,
    nonce: Vec<u8>, 
}

#[derive(Debug, Clone)]
struct ClientState {
    sender: mpsc::Sender<Message>,
    client_id: String,
    public_key: Option<Vec<u8>>,
    verified: bool,
}

type Clients = Arc<Mutex<HashMap<SocketAddr, ClientState>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030);
    let listener = TcpListener::bind(&addr).await?;
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    println!("Secure WebRTC signaling server listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let clients = Arc::clone(&clients);
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, clients).await {
                eprintln!("Connection error for {}: {}", addr, e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    clients: Clients
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel(100);
    
    let client_id = uuid::Uuid::new_v4().to_string();
    {
        let mut clients_map = clients.lock().await;
        clients_map.insert(addr, ClientState {
            sender: tx,
            client_id: client_id.clone(),
            public_key: None,
            verified: false,
        });
    }

    let clients_clone = Arc::clone(&clients);
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_sender.send(msg).await {
                eprintln!("Forward error: {}", e);
                break;
            }
        }
    });

    while let Some(Ok(message)) = ws_receiver.next().await {
        if let Message::Text(text) = message {
            if let Ok(mut signal) = serde_json::from_str::<SignalMessage>(&text) {
                signal.sender_id = client_id.clone();
                signal.timestamp = Utc::now().timestamp();

                match signal.signal_type.as_str() {
                    "secure-offer" => {
                        handle_secure_offer(&signal, addr, Arc::clone(&clients_clone)).await?;
                    }
                    "secure-answer" => {
                        handle_secure_answer(&signal, addr, Arc::clone(&clients_clone)).await?;
                    }
                    "ice-candidate" => {
                        broadcast_to_verified_peers(&signal, addr, Arc::clone(&clients_clone)).await?;
                    }
                    _ => eprintln!("Unknown signal type: {}", signal.signal_type),
                }
            }
        }
    }

    // Cleanup
    forward_task.abort();
    cleanup_client(addr, clients).await;
    Ok(())
}

async fn handle_secure_offer(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) -> Result<(), Box<dyn std::error::Error>> {
    let payload: SecureConnectionPayload = serde_json::from_str(&signal.payload)?;
    
    if !verify_signature(&payload.offer, &payload.signature, &payload.public_key) {
        eprintln!("Invalid offer signature");
        return Ok(());
    }

    {
        let mut clients_map = clients.lock().await;
        if let Some(client) = clients_map.get_mut(&sender_addr) {
            client.public_key = Some(payload.public_key.clone());
            client.verified = true;
        }
    }

    broadcast_to_verified_peers(signal, sender_addr, clients).await?;
    Ok(())
}

async fn handle_secure_answer(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) -> Result<(), Box<dyn std::error::Error>> {
    let payload: SecureConnectionPayload = serde_json::from_str(&signal.payload)?;
    
    if !verify_signature(&payload.offer, &payload.signature, &payload.public_key) {
        eprintln!("Invalid answer signature");
        return Ok(());
    }

    {
        let mut clients_map = clients.lock().await;
        if let Some(client) = clients_map.get_mut(&sender_addr) {
            client.verified = true;
        }
    }

    broadcast_to_verified_peers(signal, sender_addr, clients).await?;
    Ok(())
}

async fn broadcast_to_verified_peers(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) -> Result<(), Box<dyn std::error::Error>> {
    let clients_map = clients.lock().await;
    
    let message = serde_json::to_string(signal)?;
    
    for (addr, client) in clients_map.iter() {
        if *addr != sender_addr && client.verified {
            if let Err(e) = client.sender.send(Message::Text(message.clone())).await {
                eprintln!("Broadcast error to {}: {}", addr, e);
            }
        }
    }

    Ok(())
}

fn verify_signature(
    data: &serde_json::Value,
    signature: &[u8],
    public_key: &[u8],
) -> bool {
    if let Ok(public_key_array) = <&[u8; 32]>::try_from(public_key) {
        if let Ok(verify_key) = VerifyingKey::from_bytes(public_key_array) {
            if let Ok(message) = serde_json::to_vec(data) {
                if let Ok(sig) = <&[u8; 64]>::try_from(signature).and_then(|s| Ok(ed25519_dalek::Signature::try_from(s))) {
                    return verify_key.verify(&message, &sig.unwrap()).is_ok();
                }
            }
        }
    }
    false
}


async fn cleanup_client(addr: SocketAddr, clients: Clients) {
    let mut clients_map = clients.lock().await;
    clients_map.remove(&addr);
}