# Runtime

## Docker operating modes

**Two modes:**

```bash
# one-shot — default. Run → result → exit. Ideal for CI/CD and scripts.
docker run --env-file .env my-agent "research fusion energy trends"

# serve — HTTP server for production with parallel workflows
docker run --env-file .env -p 8080:8080 my-agent --serve
```

Both modes pass the input string as a user message to the entry agent.
Different transport (CLI arg vs HTTP body) — same semantics.

**Why one-shot as default:** simplest interface. Works in any CI.
Result goes to stdout — can be piped to other commands.

---

## Serve mode: workflow API

In serve mode, each HTTP request starts an isolated workflow instance.
The runtime runs them in parallel via tokio. The string from the request body is injected
as a user message into the entry agent.

```bash
# start a workflow
POST /run
{"task": "research fusion energy trends"}
→ {"workflow_id": "abc123"}

# status and metrics
GET /workflows/abc123
→ {"status": "running", "tokens_used": 1240, "current_agent": "search", "started_at": "..."}

# result (when ready)
GET /workflows/abc123/result
→ {"status": "done", "result": "...", "total_tokens": 4821, "duration_ms": 12400}

# streaming events
GET /workflows/abc123/stream   (SSE)
→ data: {"event": "agent_start", "agent": "search"}
→ data: {"event": "token", "text": "..."}
→ data: {"event": "agent_finish", "agent": "search", "word": "continue"}
→ data: {"event": "done", "result": "..."}
```

**User message — only two places:**

| Place | Source | Recipient |
|-------|--------|-----------|
| serve mode | HTTP request body | entry agent of the workflow |
| scatter | items from map phase's `finish(key="parallel", value='[...]')` | each worker |

Inside FSM, data flows through `finish`/`start`. Large data goes through skills.

**Why HTTP/SSE and not gRPC:**
- SSE works from a browser and curl without a special client
- gRPC requires proto files and client generation

**❌ Anti-pattern:** gRPC as the only option.
Overkill for v1. Hard to test without grpcurl.

**❌ Anti-pattern:** serve mode only, no one-shot.
Then it can't be used in CI/CD and scripts.

---

## Models — roles + override

**Decision:** two-level system.

```yaml
# in AGENT.md
tama:
  model:
    role: thinker     # reads from TAMA_MODEL_THINKER
    # name: anthropic:claude-opus-4-6  # override for a specific agent
```

```bash
# in .env
TAMA_MODEL_THINKER=anthropic:claude-opus-4-6    # expensive, for complex tasks
TAMA_MODEL_WORKER=anthropic:claude-sonnet-4-6   # main workhorse
TAMA_MODEL_CRITIC=anthropic:claude-sonnet-4-6   # for verification
```

**Why:** the user configures roles once. The agent declares which role it needs.
The specific model is a configuration detail, not an architecture detail.

**❌ Anti-pattern:** hardcoding the model in every agent.
Then switching from Claude to GPT requires changing all agents.

---

## Secrets — .env + /run/secrets auto-detection

**Decision:** the Rust runtime automatically looks in two places:
1. Environment variables (from `--env-file .env` or `-e KEY=val`)
2. `/run/secrets/KEY` (Docker secrets for production/swarm)

**For development:**
```bash
docker run --env-file .env my-agent "task"
```

**For production:**
```bash
docker secret create anthropic_key key.txt
docker run --secret anthropic_key my-agent "task"
# runtime reads /run/secrets/anthropic_key automatically
```

**❌ Anti-pattern:** secrets baked into the image.
Secrets in the image = secrets in layer history = leaked on push to registry.
