---
title: AGENT.md format
description: Complete reference for AGENT.md frontmatter fields and body.
---

`AGENT.md` defines an agent: its pattern, model configuration, and system prompt.

## File location

```
agents/{agent-name}/AGENT.md
```

The directory name must match the `name` field. Only lowercase letters, digits, and hyphens.

## Full example

```yaml
---
name: researcher
description: Research agent with web search capabilities.
version: "1.0.0"
pattern: react
max_iter: 15
env:
  - SEARCH_API_KEY
call:
  model:
    role: thinker
    temperature: 0.3
    max_tokens: 4096
  uses:
    - search-web
    - fetch-url
---

You are a thorough research assistant.

Given a research task, use search-web to find information and fetch-url to read sources.
Build a comprehensive answer with citations.

When complete, call finish with key="done" and your full research report as value.
```

## Frontmatter fields

### `name` (required)

```yaml
name: my-agent
```

- Lowercase letters, digits, and hyphens only
- Must match the directory name
- Used in logs, traces, and FSM routing

### `description` (required)

```yaml
description: A short description of what this agent does.
```

- Shown in `tama view` and the web UI
- Should describe the agent's purpose in 1-2 sentences

### `version` (required)

```yaml
version: "1.0.0"
```

Semantic version string. Used for tracking changes.

### `pattern` (required)

```yaml
pattern: react
```

One of: `oneshot`, `react`, `scatter`, `parallel`, `fsm`, `critic`, `reflexion`, `constitutional`, `chain-of-verification`, `plan-execute`, `debate`, `best-of-n`, `human`

See [Patterns overview](/patterns/overview) for details.

### Pattern-specific fields

| Pattern | Additional required fields |
|---------|--------------------------|
| `scatter` | `worker: agent-name` |
| `parallel` | `workers: [agent-a, agent-b, ...]` |
| `fsm` | `initial: state-name`, `states: {...}` |
| `debate` | `agents: [...]`, `rounds: N`, `judge: agent-name` |
| `best-of-n` | `n: N` |

### `max_iter` (optional)

```yaml
max_iter: 20
```

Maximum loop iterations for `react`, `reflexion`, `plan-execute`. Defaults:
- `react`: 10
- `reflexion`: 4

### `env` (optional)

```yaml
env:
  - API_KEY
  - SECRET_TOKEN
```

List of environment variables this agent requires. Used by `tama lint` to validate the runtime environment.

### `call` (optional)

Groups model configuration and skill declarations:

```yaml
call:
  model:
    role: thinker          # role-based selection
    # OR
    name: anthropic:claude-opus-4-6   # direct override
    temperature: 0.7       # 0.0–1.0
    max_tokens: 2048
  uses:
    - search-web
    - fetch-url
```

#### `call.model`

| Field | Description |
|-------|-------------|
| `role` | Role name → reads `TAMA_MODEL_{ROLE}` env var |
| `name` | Direct `provider:model-id` override (takes priority over `role`) |
| `temperature` | Sampling temperature (0.0–1.0) |
| `max_tokens` | Maximum output tokens |

If `call.model` is omitted, the model from `tama.toml` `[models]` section or `TAMA_MODEL_DEFAULT` is used.

#### `call.uses`

List of skill names to make available to this agent. Skills must exist in `skills/`.

## Body

The text after the `---` closing delimiter is the system prompt. This is the agent's instructions — the role it plays and how it should behave.

```yaml
---
name: summarizer
pattern: oneshot
call:
  model:
    role: thinker
---

You are a concise summarizer. Given any text, return a 2-3 sentence summary.
```

For patterns with multiple steps (`critic`, `constitutional`, etc.), the body is used for the main/first step. Other steps load their prompts from separate `.md` files. For `reflexion`, the body is unused — the actor prompt lives in `act.md`.

## FSM states syntax

```yaml
pattern: fsm
initial: draft
states:
  draft:
    - good-enough: done     # conditional: key "good-enough" → done
    - needs-work: critique  # conditional: key "needs-work" → critique
  critique: refine           # unconditional: always → refine
  refine:
    - good-enough: done
    - needs-work: critique
    - "*": error-handler    # catch-all
  done:                      # terminal (no value)
  error-handler:
```
