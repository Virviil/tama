---
title: Deploying with brew
description: How to package and deploy a tama project to production using tama brew.
---

`tama brew` compiles your project into a lean Docker image. The image contains only the `tamad` runtime, your agent and skill files, and the installed skill dependencies.

## Build the image

```bash
tama brew
# or with a custom tag
tama brew --tag registry.example.com/my-project:v1.0.0
```

## Run it

```bash
docker run \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  -e TAMA_MODEL_THINKER=anthropic:claude-opus-4-6 \
  my-project:latest \
  "research fusion energy trends"
```

The container exits when the agent finishes. The final output is written to stdout. Trace data is written to stderr.

## Push to a registry

```bash
tama brew --tag registry.example.com/my-project:v1.0.0
docker push registry.example.com/my-project:v1.0.0
```

## Kubernetes example

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: research-job
spec:
  template:
    spec:
      containers:
        - name: tamad
          image: registry.example.com/my-project:v1.0.0
          args: ["research fusion energy trends"]
          env:
            - name: ANTHROPIC_API_KEY
              valueFrom:
                secretKeyRef:
                  name: api-keys
                  key: anthropic
            - name: TAMA_MODEL_THINKER
              value: anthropic:claude-sonnet-4-6
      restartPolicy: Never
```

## Image size

The distroless base keeps images small:

| Component | Size |
|-----------|------|
| Distroless base | ~2MB |
| tamad binary | ~5MB |
| Agent/skill files | ~1MB (typical) |
| Skill dependencies | varies |
| **Total (no deps)** | **~8MB** |

## Environment variable management

Never bake API keys into the image. Pass them at runtime:

**Docker:**
```bash
docker run -e ANTHROPIC_API_KEY=... my-project:latest "task"
```

**Docker Compose:**
```yaml
services:
  agent:
    image: my-project:latest
    environment:
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - TAMA_MODEL_THINKER=anthropic:claude-sonnet-4-6
```

**Kubernetes secrets:**
```yaml
env:
  - name: ANTHROPIC_API_KEY
    valueFrom:
      secretKeyRef:
        name: api-keys
        key: anthropic
```

## Entrypoint override

Override the entrypoint without rebuilding:

```bash
docker run \
  -e TAMA_ENTRYPOINT_AGENT=summarizer \
  -e ANTHROPIC_API_KEY=... \
  my-project:latest \
  "the text to summarize..."
```

## CI/CD pipeline

```yaml
# .github/workflows/deploy.yml
name: Build and Deploy

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install tama
        run: cargo install tama  # or download binary

      - name: Lint
        run: tama lint

      - name: Build image
        run: tama brew --tag ghcr.io/${{ github.repository }}:${{ github.sha }}

      - name: Push
        run: docker push ghcr.io/${{ github.repository }}:${{ github.sha }}
```
