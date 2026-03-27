# tama 玉 — Overview

**tama** (玉, "jewel", "magic crystal") — a Markdown-native framework for AI agents.

You write agents and skills as `.md` files. `tama run` gives you full observability while you iterate.
`tama brew` compiles everything into a distroless Docker image for production.
No frameworks. No orchestration runtime to manage. No Python glue.

---

## The Workflow

```bash
# 1. scaffold
tama init my-project
tama add react my-agent
tama add skill search-web

# 2. iterate locally with full observability
tama run "research fusion energy trends"
tama runs show <run-id>          # inspect every step, every artifact
tama runs retry <run-id>         # re-run with identical input

# 3. ship to production
tama brew                        # compiles to distroless Docker image (~8MB)
docker push my-project:latest
```

---

## Why tama

Existing tools (LangGraph, CrewAI, binex) make you choose between:
- **framework code** — you assemble graphs and loops by hand in Python
- **visual editors** — drag-and-drop, hard to version-control, hard to diff

tama takes a different path:

**1. Patterns out of the box**
Agentic patterns (react, scatter, reflexion, debate, ...) built in.
You declare which pattern you need. You don't implement it.

**2. Fast iteration with full observability**
`tama run` records a complete trace — every agent step, every LLM call, every artifact.
Diff two runs. Replay a run with the same input. Understand exactly what happened.

**3. Lean production deploy**
`tama brew` compiles your agents into a distroless image.
Minimal RAM, minimal CPU, fast cold start. One command from laptop to cloud.

---

## Name

**`tama` 玉** — "jewel", "magic crystal". In Japanese alchemy, a concentrated essence.
`tama brew` sounds like casting a spell.

---

## Documentation

| File | Contents |
|------|----------|
| [cli.md](cli.md) | CLI reference: all commands, AGENT.md/SKILL.md formats |
| [primitives.md](primitives.md) | Runtime primitives, FSM, scatter, finish |
| [skills.md](skills.md) | SKILL.md format, progressive disclosure, naming |
| [build.md](build.md) | Rust, distroless, brew, dependencies, tama/tamad |
| [runtime.md](runtime.md) | Runtime modes, serve API, models, secrets |
| [principles.md](principles.md) | tools vs skills, stateless orchestration, vs Anthropic |
| [observability.md](observability.md) | OpenTelemetry tracing, spans, Langfuse, DuckDB |
| [roadmap.md](roadmap.md) | Examples, open questions, what's next |
