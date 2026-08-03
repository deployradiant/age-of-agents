/// Game loop — 2 Hz tick with agent state machine.

use rand::Rng;
use uuid::Uuid;

use crate::state::*;

pub const TICK_RATE: f64 = 2.0;

/// Advance the world by dt seconds. Returns events for broadcast.
pub fn tick_world(world: &mut GameWorld, dt: f64) -> Vec<WorldEvent> {
    world.tick_count += 1;
    world.time_elapsed += dt;
    let mut events: Vec<WorldEvent> = Vec::new();

    let agent_ids: Vec<String> = world.agents.keys().cloned().collect();
    for id in agent_ids {
        let is_dead = world
            .agents
            .get(&id)
            .is_some_and(|a| a.state == AgentState::Dead);
        if is_dead {
            continue;
        }
        tick_agent(world, &id, dt, &mut events);
    }

    // Gentle resource regen
    for r in &mut world.resources {
        if r.amount < r.max_amount {
            r.amount = (r.max_amount).min(r.amount + 0.1 * dt);
        }
    }

    events
}

fn tick_agent(world: &mut GameWorld, agent_id: &str, dt: f64, events: &mut Vec<WorldEvent>) {
    // Check state — bail if THINKING or DEAD
    {
        let agent = world.agents.get(agent_id).unwrap();
        if agent.state == AgentState::Dead || agent.state == AgentState::Thinking {
            return;
        }
    }

    // IDLE → choose an action
    if world.agents[agent_id].state == AgentState::Idle {
        choose_next_action(world, agent_id);
        let has_action = world.agents[agent_id].current_action.is_some();
        if !has_action {
            return;
        }
        // Set progress and go ACTIVE
        let agent = world.agents.get_mut(agent_id).unwrap();
        if let Some(ref mut action) = agent.current_action {
            action.progress = 0.0;
        }
        agent.state = AgentState::Active;
    }

    // ACTIVE → tick the current action
    if world.agents[agent_id].state == AgentState::Active {
        // Execute the current action logic
        run_execute_action(world, agent_id, dt, events);

        // Advance progress and check completion
        let finished = {
            let agent = world.agents.get_mut(agent_id).unwrap();
            if let Some(ref mut action) = agent.current_action {
                action.progress += dt;
                action.progress >= action.duration_seconds
            } else {
                true
            }
        };

        if finished {
            world.agents.get_mut(agent_id).unwrap().current_action = None;
            world.agents.get_mut(agent_id).unwrap().state = AgentState::Idle;
        }
    }
}

/// After tick_agent advances progress, execute_action is called separately
/// with the action already borrowed from world. This avoids NLL issues.
pub fn run_execute_action(world: &mut GameWorld, agent_id: &str, dt: f64, events: &mut Vec<WorldEvent>) {
    // Only execute when agent is Active and has an action
    let action_type = {
        let agent = match world.agents.get(agent_id) {
            Some(a) => a,
            None => return,
        };
        if agent.state != AgentState::Active {
            return;
        }
        let action = match &agent.current_action {
            Some(a) => a.action_type,
            None => return,
        };
        action
    };

    match action_type {
        ActionType::MoveTo | ActionType::Wander | ActionType::Scout => {
            exec_move(world, agent_id, dt);
        }
        ActionType::Gather => {
            exec_gather(world, agent_id, dt, events);
        }
        ActionType::Deposit => {
            exec_deposit(world, agent_id, dt, events);
        }
        ActionType::Camp => {
            exec_camp(world, agent_id, dt);
        }
        ActionType::Build => {
            exec_build(world, agent_id, dt, events);
        }
        ActionType::Attack => {
            exec_move(world, agent_id, dt);
        }
        ActionType::Idle => {}
    }
}

// ── Action selection (state machine) ─────────────────────────────────────

fn choose_next_action(world: &mut GameWorld, agent_id: &str) {
    let agent = world.agents.get(agent_id).unwrap().clone();

    // Priority 1: deposit if carrying lots
    if agent.carry_weight() > 40.0 {
        if let Some(tc_pos) = nearest_town_center(world, &agent) {
            let travel_time = agent.position.dist(&tc_pos) / agent.speed;
            let a = world.agents.get_mut(agent_id).unwrap();
            a.current_action = Some(Box::new(QueuedAction::new(
                ActionType::Deposit,
                None,
                Some(tc_pos),
                travel_time + 1.0,
            )));
            return;
        }
    }

    // Priority 2: camp if low health
    if agent.health < 25.0 {
        let a = world.agents.get_mut(agent_id).unwrap();
        a.current_action = Some(Box::new(QueuedAction::new(
            ActionType::Camp,
            None,
            None,
            5.0,
        )));
        return;
    }

    // Priority 3: gather nearest resource
    if let Some(resource) = nearest_resource(world, &agent) {
        let travel_time = agent.position.dist(&resource.position) / agent.speed;
        let res_id = resource.id.clone();  // clone before mutable borrow
        let a = world.agents.get_mut(agent_id).unwrap();
        a.current_action = Some(Box::new(QueuedAction::new(
            ActionType::Gather,
            Some(res_id),
            None,
            travel_time + 8.0,
        )));
        return;
    }

    // Priority 4: wander
    let mut rng = rand::thread_rng();
    let wander_target = Point::new(
        rng.gen_range(50.0..world.width - 50.0),
        rng.gen_range(50.0..world.height - 50.0),
    );
    let travel_time = agent.position.dist(&wander_target) / agent.speed;
    let a = world.agents.get_mut(agent_id).unwrap();
    a.current_action = Some(Box::new(QueuedAction::new(
        ActionType::Wander,
        None,
        Some(wander_target),
        travel_time,
    )));
}

// ── Movement ────────────────────────────────────────────────────────────

fn exec_move(world: &mut GameWorld, agent_id: &str, dt: f64) {
    let target = {
        let agent = world.agents.get(agent_id).unwrap();
        match &agent.current_action {
            Some(a) => match &a.target_position {
                Some(p) => *p,
                None => return,
            },
            None => return,
        }
    };

    let agent = world.agents.get_mut(agent_id).unwrap();
    let dx = target.x - agent.position.x;
    let dy = target.y - agent.position.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 5.0 {
        agent.position.x = target.x;
        agent.position.y = target.y;
        if let Some(ref mut action) = agent.current_action {
            action.progress = action.duration_seconds;
        }
        return;
    }

    let step = agent.speed * dt;
    if step >= dist {
        agent.position.x = target.x;
        agent.position.y = target.y;
    } else {
        agent.position.x += (dx / dist) * step;
        agent.position.y += (dy / dist) * step;
    }
}

// ── Gathering ───────────────────────────────────────────────────────────

fn exec_gather(world: &mut GameWorld, agent_id: &str, dt: f64, events: &mut Vec<WorldEvent>) {
    let target_id = {
        let agent = world.agents.get(agent_id).unwrap();
        match &agent.current_action {
            Some(a) => a.target_id.clone(),
            None => return,
        }
    };

    let target_id = match target_id {
        Some(id) => id,
        None => return,
    };

    // Find resource index
    let res_idx = world.resources.iter().position(|r| r.id == target_id && r.alive());

    let res_idx = match res_idx {
        Some(idx) => idx,
        None => {
            // Resource depleted — agent will repick on next idle cycle
            // Just let this action finish
            let agent = world.agents.get_mut(agent_id).unwrap();
            if let Some(ref mut action) = agent.current_action {
                action.progress = action.duration_seconds;
            }
            return;
        }
    };

    // Move to resource if far
    {
        let agent = world.agents.get(agent_id).unwrap();
        let res = &world.resources[res_idx];
        if agent.position.dist(&res.position) > 25.0 {
            let _ = agent;
            let agent = world.agents.get_mut(agent_id).unwrap();
            let res = &world.resources[res_idx];
            let dx = res.position.x - agent.position.x;
            let dy = res.position.y - agent.position.y;
            let dist = (dx * dx + dy * dy).sqrt();
            let step = agent.speed * dt;
            if step >= dist {
                agent.position.x = res.position.x;
                agent.position.y = res.position.y;
            } else {
                agent.position.x += (dx / dist) * step;
                agent.position.y += (dy / dist) * step;
            }
            return;
        }
    }

    // Gather
    let agent = world.agents.get_mut(agent_id).unwrap();
    let resource = &mut world.resources[res_idx];
    let rate = 12.0 * dt;
    let gathered = rate.min(resource.amount);
    resource.amount -= gathered;

    match resource.kind {
        ResourceKind::Wood => agent.wood += gathered,
        ResourceKind::Gold => agent.gold += gathered,
        ResourceKind::Food => agent.food += gathered,
        ResourceKind::Stone => agent.stone += gathered,
    }

    events.push(WorldEvent {
        event_type: "gather".to_string(),
        agent_id: Some(agent_id.to_string()),
        resource: Some(format!("{:?}", resource.kind).to_lowercase()),
        amount: Some((gathered * 10.0).round() / 10.0),
    });
}

// ── Deposit ─────────────────────────────────────────────────────────────

fn exec_deposit(world: &mut GameWorld, agent_id: &str, dt: f64, events: &mut Vec<WorldEvent>) {
    let tc_pos = {
        let agent = world.agents.get(agent_id).unwrap();
        nearest_town_center(world, agent)
    };

    let tc_pos = match tc_pos {
        Some(p) => p,
        None => {
            let agent = world.agents.get_mut(agent_id).unwrap();
            if let Some(ref mut action) = agent.current_action {
                action.progress = action.duration_seconds;
            }
            return;
        }
    };

    // Move to town center if far
    {
        let agent = world.agents.get(agent_id).unwrap();
        if agent.position.dist(&tc_pos) > 20.0 {
            let _ = agent;
            let agent = world.agents.get_mut(agent_id).unwrap();
            let dx = tc_pos.x - agent.position.x;
            let dy = tc_pos.y - agent.position.y;
            let dist = (dx * dx + dy * dy).sqrt();
            let step = agent.speed * dt;
            if step >= dist {
                agent.position.x = tc_pos.x;
                agent.position.y = tc_pos.y;
            } else {
                agent.position.x += (dx / dist) * step;
                agent.position.y += (dy / dist) * step;
            }
            return;
        }
    }

    // Deposit
    let agent = world.agents.get_mut(agent_id).unwrap();
    let total = agent.wood + agent.gold + agent.stone + agent.food;
    agent.wood = 0.0;
    agent.gold = 0.0;
    agent.stone = 0.0;
    agent.food = 0.0;

    if total > 0.0 {
        events.push(WorldEvent {
            event_type: "deposited".to_string(),
            agent_id: Some(agent_id.to_string()),
            resource: None,
            amount: Some((total * 10.0).round() / 10.0),
        });
    }

    if let Some(ref mut action) = world.agents.get_mut(agent_id).unwrap().current_action {
        action.progress = action.duration_seconds;
    }
}

// ── Camp (heal) ─────────────────────────────────────────────────────────

fn exec_camp(world: &mut GameWorld, agent_id: &str, dt: f64) {
    let agent = world.agents.get_mut(agent_id).unwrap();
    agent.health = 100.0f64.min(agent.health + 15.0 * dt);
}

// ── Build ───────────────────────────────────────────────────────────────

fn exec_build(world: &mut GameWorld, agent_id: &str, dt: f64, events: &mut Vec<WorldEvent>) {
    let target = {
        let agent = world.agents.get(agent_id).unwrap();
        match &agent.current_action {
            Some(a) => a.target_position,
            None => return,
        }
    };

    let target = match target {
        Some(p) => p,
        None => return,
    };

    // Move to build site
    {
        let agent = world.agents.get(agent_id).unwrap();
        if agent.position.dist(&target) > 20.0 {
            let _ = agent;
            let agent = world.agents.get_mut(agent_id).unwrap();
            let dx = target.x - agent.position.x;
            let dy = target.y - agent.position.y;
            let dist = (dx * dx + dy * dy).sqrt();
            let step = agent.speed * dt;
            if step >= dist {
                agent.position.x = target.x;
                agent.position.y = target.y;
            } else {
                agent.position.x += (dx / dist) * step;
                agent.position.y += (dy / dist) * step;
            }
            return;
        }
    }

    // Build
    let agent = world.agents.get_mut(agent_id).unwrap();
    if agent.wood >= 50.0 {
        agent.wood -= 50.0;
        let building = Building {
            id: Uuid::new_v4().to_string()[..8].to_string(),
            kind: "town_center".to_string(),
            position: target,
            health: 100.0,
            owner: agent.id.clone(),
        };
        world.buildings.push(building);
        events.push(WorldEvent {
            event_type: "built".to_string(),
            agent_id: Some(agent_id.to_string()),
            resource: None,
            amount: None,
        });
    }

    if let Some(ref mut action) = world.agents.get_mut(agent_id).unwrap().current_action {
        action.progress = action.duration_seconds;
    }
}

// ── World queries ───────────────────────────────────────────────────────

fn nearest_town_center(world: &GameWorld, agent: &Agent) -> Option<Point> {
    world
        .buildings
        .iter()
        .filter(|b| b.kind == "town_center")
        .min_by(|a, b| {
            let da = agent.position.dist(&a.position);
            let db = agent.position.dist(&b.position);
            da.partial_cmp(&db).unwrap()
        })
        .map(|b| b.position)
}

fn nearest_resource<'a>(world: &'a GameWorld, agent: &Agent) -> Option<&'a ResourceNode> {
    world
        .resources
        .iter()
        .filter(|r| r.alive())
        .min_by(|a, b| {
            let da = agent.position.dist(&a.position);
            let db = agent.position.dist(&b.position);
            da.partial_cmp(&db).unwrap()
        })
}