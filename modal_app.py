"""Age of Agents — Modal deployment entrypoint.

Run locally:  modal serve modal_app.py
Deploy:       modal deploy modal_app.py
"""

import modal

from backend.server import web_app

app = modal.App("age-of-agents")

image = modal.Image.debian_slim().pip_install(
    "fastapi>=0.110",
    "uvicorn[standard]>=0.27",
    "websockets>=12.0",
    "pydantic>=2.0",
)


@app.function(
    image=image,
    allow_concurrent_inputs=20,
    mounts=[
        modal.Mount.from_local_dir(
            local_path="./frontend",
            remote_path="/root/frontend",
        ),
    ],
    secrets=[
        modal.Secret.from_name("openrouter-api-key"),
    ],
    memory=1024,
)
@modal.asgi_app()
def fastapi_app():
    return web_app