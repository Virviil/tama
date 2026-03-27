---
title: Core Concepts
description: Agents, skills, patterns, and how data flows through a tama project.
---

## Agents and Skills

Everything in tama is built from two kinds of file:

| File | What it is |
|------|-----------|
| `AGENT.md` | An execution unit. Has a pattern + system prompt. LLM-driven. |
| `SKILL.md` | A tool. Does one thing. Deterministic. No LLM. |

```
my-project/
├── agents/
│   ├── researcher/AGENT.md      ← LLM-driven, pattern: react
│   └── summarizer/AGENT.md      ← LLM-driven, pattern: oneshot
└── skills/
    ├── search-web/SKILL.md      ← tool: DuckDuckGo search
    └── fetch-url/SKILL.md       ← tool: HTTP fetch
```

## Patterns

A **pattern** describes the control flow of an agent. You declare the pattern in `AGENT.md` frontmatter — tama implements it.

There are 13 patterns, from the simplest:

| Pattern | What it does |
|---------|-------------|
| `oneshot` | Single LLM call. Input → LLM → output. |
| `react` | Tool-use loop. Runs until the model calls `finish`. |

...to compositions:

| Pattern | What it does |
|---------|-------------|
| `critic` | draft → critique → refine |
| `scatter` | map → parallel workers → reduce |
| `debate` | position-a → position-b → judge |
| `fsm` | User-defined state machine |

See [Patterns overview](/patterns/overview) for the full list.

## Data flow

Agents are **stateless**. Data flows only through two mechanisms:

### `start()` — receive input

Every agent's first action is to call `start()` to receive its input. What `start()` returns depends on where the agent sits:

| Agent position | `start()` returns |
|----------------|------------------|
| Entry agent | The CLI input from `tama run "..."` |
| FSM non-initial state | `value` from the previous agent's `finish` |
| Scatter worker | One item from the map phase's output array |
| Parallel worker | The same input all workers received |

### `finish(key, value)` — send output

To complete, an agent calls `finish(key, value)`:

- `key` — routing word used by FSM to select the next state
- `value` — the data passed to the next agent via `start()`

```
agent-a calls: finish(key="approved", value="Here is the result...")
                      ↓
FSM routes "approved" → agent-b
                      ↓
agent-b calls: start() → "Here is the result..."
```

:::note
`oneshot` agents don't call `finish` explicitly — the runtime handles it automatically. The LLM response becomes the `value` and the key is always `"result"`.
:::

## Skills and progressive disclosure

Skills follow a two-level disclosure model:

**Level 1** — always visible: the skill's `name` and `description` are injected into the system prompt. The agent sees what's available.

**Level 2** — on demand: the agent calls `read_skill("skill-name")` to load the full instructions. This also unlocks any built-in runtime tools the skill declares.

This keeps the context window lean — agents only load the full instructions for skills they actually decide to use.

## The three universal tools

Every `react` agent always has access to three tools:

| Tool | Purpose |
|------|---------|
| `start()` | Get the input assigned to this agent |
| `finish(key, value)` | Signal completion and pass output |
| `read_skill(name)` | Load a skill's full instructions and unlock its tools |

Additional tools come exclusively through skills. Tools are never given to agents by default.

## Entrypoint

When you run `tama run "task"`, it looks up the `entrypoint` in `tama.toml` and starts there:

```toml
[project]
name = "my-project"
entrypoint = "researcher"
```

Override at runtime:

```bash
TAMA_ENTRYPOINT_AGENT=summarizer tama run "summarize this text..."
```

## Models and roles

Agents reference models by **role**, not by name:

```yaml
call:
  model:
    role: thinker
```

Roles map to actual models via environment variables:

```bash
export TAMA_MODEL_THINKER=anthropic:claude-opus-4-6
```

This decouples agent definitions from model choices — swap models without editing any agent files. You can also override a specific agent:

```yaml
call:
  model:
    name: anthropic:claude-haiku-4-5   # direct override
    temperature: 0.3
    max_tokens: 1024
```
