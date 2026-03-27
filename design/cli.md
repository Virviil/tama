# CLI Reference

## tama — compiler (developer machine)

```
tama <command> [args]
```

---

## tama init \<name\>

Creates a new project from scratch.

```bash
tama init my-project
```

Creates:
```
my-project/
├── tama.toml              # project config: name, entrypoint, model roles
├── .env.example           # environment variable template
├── .gitignore
└── agents/
    └── my-project-agent/
        └── AGENT.md       # starter react agent
```

After:
```bash
cd my-project
cp .env.example .env       # add ANTHROPIC_API_KEY
ANTHROPIC_API_KEY=... tamad "hello world"
```

---

## tama add \<pattern\> \<name\>

Scaffolds an agent or skill in the current project.

Names are kebab-case: `my-agent`, `search-web`. Not `myAgent`, not `my_agent`.

### Agents

```bash
tama add react my-agent              # tool-use loop, core primitive
tama add critic essay-critic         # draft → critique → refine
tama add parallel multi-check        # fixed list of agents in parallel
tama add fsm research-flow           # finite state machine with word-based routing
tama add scatter bulk-processor      # map react → parallel workers → reduce react, worker declared in YAML
tama add debate topic-explorer       # position-a → position-b → judge
tama add reflexion code-improver     # act → reflect → loop
tama add constitutional safe-writer  # generate → check principles → revise
tama add chain-of-verification fact-checker  # generate → verify claims → revise
tama add plan-execute task-runner    # plan → execute → verify → loop
tama add best-of-n creative-writer   # N parallel variants → judge picks best
```

Creates `agents/<name>/` directory with AGENT.md and required prompt files.

### Skill

```bash
tama add skill search-web
```

Creates `skills/search-web/SKILL.md` — an Anthropic-compatible tool definition.

---

## tama brew

Builds a Docker image from the current project.

```bash
tama brew
```

- Reads `tama.toml` for project name and entrypoint
- Scans all SKILL.md files to collect dependencies (`tama.depends.apt/uv/bins`)
- Generates a Dockerfile in memory and pipes it via stdin to `docker build`
- Final image: `gcr.io/distroless/cc-debian12` (~8MB)

Requires Docker daemon.

---

## tama lint \<path\>

Checks that an agent has all required files for its pattern.

```bash
tama lint agents/my-agent
tama lint agents/essay-critic
```

Required files by pattern:

| Pattern | Required files |
|---|---|
| `react` | _(none — system prompt is the AGENT.md body)_ |
| `scatter` | _(none — system prompt is the AGENT.md body)_ |
| `critic` | `draft.md`, `critique.md`, `refine.md` |
| `reflexion` | `reflect.md` |
| `constitutional` | `critique.md`, `revise.md` |
| `chain-of-verification` | `verify.md`, `check.md`, `revise.md` |
| `plan-execute` | `execute.md`, `verify.md` |
| `debate` | `position-a.md`, `position-b.md`, `judge.md` |
| `best-of-n` | `judge.md` |
| `parallel`, `fsm` | _(none — everything in AGENT.md)_ |

---

## tama run

Runs the agent in **dev mode** with full trace recording to `.tama/`.

```bash
tama run "research fusion energy trends"
tama run --agent my-agent "task"
```

- Uses entrypoint and model roles from `tama.toml` (same resolution as `tamad`)
- Writes a full trace to `.tama/runs.duckdb` — every step, every artifact
- Streams progress to stderr, final result to stdout
- `.tama/` is gitignored

After a run, analysis commands are available:

```bash
tama runs                          # list recent runs
tama runs show <run-id>            # detailed trace: agents, tokens, timing
tama runs show <run-id> --llm      # include full LLM prompts and responses
tama runs retry <run-id>           # re-run with identical input
```

**Why a separate command from `tamad`:** `tamad` is a minimal binary with no observability,
built for distroless images. `tama run` is a dev tool with DuckDB and tracing.

---

## tama models

Lists available models from configured providers (Anthropic, OpenAI, Google).

```bash
tama models
```

---

## tamad — runtime

`tamad` is a standalone binary. In production it lives inside the Docker image (distroless);
in development it runs directly from the project directory.

```bash
tamad "research fusion energy trends"
tamad "summarize this document"
```

- Takes the task as a positional CLI argument
- Reads entrypoint from `TAMA_ENTRYPOINT_AGENT` or from `tama.toml → project.entrypoint`
- Reads model roles from `[models]` in `tama.toml` or from env (`TAMA_MODEL_THINKER`, etc.)
- Looks for agents in `./agents/<name>/`
- Writes result to stdout
- Runs from the project root (in Docker — `/`, in dev — project directory)

In Docker, `TAMA_ENTRYPOINT_AGENT` is set by `tama brew` and baked into the image.

---

## AGENT.md format

```markdown
---
name: my-agent
description: What this agent does and when to use it.

tama:
  version: 1.0.0
  pattern: react           # one of the 12 patterns
  uses: [search-web]       # skills (react/scatter only)
  model:
    role: thinker          # → TAMA_MODEL_THINKER from .env
    # name: anthropic:claude-opus-4-6  # override
---

System prompt goes here.
For react: this is the full system prompt. Input arrives as the user message.
```

## SKILL.md format

```markdown
---
name: search-web
description: Search the web using DuckDuckGo. Use when the user asks to find information online.
license: MIT
tama:
  depends:
    apt:
      - duckduckgo-cli
    bins:
      - duckduckgo-search
---

Use bash to run `duckduckgo-search "$QUERY"` and return the top results.
```

## tama.toml

```toml
[project]
name = "my-project"
entrypoint = "my-project-agent"  # directory name under agents/

[models]
thinker = "anthropic:claude-opus-4-6"
worker  = "anthropic:claude-sonnet-4-6"
critic  = "anthropic:claude-sonnet-4-6"
```
