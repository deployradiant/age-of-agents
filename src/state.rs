/// Game world state — agents, resources, buildings, and actions.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rand::Rng;

pub type WorldRef = Arc<Mutex<GameWorld>>;

// ── Player commands (from WebSocket) ─────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PlayerCommand {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub command: String,
    pub agent_id: Option<String>,
    pub resource_id: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub name: Option<String>,
}

// ── Position ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dist(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

// ── Resources ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResourceKind {
    #[serde(rename = "wood")]
    Wood,
    #[serde(rename = "gold")]
    Gold,
    #[serde(rename = "food")]
    Food,
    #[serde(rename = "stone")]
    Stone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: String,
    pub kind: ResourceKind,
    pub position: Point,
    pub amount: f64,
    pub max_amount: f64,
}

impl ResourceNode {
    pub fn alive(&self) -> bool {
        self.amount > 0.0
    }
}

// ── Buildings ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub kind: String, // "town_center", "barracks", etc.
    pub position: Point,
    pub health: f64,
    pub owner: String,
}

// ── Agent state machine ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "thinking")]
    Thinking,
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "dead")]
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    #[serde(rename = "move_to")]
    MoveTo,
    #[serde(rename = "gather")]
    Gather,
    #[serde(rename = "build")]
    Build,
    #[serde(rename = "attack")]
    Attack,
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "wander")]
    Wander,
    #[serde(rename = "camp")]
    Camp,
    #[serde(rename = "scout")]
    Scout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedAction {
    pub action_type: ActionType,
    pub target_id: Option<String>,
    pub target_position: Option<Point>,
    pub progress: f64,
    pub duration_seconds: f64,
}

impl QueuedAction {
    pub fn new(
        action_type: ActionType,
        target_id: Option<String>,
        target_position: Option<Point>,
        duration_seconds: f64,
    ) -> Self {
        Self {
            action_type,
            target_id,
            target_position,
            progress: 0.0,
            duration_seconds,
        }
    }
}

// ── Agent ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub position: Point,
    pub state: AgentState,
    pub speed: f64,
    pub health: f64,
    pub wood: f64,
    pub gold: f64,
    pub stone: f64,
    pub food: f64,
    pub color: String,
    pub current_action: Option<Box<QueuedAction>>,
    pub action_queue: Vec<QueuedAction>,
}

impl Agent {
    pub fn carry_weight(&self) -> f64 {
        self.wood + self.gold + self.stone + self.food
    }
}

// ── Game world ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWorld {
    pub width: f64,
    pub height: f64,
    pub agents: HashMap<String, Agent>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<Building>,
    pub tick_count: u64,
    pub time_elapsed: f64,
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            agents: HashMap::new(),
            resources: Vec::new(),
            buildings: Vec::new(),
            tick_count: 0,
            time_elapsed: 0.0,
        }
    }
}

// ── Serialized state for WebSocket broadcast ────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SerializedAgent {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub color: String,
    pub health: f64,
    pub state: String,
    pub wood: f64,
    pub gold: f64,
    pub stone: f64,
    pub food: f64,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SerializedResource {
    pub id: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SerializedBuilding {
    pub id: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub health: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub agent_id: Option<String>,
    pub resource: Option<String>,
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldStateMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub tick: u64,
    pub time: f64,
    pub agents: HashMap<String, SerializedAgent>,
    pub resources: Vec<SerializedResource>,
    pub buildings: Vec<SerializedBuilding>,
    pub events: Vec<WorldEvent>,
}

// ── World creation ──────────────────────────────────────────────────────

pub fn create_default_world() -> GameWorld {
    let mut world = GameWorld::new();

    let mut rng = rand::thread_rng();

    // Scatter resources
    for _ in 0..20 {
        world.resources.push(ResourceNode {
            id: Uuid::new_v4().to_string()[..8].to_string(),
            kind: ResourceKind::Wood,
            position: Point::new(rng.gen_range(50.0..1150.0), rng.gen_range(50.0..750.0)),
            amount: rng.gen_range(50.0..150.0),
            max_amount: 100.0,
        });
    }
    for _ in 0..8 {
        world.resources.push(ResourceNode {
            id: Uuid::new_v4().to_string()[..8].to_string(),
            kind: ResourceKind::Gold,
            position: Point::new(rng.gen_range(50.0..1150.0), rng.gen_range(50.0..750.0)),
            amount: rng.gen_range(30.0..80.0),
            max_amount: 60.0,
        });
    }
    for _ in 0..12 {
        world.resources.push(ResourceNode {
            id: Uuid::new_v4().to_string()[..8].to_string(),
            kind: ResourceKind::Food,
            position: Point::new(rng.gen_range(50.0..1150.0), rng.gen_range(50.0..750.0)),
            amount: rng.gen_range(40.0..120.0),
            max_amount: 80.0,
        });
    }

    // Spawn agents
    let agent_data = vec![
        ("aldric", "Aldric", Point::new(100.0, 400.0), "#4488ff"),
        ("brom", "Brom", Point::new(180.0, 400.0), "#ff4488"),
        ("cedric", "Cedric", Point::new(260.0, 400.0), "#44ff88"),
        ("doran", "Doran", Point::new(340.0, 400.0), "#ffaa44"),
        ("elara", "Elara", Point::new(420.0, 400.0), "#aa44ff"),
    ];

    for (id, name, pos, color) in agent_data {
        world.agents.insert(
            id.to_string(),
            Agent {
                id: id.to_string(),
                name: name.to_string(),
                position: pos,
                state: AgentState::Idle,
                speed: 50.0,
                health: 100.0,
                wood: 0.0,
                gold: 0.0,
                stone: 0.0,
                food: 0.0,
                color: color.to_string(),
                current_action: None,
                action_queue: Vec::new(),
            },
        );
    }

    // Starting town center
    world.buildings.push(Building {
        id: Uuid::new_v4().to_string()[..8].to_string(),
        kind: "town_center".to_string(),
        position: Point::new(600.0, 400.0),
        health: 100.0,
        owner: "aldric".to_string(),
    });

    world
}

// ── Serialization helpers ───────────────────────────────────────────────

impl GameWorld {
    /// Apply a player command (from WebSocket) to the world.
    pub fn apply_command(&mut self, cmd: &PlayerCommand) -> String {
        match cmd.command.as_str() {
            "move_to" => {
                let agent_id = match &cmd.agent_id {
                    Some(id) => id.clone(),
                    None => return r#"{"status":"error","message":"missing agent_id"}"#.to_string(),
                };
                let x = match cmd.x {
                    Some(v) => v,
                    None => return r#"{"status":"error","message":"missing x"}"#.to_string(),
                };
                let y = match cmd.y {
                    Some(v) => v,
                    None => return r#"{"status":"error","message":"missing y"}"#.to_string(),
                };
                if let Some(agent) = self.agents.get_mut(&agent_id) {
                    let travel_time = agent.position.dist(&Point::new(x, y)) / agent.speed;
                    agent.current_action = Some(Box::new(QueuedAction::new(
                        ActionType::MoveTo,
                        None,
                        Some(Point::new(x, y)),
                        travel_time + 1.0,
                    )));
                    agent.state = AgentState::Active;
                    format!(r#"{{"status":"ok","agent":"{agent_id}","command":"move_to"}}"#)
                } else {
                    format!(r#"{{"status":"error","message":"agent {agent_id} not found"}}"#)
                }
            }
            "gather" => {
                let agent_id = match &cmd.agent_id {
                    Some(id) => id.clone(),
                    None => return r#"{"status":"error","message":"missing agent_id"}"#.to_string(),
                };
                let resource_id = match &cmd.resource_id {
                    Some(id) => id.clone(),
                    None => return r#"{"status":"error","message":"missing resource_id"}"#.to_string(),
                };
                if let Some(agent) = self.agents.get_mut(&agent_id) {
                    if let Some(res) = self.resources.iter().find(|r| r.id == resource_id && r.alive()) {
                        let travel_time = agent.position.dist(&res.position) / agent.speed;
                        agent.current_action = Some(Box::new(QueuedAction::new(
                            ActionType::Gather,
                            Some(resource_id),
                            None,
                            travel_time + 8.0,
                        )));
                        agent.state = AgentState::Active;
                        format!(r#"{{"status":"ok","agent":"{agent_id}","command":"gather"}}"#)
                    } else {
                        format!(r#"{{"status":"error","message":"resource {resource_id} not found or depleted"}}"#)
                    }
                } else {
                    format!(r#"{{"status":"error","message":"agent {agent_id} not found"}}"#)
                }
            }
            _ => format!(r#"{{"status":"error","message":"unknown command: {}"}}"#, cmd.command),
        }
    }

    pub fn serialize_state(&self, events: Vec<WorldEvent>) -> WorldStateMessage {
        let mut agents = HashMap::new();
        for (id, a) in &self.agents {
            agents.insert(
                id.clone(),
                SerializedAgent {
                    id: a.id.clone(),
                    name: a.name.clone(),
                    x: (a.position.x * 10.0).round() / 10.0,
                    y: (a.position.y * 10.0).round() / 10.0,
                    color: a.color.clone(),
                    health: (a.health * 10.0).round() / 10.0,
                    state: match a.state {
                        AgentState::Active => "active".to_string(),
                        AgentState::Thinking => "thinking".to_string(),
                        AgentState::Idle => "idle".to_string(),
                        AgentState::Dead => "dead".to_string(),
                    },
                    wood: (a.wood * 10.0).round() / 10.0,
                    gold: (a.gold * 10.0).round() / 10.0,
                    stone: (a.stone * 10.0).round() / 10.0,
                    food: (a.food * 10.0).round() / 10.0,
                    action: a
                        .current_action
                        .as_ref()
                        .map(|act| format!("{:?}", act.action_type).to_lowercase()),
                },
            );
        }

        let resources = self
            .resources
            .iter()
            .filter(|r| r.alive())
            .map(|r| SerializedResource {
                id: r.id.clone(),
                kind: format!("{:?}", r.kind).to_lowercase(),
                x: (r.position.x * 10.0).round() / 10.0,
                y: (r.position.y * 10.0).round() / 10.0,
                amount: (r.amount * 10.0).round() / 10.0,
            })
            .collect();

        let buildings = self
            .buildings
            .iter()
            .map(|b| SerializedBuilding {
                id: b.id.clone(),
                kind: b.kind.clone(),
                x: (b.position.x * 10.0).round() / 10.0,
                y: (b.position.y * 10.0).round() / 10.0,
                health: (b.health * 10.0).round() / 10.0,
            })
            .collect();

        WorldStateMessage {
            msg_type: "state".to_string(),
            tick: self.tick_count,
            time: (self.time_elapsed * 10.0).round() / 10.0,
            agents,
            resources,
            buildings,
            events,
        }
    }
}