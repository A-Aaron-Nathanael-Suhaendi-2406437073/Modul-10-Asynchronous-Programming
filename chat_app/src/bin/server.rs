use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast::{Sender, channel}, Mutex};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: String,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    active_users: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();
    let mut my_username = String::new();

    // A continuous loop for concurrently performing two tasks: (1) receiving
    // messages from `ws_stream` and broadcasting them, and (2) receiving
    // messages on `bcast_rx` and sending them to the client.
    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From client {addr:?} {text:?}");

                            if let Ok(parsed_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                                if parsed_msg.message_type == "register" {
                                    if let Some(name) = parsed_msg.data {
                                        my_username = name.clone();
                                        let mut users = active_users.lock().await;
                                        if !users.contains(&name) {
                                            users.push(name);
                                        }

                                        let users_msg = WebSocketMessage {
                                            message_type: "users".to_string(),
                                            data_array: Some(users.clone()),
                                            data: None,
                                        };
                                        let _ = bcast_tx.send(serde_json::to_string(&users_msg).unwrap());
                                    }
                                } else if parsed_msg.message_type == "message" {
                                    if let Some(isi_pesan) = parsed_msg.data {
                                        let msg_data_json = format!(r#"{{"from":"{}","message":"{}"}}"#, my_username, isi_pesan);

                                        let forward_msg = WebSocketMessage {
                                            message_type: "message".to_string(),
                                            data_array: None,
                                            data: Some(msg_data_json),
                                        };
                                        let _ = bcast_tx.send(serde_json::to_string(&forward_msg).unwrap());
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => break,
                }
            }
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }

    if !my_username.is_empty() {
        let mut users = active_users.lock().await;
        users.retain(|u| u != &my_username);
        let users_msg = WebSocketMessage {
            message_type: "users".to_string(),
            data_array: Some(users.clone()),
            data: None,
        };
        let _ = bcast_tx.send(serde_json::to_string(&users_msg).unwrap());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);
    let active_users = Arc::new(Mutex::new(Vec::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from Aaron's Computer {addr}");
        let bcast_tx = bcast_tx.clone();
        let active_users = Arc::clone(&active_users);

        tokio::spawn(async move {
            // Wrap the raw TCP stream into a websocket.
            if let Ok((_req, ws_stream)) = ServerBuilder::new().accept(socket).await {
                let _ = handle_connection(addr, ws_stream, bcast_tx, active_users).await;
            }
        });
    }
}