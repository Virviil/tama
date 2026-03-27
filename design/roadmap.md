# Roadmap

## Examples

| # | Project | Patterns used |
|---|---------|---------------|
| 01 | `react-search` | react + search-web skill |
| 02 | `deep-research` | scatter (react map → parallel workers → react reduce) |
| 03 | `poem-pipeline` | nested FSMs, parallel, scatter, react, mem-set/mem-get skills |

---

## What's implemented

- All 12 patterns: react, fsm, parallel, scatter, critic, reflexion, constitutional, chain-of-verification, plan-execute, debate, best-of-n, orchestrator
- Progressive skill disclosure: Level 1 (description in system prompt) + Level 2 (`read_skill` loads body + unlocks tools)
- Tool-unlock via `tools:` field in SKILL.md
- Step-through debugger with queue-based parallel serialisation
- `after_agent` hook with retry
- OpenTelemetry tracer + DuckDB tracer
- `tama init`, `tama add`, `tama lint`, `tama brew`, `tama run`, `tama runs`

---

## Open questions

- **Skill registry:** where to host? GitHub Releases? Separate server? `tama.dev`?
- **Skill versioning:** how to resolve conflicts when two skills require different versions of the same pip package?
- **`tama runs diff`:** compare two runs side by side
- **Serve mode:** HTTP server for production (`tamad --serve`)
- **Artifact lineage:** track which output came from which agent + run
