mod game;
mod store;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use game::{Command, GameWorld};
use serde::{Deserialize, Serialize};
use store::Store;
use tokio::sync::{Mutex, broadcast};
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

const TICK_DURATION: Duration = Duration::from_millis(100);
const SAVE_EVERY_TICKS: u64 = 10;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ClientMessage {
    Command {
        request_id: String,
        command: Command,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    Snapshot {
        world: GameWorld,
    },
    CommandResult {
        request_id: String,
        ok: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
}

impl ServerMessage {
    fn snapshot(world: &GameWorld) -> Self {
        Self::Snapshot {
            world: world.clone(),
        }
    }
}

struct AppState {
    world: Mutex<GameWorld>,
    snapshots: broadcast::Sender<ServerMessage>,
    store: Store,
}

type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let store = Store::configured();
    store.initialize()?;
    let world = store.load()?.unwrap_or_default();
    tracing::info!(database = %store.path().display(), "world store ready");

    let (snapshots, _) = broadcast::channel(32);
    let state = Arc::new(AppState {
        world: Mutex::new(world),
        snapshots,
        store,
    });
    tokio::spawn(game_loop(state.clone()));

    let app = Router::new()
        .route("/", get(index))
        .route("/state", get(get_state))
        .route("/ws", get(websocket))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/frontend", ServeDir::new("frontend"))
        .with_state(state);

    let address = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn game_loop(state: SharedState) {
    let mut interval = tokio::time::interval(TICK_DURATION);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        let message = {
            let mut world = state.world.lock().await;
            let buildings = world.buildings.len();
            let active_units = world
                .units
                .iter()
                .filter(|unit| !matches!(unit.action, game::UnitAction::Idle))
                .count();
            world.tick(TICK_DURATION.as_secs_f64());
            let completed_action = world.buildings.len() != buildings
                || world
                    .units
                    .iter()
                    .filter(|unit| !matches!(unit.action, game::UnitAction::Idle))
                    .count()
                    < active_units;
            if (world.tick % SAVE_EVERY_TICKS == 0 || completed_action)
                && let Err(error) = state.store.save(&world)
            {
                tracing::error!(%error, "world save failed");
            }
            ServerMessage::snapshot(&world)
        };
        let _ = state.snapshots.send(message);
    }
}

async fn index() -> Html<String> {
    match tokio::fs::read_to_string("frontend/index.html").await {
        Ok(content) => Html(content),
        Err(_) => Html("<h1>Age of Agents</h1><p>Frontend not found.</p>".into()),
    }
}

async fn get_state(State(state): State<SharedState>) -> Json<GameWorld> {
    Json(state.world.lock().await.clone())
}

async fn websocket(ws: WebSocketUpgrade, State(state): State<SharedState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: SharedState) {
    let initial = {
        let world = state.world.lock().await;
        ServerMessage::snapshot(&world)
    };
    if send_message(&mut socket, &initial).await.is_err() {
        return;
    }

    let mut snapshots = state.snapshots.subscribe();
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(Ok(message)) = incoming else { break };
                match message {
                    Message::Text(text) => {
                        let response = handle_client_message(&state, &text).await;
                        if send_message(&mut socket, &response).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            snapshot = snapshots.recv() => {
                match snapshot {
                    Ok(message) => {
                        if send_message(&mut socket, &message).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        let current = {
                            let world = state.world.lock().await;
                            ServerMessage::snapshot(&world)
                        };
                        if send_message(&mut socket, &current).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

async fn handle_client_message(state: &SharedState, text: &str) -> ServerMessage {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(ClientMessage::Command {
            request_id,
            command,
        }) => {
            let (result, snapshot) = {
                let mut world = state.world.lock().await;
                let mut candidate = world.clone();
                let result = candidate
                    .apply_command(command)
                    .map_err(|error| error.to_string())
                    .and_then(|()| {
                        state.store.save(&candidate).map_err(|error| {
                            tracing::error!(%error, "accepted command could not be saved");
                            "command could not be saved".to_owned()
                        })?;
                        *world = candidate;
                        Ok(())
                    });
                (result, ServerMessage::snapshot(&world))
            };
            if result.is_ok() {
                let _ = state.snapshots.send(snapshot);
            }
            ServerMessage::CommandResult {
                request_id,
                ok: result.is_ok(),
                error: result.err().map(|error| error.to_string()),
            }
        }
        Err(error) => {
            let request_id = serde_json::from_str::<serde_json::Value>(text)
                .ok()
                .and_then(|value| value.get("request_id")?.as_str().map(str::to_owned))
                .unwrap_or_default();
            ServerMessage::CommandResult {
                request_id,
                ok: false,
                error: Some(format!("invalid command: {error}")),
            }
        }
    }
}

async fn send_message(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), axum::Error> {
    let json = serde_json::to_string(message).expect("server messages are serializable");
    socket.send(Message::Text(json.into())).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_command_and_result_use_request_id() {
        let message: ClientMessage = serde_json::from_str(
            r#"{"type":"command","request_id":"request-7","command":{"type":"gather","unit_id":"villager-1","tree_id":"tree-1"}}"#,
        )
        .unwrap();
        assert!(matches!(
            message,
            ClientMessage::Command { request_id, .. } if request_id == "request-7"
        ));

        let result = ServerMessage::CommandResult {
            request_id: "request-7".into(),
            ok: true,
            error: None,
        };
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["type"], "command_result");
        assert_eq!(json["request_id"], "request-7");
    }
}
