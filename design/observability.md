# Observability

tama emits OpenTelemetry traces for every run. Any OTLP-compatible backend works out of the box — Langfuse, Jaeger, Honeycomb, Grafana Tempo, etc.

---

## Trace structure

Every run produces one trace. Every agent invocation, LLM call, tool call, and skill load gets its own span. Spans form a hierarchy that mirrors the agent graph.

```
trace: abc123
  span: run                           ← entire run, root span
    span: agent:deep-research         ← agent invocation
      span: llm:decompose             ← LLM call (pattern step)
      span: agent:research-angle      ← sub-agent invocation
        span: llm:react:iter-1        ← react loop iteration
        span: tool:search-web         ← tool execution
        span: skill:search-web        ← skill load (read_skill)
        span: llm:react:iter-2
        span: finish                  ← finish(key, value) call
      span: agent:research-angle      ← parallel instance
        span: llm:react:iter-1
        span: tool:search-web
        span: finish
      span: llm:merge
```

---

## trace_id

Each run has a `trace_id`. It is resolved in priority order:

1. **CLI flag:** `tamad --trace-id abc123 "task"`
2. **A2A header:** W3C `traceparent` header from the incoming request
3. **Auto-generated:** new UUID v4 if none of the above are present

The `trace_id` is propagated through the entire agent graph — all sub-agents, all parallel workers — so every span belongs to the same trace.

---

## Span types and attributes

| Span name | Triggered by | Key attributes |
|-----------|-------------|----------------|
| `run` | start of `tamad` execution | `trace_id`, `entrypoint`, `task` |
| `agent:{name}` | `run_node()` called | `agent.name`, `agent.pattern`, `parent_span_id` |
| `llm:{step}` | LLM API call | `model`, `input_tokens`, `output_tokens`, `duration_ms` |
| `tool:{name}` | tool executed in react loop | `tool.name`, `tool.args`, `duration_ms` |
| `skill:{name}` | `read_skill` called | `skill.name` |
| `finish` | `finish(key, value)` called | `finish.key`, `finish.value_length` |
| `human:pause` | `human` pattern pause | `channel_id` |
| `human:resume` | human response received | `channel_id`, `wait_ms` |

---

## Configuration

Standard OpenTelemetry environment variables:

```bash
# OTLP endpoint (required to enable tracing)
OTEL_EXPORTER_OTLP_ENDPOINT=https://cloud.langfuse.com/api/public/otel

# Auth headers (provider-specific)
OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic <base64>

# Service name shown in the backend
OTEL_SERVICE_NAME=my-tama-project

# Optional: disable tracing entirely
OTEL_SDK_DISABLED=true
```

If `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, tracing is a no-op — zero overhead in production.

---

## Dev mode

In `tama run`, traces are also stored locally in `.tama/runs.duckdb` alongside the run record. This enables `tama runs show`, `tama runs diff`, and `tama runs replay` without any external backend.

The DuckDB trace is always written in dev mode regardless of OTLP configuration.

---

## Dev run storage — `.tama/runs.duckdb`

In `tama run`, every run is fully persisted to `.tama/runs.duckdb`. This is the source of truth for local dev tooling — independent of any OTLP backend.

### What gets stored

**Per run:**
- `trace_id`, `timestamp`, `entrypoint`, `task` (original input), `status`, `duration_ms`

**Per span:**
- `span_id`, `parent_span_id`, `trace_id`, `name`, `start_time`, `end_time`

**Per LLM call** (enables replay):
- Full system prompt, full messages array (input)
- Full LLM response (output)
- `model`, `input_tokens`, `output_tokens`

**Per tool call** (enables replay):
- `tool_name`, `args` (input)
- `result` (output)

**Per finish:**
- `key`, `value`

Storing full inputs/outputs is what makes **replay** possible — the runtime can short-circuit LLM calls and return cached responses instead.

### `tama runs` commands

```bash
tama runs                            # list recent runs: id, timestamp, status, duration, task preview
tama runs show <run-id>              # full trace tree: all spans with timing and token counts
tama runs show <run-id> --llm        # include full LLM inputs/outputs
tama runs diff <run-id> <run-id>     # side-by-side: which spans changed, token delta, output diff
tama runs retry <run-id>             # re-run with the same task input (fresh LLM calls)
tama runs replay <run-id>            # re-run with cached LLM responses — deterministic, free
tama runs replay <run-id> --from <span-id>  # replay from a specific span, live LLM after that
```

### retry vs replay

| | `retry` | `replay` |
|---|---------|---------|
| Input | same task string | same task string |
| LLM calls | fresh (costs tokens) | cached (free, deterministic) |
| Tool calls | fresh execution | fresh execution |
| Use case | "run again with same input" | "debug exactly what happened" |

`replay --from <span-id>` is the most powerful: cached responses up to a point, then live LLM from there. Useful for fixing a specific step without re-running the whole pipeline.

---

## Implementation

**Rust crates:**
- `opentelemetry` — core API and SDK
- `opentelemetry-otlp` — OTLP exporter (HTTP/gRPC)
- `opentelemetry_sdk` — tracer provider, batch exporter

**Context propagation:**
The current `trace: &mut Vec<TraceEntry>` is replaced by an OTel `Context` threaded through `run_node` and all pattern functions. Each function creates a child span from the parent context.

```rust
// run_node receives and propagates OTel context
pub fn run_node<'a>(
    graph: &'a AgentGraph,
    name: &'a str,
    client: &'a LlmClient,
    task: &'a str,
    ctx: &'a Context,     // ← OTel context replaces Vec<TraceEntry>
) -> Pin<Box<dyn Future<Output = Result<AgentOutput>> + 'a>>
```

**❌ Anti-pattern:** keeping `Vec<TraceEntry>` and mapping to OTel after the fact.
Spans need start/end times captured at the moment of execution, not reconstructed later.
