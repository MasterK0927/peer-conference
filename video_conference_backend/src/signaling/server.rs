use crate::models::{Client, SignalMessage};
use crate::signaling::handlers;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use chrono::Utc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

pub async fn run_signaling_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&addr).await?;
    let clients: Arc<Mutex<HashMap<SocketAddr, Client>>> = Arc::new(Mutex::new(HashMap::new()));

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
    clients: Arc<Mutex<HashMap<SocketAddr, Client>>>
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel(100);
    
    let client_id = uuid::Uuid::new_v4().to_string();
    {
        let mut clients_map = clients.lock().await;
        clients_map.insert(addr, Client::new(tx, client_id.clone(), addr));
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
                        handlers::handle_secure_offer(&signal, addr, Arc::clone(&clients_clone)).await?;
                    }
                    "secure-answer" => {
                        handlers::handle_secure_answer(&signal, addr, Arc::clone(&clients_clone)).await?;
                    }
                    "ice-candidate" => {
                        handlers::broadcast_to_verified_peers(&signal, addr, Arc::clone(&clients_clone)).await?;
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

async fn cleanup_client(addr: SocketAddr, clients: Arc<Mutex<HashMap<SocketAddr, Client>>>) {
    let mut clients_map = clients.lock().await;
    clients_map.remove(&addr);
}