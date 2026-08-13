mod game;
mod navigation;
mod store;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use game::{Command, GameWorld, WorldSnapshot};
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
        sequence: u64,
        world: Box<WorldSnapshot>,
    },
    CommandResult {
        request_id: String,
        ok: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        applied_sequence: Option<u64>,
    },
}

impl ServerMessage {
    fn snapshot(sequence: u64, world: &GameWorld) -> Self {
        Self::Snapshot {
            sequence,
            world: Box::new(world.snapshot()),
        }
    }
}

struct AppState {
    world: Mutex<GameWorld>,
    snapshots: broadcast::Sender<ServerMessage>,
    next_snapshot_sequence: AtomicU64,
    store: Store,
}

impl AppState {
    fn snapshot(&self, world: &GameWorld) -> (u64, ServerMessage) {
        let sequence = self.next_snapshot_sequence.fetch_add(1, Ordering::Relaxed) + 1;
        (sequence, ServerMessage::snapshot(sequence, world))
    }

    fn publish_snapshot(&self, world: &GameWorld) -> u64 {
        let (sequence, message) = self.snapshot(world);
        let _ = self.snapshots.send(message);
        sequence
    }
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
        next_snapshot_sequence: AtomicU64::new(0),
        store,
    });
    tokio::spawn(game_loop(state.clone()));

    let app = Router::new()
        .route("/", get(index))
        .route("/state", get(get_state))
        .route("/reset", post(reset_world))
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
        let mut world = state.world.lock().await;
        let buildings = world.buildings.len();
        let active_units = world
            .units
            .iter()
            .filter(|unit| !matches!(unit.action, game::UnitAction::Idle))
            .count();
        let previous_tick = world.tick;
        world.tick(TICK_DURATION.as_secs_f64());
        let advanced = world.tick != previous_tick;
        let completed_action = world.buildings.len() != buildings
            || world
                .units
                .iter()
                .filter(|unit| !matches!(unit.action, game::UnitAction::Idle))
                .count()
                < active_units;
        if ((advanced && world.tick % SAVE_EVERY_TICKS == 0) || completed_action)
            && let Err(error) = state.store.save(&world)
        {
            tracing::error!(%error, "world save failed");
        }
        // Publish while holding the same lock that orders world mutations. This
        // prevents a command snapshot from overtaking a tick snapshot.
        state.publish_snapshot(&world);
    }
}

async fn index() -> Html<String> {
    match tokio::fs::read_to_string("frontend/index.html").await {
        Ok(content) => Html(content),
        Err(_) => Html("<h1>Age of Agents</h1><p>Frontend not found.</p>".into()),
    }
}

async fn get_state(State(state): State<SharedState>) -> Json<WorldSnapshot> {
    Json(state.world.lock().await.snapshot())
}

async fn reset_world(
    State(state): State<SharedState>,
) -> Result<Json<WorldSnapshot>, (StatusCode, String)> {
    let mut world = state.world.lock().await;
    let fresh = GameWorld::default();
    state.store.save(&fresh).map_err(|error| {
        tracing::error!(%error, "world reset could not be saved");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "world reset could not be saved".to_owned(),
        )
    })?;
    *world = fresh;
    state.publish_snapshot(&world);
    Ok(Json(world.snapshot()))
}

async fn websocket(ws: WebSocketUpgrade, State(state): State<SharedState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: SharedState) {
    let initial = {
        let world = state.world.lock().await;
        state.snapshot(&world).1
    };
    if send_message(&mut socket, &initial).await.is_err() {
        return;
    }

    let mut snapshots = state.snapshots.subscribe();
    loop {
        tokio::select! {
            // Drain already-published world states before accepting another command,
            // so a command result is never followed by an older queued snapshot.
            biased;
            snapshot = snapshots.recv() => {
                match snapshot {
                    Ok(message) => {
                        if send_message(&mut socket, &message).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Replace retained stale messages while holding the world lock.
                        // Every publisher holds this lock too, so anything queued after
                        // the resubscribe is newer than the synthesized current state.
                        let current = {
                            let world = state.world.lock().await;
                            snapshots = state.snapshots.subscribe();
                            state.snapshot(&world).1
                        };
                        if send_message(&mut socket, &current).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
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
        }
    }
}

async fn handle_client_message(state: &SharedState, text: &str) -> ServerMessage {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(ClientMessage::Command {
            request_id,
            command,
        }) => {
            let (result, applied_sequence) = {
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
                let applied_sequence = result.is_ok().then(|| state.publish_snapshot(&world));
                (result, applied_sequence)
            };
            ServerMessage::CommandResult {
                request_id,
                ok: result.is_ok(),
                error: result.err().map(|error| error.to_string()),
                applied_sequence,
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
                applied_sequence: None,
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
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[tokio::test]
    async fn reset_replaces_and_persists_the_default_world() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "age-of-agents-reset-{}-{nonce}.db",
            std::process::id()
        ));
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let mut modified = GameWorld::default();
        modified.stockpile.wood = 99.0;
        store.save(&modified).unwrap();
        let (snapshots, _) = broadcast::channel(4);
        let state = Arc::new(AppState {
            world: Mutex::new(modified),
            snapshots,
            next_snapshot_sequence: AtomicU64::new(0),
            store: store.clone(),
        });

        let Json(snapshot) = reset_world(State(state)).await.unwrap();

        assert_eq!(snapshot.tick, 0);
        assert_eq!(snapshot.stockpile.wood, 0.0);
        assert_eq!(store.load().unwrap(), Some(GameWorld::default()));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn typed_command_and_result_use_request_id() {
        let message: ClientMessage = serde_json::from_str(
            r#"{"type":"command","request_id":"request-7","command":{"type":"gather","unit_id":"villager-1","resource_id":"tree-1"}}"#,
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
            applied_sequence: Some(9),
        };
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["type"], "command_result");
        assert_eq!(json["request_id"], "request-7");
        assert_eq!(json["applied_sequence"], 9);
    }

    #[test]
    fn group_move_is_one_typed_command() {
        let message: ClientMessage = serde_json::from_str(
            r#"{"type":"command","request_id":"group-1","command":{"type":"group_move","unit_ids":["villager-1","villager-2"],"x":400,"y":500}}"#,
        ).unwrap();
        assert!(matches!(message, ClientMessage::Command {
            command: Command::GroupMove { unit_ids, .. }, ..
        } if unit_ids.len() == 2));
    }

    #[test]
    fn snapshots_serialize_typed_visibility_without_unseen_entities() {
        let world = GameWorld::default();
        let message = ServerMessage::snapshot(4, &world);
        let json = serde_json::to_value(message).unwrap();
        assert_eq!(json["type"], "snapshot");
        assert_eq!(json["sequence"], 4);
        assert_eq!(json["world"]["width"], 2400.0);
        let terrain = json["world"]["terrain"].as_array().unwrap();
        assert!(terrain.iter().all(|cell| matches!(
            cell["visibility"].as_str(),
            Some("unseen" | "explored" | "visible")
        )));
        assert!(terrain.iter().all(|cell| {
            let is_unseen = cell["visibility"] == "unseen";
            is_unseen == cell.get("biome").is_none()
        }));
        assert!(json["world"]["resources"].as_array().unwrap().len() < world.resources.len());
    }
}
