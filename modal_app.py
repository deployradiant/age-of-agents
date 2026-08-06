"""Modal deployment for Age of Agents Rust backend.

Build the Rust binary directly in a Modal image.
"""

import modal

app = modal.App("age-of-agents")
volume = modal.Volume.from_name("age-of-agents-data", create_if_missing=True)

# The add_local_* calls must be last, with copy=True since we run build after
image = (
    modal.Image.debian_slim()
    .apt_install("curl", "build-essential", "pkg-config", "libssl-dev")
    .run_commands(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
        ". $HOME/.cargo/env && cargo --version",
    )
    .workdir("/app")
    .run_commands(
        "mkdir -p src",
    )
    .add_local_file("Cargo.toml", "/app/Cargo.toml", copy=True)
    .add_local_file("Cargo.lock", "/app/Cargo.lock", copy=True)
    .run_commands(
        "echo 'fn main() {}' > src/main.rs",
        ". $HOME/.cargo/env && cargo build --release --locked || true",
    )
    .add_local_dir("src", "/app/src", copy=True)
    .run_commands(
        ". $HOME/.cargo/env && cargo build --release --locked --bin age-of-agents",
    )
    .add_local_dir("frontend", "/app/frontend", copy=True)
    .add_local_dir("assets", "/app/assets", copy=True)
)


@app.function(
    image=image,
    memory=256,
    min_containers=1,
    max_containers=1,
    volumes={"/data": volume},
)
@modal.concurrent(max_inputs=100)
@modal.web_server(port=8000, startup_timeout=30)
def web():
    """Serve Age of Agents via the Rust binary."""
    import subprocess
    import os

    env = {**os.environ, "AGE_OF_AGENTS_DB": "/data/age_of_agents.db"}
    proc = subprocess.Popen(
        ["/app/target/release/age-of-agents"],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    # Don't call wait() — Modal needs the function to return so the
    # web_server can start proxying. The Rust binary runs in background.
    print(f"Started Rust server (pid={proc.pid})")