use axum::{extract::Json, response::IntoResponse, routing::post, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[derive(Deserialize, Serialize)]
struct UdpMessage {
    header: u8,
    client_identifier: u64,
    session_id: u32,
    player_ids: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct UdpResponse {
    status: String,
    response: String,
}

async fn send_udp_message(Json(payload): Json<UdpMessage>) -> impl IntoResponse {
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();

    let header = payload.header;
    let client_identifier = payload.client_identifier;
    let session_id = payload.session_id;
    let player_ids = payload.player_ids;

    let mut buffer = Vec::new();

    buffer.extend_from_slice(&header.to_le_bytes());
    buffer.extend_from_slice(&client_identifier.to_le_bytes());
    buffer.extend_from_slice(&session_id.to_le_bytes());
    buffer.extend_from_slice(&(player_ids.len() as u16).to_le_bytes());
    for player_id in player_ids {
        let mut fixed_length_id = [0u8; 16];
        let bytes = player_id.as_bytes();
        let copy_len = bytes.len().min(16);
        fixed_length_id[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buffer.extend_from_slice(&fixed_length_id);
    }

    // let address = SocketAddr::from(([127, 0, 0, 1], 5001));
    let address = SocketAddr::from(([192, 168, 1, 151], 5001));
    // let address = SocketAddr::from(([176, 40, 120, 89], 5000));
    match socket.send_to(&buffer, address).await {
        Ok(_) => Json(UdpResponse {
            status: "success".to_string(),
            response: vec_u8_to_byte_array_string(&buffer),
        }),
        Err(e) => Json(UdpResponse {
            status: "error".to_string(),
            response: format!("Failed to send message: {}", e),
        }),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/create_session", post(send_udp_message));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn vec_u8_to_byte_array_string(vec: &Vec<u8>) -> String {
    let bytes_str: Vec<String> = vec.iter().map(|b| b.to_string()).collect();
    format!("[{}]", bytes_str.join(", "))
}
