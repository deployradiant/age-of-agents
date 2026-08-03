"""Game loop — ticks the world, processes agent state machines."""

from __future__ import annotations

import math
import random
from typing import Optional

from backend.state import (
    ActionType,
    Agent,
    AgentState,
    Building,
    GameWorld,
    Point,
    ResourceNode,
    ResourceType,
)

TICK_RATE = 2  # ticks per second


def tick_world(world: GameWorld, dt: float) -> list[dict]:
    """Advance the world by dt seconds. Returns events for broadcast."""
    world.tick_count += 1
    world.time_elapsed += dt
    events: list[dict] = []

    for agent in list(world.agents.values()):
        if agent.state == AgentState.DEAD:
            continue
        tick_agent(world, agent, dt, events)

    # Gentle resource regen
    for r in world.resources:
        if r.amount < r.max_amount:
            r.amount = min(r.max_amount, r.amount + 0.1 * dt)

    return events


# ── State Machine ────────────────────────────────────────────────────────
# Agents cycle: IDLE → (choose task) → ACTIVE → (task done) → IDLE
# No LLM needed yet — agents use simple heuristic rules.
# Later: swap in async LLM calls at the IDLE→choose-task transition.


def tick_agent(world: GameWorld, agent: Agent, dt: float, events: list[dict]) -> None:
    # DEAD or THINKING agents do nothing
    if agent.state in (AgentState.DEAD, AgentState.THINKING):
        return

    # IDLE → pick next action
    if agent.state == AgentState.IDLE:
        _choose_next_action(world, agent)
        if agent.current_action is None:
            # Nothing to do — stay idle
            return
        agent.action_progress = 0.0
        agent.state = AgentState.ACTIVE

    # ACTIVE → execute current action
    if agent.state == AgentState.ACTIVE and agent.current_action:
        action = agent.current_action
        action.progress += dt
        _execute_action(world, agent, action, dt, events)

        # Check completion
        if action.progress >= action.duration_seconds:
            on_action_done(world, agent, action, events)
            agent.current_action = None
            agent.action_progress = 0.0
            agent.state = AgentState.IDLE


def on_action_done(world: GameWorld, agent: Agent, action, events: list[dict]) -> None:
    """Side effects when an action completes."""
    pass  # post-completion effects handled inline per action


def _choose_next_action(world: GameWorld, agent: Agent) -> None:
    """Heuristic decision: pick what the agent does next based on state."""
    # Priority 1: deposit if carrying lots
    if agent.carry_weight() > 40:
        tc = _nearest_town_center(world, agent)
        if tc:
            agent.current_action = world._make_action(
                ActionType.DEPOSIT,
                duration_seconds=agent.position.dist(tc.position) / agent.speed + 1,
            )
            return

    # Priority 2: retreat if low health
    if agent.health < 25:
        tc = _nearest_town_center(world, agent)
        if tc:
            agent.current_action = world._make_action(
                ActionType.CAMP,
                duration_seconds=5.0,
            )
            return

    # Priority 3: gather nearest resource
    nearest = _nearest_resource(world, agent)
    if nearest:
        travel_time = agent.position.dist(nearest.position) / agent.speed
        gather_time = 8.0
        agent.current_action = world._make_action(
            ActionType.GATHER,
            target_id=nearest.id,
            duration_seconds=travel_time + gather_time,
        )
        return

    # Priority 4: wander
    wander_target = Point(
        random.uniform(50, world.width - 50),
        random.uniform(50, world.height - 50),
    )
    agent.current_action = world._make_action(
        ActionType.WANDER,
        target_position=wander_target,
        duration_seconds=agent.position.dist(wander_target) / agent.speed,
    )


def _execute_action(world: GameWorld, agent: Agent, action, dt: float, events: list[dict]) -> None:
    if action.action_type == ActionType.MOVE_TO:
        _exec_move(agent, action, dt)
    elif action.action_type == ActionType.WANDER:
        _exec_wander(agent, action, dt)
    elif action.action_type == ActionType.GATHER:
        _exec_gather(world, agent, action, dt, events)
    elif action.action_type == ActionType.DEPOSIT:
        _exec_deposit(world, agent, action, dt, events)
    elif action.action_type == ActionType.CAMP:
        _exec_camp(agent, action, dt)
    elif action.action_type == ActionType.BUILD:
        _exec_build(world, agent, action, dt, events)
    elif action.action_type == ActionType.IDLE:
        pass  # just wait


def _exec_move(agent: Agent, action, dt: float) -> None:
    if not action.target_position:
        action.progress = action.duration_seconds
        return
    _move_toward(agent, action.target_position, dt, action.duration_seconds)


def _exec_wander(agent: Agent, action, dt: float) -> None:
    if action.target_position:
        _move_toward(agent, action.target_position, dt, action.duration_seconds)


def _exec_gather(world: GameWorld, agent: Agent, action, dt: float, events: list[dict]) -> None:
    target = _find_resource(world, action.target_id)
    if target is None or not target.alive:
        # Resource depleted — find another
        new_target = _nearest_resource(world, agent)
        if new_target:
            action.target_id = new_target.id
            # Extend duration to account for travel
            travel = agent.position.dist(new_target.position) / agent.speed
            action.duration_seconds = action.progress + travel + 8.0
        else:
            action.progress = action.duration_seconds
        return

    # Move to resource if far
    if agent.position.dist(target.position) > 25:
        _move_toward(agent, target.position, dt, action.duration_seconds)
        return

    # Gather
    rate = 12.0 * dt
    gathered = min(rate, target.amount)
    target.amount -= gathered
    setattr(agent, target.kind.value, getattr(agent, target.kind.value, 0) + gathered)
    events.append({"type": "gather", "agent_id": agent.id, "resource": target.kind.value, "amount": round(gathered, 1)})

    if target.amount <= 0:
        action.target_id = None  # will repick next tick


def _exec_deposit(world: GameWorld, agent: Agent, action, dt: float, events: list[dict]) -> None:
    tc = _nearest_town_center(world, agent)
    if not tc:
        action.progress = action.duration_seconds
        return

    if agent.position.dist(tc.position) > 20:
        _move_toward(agent, tc.position, dt, action.duration_seconds)
        return

    # Deposit all carried resources
    total = 0
    for res in ("wood", "gold", "stone", "food"):
        val = getattr(agent, res, 0)
        total += val
        setattr(agent, res, 0)
    if total > 0:
        events.append({"type": "deposited", "agent_id": agent.id, "amount": round(total, 1)})
    action.progress = action.duration_seconds


def _exec_camp(agent: Agent, action, dt: float) -> None:
    agent.health = min(100.0, agent.health + 15.0 * dt)


def _exec_build(world: GameWorld, agent: Agent, action, dt: float, events: list[dict]) -> None:
    if not action.target_position:
        action.progress = action.duration_seconds
        return

    if agent.position.dist(action.target_position) > 20:
        _move_toward(agent, action.target_position, dt, action.duration_seconds)
        return

    # Spend resources and place building
    if agent.wood >= 50:
        agent.wood -= 50
        world.buildings.append(Building(
            kind=ResourceType.TOWN_CENTER,
            position=action.target_position,
            owner=agent.id,
        ))
        events.append({"type": "built", "agent_id": agent.id})
    action.progress = action.duration_seconds


# ── Movement helper ─────────────────────────────────────────────────────
def _move_toward(agent: Agent, target: Point, dt: float, timeout: float) -> None:
    dx = target.x - agent.position.x
    dy = target.y - agent.position.y
    dist = math.sqrt(dx * dx + dy * dy)

    if dist < 5:
        agent.position.x, agent.position.y = target.x, target.y
        return

    step = agent.speed * dt
    if step >= dist:
        agent.position.x, agent.position.y = target.x, target.y
    else:
        agent.position.x += (dx / dist) * step
        agent.position.y += (dy / dist) * step


# ── World queries ────────────────────────────────────────────────────────
def _nearest_town_center(world: GameWorld, agent: Agent) -> Optional[Point]:
    tcs = [b for b in world.buildings if b.kind == ResourceType.TOWN_CENTER]
    if not tcs:
        return None
    return min(tcs, key=lambda b: agent.position.dist(b.position)).position


def _nearest_resource(world: GameWorld, agent: Agent) -> Optional[ResourceNode]:
    alive = [r for r in world.resources if r.alive]
    if not alive:
        return None
    return min(alive, key=lambda r: agent.position.dist(r.position))


def _find_resource(world: GameWorld, res_id: Optional[str]) -> Optional[ResourceNode]:
    if res_id is None:
        return None
    for r in world.resources:
        if r.id == res_id:
            return r
    return None