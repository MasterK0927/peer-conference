use crate::models::{Client, SignalMessage};
use crate::models::message::SecureConnectionPayload;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use ed25519_dalek::{Verifier, VerifyingKey};
use tokio_tungstenite::tungstenite::protocol::Message;

pub async fn handle_secure_offer(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Arc<Mutex<HashMap<SocketAddr, Client>>>
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

pub async fn handle_secure_answer(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Arc<Mutex<HashMap<SocketAddr, Client>>>
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

pub async fn broadcast_to_verified_peers(
    signal: &SignalMessage,
    sender_addr: SocketAddr,
    clients: Arc<Mutex<HashMap<SocketAddr, Client>>>
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