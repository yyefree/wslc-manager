mod api;
mod wslc;
mod ws;

use axum::{
    Router,
    routing::{get, post, delete},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use api::AppState;
use wslc::WslcClient;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(RwLock::new(AppState {
        client: WslcClient::new(),
    }));

    let api_routes = Router::new()
        .route("/system", get(api::get_system_info))
        .route("/containers", get(api::list_containers))
        .route("/containers/run", post(api::run_container))
        .route("/containers/{name}/stop", post(api::stop_container))
        .route("/containers/{name}/start", post(api::start_container))
        .route("/containers/{name}/restart", post(api::restart_container))
        .route("/containers/{name}/remove", delete(api::remove_container))
        .route("/containers/{name}/logs", get(api::get_container_logs))
        .route("/containers/{name}/exec", post(api::exec_container_command))
        .route("/containers/{name}/inspect", get(api::inspect_container))
        .route("/containers/{name}/stats", get(api::get_container_stats))
        .route("/containers/{name}/top", get(api::get_container_top))
        .route("/containers/{name}/commit", post(api::commit_container))
        .route("/containers/{name}/clone", post(api::clone_container))
        .route("/containers/{name}/export", post(api::export_container))
        .route("/containers/{name}/compose", get(api::export_compose))
        .route("/containers/{old_name}/rename/{new_name}", post(api::rename_container))
        .route("/containers/batch/stop", post(api::batch_stop))
        .route("/containers/batch/start", post(api::batch_start))
        .route("/containers/batch/remove", post(api::batch_remove))
        .route("/containers/batch/remove-stopped", post(api::remove_all_stopped))
        .route("/compose/import", post(api::import_compose))
        .route("/images", get(api::list_images))
        .route("/images/pull", post(api::pull_image))
        .route("/images/{image}/remove", delete(api::remove_image))
        .route("/volumes", get(api::list_volumes))
        .route("/volumes/create", post(api::create_volume))
        .route("/volumes/{name}/remove", delete(api::remove_volume))
        .route("/networks", get(api::list_networks))
        .route("/networks/create", post(api::create_network))
        .route("/networks/{name}/remove", delete(api::remove_network));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/ws", get(ws::ws_handler))
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("WSLC Manager starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
