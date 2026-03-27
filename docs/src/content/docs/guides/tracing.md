---
title: Tracing & Observability
description: How tama traces every run and how to inspect traces.
---

Every `tamad` run is fully traced. Every agent start/end, every LLM call, every tool call is recorded — including inputs, outputs, token counts, and timing.

## Storage

Traces are stored in `.tama/runs.duckdb` in your project directory. This is a local DuckDB file — no external service required.

```
my-project/
├── .tama/
│   └── runs.duckdb   ← all run traces
├── agents/
└── skills/
```

## Web UI

tama includes a built-in web UI for exploring runs:

```bash
tama view    # opens http://localhost:3000
```

The UI shows:
- List of all runs with status, duration, input, and output
- Agent call tree for each run (with pattern color coding)
- Individual LLM calls with full system prompt, input, output
- Token counts and timing for cost/performance analysis
- Diff view between two runs

## What's traced

Every run creates a tree of spans:

| Span type | Captured fields |
|-----------|----------------|
| `agent` | agent name, pattern, input, output key, output value, duration |
| `llm` | model, system prompt, messages, response, input tokens, output tokens, duration |
| `tool` | tool name, arguments, result |

## Trace context

Each run has a **trace ID** (unique per `tamad` invocation) and each span has a **span ID**. Parent-child relationships are tracked — you can see exactly which LLM call happened inside which agent.

## Querying traces directly

For advanced analysis, query the DuckDB file directly:

```bash
duckdb .tama/runs.duckdb
```

```sql
-- Most recent runs
SELECT trace_id, started_at, duration_ms, input, output
FROM traces
ORDER BY started_at DESC
LIMIT 10;

-- All LLM calls in a specific run
SELECT agent_name, model, input_tokens, output_tokens, duration_ms
FROM spans
WHERE trace_id = 'abc123...'
  AND span_type = 'llm'
ORDER BY started_at;

-- Token usage by agent across all runs
SELECT agent_name,
       SUM(input_tokens) as total_input,
       SUM(output_tokens) as total_output,
       COUNT(*) as call_count
FROM spans
WHERE span_type = 'llm'
GROUP BY agent_name
ORDER BY total_output DESC;
```

## Interactive debugger

The debugger adds observability during development:

```bash
tamad --debug "task input"
```

The debugger pauses:
- **Before each LLM call** — lets you inspect the full context, edit the system prompt, or skip
- **After each agent completes** — lets you approve the output or retry the entire agent

On retry, the agent restarts from scratch. Any side effects are rolled back (tool calls, memory writes).

## Comparing runs

The web UI supports run comparison — select two runs and diff their agent trees, prompt changes, and outputs side by side.

## Privacy

All traces stay local in `.tama/runs.duckdb`. Nothing is sent to any external service. Add `.tama/` to `.gitignore` to keep traces out of version control:

```
# .gitignore
.tama/
```
