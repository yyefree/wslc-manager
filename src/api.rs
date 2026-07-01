use axum::{
    Json,
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::wslc::WslcClient;

pub struct AppState {
    pub client: WslcClient,
}

#[derive(Deserialize)]
pub struct RunContainerRequest {
    pub image: String,
    pub name: Option<String>,
    pub ports: Option<String>,
    pub volumes: Option<String>,
    pub envs: Option<Vec<String>>,
    pub command: Option<String>,
}

#[derive(Deserialize)]
pub struct LogQuery {
    pub lines: Option<u32>,
    pub follow: Option<bool>,
}

#[derive(Deserialize)]
pub struct BatchRequest {
    pub names: Vec<String>,
}

#[derive(Deserialize)]
pub struct ComposeRequest {
    pub name: String,
    pub services: Vec<ComposeService>,
}

#[derive(Deserialize)]
pub struct ComposeService {
    pub name: String,
    pub image: String,
    pub ports: Option<String>,
    pub volumes: Option<String>,
    pub envs: Option<Vec<String>>,
    pub command: Option<String>,
}

pub async fn get_system_info(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.get_system_info() {
        Ok(info) => Json(info).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn list_containers(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.list_containers(true) {
        Ok(containers) => Json(containers).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn list_images(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.list_images() {
        Ok(images) => Json(images).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn list_volumes(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.list_volumes() {
        Ok(volumes) => Json(volumes).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn list_networks(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.list_networks() {
        Ok(networks) => Json(networks).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn inspect_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.inspect_container(&name) {
        Ok(detail) => Json(detail).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn run_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(req): Json<RunContainerRequest>,
) -> impl IntoResponse {
    if req.image.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "镜像名称不能为空"}))).into_response();
    }
    let state = state.read().await;
    match state.client.run_container(
        &req.image,
        req.name.as_deref(),
        req.ports.as_deref(),
        req.volumes.as_deref(),
        req.envs,
        true,
        req.command.as_deref(),
    ) {
        Ok(output) => Json(serde_json::json!({"success": true, "output": output})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn stop_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.stop_container(&name) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn start_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.start_container(&name) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn restart_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.restart_container(&name) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn remove_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let force = params.get("force").map(|v| v == "true").unwrap_or(false);
    let state = state.read().await;
    match state.client.remove_container(&name, force) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn get_container_logs(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.get_logs(&name, query.lines, query.follow.unwrap_or(false)) {
        Ok(logs) => Json(serde_json::json!({"logs": logs})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn exec_container_command(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let command = req["command"].as_str().unwrap_or("echo 'No command'");
    let state = state.read().await;
    match state.client.exec_command(&name, command) {
        Ok(output) => Json(serde_json::json!({"output": output})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn pull_image(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let image = req["image"].as_str().unwrap_or("").trim();
    if image.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "镜像名称不能为空"}))).into_response();
    }
    let state = state.read().await;
    match state.client.pull_image(image) {
        Ok(output) => Json(serde_json::json!({"success": true, "output": output})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn remove_image(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(image): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.remove_image(&image, false) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn create_volume(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let name = req["name"].as_str().unwrap_or("").trim();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "卷名称不能为空"}))).into_response();
    }
    let state = state.read().await;
    match state.client.create_volume(name) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn remove_volume(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.remove_volume(&name) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn create_network(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let name = req["name"].as_str().unwrap_or("").trim();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "网络名称不能为空"}))).into_response();
    }
    let driver = req["driver"].as_str();
    let state = state.read().await;
    match state.client.create_network(name, driver) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn remove_network(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.remove_network(&name) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn get_container_stats(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.get_container_stats(&name) {
        Ok(stats) => Json(serde_json::json!({"stats": stats})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn get_container_top(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.get_container_top(&name) {
        Ok(output) => Json(serde_json::json!({"output": output})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn commit_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let image = req["image"].as_str().unwrap_or("").trim();
    if image.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "镜像名称不能为空"}))).into_response();
    }
    let state = state.read().await;
    match state.client.commit_container(&name, image) {
        Ok(output) => Json(serde_json::json!({"success": true, "output": output})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn rename_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path((old_name, new_name)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.rename_container(&old_name, &new_name) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn export_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let output = req["output"].as_str().unwrap_or("").trim();
    if output.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "导出路径不能为空"}))).into_response();
    }
    let state = state.read().await;
    match state.client.export_container(&name, output) {
        Ok(_) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn clone_container(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let new_name = req["name"].as_str().unwrap_or("").trim();
    if new_name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "新容器名称不能为空"}))).into_response();
    }
    let state = state.read().await;
    
    // Get container info first
    match state.client.inspect_container(&name) {
        Ok(detail) => {
            // Commit container as new image
            let temp_image = format!("temp-clone-{}", name);
            if let Err(e) = state.client.commit_container(&name, &temp_image) {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response();
            }
            
            // Run new container from committed image
            let ports = detail.ports.first().map(|p| format!("{}:{}", p.host_port, p.container_port));
            match state.client.run_container(&temp_image, Some(new_name), ports.as_deref(), None, None, true, None) {
                Ok(_) => {
                    // Clean up temp image
                    let _ = state.client.remove_image(&temp_image, true);
                    Json(serde_json::json!({"success": true})).into_response()
                }
                Err(e) => {
                    let _ = state.client.remove_image(&temp_image, true);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response()
                }
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn batch_stop(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(req): Json<BatchRequest>,
) -> impl IntoResponse {
    let state = state.read().await;
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    for name in &req.names {
        match state.client.stop_container(name) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", name, e));
            }
        }
    }
    
    Json(serde_json::json!({
        "success": true,
        "stopped": success,
        "failed": failed,
        "errors": errors
    })).into_response()
}

pub async fn batch_start(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(req): Json<BatchRequest>,
) -> impl IntoResponse {
    let state = state.read().await;
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    for name in &req.names {
        match state.client.start_container(name) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", name, e));
            }
        }
    }
    
    Json(serde_json::json!({
        "success": true,
        "started": success,
        "failed": failed,
        "errors": errors
    })).into_response()
}

pub async fn batch_remove(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(req): Json<BatchRequest>,
) -> impl IntoResponse {
    let state = state.read().await;
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    for name in &req.names {
        match state.client.remove_container(name, true) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", name, e));
            }
        }
    }
    
    Json(serde_json::json!({
        "success": true,
        "removed": success,
        "failed": failed,
        "errors": errors
    })).into_response()
}

pub async fn remove_all_stopped(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    let containers = state.client.list_containers(true).unwrap_or_default();
    let stopped: Vec<String> = containers.iter()
        .filter(|c| !c.is_running)
        .map(|c| c.name.clone())
        .collect();
    
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    for name in &stopped {
        match state.client.remove_container(name, true) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", name, e));
            }
        }
    }
    
    Json(serde_json::json!({
        "success": true,
        "removed": success,
        "failed": failed,
        "errors": errors
    })).into_response()
}

pub async fn export_compose(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.client.inspect_container(&name) {
        Ok(detail) => {
            let mut yaml = String::from("version: '3.8'\nservices:\n");
            yaml.push_str(&format!("  {}:\n", name));
            yaml.push_str(&format!("    image: {}\n", detail.image));
            if !detail.cmd.is_empty() {
                yaml.push_str(&format!("    command: {}\n", detail.cmd.join(" ")));
            }
            for port in &detail.ports {
                yaml.push_str(&format!("    ports:\n      - \"{}:{}\"\n", port.host_port, port.container_port));
            }
            for env in &detail.env {
                yaml.push_str(&format!("    environment:\n      - {}\n", env));
            }
            for mount in &detail.mounts {
                yaml.push_str(&format!("    volumes:\n      - \"{}:{}\"\n", mount.source, mount.destination));
            }
            Json(serde_json::json!({"success": true, "compose": yaml})).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))).into_response(),
    }
}

pub async fn import_compose(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(req): Json<ComposeRequest>,
) -> impl IntoResponse {
    if req.name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "项目名称不能为空"}))).into_response();
    }
    
    let state = state.read().await;
    let mut results = Vec::new();
    
    for service in &req.services {
        let container_name = format!("{}-{}", req.name, service.name);
        match state.client.run_container(
            &service.image,
            Some(&container_name),
            service.ports.as_deref(),
            service.volumes.as_deref(),
            service.envs.clone(),
            true,
            service.command.as_deref(),
        ) {
            Ok(_) => results.push(serde_json::json!({"service": service.name, "container": container_name, "status": "started"})),
            Err(e) => results.push(serde_json::json!({"service": service.name, "error": e, "status": "failed"})),
        }
    }
    
    Json(serde_json::json!({"success": true, "results": results})).into_response()
}
