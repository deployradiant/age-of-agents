"""LLM integration — async calls to OpenRouter for agent decision-making."""

from __future__ import annotations

import json
import os
from typing import Optional
from openai import AsyncOpenAI
import httpx

# Use OpenRouter by default
DEFAULT_BASE_URL = "https://openrouter.ai/api/v1"
# The model we're currently running through — DeepSeek V4 Flash
DEFAULT_MODEL = "deepseek/deepseek-v4-flash"

_client: Optional[AsyncOpenAI] = None


def get_client() -> AsyncOpenAI:
    global _client
    if _client is None:
        api_key = os.environ.get("OPENROUTER_API_KEY") or os.environ.get("OPENAI_API_KEY", "")
        base_url = os.environ.get("OPENAI_BASE_URL", DEFAULT_BASE_URL)
        _client = AsyncOpenAI(api_key=api_key, base_url=base_url)
    return _client


SYSTEM_PROMPT = """You are the brain of an NPC in a real-time strategy village simulation (Age of Agents).

You control ONE agent. Your goal is survival, growth, and contributing to your village.

## Available Actions
Return a JSON array of actions for your agent. Each action has:
- action_type: one of "move_to", "gather", "build", "attack", "idle", "deposit", "wander", "scout", "camp"
- target_id: optional resource/building/agent id
- target_position: [x, y] coordinates (used for move_to and build)
- duration_seconds: how long to spend (gather: 5-10, idle: 2-5, wander: 3-8)

## Strategy Guidelines
- If carrying lots of resources, go deposit at the town center
- If low on health, retreat to town center or camp
- If idle, find the nearest useful resource and gather it
- If you see enemies nearby, consider attacking or retreating depending on health
- You can build buildings: town_center costs 200 wood + 100 gold
- Scout unknown areas to find more resources
- Stay alive — don't attack when outnumbered or low health

## World State
You'll receive your current status, visible resources, nearby agents, and village inventory.

Return ONLY a valid JSON array. No markdown, no explanation.
Example: [{"action_type": "move_to", "target_position": [400, 300], "duration_seconds": 3}]
"""


async def ask_agent(agent_name: str, world_snapshot: dict, model: str = DEFAULT_MODEL, timeout: float = 15.0) -> list[dict]:
    """Ask the LLM what actions this agent should take next."""
    client = get_client()

    world_summary = json.dumps(world_snapshot, indent=2)

    try:
        response = await client.chat.completions.create(
            model=model,
            messages=[
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": f"Agent name: {agent_name}\n\nCurrent world state:\n{world_summary}\n\nWhat do you do next?"},
            ],
            temperature=0.7,
            max_tokens=512,
            timeout=httpx.Timeout(timeout=timeout),
            extra_headers={
                "HTTP-Referer": "https://age-of-agents.modal.run",
                "X-Title": "Age of Agents",
            },
        )
        content = response.choices[0].message.content or "[]"
        # Strip code fences if present
        if "```" in content:
            content = content.split("```")[1]
            if content.startswith("json"):
                content = content[4:]
        content = content.strip()
        actions = json.loads(content)
        if isinstance(actions, dict):
            actions = [actions]
        return actions[:3]  # max 3 actions per thought
    except Exception as e:
        print(f"[LLM] Error asking {agent_name}: {e}")
        # Fallback: wander to a random nearby spot
        return [
            {"action_type": "wander", "target_position": None, "duration_seconds": 5.0}
        ]


async def ask_agent_streaming(agent_name: str, world_snapshot: dict, model: str = DEFAULT_MODEL, timeout: float = 20.0) -> list[dict]:
    """Streaming version — useful if we want to show agent's 'inner monologue' on the client later."""
    client = get_client()
    world_summary = json.dumps(world_snapshot, indent=2)

    full_content = ""
    try:
        stream = await client.chat.completions.create(
            model=model,
            messages=[
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": f"Agent name: {agent_name}\n\nCurrent world state:\n{world_summary}\n\nWhat do you do next?"},
            ],
            temperature=0.7,
            max_tokens=512,
            timeout=httpx.Timeout(timeout=timeout),
            stream=True,
            extra_headers={
                "HTTP-Referer": "https://age-of-agents.modal.run",
                "X-Title": "Age of Agents",
            },
        )
        async for chunk in stream:
            delta = chunk.choices[0].delta.content or ""
            full_content += delta
    except Exception as e:
        print(f"[LLM] Error asking {agent_name}: {e}")
        return [{"action_type": "wander", "target_position": None, "duration_seconds": 5.0}]

    content = full_content.strip()
    if "```" in content:
        content = content.split("```")[1]
        if content.startswith("json"):
            content = content[4:]
    content = content.strip()
    try:
        actions = json.loads(content)
        if isinstance(actions, dict):
            actions = [actions]
        return actions[:3]
    except json.JSONDecodeError:
        print(f"[LLM] Bad JSON from {agent_name}: {content[:200]}")
        return [{"action_type": "wander", "target_position": None, "duration_seconds": 5.0}]