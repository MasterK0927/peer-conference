use crate::models::{Client, SignalMessage};
use crate::models::message::SecureConnectionPayload;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::protocol::Message;
use p256::ecdsa::signature::Verifier;

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
    // Check public key length - P-256 public keys are uncompressed (65 bytes) or compressed (33 bytes)
    if public_key.len() != 65 && public_key.len() != 33 {
        eprintln!("[ERROR] Invalid public key length: expected 65 or 33 bytes, got {}", public_key.len());
        return false;
    }

    // Check signature length - P-256 ECDSA signatures are typically 64 bytes (r and s components)
    if signature.len() != 64 {
        eprintln!("[ERROR] Secure offer handling error: Uint8Array of valid length expected - got {} bytes", signature.len());
        return false;
    }

    // Convert the message to bytes
    let message = match serde_json::to_vec(data) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("[ERROR] Failed to serialize data: {}", e);
            return false;
        }
    };

    // Use p256 crate for verification
    use p256::ecdsa::{Signature, VerifyingKey};
    use p256::{EncodedPoint, FieldBytes};
    
    // Import public key
    let encoded_point = match EncodedPoint::from_bytes(public_key) {
        Ok(point) => point,
        Err(e) => {
            eprintln!("[ERROR] Failed to parse public key: {}", e);
            return false;
        }
    };

    let verifying_key = match VerifyingKey::from_encoded_point(&encoded_point) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("[ERROR] Invalid verifying key: {}", e);
            return false;
        }
    };

    // Create signature object from raw bytes - use clone_from_slice to get owned values
    let signature = match Signature::from_scalars(
        FieldBytes::clone_from_slice(&signature[..32]),
        FieldBytes::clone_from_slice(&signature[32..])
    ) {
        Ok(sig) => sig,
        Err(e) => {
            eprintln!("[ERROR] Failed to parse signature: {}", e);
            return false;
        }
    };

    // Verify the signature
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&message);
    let digest = hasher.finalize();
    
    match verifying_key.verify(&digest, &signature) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("[ERROR] Signature verification failed: {}", e);
            false
        }
    }
}