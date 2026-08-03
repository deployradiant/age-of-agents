# Age of Agents 🏰

*A real-time strategy village simulation with LLM-driven NPC agents.*

Age of Agents is an Age of Empires-inspired sandbox where every NPC is driven by an LLM (or a state machine). Watch autonomous agents gather resources, build structures, fight, and cooperate — all driven by thinking models deciding their next move in real time.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 Modal Server                     │
│  ┌──────────┐  ┌────────────┐  ┌───────────┐   │
│  │ FastAPI  │◄─┤ WebSocket  ├─►│ Game Loop │   │
│  │ REST API │  │ Broadcast  │  │ (2 Hz)    │   │
│  └──────────┘  └────────────┘  └─────┬─────┘   │
│                                       │         │
│  ┌────────────────────────────────────▼──────┐  │
│  │            Agent State Machine             │  │
│  │  IDLE → pick task → ACTIVE → execute →    │  │
│  └────────────────────────────────────────────┘  │
│                                       │         │
│  ┌────────────────────────────────────▼──────┐  │
│  │  (Future) Async LLM Agent Thinker         │  │
│  │  Decoupled — agents think async,          │  │
│  │  results feed back into game loop         │  │
│  └────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
         │ WebSocket (state updates)
         ▼
┌───────────────────┐
│  HTML Frontend    │
│  Three.js Canvas  │
│  OrbitControls    │
│  HUD Sidebar      │
└───────────────────┘
```

## Quick Start

### Local Dev

```bash
# Install dependencies
pip install fastapi uvicorn websockets pydantic

# Run the server
uvicorn backend.server:web_app --reload --port 8000

# Open http://localhost:8000
```

### Modal Deploy

```bash
# Ensure you have a Modal secret called "openrouter-api-key" with key OPENROUTER_API_KEY
modal deploy modal_app.py
```

## Current Status

- ✅ Game world with agents, resources, buildings
- ✅ State machine NPCs (gather, deposit, camp, wander, build)
- ✅ Three.js 3D frontend with orbit controls
- ✅ WebSocket real-time state broadcast
- ❌ Async LLM agent thinking — next milestone

## Roadmap

1. **Async LLM Agent Thinker** — Replace state machine decisions with async DeepSeek V4 Flash calls
2. **Multi-model agents** — Assign different LLMs to different NPCs
3. **Agent interaction** — Communication, trading, alliances between agents
4. **Persistent world** — Save/load game state via Modal volumes
5. **Web UI controls** — Click to select agents, issue commands

## Built With

- **Backend**: Python + FastAPI + WebSockets on [Modal](https://modal.com)
- **Frontend**: Three.js with OrbitControls
- **LLMs (future)**: OpenRouter / DeepSeek V4 Flash