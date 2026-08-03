/// Age of Agents — Axum WebSocket game server.
///
/// Run locally: cargo run
/// Deploy:      via Modal (see Dockerfile + modal_app.py)

mod game_loop;
mod state;

use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    http::Method,
    response::Html,
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{Mutex, broadcast};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use state::*;
use game_loop::*;

/// Application shared state.
struct AppState {
    world: WorldRef,
    broadcaster: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // Create world and broadcast channel
    let world = Arc::new(Mutex::new(create_default_world()));
    let (broadcaster, _) = broadcast::channel::<String>(32);

    let state = Arc::new(AppState {
        world: world.clone(),
        broadcaster: broadcaster.clone(),
    });

    // Spawn the game loop
    let loop_world = world.clone();
    let loop_broadcaster = broadcaster.clone();
    tokio::spawn(async move {
        game_loop_task(loop_world, loop_broadcaster).await;
    });

    // Build router with CORS for local dev
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/reset", post(reset_handler))
        .route("/state", get(state_handler))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:8000";
    tracing::info!("🚀 Age of Agents starting on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ── Game loop task ──────────────────────────────────────────────────────

async fn game_loop_task(world: WorldRef, broadcaster: broadcast::Sender<String>) {
    let tick_interval = std::time::Duration::from_secs_f64(1.0 / TICK_RATE);
    let mut tick_start = Instant::now();

    loop {
        let dt = tick_interval.as_secs_f64();

        let state_msg = {
            let mut w = world.lock().await;
            let events = tick_world(&mut w, dt);
            let msg = w.serialize_state(events);
            serde_json::to_string(&msg).unwrap_or_default()
        };

        // Broadcast to all WebSocket clients (ignore errors from no receivers)
        let _ = broadcaster.send(state_msg);

        let elapsed = tick_start.elapsed();
        if elapsed < tick_interval {
            tokio::time::sleep(tick_interval - elapsed).await;
        }
        tick_start = Instant::now();
    }
}

// ── Handlers ────────────────────────────────────────────────────────────

async fn index_handler(State(_state): State<Arc<AppState>>) -> Html<String> {
    // Try multiple locations for the frontend HTML
    let paths = [
        "frontend/index.html",
        "../frontend/index.html",
        "/root/frontend/index.html",
    ];
    for path in &paths {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            return Html(content);
        }
    }

    // Embedded fallback — minimal page if file not found
    Html(
        r#"<html><body><h1>Age of Agents</h1><p>Frontend not found.</p></body></html>"#
            .to_string(),
    )
}

async fn reset_handler(State(state): State<Arc<AppState>>) -> &'static str {
    let mut w = state.world.lock().await;
    *w = create_default_world();
    "{\"status\":\"ok\"}"
}

async fn state_handler(State(state): State<Arc<AppState>>) -> String {
    let w = state.world.lock().await;
    let msg = w.serialize_state(Vec::new());
    serde_json::to_string(&msg).unwrap_or_default()
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

// ── WebSocket handler ───────────────────────────────────────────────────

async fn handle_ws(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to game state broadcasts
    let mut rx = state.broadcaster.subscribe();

    // Send initial state immediately
    {
        let w = state.world.lock().await;
        let msg = w.serialize_state(Vec::new());
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    // Task: forward broadcast messages to this client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task: receive messages from client (ping/pong, future commands)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
            // Future: handle client commands here
        }
    });

    // Wait for either task to finish (client disconnect)
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}