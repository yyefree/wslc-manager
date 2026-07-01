use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<RwLock<AppState>>) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial state
    let state_read = state.read().await;
    if let Ok(info) = state_read.client.get_system_info() {
        let msg = serde_json::json!({
            "type": "system_info",
            "data": info
        });
        if let Err(e) = sender.send(Message::Text(msg.to_string().into())).await {
            tracing::warn!("Failed to send system_info: {}", e);
            return;
        }
    }

    if let Ok(containers) = state_read.client.list_containers(true) {
        let msg = serde_json::json!({
            "type": "containers",
            "data": containers
        });
        if let Err(e) = sender.send(Message::Text(msg.to_string().into())).await {
            tracing::warn!("Failed to send containers: {}", e);
            return;
        }
    }

    if let Ok(images) = state_read.client.list_images() {
        let msg = serde_json::json!({
            "type": "images",
            "data": images
        });
        if let Err(e) = sender.send(Message::Text(msg.to_string().into())).await {
            tracing::warn!("Failed to send images: {}", e);
            return;
        }
    }
    drop(state_read);

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(text) => {
                    if let Ok(req) = serde_json::from_str::<serde_json::Value>(&text) {
                        let response = handle_ws_message(&state, &req).await;
                        if let Ok(resp_str) = serde_json::to_string(&response) {
                            let _ = sender.send(Message::Text(resp_str.into())).await;
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        } else {
            break;
        }
    }
}

async fn handle_ws_message(
    state: &Arc<RwLock<AppState>>,
    req: &serde_json::Value,
) -> serde_json::Value {
    let cmd = req["cmd"].as_str().unwrap_or("");

    match cmd {
        "refresh" => {
            let state = state.read().await;
            let containers = state.client.list_containers(true).unwrap_or_default();
            let images = state.client.list_images().unwrap_or_default();
            let info = state.client.get_system_info().ok();

            serde_json::json!({
                "type": "refresh",
                "containers": containers,
                "images": images,
                "system_info": info
            })
        }
        "stop" => {
            let name = req["name"].as_str().unwrap_or("");
            let state = state.read().await;
            match state.client.stop_container(name) {
                Ok(_) => serde_json::json!({"type": "success", "message": format!("容器 {} 已停止", name)}),
                Err(e) => serde_json::json!({"type": "error", "message": e}),
            }
        }
        "start" => {
            let name = req["name"].as_str().unwrap_or("");
            let state = state.read().await;
            match state.client.start_container(name) {
                Ok(_) => serde_json::json!({"type": "success", "message": format!("容器 {} 已启动", name)}),
                Err(e) => serde_json::json!({"type": "error", "message": e}),
            }
        }
        "restart" => {
            let name = req["name"].as_str().unwrap_or("");
            let state = state.read().await;
            match state.client.restart_container(name) {
                Ok(_) => serde_json::json!({"type": "success", "message": format!("容器 {} 已重启", name)}),
                Err(e) => serde_json::json!({"type": "error", "message": e}),
            }
        }
        "remove" => {
            let name = req["name"].as_str().unwrap_or("");
            let state = state.read().await;
            match state.client.remove_container(name, true) {
                Ok(_) => serde_json::json!({"type": "success", "message": format!("容器 {} 已删除", name)}),
                Err(e) => serde_json::json!({"type": "error", "message": e}),
            }
        }
        "logs" => {
            let name = req["name"].as_str().unwrap_or("");
            let lines = req["lines"].as_u64().map(|n| n as u32);
            let state = state.read().await;
            match state.client.get_logs(name, lines, false) {
                Ok(logs) => serde_json::json!({"type": "logs", "name": name, "logs": logs}),
                Err(e) => serde_json::json!({"type": "error", "message": e}),
            }
        }
        "exec" => {
            let name = req["name"].as_str().unwrap_or("");
            let command = req["command"].as_str().unwrap_or("");
            let state = state.read().await;
            match state.client.exec_command(name, command) {
                Ok(output) => serde_json::json!({"type": "exec_output", "name": name, "output": output}),
                Err(e) => serde_json::json!({"type": "error", "message": e}),
            }
        }
        _ => serde_json::json!({"type": "error", "message": "Unknown command"}),
    }
}
