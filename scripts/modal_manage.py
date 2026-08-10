#!/usr/bin/env python3
"""Manage and verify the Age of Agents Modal deployment."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP_NAME = "age-of-agents"
BASE_URL = "https://koogle-frick--age-of-agents-web.modal.run"
DIRECTIONAL_ASSETS = (
    "agent_walk_diag_toward_01.png",
    "agent_walk_diag_toward_02.png",
    "agent_walk_down_01.png",
    "agent_walk_down_02.png",
    "agent_walk_up_01.png",
    "agent_walk_up_02.png",
)


def modal(*arguments: str) -> None:
    subprocess.run([sys.executable, "-m", "modal", *arguments], cwd=ROOT, check=True)


def fetch(path: str, timeout: int = 120) -> bytes:
    request = urllib.request.Request(f"{BASE_URL}{path}", headers={"User-Agent": "age-of-agents-deploy-check"})
    with urllib.request.urlopen(request, timeout=timeout) as response:
        if response.status != 200:
            raise RuntimeError(f"GET {path} returned HTTP {response.status}")
        return response.read()


def verify_once() -> None:
    comparisons = {
        "/": ROOT / "frontend/index.html",
        "/frontend/app.js": ROOT / "frontend/app.js",
        **{
            f"/assets/game/{name}": ROOT / "assets/game" / name
            for name in DIRECTIONAL_ASSETS
        },
    }
    for remote_path, local_path in comparisons.items():
        remote = fetch(remote_path)
        local = local_path.read_bytes()
        if remote != local:
            raise RuntimeError(f"production {remote_path} does not match {local_path.relative_to(ROOT)}")

    state = json.loads(fetch("/state"))
    terrain = state.get("terrain", [])
    units = state.get("units", [])
    if len(terrain) != 600 or len(units) != 2:
        raise RuntimeError(f"unexpected world shape: terrain={len(terrain)}, units={len(units)}")
    unseen = [cell for cell in terrain if cell.get("visibility") == "unseen"]
    if not unseen:
        raise RuntimeError("production state has no unseen terrain to verify")
    if any("biome" in cell for cell in unseen):
        raise RuntimeError("production leaks biome data for unseen terrain")

    print(
        "PASS production matches checkout; "
        f"terrain={len(terrain)}, units={len(units)}, unseen={len(unseen)}, "
        f"directional_assets={len(DIRECTIONAL_ASSETS)}"
    )


def verify(attempts: int = 24, delay_seconds: int = 5) -> None:
    last_error: Exception | None = None
    for attempt in range(1, attempts + 1):
        try:
            verify_once()
            return
        except (RuntimeError, urllib.error.URLError, TimeoutError) as error:
            last_error = error
            if attempt < attempts:
                print(f"production not ready ({attempt}/{attempts}): {error}", file=sys.stderr)
                time.sleep(delay_seconds)
    raise RuntimeError(f"production did not converge after {attempts} attempts") from last_error


def deploy(skip_verify: bool) -> None:
    modal("deploy", "modal_app.py")
    if not skip_verify:
        verify()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    deploy_parser = subparsers.add_parser("deploy", help="Deploy current checkout and verify production")
    deploy_parser.add_argument("--skip-verify", action="store_true")

    subparsers.add_parser("verify", help="Compare production with the current checkout")
    subparsers.add_parser("status", help="List Modal apps and containers")
    subparsers.add_parser("history", help="Show deployment history")

    logs_parser = subparsers.add_parser("logs", help="Show recent application logs")
    logs_parser.add_argument("--tail", type=int, default=100)
    logs_parser.add_argument("--since")
    logs_parser.add_argument("--follow", action="store_true")

    subparsers.add_parser("rollover", help="Restart production containers without rebuilding")

    stop_parser = subparsers.add_parser("stop", help="Permanently stop the deployed app")
    stop_parser.add_argument(
        "--confirm",
        metavar="APP_NAME",
        help=f"required safety confirmation; pass exactly {APP_NAME!r}",
    )
    return parser


def main() -> None:
    args = build_parser().parse_args()
    if args.command == "deploy":
        deploy(args.skip_verify)
    elif args.command == "verify":
        verify()
    elif args.command == "status":
        modal("app", "list")
        modal("container", "list")
    elif args.command == "history":
        modal("app", "history", APP_NAME)
    elif args.command == "logs":
        command = ["app", "logs", APP_NAME, "--tail", str(args.tail)]
        if args.since:
            command.extend(("--since", args.since))
        if args.follow:
            command.append("--follow")
        modal(*command)
    elif args.command == "rollover":
        modal("app", "rollover", APP_NAME)
    elif args.command == "stop":
        if args.confirm != APP_NAME:
            raise SystemExit(f"refusing to stop production; pass --confirm {APP_NAME}")
        modal("app", "stop", APP_NAME, "--yes")


if __name__ == "__main__":
    main()
