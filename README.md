# Age of Agents 🏰

*A real-time strategy village simulation with LLM-driven NPC agents.*

Age of Agents is an Age of Empires-inspired sandbox where every NPC is driven by an LLM (or a state machine). Watch autonomous agents gather resources, build structures, fight, and cooperate — all driven by thinking models deciding their next move in real time.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 Modal Server                     │
│  ┌──────────┐  ┌────────────┐  ┌───────────┐   │
│  │  Axum    │◄─┤ WebSocket  ├─►│ Game Loop │   │
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
# Prerequisites: Rust toolchain (rustup)

# Run the server directly
cargo run --release

# Open http://localhost:8000
```

### Docker

```bash
docker build -t age-of-agents .
docker run -p 8000:8000 age-of-agents
```

### Modal Deploy

```bash
modal deploy modal_app.py
```

The deployment uses a multi-stage Docker build (see `Dockerfile`):
- **Stage 1** (`rust:latest`): Compiles the Cargo project in release mode
- **Stage 2** (`debian:bookworm-slim`): Copies the binary and `frontend/` directory, exposes port 8000

The Modal app (`modal_app.py`):
- Builds the image via `modal.Image.from_dockerfile("Dockerfile")`
- Mounts the local `frontend/` directory to `/root/frontend/` in the container (matching the path the Rust binary expects)
- Exposes the Axum HTTP server on port 8000 using `@modal.web_server`
- Allocates 256 MB memory with 1 warm container

## Current Status

- ✅ Game world with agents, resources, buildings
- ✅ State machine NPCs (gather, deposit, camp, wander, build)
- ✅ Three.js 3D frontend with orbit controls
- ✅ WebSocket real-time state broadcast
- ✅ Rust/Axum backend with Docker + Modal deployment
- ❌ Async LLM agent thinking — next milestone

## Roadmap

1. **Async LLM Agent Thinker** — Replace state machine decisions with async DeepSeek V4 Flash calls
2. **Multi-model agents** — Assign different LLMs to different NPCs
3. **Agent interaction** — Communication, trading, alliances between agents
4. **Persistent world** — Save/load game state via Modal volumes
5. **Web UI controls** — Click to select agents, issue commands

## Built With

- **Backend**: Rust + Axum + WebSockets on [Modal](https://modal.com)
- **Frontend**: Three.js with OrbitControls
- **LLMs (future)**: OpenRouter / DeepSeek V4 Flash