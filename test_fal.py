#!/usr/bin/env python3
"""Test OpenRouter image models"""
import subprocess, json, httpx, sys

# Get API key
r = subprocess.run(
    ["bash", "-c", 'source ~/.hermes/.env 2>/dev/null; echo -n "$OPENROUTER_API_KEY"'],
    capture_output=True, text=True
)
key = r.stdout.strip()
print(f"Key length: {len(key)}, starts: {key[:15]}...", flush=True)

models = [
    "black-forest-labs/flux-schnell",
    "black-forest-labs/flux-dev",
    "black-forest-labs/flux-pro",
    "stabilityai/stable-diffusion-3.5-large",
    "openai/dall-e-3",
]

for model in models:
    try:
        resp = httpx.post(
            "https://openrouter.ai/api/v1/completions",
            headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
            json={"model": model, "prompt": "test", "max_tokens": 5},
            timeout=10
        )
        data = resp.json()
        err = data.get("error", {})
        msg = err.get("message", str(data)[:100])
        print(f"  {model}: {msg[:80]}", flush=True)
    except Exception as e:
        print(f"  {model}: ERROR {e}", flush=True)