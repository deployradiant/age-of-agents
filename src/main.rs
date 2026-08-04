/// Age of Agents — Axum WebSocket game server with SQLite persistence.
///
/// Run locally: cargo run
/// Deploy:      modal deploy modal_app.py

mod db;
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
use rusqlite::Connection;
use tokio::sync::{Mutex, broadcast};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use state::*;
use game_loop::*;
use db::{open_db, load_world, should_save, background_save, save_world};

/// Application shared state.
struct AppState {
    world: WorldRef,
    broadcaster: broadcast::Sender<String>,
    db: Arc<Mutex<Option<Connection>>>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // Open SQLite database
    let db_path = std::env::var("AGE_OF_AGENTS_DB").unwrap_or_else(|_| "age_of_agents.db".to_string());
    let db_conn = db::open_db(Some(&db_path)).expect("Failed to open database");
    let db = Arc::new(Mutex::new(Some(db_conn)));

    // Load world from DB or create fresh
    let world = {
        let conn_guard = db.lock().await;
        let conn = conn_guard.as_ref().unwrap();
        match db::load_world(conn) {
            Ok(Some(w)) => {
                tracing::info!("📦 Loaded world from DB (tick {})", w.tick_count);
                Arc::new(Mutex::new(w))
            }
            _ => {
                tracing::info!("🌱 Creating new world");
                Arc::new(Mutex::new(create_default_world()))
            }
        }
    };

    let (broadcaster, _) = broadcast::channel::<String>(32);

    let state = Arc::new(AppState {
        world: world.clone(),
        broadcaster: broadcaster.clone(),
        db: db.clone(),
    });

    // Spawn the game loop
    let loop_world = world.clone();
    let loop_broadcaster = broadcaster.clone();
    let loop_db = db.clone();
    tokio::spawn(async move {
        game_loop_task(loop_world, loop_broadcaster, loop_db).await;
    });

    // Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/reset", post(reset_handler))
        .route("/state", get(state_handler))
        .route("/save", post(save_handler))
                .route("/ws", get(ws_handler))
                .nest_service("/assets", ServeDir::new("assets"))
                .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:8000";
    tracing::info!("🚀 Age of Agents starting on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ── Game loop task ──────────────────────────────────────────────────────

async fn game_loop_task(
    world: WorldRef,
    broadcaster: broadcast::Sender<String>,
    db: Arc<Mutex<Option<Connection>>>,
) {
    let tick_interval = std::time::Duration::from_secs_f64(1.0 / TICK_RATE);
    let mut tick_start = Instant::now();
    let mut last_save_tick: u64 = 0;

    loop {
        let dt = tick_interval.as_secs_f64();

        let state_msg = {
            let mut w = world.lock().await;
            let events = tick_world(&mut w, dt);
            let msg = w.serialize_state(events);

            // Auto-save to DB periodically
            if db::should_save(w.tick_count) && w.tick_count != last_save_tick {
                last_save_tick = w.tick_count;
                if let Ok(json) = serde_json::to_string(&*w) {
                    db::background_save("age_of_agents.db".to_string(), json);
                    tracing::info!("💾 Auto-saved world at tick {}", w.tick_count);
                }
            }

            serde_json::to_string(&msg).unwrap_or_default()
        };

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
    let paths = [
        "frontend/index.html",
        "../frontend/index.html",
        "/root/frontend/index.html",
        "/app/frontend/index.html",
    ];
    for path in &paths {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            return Html(content);
        }
    }
    Html(r#"<html><body><h1>Age of Agents</h1><p>Frontend not found.</p></body></html>"#.to_string())
}

async fn reset_handler(State(state): State<Arc<AppState>>) -> &'static str {
    let mut w = state.world.lock().await;
    *w = create_default_world();
    "{\"status\":\"ok\"}"
}

async fn save_handler(State(state): State<Arc<AppState>>) -> String {
    let w = state.world.lock().await;
    let db_guard = state.db.lock().await;
    if let Some(ref conn) = *db_guard {
        match db::save_world(conn, &w) {
            Ok(()) => format!(r#"{{"status":"ok","tick":{}}}"#, w.tick_count),
            Err(e) => format!(r#"{{"status":"error","message":"{e}"}}"#),
        }
    } else {
        r#"{"status":"error","message":"database not available"}"#.to_string()
    }
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
    let mut rx = state.broadcaster.subscribe();

    // Send initial state
    {
        let w = state.world.lock().await;
        let msg = w.serialize_state(Vec::new());
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    // Forward broadcast messages to this client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Receive messages from client (player commands)
    let recv_state = state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Parse player command
                    if let Ok(cmd) = serde_json::from_str::<state::PlayerCommand>(&text) {
                        if cmd.msg_type == "command" {
                            let response = {
                                let mut w = recv_state.world.lock().await;
                                w.apply_command(&cmd)
                            };
                            tracing::info!("📜 Command: {} → {}", cmd.command, &response[..50.min(response.len())]);
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}