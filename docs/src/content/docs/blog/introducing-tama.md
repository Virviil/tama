---
title: Introducing tama — Markdown-native AI agent orchestration
date: 2026-03-26 00:00:00
authors:
  - tama
tags:
  - announcement
  - agents
  - open-source
cover:
  alt: tama — Markdown-native AI agent orchestration
  image: ../../../assets/cover-introducing-tama.png
excerpt: >
  Aren't you tired of picking a language just to experiment with multiagent AI? Of pip installing the same deps over and over? tama is the answer you didn't know you needed.
---

Aren't you pissed off that you have to **choose a programming language** just to experiment with something as simple as a multiagent system?

You want to wire up two agents that talk to each other. That's it. And before you've written a single prompt you're already:

- Picking Python because "that's where the AI libraries are"
- Creating a virtual environment. Again.
- `pip install langchain langgraph anthropic pydantic` — and watching 47 transitive dependencies appear
- Writing a class. Then another class. Then a decorator. Then wondering why the decorator broke your async context
- Fighting a framework that was designed for someone else's problem

And you haven't even written a prompt yet.

---

Are you tired of copy-pasting the same boilerplate every single time? The same `async def run_agent`, the same `StateGraph()`, the same `add_node` / `add_edge` / `compile` / `invoke` ritual — for every toy project, every experiment, every "let me just quickly try something"?

Do you hate that your agent's **entire behavior** is scattered across a Python file, a config dict, a prompt string defined three functions away, and a state type that lives in yet another file — and you need all four open simultaneously just to understand what the thing does?

Does it infuriate you that you can't just open a file and *read* what an agent does? That you have to mentally execute a graph construction function just to visualize the topology?

---

**This is exactly why tama exists.**

tama is a Markdown-native framework for AI agents. Your agents are `.md` files. Your skills are `.md` files. You pick a pattern with one keyword. tama runs it. No language lock-in. No boilerplate. No archaeology.

`tama init`. Write some Markdown. `tamad "do the thing"`. Ship.

That's it.

## Everything is a Markdown file

An agent in tama is an `AGENT.md` file with a frontmatter header and a system prompt:

```yaml
---
name: researcher
description: Searches the web and collects findings on any topic.
version: 1.0.0
pattern: react
call:
  model:
    role: default
  uses:
    - search-web
---

You are a rigorous research assistant. Use the search-web skill to find
primary sources, statistics, and expert opinions on the topic you receive.
Synthesize your findings into a structured brief.

Call start() to receive your research topic, then finish() when done.
```

That's the whole agent. No class inheritance. No decorator chains. No `__init__` methods.

A skill — a tool the agent can use — is equally simple:

```yaml
---
name: search-web
description: Search the web using DuckDuckGo.
tools:
  - tama_search_web
---

Use tama_search_web(query) to search. Prefer precise queries over broad ones.
Run 2–3 searches to triangulate facts before concluding.
```

Two file types. Unlimited composability.

## The routing problem

Every multi-agent framework has a routing problem: how does the system decide which agent runs next?

Most solve it one of two ways:

**Option 1 — write code.** LangGraph gives you conditional edge functions. Agno gives you a `Router(selector=fn)`. You write a Python function for every branch, every cycle, every conditional. Your control flow lives in code alongside your prompts.

**Option 2 — let the LLM decide.** Docker Agent's handoffs, CrewAI's hierarchical mode, AutoGen's group chat — the model decides which agent to delegate to. This works until it doesn't. When it breaks, it breaks silently, mid-pipeline.

tama does neither. The LLM calls `finish(key="billing", value="issue summary")`. The FSM state table maps `"billing"` to `billing-agent`. Deterministically. No code. No LLM making routing decisions.

```yaml
---
name: support
pattern: fsm
initial: triage
states:
  triage:
    - billing: billing-agent
    - technical: tech-agent
    - general: general-agent
  billing-agent:
    - done: ~
    - escalate: triage      # explicit cycle — no prompt engineering needed
  tech-agent: ~
  general-agent: ~
---
```

The triage agent never knows the routing table exists. Its job is to call `finish` with the right key. The runtime does the rest. Cycles, escalation paths, retry loops — all declared, all auditable, all enforced by the Rust runtime before any LLM call is made.

## Patterns, not plumbing

The real insight behind tama is that most multiagent workflows follow a small set of recurring shapes. We identified 13 of them and made each one a keyword:

| Pattern | What it does |
|---------|-------------|
| `react` | Tool-use loop — runs until the model calls `finish` |
| `fsm` | State machine — routing determined by the `key` each agent returns |
| `scatter` | Fan out — same worker runs in parallel on different inputs |
| `parallel` | Fork — different workers run simultaneously on the same input |
| `critic` | Draft → critique → refine |
| `reflexion` | Act → reflect → retry until quality threshold is met |
| `debate` | Two positions argued for N rounds → judge synthesizes |
| `best-of-n` | N variants generated in parallel → judge picks the best |
| `chain-of-verification` | Generate → extract claims → verify each → revise |
| `constitutional` | Generate → critique against principles → revise |
| `plan-execute` | Plan steps (JSON) → execute each → verify completeness |
| `orchestrator` | Decompose task → parallel workers → merge results |
| `oneshot` | Single LLM call — no tools, no loop |

You pick the pattern that matches your problem. tama implements it. You write the prompts.

## Composing into systems

Patterns compose naturally. An FSM connects agents sequentially with conditional routing:

```yaml
---
name: support-pipeline
pattern: fsm
initial: triage
states:
  triage: classify                  # unconditional — always goes to classify
  classify:
    - billing: billing-agent
    - technical: tech-agent
    - general: general-agent
  billing-agent: ~
  tech-agent: ~
  general-agent: ~
---
```

The `parallel` pattern runs different workers on the same input simultaneously:

```yaml
---
name: specialists
pattern: parallel
workers: [activities, hotels, transport, restaurants]
---
```

And you can nest them — an FSM state can itself be a parallel agent, which contains scatter workers, each running a reflexion loop. The patterns compose recursively with no special configuration.

## Skills and progressive disclosure

Skills follow a two-level disclosure model that keeps context windows lean.

Every agent always sees a list of available skill names and descriptions. When the agent decides it needs a skill, it calls `read_skill("search-web")` — this loads the full instructions and unlocks the underlying runtime tools. Agents only load what they use.

This matters at scale. A complex agent with access to 10 skills doesn't pay the token cost of all 10 skill prompts on every turn.

## Data flow: two operations

Agents are stateless. Data flows through exactly two operations:

- **`start()`** — receive input (CLI arg, or the previous agent's `finish` value)
- **`finish(key, value)`** — complete and pass output downstream

The `key` is a routing word that tells the FSM which state to go to next. The `value` is the data passed to the next agent via `start()`.

For shared state across agents — when you need a researcher to write findings that a separate reporter can read — tama provides `mem-set`, `mem-get`, and `mem-append` skills backed by in-process shared memory. No databases, no message queues.

## The toolchain

Three commands cover the full lifecycle:

```bash
# scaffold
tama init my-project
tama add react researcher
tama add oneshot summarizer

# validate
tama lint

# run
ANTHROPIC_API_KEY=sk-... tamad "research the current state of fusion energy"

# ship (coming soon)
tama brew
docker push my-project:latest
```

`tama brew` will compile your entire project — all agents, all skills, all prompt files — into a self-contained Docker image. If your skills use Python, the image includes a distroless Python base with uv-installed deps. If they don't, there's no Python runtime at all — just the `tamad` binary. Only what you actually use. **`tama brew` is on the roadmap and not yet released.**

## Why Markdown?

Several reasons:

**Diffability.** Every change to an agent's behavior is a text diff in version control. You can review agent changes the same way you review code changes — line by line, in a PR.

**Portability.** An `AGENT.md` file is readable by anyone. You can open it in any editor, share it in a GitHub issue, paste it into a discussion. The system prompt isn't buried inside a framework object.

**Composability.** Because agents are just files in a directory, you can copy an agent from one project to another with `cp`. You can publish agents as packages. You can scaffold them with a CLI.

**Separation of concerns.** The runtime logic (how `react` works, how `reflexion` iterates) lives in tama. The domain logic (what to research, how to critique) lives in your Markdown. Neither bleeds into the other.

## What's next

tama is early. The 13 patterns cover the patterns we've seen matter most in practice, but we expect to grow the pattern library as the community identifies new shapes.

The examples directory ships with 23 worked examples ported from real tutorials — from a simple airline customer service FSM to a stock analysis platform with nested scatter workers and shared memory. They're the best way to see what tama looks like at various scales.

Get started:

- [Quickstart](/getting-started/quickstart) — your first agent in 5 minutes
- [Patterns overview](/patterns/overview) — all 13 patterns with examples
- [Examples on GitHub](https://github.com/mlnja/tama/tree/main/examples) — 23 worked examples

We're glad you're here.
