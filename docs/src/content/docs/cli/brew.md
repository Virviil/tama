---
title: tama brew
description: Compile your project into a distroless Docker image for production.
---

`tama brew` compiles your tama project into a lean, production-ready Docker image. It runs `tama lint` first, installs all skill dependencies, and packages everything with the `tamad` runtime binary.

## Usage

```bash
tama brew
tama brew --tag my-project:v1.2.0
```

## What it does

1. **Lint** — runs `tama lint` and exits if any checks fail
2. **Build base** — starts from a distroless base image
3. **Install dependencies** — runs `apt-get install` and `uv pip install` for all skill `depends:`
4. **Copy project files** — copies `agents/`, `skills/`, `tama.toml`
5. **Bundle binary** — includes the `tamad` runtime binary
6. **Tag image** — tags as `<project-name>:latest` (or custom tag)

## Output

A Docker image that runs `tamad` as the entrypoint:

```bash
docker run -e ANTHROPIC_API_KEY=sk-... \
           -e TAMA_MODEL_THINKER=anthropic:claude-opus-4-6 \
           my-project:latest \
           "research fusion energy trends"
```

## Image size

The output image is typically **~8–15MB** because it uses a distroless base (no shell, no apt, no unnecessary tools). This means fast cold starts and a minimal attack surface.

## Configuration

Environment variables are **not** baked into the image. Pass them at runtime:

```bash
docker run \
  -e ANTHROPIC_API_KEY=... \
  -e TAMA_MODEL_THINKER=... \
  my-project:latest \
  "task input"
```

## Custom tag

```bash
tama brew --tag my-project:v2.0.0
tama brew --tag registry.example.com/my-project:latest
```

## Pushing to a registry

```bash
tama brew --tag registry.example.com/my-project:latest
docker push registry.example.com/my-project:latest
```
