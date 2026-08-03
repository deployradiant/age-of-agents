"""Game state models — the world, agents, resources, and actions."""

from __future__ import annotations

import time
import uuid
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class ActionType(str, Enum):
    MOVE_TO = "move_to"
    GATHER = "gather"
    BUILD = "build"
    ATTACK = "attack"
    IDLE = "idle"
    SCOUT = "scout"
    CAMP = "camp"
    DEPOSIT = "deposit"
    WANDER = "wander"


class ResourceType(str, Enum):
    WOOD = "wood"
    GOLD = "gold"
    STONE = "stone"
    FOOD = "food"
    # buildings the agent can build
    TOWN_CENTER = "town_center"
    BARRACKS = "barracks"
    


@dataclass
class Point:
    x: float
    y: float

    def dist(self, other: Point) -> float:
        return ((self.x - other.x) ** 2 + (self.y - other.y) ** 2) ** 0.5


@dataclass
class ResourceNode:
    id: str = field(default_factory=lambda: uuid.uuid4().hex[:8])
    kind: ResourceType = ResourceType.WOOD
    position: Point = field(default_factory=lambda: Point(0, 0))
    amount: float = 100.0
    max_amount: float = 100.0

    @property
    def alive(self) -> bool:
        return self.amount > 0


@dataclass
class Building:
    id: str = field(default_factory=lambda: uuid.uuid4().hex[:8])
    kind: ResourceType = ResourceType.TOWN_CENTER
    position: Point = field(default_factory=lambda: Point(0, 0))
    health: float = 100.0
    owner: str = ""  # agent id


@dataclass
class QueuedAction:
    action_type: ActionType
    target_id: Optional[str] = None
    target_position: Optional[Point] = None
    progress: float = 0.0  # 0..1 for timed actions
    duration_seconds: float = 0.0


class AgentState(str, Enum):
    ACTIVE = "active"
    THINKING = "thinking"      # waiting for LLM response
    IDLE = "idle"              # needs a plan, will trigger LLM
    DEAD = "dead"


@dataclass
class Agent:
    id: str = field(default_factory=lambda: uuid.uuid4().hex[:8])
    name: str = ""
    position: Point = field(default_factory=lambda: Point(0, 0))
    state: AgentState = AgentState.IDLE
    move_target: Optional[Point] = None

    # resources carried
    wood: float = 0
    gold: float = 0
    stone: float = 0
    food: float = 0

    # capabilities
    speed: float = 50.0  # pixels per second
    health: float = 100.0

    # action queue
    action_queue: list[QueuedAction] = field(default_factory=list)
    current_action: Optional[QueuedAction] = None
    action_progress: float = 0.0

    # thinking
    thought_requested_at: Optional[float] = None
    last_thought: Optional[str] = None
    last_thought_result: list[QueuedAction] = field(default_factory=list)

    # which LLM model drives this agent (future feature)
    model: str = "deepseek/deepseek-v4-flash"

    # cosmetic
    color: str = "#4488ff"
    
    @property
    def pos_tuple(self) -> tuple[float, float]:
        return (self.position.x, self.position.y)

    def carry_weight(self) -> float:
        return self.wood + self.gold + self.stone + self.food


@dataclass
class GameWorld:
    width: float = 800
    height: float = 600
    agents: dict[str, Agent] = field(default_factory=dict)
    resources: list[ResourceNode] = field(default_factory=list)
    buildings: list[Building] = field(default_factory=list)
    tick_count: int = 0
    time_elapsed: float = 0.0
    last_tick: float = field(default_factory=time.time)

    def _make_action(self, action_type: ActionType, target_id: str | None = None, target_position: Point | None = None, duration_seconds: float = 3.0) -> QueuedAction:
        return QueuedAction(
            action_type=action_type,
            target_id=target_id,
            target_position=target_position,
            duration_seconds=duration_seconds,
        )

    def get_agent(self, agent_id: str) -> Agent | None:
        return self.agents.get(agent_id)

    def get_visible_resources(self, agent: Agent, radius: float = 400) -> list[ResourceNode]:
        return [r for r in self.resources if r.alive and agent.position.dist(r.position) < radius]

    def get_nearby_agents(self, agent: Agent, radius: float = 300) -> list[Agent]:
        return [a for a in self.agents.values() if a.id != agent.id and a.state != AgentState.DEAD and agent.position.dist(a.position) < radius]

    def snapshot_for_llm(self, agent_id: str) -> dict:
        agent = self.get_agent(agent_id)
        if not agent:
            return {}
        visible_res = self.get_visible_resources(agent)
        nearby = self.get_nearby_agents(agent)
        return {
            "agent": {
                "id": agent.id,
                "name": agent.name,
                "position": (round(agent.position.x, 1), round(agent.position.y, 1)),
                "carrying": {r: getattr(agent, r, 0) for r in ["wood", "gold", "stone", "food"]},
                "health": round(agent.health, 1),
                "state": agent.state.value,
            },
            "visible_resources": [
                {"kind": r.kind.value, "position": (round(r.position.x, 1), round(r.position.y, 1)), "amount": round(r.amount, 1)}
                for r in visible_resources
            ],
            "nearby_agents": [
                {"name": a.name, "position": (round(a.position.x, 1), round(a.position.y, 1)), "health": round(a.health, 1)}
                for a in nearby
            ],
            "inventory": {
                "total_wood": sum(a.wood for a in self.agents.values()),
                "total_gold": sum(a.gold for a in self.agents.values()),
            },
            "buildings": [
                {"kind": b.kind.value, "position": (round(b.position.x, 1), round(b.position.y, 1))}
                for b in self.buildings if b.owner == agent_id
            ],
        }


def create_default_world() -> GameWorld:
    world = GameWorld(width=1200, height=800)

    # scatter resources
    import random
    random.seed(42)
    for _ in range(20):
        world.resources.append(ResourceNode(
            kind=ResourceType.WOOD,
            position=Point(random.uniform(50, 1150), random.uniform(50, 750)),
            amount=random.uniform(50, 150),
        ))
    for _ in range(8):
        world.resources.append(ResourceNode(
            kind=ResourceType.GOLD,
            position=Point(random.uniform(50, 1150), random.uniform(50, 750)),
            amount=random.uniform(30, 80),
        ))
    for _ in range(12):
        world.resources.append(ResourceNode(
            kind=ResourceType.FOOD,
            position=Point(random.uniform(50, 1150), random.uniform(50, 750)),
            amount=random.uniform(40, 120),
        ))

    # spawn a few agents
    names = ["Aldric", "Brom", "Cedric", "Doran", "Elara"]
    for i, name in enumerate(names):
        world.agents[name.lower()] = Agent(
            name=name,
            position=Point(100 + i * 80, 400),
            color=["#4488ff", "#ff4488", "#44ff88", "#ffaa44", "#aa44ff"][i],
            model="deepseek/deepseek-v4-flash",
        )

    # starting town center
    world.buildings.append(Building(
        kind=ResourceType.TOWN_CENTER,
        position=Point(600, 400),
        owner="aldric",
    ))

    return world