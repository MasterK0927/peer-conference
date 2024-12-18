use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use serde_json::json;
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
use p256::ecdsa::{Signature, VerifyingKey};
use ed25519_dalek::Verifier;

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
    signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectionVerificationPayload {
    challenge: Vec<u8>,
    offer: serde_json::Value,
    encrypted_data: Option<EncryptedPayload>,
    connection_message: Option<String>,
    public_key: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ClientState {
    sender: mpsc::Sender<Message>,
    connected_peers: Vec<SocketAddr>,
    is_screen_sharing: bool,
    challenges: HashMap<Vec<u8>, bool>,
}

type Clients = Arc<Mutex<HashMap<SocketAddr, ClientState>>>;

// fn default_timestamp() -> i64 {
//     Utc::now().timestamp()
// }

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    text: String,
    sender: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChallengeResponsePayload {
    challenge: Vec<u8>,
    challenge_response: Vec<u8>,
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

fn verify_payload_signature(
    payload: &ConnectionVerificationPayload
) -> bool {
    println!("Payload for verification: {:?}", payload);

    // Check challenge length
    if payload.challenge.len() != 32 {
        println!("Invalid challenge length. Expected 32 bytes, got {}", payload.challenge.len());
        return false;
    }

    // Check public key length
    if payload.public_key.len() != 294 {
        println!("Invalid public key length. Expected 294 bytes, got {}", payload.public_key.len());
        return false;
    }

    // Check signature length
    if payload.signature.len() != 64 {
        println!("Invalid signature length. Expected 64 bytes, got {}", payload.signature.len());
        return false;
    }

    // Reconstruct payload for signature verification
    let verification_payload = json!({
        "challenge": payload.challenge,
        "offer": payload.offer,
        "encrypted_data": payload.encrypted_data,
        "connection_message": payload.connection_message,
    });

    // Parse the public key
    let verifying_key = match VerifyingKey::from_sec1_bytes(&payload.public_key) {
        Ok(key) => key,
        Err(e) => {
            println!("Failed to create verifying key: {:?}", e);
            return false;
        }
    };

    // Parse the signature
    let signature = match Signature::from_bytes(payload.signature.as_slice().try_into().unwrap()) {
        Ok(sig) => sig,
        Err(e) => {
            println!("Failed to parse signature: {:?}", e);
            return false;
        }
    };

    // Verify the signature
    match verifying_key.verify(
        serde_json::to_string(&verification_payload)
            .unwrap()
            .as_bytes(),
        &signature
    ) {
        Ok(_) => {
            println!("Signature verification successful.");
            true
        }
        Err(e) => {
            println!("Signature verification failed: {:?}", e);
            false
        }
    }
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
            challenges: HashMap::new(),
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
                    signal.sender_ip = Some(addr.ip().to_string());
                    signal.timestamp = chrono::Utc::now().timestamp();

                    match signal.signal_type.as_str() {
                        "offer-with-challenge" => handle_offer_with_challenge(&signal, addr, Arc::clone(&clients_clone)).await,
                        "challenge-response" => handle_challenge_response(&signal, addr, Arc::clone(&clients_clone)).await,
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

fn validate_encrypted_payload(payload: &EncryptedPayload) -> Result<(), String> {
    if payload.encrypted.is_empty() {
        return Err("Missing 'encrypted' field".to_string());
    }
    if payload.iv.is_empty() {
        return Err("Missing 'iv' field".to_string());
    }
    if payload.sender_ip.is_empty() {
        return Err("Missing 'sender_ip' field".to_string());
    }
    if payload.signature.is_empty() {
        return Err("Missing 'signature' field".to_string());
    }
    Ok(())
}
async fn handle_offer_with_challenge(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients,
) {
    let payload: ConnectionVerificationPayload = match serde_json::from_str(&signal.payload) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Invalid offer payload: {}", e);
            return;
        }
    };

    if let Some(encrypted_data) = &payload.encrypted_data {
        if let Err(err) = validate_encrypted_payload(encrypted_data) {
            eprintln!("Invalid encrypted payload: {}", err);
            return;
        }
    } else {
        eprintln!("Invalid encrypted data: missing required fields.");
        return;
    }

    // Continue processing if payload is valid
    println!("Encrypted payload validated successfully: {:?}", payload.encrypted_data);

    // Verify payload signature
    if !verify_payload_signature(&payload) {
        eprintln!("Invalid payload signature");
        return;
    }

    // Store the challenge for later verification
    {
        let mut clients_map = clients.lock().await;
        if let Some(client_state) = clients_map.get_mut(&sender_addr) {
            client_state.challenges.insert(payload.challenge.clone(), false);
        }
    }

    // Broadcast the offer payload to other clients
    let challenge_response_signal = SignalMessage {
        signal_type: "challenge-response".to_string(),
        payload: serde_json::to_string(&payload).unwrap(),
        sender_ip: Some(sender_addr.ip().to_string()),
        timestamp: Utc::now().timestamp(),
    };

    broadcast_to_peers(&challenge_response_signal, sender_addr, clients).await;
}



async fn handle_challenge_response(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) {
    let payload: ChallengeResponsePayload = match serde_json::from_str(&signal.payload) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Invalid challenge response: {}", e);
            return;
        }
    };

    // Verify the challenge response
    let mut is_valid = false;
    {
        let mut clients_map = clients.lock().await;
        if let Some(client_state) = clients_map.get_mut(&sender_addr) {
            // Check if the challenge exists and matches
            if client_state.challenges.contains_key(&payload.challenge) {
                is_valid = true;
                client_state.challenges.insert(payload.challenge.clone(), true);
            }
        }
    }

    if is_valid {
        let verification_signal = SignalMessage {
            signal_type: "connection-verified".to_string(),
            payload: "Connection established".to_string(),
            sender_ip: Some(sender_addr.ip().to_string()),
            timestamp: Utc::now().timestamp(),
        };

        broadcast_to_peers(&verification_signal, sender_addr, clients).await;
    }
}

async fn handle_answer(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Clients
) {
    let _ = sender_addr;
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
    if let Ok(_chat_msg) = serde_json::from_str::<ChatMessage>(&signal.payload) {
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
    let _ = payload;
    None // Placeholder
}
