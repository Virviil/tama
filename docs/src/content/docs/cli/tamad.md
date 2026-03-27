---
title: tamad (runtime)
description: The runtime binary that executes your agent graph.
---

`tamad` is the runtime binary that executes your agent graph. It reads `tama.toml`, builds the agent graph, and runs the entrypoint agent with the provided input.

## Usage

```bash
tamad "<task input>"
tamad --debug "<task input>"
```

Run from the project root (where `tama.toml` lives).

## Arguments

| Argument | Description |
|----------|-------------|
| `<task>` | The input string passed to the entrypoint agent's `start()` |

## Flags

| Flag | Description |
|------|-------------|
| `--debug` | Enable interactive step-through debugger |

## Environment variables

| Variable | Description |
|----------|-------------|
| `TAMA_ENTRYPOINT_AGENT` | Override the entrypoint from `tama.toml` |
| `TAMA_MODEL_<ROLE>` | Set model for a role (e.g. `TAMA_MODEL_THINKER`) |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `GEMINI_API_KEY` | Google API key |

## Output

`tamad` writes the final agent output to stdout. All trace/debug output goes to stderr.

```bash
# capture output only
tamad "research fusion energy" > output.txt

# see trace output in real time
tamad "research fusion energy" 2>&1 | tee full-output.txt
```

## Interactive debugger

```bash
tamad --debug "research fusion energy"
```

The debugger pauses before each LLM call and lets you:
- Inspect the current input/context
- Edit the system prompt before the call
- Press Enter to continue
- After each agent completes: proceed or retry the entire agent

## Difference from `tama`

| | `tama` | `tamad` |
|--|--------|---------|
| Purpose | Developer tool | Runtime |
| Commands | `init`, `add`, `lint`, `brew` | _(takes task as argument)_ |
| Runs agents? | No | Yes |
| In Docker image? | No | Yes |

`tamad` is the binary that runs in production Docker images. `tama` stays on your development machine.

## Tracing

Every `tamad` run produces a trace in `.tama/runs.duckdb`. View runs with the web UI:

```bash
tama runs       # not yet implemented — use web UI
```

Or query directly:

```sql
duckdb .tama/runs.duckdb
SELECT * FROM spans ORDER BY started_at DESC LIMIT 20;
```

See [Tracing & Observability](/guides/tracing) for details.
