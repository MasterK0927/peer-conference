use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignalMessage {
    pub signal_type: String,
    pub payload: String,
    pub sender_id: String,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecureConnectionPayload {
    pub offer: serde_json::Value,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub nonce: Vec<u8>,
}