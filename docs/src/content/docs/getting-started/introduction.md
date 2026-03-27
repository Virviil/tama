---
title: Introduction
description: What tama is, why it exists, and how it compares to other agent frameworks.
---

**tama** (玉, "jewel", "magic crystal") is a Markdown-native framework for AI agents.

You write agents and skills as `.md` files. `tama` is the developer tool for scaffolding, linting, and building. `tama run` is the runtime that executes them. `tama brew` compiles everything into a production Docker image.

---

## The core idea

Existing agent frameworks make you choose between:

- **Framework code** — assemble graphs and loops by hand in Python
- **Visual editors** — drag-and-drop, hard to version-control, hard to diff

tama takes a different path: **declare your pattern, write your prompts, done.**

```yaml
# agents/essay-critic/AGENT.md
---
name: essay-critic
description: Iteratively drafts and critiques an essay.
version: "1.0.0"
pattern: critic
call:
  model:
    role: writer
---

You are an expert essay writer. Write clear, structured prose.
```

The `critic` pattern automatically runs: draft → critique → refine. You don't implement it. You declare it.

---

## One CLI, four commands

| Command | Purpose |
|---------|---------|
| `tama init` | Scaffold a new project |
| `tama add` | Add agents and skills |
| `tama lint` | Validate your project |
| `tama run` | Execute your agent graph |
| `tama brew` | Build a production Docker image |

```bash
# develop
tama init my-project && tama add react my-agent

# run
ANTHROPIC_API_KEY=sk-... tama run "write a blog post about Rust"

# ship
tama brew && docker push my-project:latest
```

---

## Two entities

The entire system is built from two kinds of thing:

**AGENT.md** — an execution unit. Has a pattern + system prompt. LLM-driven.

**SKILL.md** — a tool. Does one thing. Deterministic. No LLM.

That's it. No other primitives.

---

## Two composition primitives

Under the hood, all 13 patterns are built from exactly two primitives:

| Primitive | Semantics |
|-----------|-----------|
| **FSM** | Sequential steps; routing determined by the `key` returned by each step |
| **Parallel** | Concurrent steps; results collected before proceeding |

The named patterns (`critic`, `reflexion`, `debate`, ...) are pre-wired FSM/Parallel compositions with fixed structure and named prompt files. The `fsm` pattern is the escape hatch when the fixed structures don't fit.

---

## What's next

- [Quickstart](/getting-started/quickstart) — run your first agent in 5 minutes
- [Core Concepts](/getting-started/concepts) — understand patterns, skills, and data flow
- [Patterns overview](/patterns/overview) — all 13 patterns at a glance
