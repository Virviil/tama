# Principles

## Tools vs Skills — two levels, not one

**Decision:** the runtime has a fixed set of built-in tools. Users only work with skills.

```
Runtime built-in tools        Skills (written by the user)
─────────────────────         ────────────────────────────
tama_bash       ←───────────── search-web/SKILL.md  "Use tama_bash: duckduckgo-search..."
tama_http_get   ←───────────── fetch-url/SKILL.md   "Use tama_http_get to fetch..."
tama_read_file  ←───────────── skills with tools: [tama_read_file]
tama_write_file
tama_mem_set    ←───────────── mem-set/SKILL.md     tools: [tama_mem_set]
tama_mem_get    ←───────────── mem-get/SKILL.md     tools: [tama_mem_get]
start         ← always on — receive assigned task
finish        ← always on — signal completion, pass value to next agent
read_skill    ← always on — load skill body + unlock its declared tools
```

- **Tool** — a runtime primitive. Users don't create tools.
- **Skill** — an instruction for how to use tools for a specific task. This is the unit of extensibility.

**Analogy with Claude Code:** Claude Code does the same — it has built-in tools (Bash, Read, Write...) and skills are instructions for Claude on how to use them. In tama, `tamad` plays the role of Claude; the built-in tools are equivalent in spirit.

A custom tool is just a skill that calls `bash`:

```markdown
# skills/run-sql/SKILL.md
Use bash: `psql $DATABASE_URL -c "$query"` and return the result.
```

**❌ Anti-pattern:** skills as direct subprocesses (skill = executable).
That requires a separate execution model, skill → tool definition conversion, and environment issues. Bash solves this for free.

**❌ Anti-pattern:** WASM plugin system for custom tools.
Excessive complexity. `bash` is already a universal plugin runtime.

---

## Stateless orchestration, stateful skills

**Decision:** agent orchestration carries no state. State lives exclusively in skills.

```
Orchestration (FSM / scatter / parallel)  →  ordering and routing only
State                                      →  skills only (S3, DB, files)
Runtime                                    →  guarantees ordering, not state
```

Data between agents flows explicitly through `start()`/`finish()` tools — never as hidden parameters injected by the runtime. The orchestration layer only decides *when* agents run and *which* agent runs next.

```markdown
# agents/process-page/AGENT.md  (scatter worker)
Call start() to get your S3 key.
Read the page content using skill read-from-s3.
Process it and save result using skill write-to-s3.
Call finish("done", "summary of what was saved").

# agents/merge-results/AGENT.md  (next FSM agent)
Read all processed results from S3 using skill read-from-s3.
Merge and return final summary.
```

The runtime guarantees one thing: the next agent starts only after the previous one calls `finish`. Routing is based on the `key`. The `value` is made available to the next agent via `start()` — but it's a small handoff string, not a data pipeline. Large data always goes through skills.

**❌ Anti-pattern:** passing large data between agents through `finish` value.
`finish(key, value)` is for task descriptions and short summaries — not file contents or datasets.
Large data belongs in storage (S3, files, DB), accessible via skills.

**❌ Anti-pattern:** passing data between agents as implicit user messages.
The orchestration layer becomes stateful; data is limited by the LLM context window.

### Why this is better than stateful orchestration (e.g. LangGraph)

In LangGraph, state is a first-class object in the graph. Agents know the shared state structure.
In tama, agents **don't know about each other** — they only know about skills.
Orchestration and storage are separate layers with separate responsibilities.

| | LangGraph | tama |
|---|---|---|
| State lives in | the graph (TypedDict) | skills (S3/DB/files) |
| Agents know about | state schema | read/write skills |
| Coupling | agent ↔ state schema | agent ↔ storage skill |
| Scale | limited by process memory | limited by storage |

---

## tama vs Anthropic Agent Skills

Anthropic has its own Agent Skills format (SKILL.md). tama is intentionally compatible with this format — all skills are valid per the Anthropic spec. But the execution model is fundamentally different.

| | Anthropic Skills (API) | tama |
|---|---|---|
| Execution environment | shared code execution VM | dedicated Docker image |
| Python packages | pre-installed in VM only | any, declared in `tama:` |
| apt packages | ❌ unavailable | ✅ via dpkg+ldd discovery |
| Custom binaries | ❌ | ✅ |
| Reproducibility | depends on VM state | deterministic image |
| Isolation | none (shared) | full (separate container) |
| Deploy target | Anthropic platform | any Docker-compatible host |

**Key advantage of tama:** dependencies are declared in the skill and guaranteed to be present in the image. In Anthropic Skills API, `from requests import requests` only works if `requests` is already installed in their VM. You can't add a package — you can't write a skill that requires it.

In tama:
```yaml
tama:
  depends:
    uv:
      - requests>=2.31.0   # guaranteed to be in the image
```

**Compatibility:** tama skills are readable by Claude Code as standard Skills (top-level `name` and `description` fields). The `tama:` field is ignored by Claude Code. The skill works in both contexts.
