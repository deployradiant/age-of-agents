"""Age of Agents — FastAPI backend with WebSocket game server.

Run locally:  uvicorn backend.server:web_app --reload --port 8000
Deploy:       modal deploy modal_app.py
"""

from __future__ import annotations

import asyncio
import json
import os
import time
import traceback
from typing import Optional

FRONTEND_DIR = os.path.join(os.path.dirname(__file__), "..", "frontend")

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.responses import HTMLResponse

from backend.state import create_default_world, GameWorld, AgentState
from backend.game_loop import tick_world, TICK_RATE

web_app = FastAPI(title="Age of Agents")

# ── Game State (in-memory) ───────────────────────────────────────────────
world: Optional[GameWorld] = None
game_task: Optional[asyncio.Task] = None
connected_clients: set[WebSocket] = set()


def ensure_world():
    global world
    if world is None:
        world = create_default_world()


# ── Game loop (async, runs forever) ──────────────────────────────────────
async def game_loop():
    global world
    ensure_world()
    tick_interval = 1.0 / TICK_RATE

    while True:
        tick_start = time.time()
        dt = tick_interval  # fixed step

        events = tick_world(world, dt)

        # Broadcast state to all connected clients
        if connected_clients:
            state_msg = _serialize_world(world, events)
            await _broadcast(state_msg)

        # Sleep for remainder of tick
        elapsed = time.time() - tick_start
        sleep_time = max(0, tick_interval - elapsed)
        await asyncio.sleep(sleep_time)


def _serialize_world(w: GameWorld, events: list[dict]) -> str:
    agents = {}
    for aid, a in w.agents.items():
        agents[aid] = {
            "id": a.id,
            "name": a.name,
            "x": round(a.position.x, 1),
            "y": round(a.position.y, 1),
            "color": a.color,
            "health": round(a.health, 1),
            "state": a.state.value,
            "wood": round(a.wood, 1),
            "gold": round(a.gold, 1),
            "stone": round(a.stone, 1),
            "food": round(a.food, 1),
            "action": a.current_action.action_type.value if a.current_action else None,
        }

    resources = [
        {"id": r.id, "kind": r.kind.value, "x": round(r.position.x, 1), "y": round(r.position.y, 1), "amount": round(r.amount, 1)}
        for r in w.resources if r.alive
    ]

    buildings = [
        {"id": b.id, "kind": b.kind.value, "x": round(b.position.x, 1), "y": round(b.position.y, 1), "health": round(b.health, 1)}
        for b in w.buildings
    ]

    data = {
        "type": "state",
        "tick": w.tick_count,
        "time": round(w.time_elapsed, 1),
        "agents": agents,
        "resources": resources,
        "buildings": buildings,
        "events": events,
    }
    return json.dumps(data)


async def _broadcast(msg: str):
    dead = set()
    for ws in connected_clients:
        try:
            await ws.send_text(msg)
        except Exception:
            dead.add(ws)
    connected_clients -= dead


# ── REST endpoints ───────────────────────────────────────────────────────
@web_app.get("/")
async def index():
    """Serve the game frontend."""
    paths_to_try = [
        os.path.join(FRONTEND_DIR, "index.html"),
        os.path.join(os.path.dirname(__file__), "..", "frontend", "index.html"),
        os.path.join(os.path.dirname(__file__), "frontend", "index.html"),
        "/root/frontend/index.html",
    ]
    for p in paths_to_try:
        if os.path.exists(p):
            with open(p) as f:
                return HTMLResponse(f.read())
    return HTMLResponse("<h1>Age of Agents</h1><p>Frontend not found.</p>")


@web_app.post("/reset")
async def reset():
    global world
    world = create_default_world()
    return {"status": "ok", "message": "World reset"}


@web_app.get("/state")
async def get_state():
    ensure_world()
    return json.loads(_serialize_world(world, []))


# ── WebSocket ────────────────────────────────────────────────────────────
@web_app.websocket("/ws")
async def ws_endpoint(ws: WebSocket):
    await ws.accept()
    connected_clients.add(ws)

    # Send initial state immediately
    ensure_world()
    initial = _serialize_world(world, [])
    await ws.send_text(initial)

    try:
        while True:
            data = await ws.receive_text()
            msg = json.loads(data)
            msg_type = msg.get("type")

            if msg_type == "ping":
                await ws.send_text(json.dumps({"type": "pong"}))

    except WebSocketDisconnect:
        pass
    except Exception:
        traceback.print_exc()
    finally:
        connected_clients.discard(ws)


# ── Startup ──────────────────────────────────────────────────────────────
@web_app.on_event("startup")
async def startup():
    global game_task
    ensure_world()
    game_task = asyncio.create_task(game_loop())