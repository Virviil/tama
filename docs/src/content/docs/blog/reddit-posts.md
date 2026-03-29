---
title: Reddit launch posts
description: Tailored posts for 10 Reddit communities. Do not publish as-is — review and personalize before posting.
date: 2026-03-29
draft: true
---

## Posting order

r/rust → r/LocalLLaMA → r/SideProject → others.
Wait for traction on each before moving to the next.

**Rules for all posts:**
- Do NOT mention `tama brew` / Docker — not yet released
- Always link to https://github.com/mlnja/tama
- Engage in comments — answer questions, don't just drop the link
- Lead with FSM / "LLM produces content, runtime owns control flow"

---

## 1. r/rust

**Title:** I built an AI agent runtime in Rust where routing is a state machine, not a DAG

**Body:**

Every multi-agent framework I tried has the same problem: routing is either Python conditional functions (LangGraph) or the LLM itself decides which agent to call next (Docker Agent, CrewAI hierarchical). Both are painful — one requires code for every branch, the other is non-deterministic.

So I built tama — agents are Markdown files, and routing is a real FSM.

The LLM calls `finish(key="billing", value="...")`. The runtime maps `"billing"` to the next state via a declared table. The LLM never decides which agent to call. Control flow is owned by the YAML, not the model.

```yaml
---
name: support
pattern: fsm
initial: triage
states:
  triage:
    - billing: billing-agent
    - technical: tech-agent
  billing-agent:
    - done: ~
    - escalate: triage     # cycle — declared, not hoped for
  tech-agent: ~
---
```

This is the airline customer service example from OpenAI's Swarm tutorial — ported to 4 Markdown files with zero Python.

The runtime (`tamad`) is a single Rust binary. Agents are stateless. I've implemented 12 patterns: fsm, react, scatter, parallel, reflexion, critic, debate, best-of-n, constitutional, chain-of-verification, plan-execute, oneshot. 23 worked examples in the repo.

GitHub: https://github.com/mlnja/tama

Curious what the Rust community thinks about the FSM-based routing approach specifically.

---

## 2. r/LocalLLaMA

**Title:** Tired of LLM-driven routing in agent frameworks? I built a real FSM for multi-agent systems

**Body:**

The problem with most agent frameworks: either you write Python to route between agents, or you let the LLM decide where to go next. The first requires code for every branch. The second means your control flow is only as reliable as your prompt engineering.

tama uses a state machine instead. The LLM returns a routing word. The YAML table maps it to the next state. No code. No LLM choosing which agent to call.

```yaml
---
name: pipeline
pattern: fsm
initial: triage
states:
  triage:
    - billing: billing-agent
    - technical: tech-agent
    - general: general-agent
  billing-agent:
    - done: ~
    - escalate: triage
  tech-agent: ~
  general-agent: ~
---
```

Every agent calls `finish(key="billing", value="issue summary")`. The FSM routes deterministically. Cycles, escalation paths, retry loops — all declared in YAML, none requiring code or relying on LLM routing decisions.

Beyond FSM: 12 patterns total. reflexion (act → reflect → loop), debate (two positions + judge), best-of-n (N parallel variants + judge picks), scatter (map → parallel workers → reduce). All declared in one line, no code.

Agents are Markdown files. Runtime is a Rust binary. 23 examples. Works with Anthropic, OpenAI, Google.

GitHub: https://github.com/mlnja/tama

---

## 3. r/MachineLearning

**Title:** tama: FSM-based multi-agent routing — the LLM produces content, the runtime owns control flow

**Body:**

Most multi-agent frameworks have a routing problem. LangGraph solves it with Python conditional edge functions. Docker Agent solves it by letting the LLM decide. CrewAI's hierarchical mode delegates to a manager LLM.

All three put routing in the wrong place — either in code that needs to be written and maintained, or in the LLM where it's non-deterministic.

tama takes a different approach: the agent outputs a routing word, the FSM state table maps it deterministically. The LLM produces content. The runtime owns control flow.

```yaml
pattern: fsm
states:
  classify:
    - approved: publish
    - needs-revision: revise
    - reject: ~
  revise:
    - done: classify     # loops back
  publish: ~
```

This composes with 12 other patterns. A scatter worker can internally run a reflexion loop. A debate agent feeds into an FSM pipeline. Composition is recursive — each agent in the graph declares its own pattern independently.

| Pattern | What it does |
|---------|-------------|
| `fsm` | Declared state machine, LLM routing words → deterministic transitions |
| `reflexion` | act → reflect → retry until DONE |
| `debate` | N rounds, two positions, judge synthesizes |
| `best-of-n` | N parallel variants → judge picks best |
| `constitutional` | generate → critique against principles → revise |
| `chain-of-verification` | generate claims → verify each → revise |
| `scatter` | map → parallel workers → reduce |

Rust runtime, local-first, 23 runnable examples. GitHub: https://github.com/mlnja/tama

---

## 4. r/programming

**Title:** What if multi-agent routing was a state machine declared in YAML, not Python code or LLM decisions?

**Body:**

Every multi-agent framework I've used puts routing in the wrong place.

LangGraph: write a `def should_continue(state)` Python function for every branch. Fine if you know Python, painful if you're iterating on prompts.

Docker Agent / CrewAI hierarchical: the LLM decides which agent to call next. Non-deterministic. Your control flow is only as reliable as your instruction engineering.

I wanted something different: declare the state machine in YAML, let the LLM return routing words, have the runtime route deterministically.

```yaml
---
name: review-pipeline
pattern: fsm
initial: draft
states:
  draft:
    - ready: review
  review:
    - approved: publish
    - revise: draft        # sends back to drafting
  publish: ~
---
```

The draft agent calls `finish(key="ready", value="my essay")`. The runtime routes to `review`. The review agent calls `finish(key="revise", value="feedback")`. The runtime routes back to `draft`. No Python. No LLM deciding the routing.

This is tama — agents as Markdown files, routing as a state machine. 12 patterns, 23 examples, Rust runtime.

GitHub: https://github.com/mlnja/tama

---

## 5. r/SideProject

**Title:** I built tama — multi-agent AI where routing is a state machine, not code or LLM decisions

**Body:**

I've built a few multi-agent projects and hit the same wall every time: routing between agents is either code I have to write and maintain, or the LLM deciding where to go next (which works until it doesn't).

So I built tama. The core idea: agents are Markdown files, and routing is a real FSM.

The LLM returns a word. The YAML table routes deterministically. Here's the airline customer service example from OpenAI's Swarm tutorial — the one they use to showcase their framework — ported to 4 Markdown files:

```
triage agent    →  calls finish(key="billing")
FSM routes      →  billing-agent
billing-agent   →  calls finish(key="escalate")
FSM routes      →  triage (cycle back)
```

The whole routing table is 8 lines of YAML. No Python. No "please route to the right agent" instruction engineering.

Beyond FSM: 12 patterns total (reflexion, debate, scatter, best-of-n, etc.), 23 worked examples, Rust binary, works with Anthropic/OpenAI/Google.

GitHub: https://github.com/mlnja/tama

Happy to answer questions about the Rust implementation or the FSM design specifically.

---

## 6. r/Python

**Title:** I got tired of writing Python routing functions for AI agents — so I built a state machine in YAML

**Body:**

Not here to dunk on Python. But for multi-agent routing specifically, I kept writing the same thing:

```python
def should_continue(state: State) -> Literal["billing", "technical", "end"]:
    last_msg = state["messages"][-1]
    if "billing" in last_msg.content.lower():
        return "billing"
    ...
```

Every branch is a function. Every cycle is a conditional. And if I want to change the routing topology I'm editing Python, not the agent logic.

I built tama to make routing declarative. The LLM returns a routing word from `finish(key="billing")`. A YAML state table maps it to the next agent. No Python functions for control flow.

```yaml
states:
  triage:
    - billing: billing-agent
    - technical: tech-agent
  billing-agent:
    - done: ~
    - escalate: triage
```

Beyond routing: 12 patterns, agents as Markdown files, Rust runtime, no Python required to run. But if your skills use Python — file processing, data work — the build system supports distroless Python images with uv-installed deps.

GitHub: https://github.com/mlnja/tama

Genuinely curious: do people find LangGraph's conditional edge functions annoying, or is that not the pain point for you?

---

## 7. r/learnmachinelearning

**Title:** The two hardest things in multi-agent AI — routing and composition — are easier than you think

**Body:**

If you're learning to build multi-agent systems, two things will frustrate you quickly:

**Routing** — how does one agent decide which agent to call next? Most frameworks give you either Python code or an LLM making the routing decision. Both are harder than they should be.

**Composition** — how do you wire agents together without it becoming a tangled graph? Most frameworks give you nodes and edges that you assemble by hand.

tama solves routing with a state machine. The LLM returns a routing word; a YAML table maps it to the next state. No code, no LLM deciding where to go.

```yaml
pattern: fsm
states:
  triage:
    - billing: billing-agent   # LLM says "billing" → goes here
    - technical: tech-agent    # LLM says "technical" → goes here
  billing-agent:
    - escalate: triage         # can loop back
```

tama solves composition with 12 named patterns. `pattern: reflexion` means act → reflect → retry until DONE. `pattern: debate` means two positions argued for N rounds, a judge synthesizes. You declare the pattern; the runtime implements it.

There are 23 runnable examples in the repo covering every pattern from simple ReAct to a 4-analyst stock analysis system. Each example is just Markdown files — you can read exactly what every agent does.

GitHub: https://github.com/mlnja/tama

---

## 8. r/artificial

**Title:** tama — multi-agent AI with a real state machine, not LLM-driven routing

**Body:**

Most multi-agent frameworks let the LLM decide where to go next. The supervisor chooses which agent to delegate to. The router LLM picks the next step. This works until it doesn't — and when it breaks, it breaks silently.

tama separates content from control flow. The LLM produces content and returns a routing word. The state machine routes deterministically. No LLM ever decides "which agent should handle this."

```yaml
---
name: debate
pattern: debate
agents: [pro, con]
rounds: 2
judge: judge
---
```

```bash
tamad "Resolved: remote work is more productive than office work"
```

Pro and con argue for 2 rounds. The judge synthesizes a nuanced verdict. The routing between rounds is handled by the runtime — no LLM decides "it's time for round 2."

12 patterns, all declared in one line. 23 examples. Markdown files, Rust runtime, works with Claude/GPT-4/Gemini.

GitHub: https://github.com/mlnja/tama

---

## 9. r/ChatGPT

**Title:** Built a framework where you describe AI workflows in Markdown — routing is automatic, no code required

**Body:**

Wanted to share something I built for people who want to chain AI agents together without writing code.

The problem I kept hitting: if you want agent A to pass to agent B under condition X, most tools require either Python code or hoping the LLM routes correctly.

tama works differently. You declare a state machine in YAML — "if the agent says billing, go to billing-agent; if it says escalate, loop back to triage." The agent just says a word. The routing is automatic.

Example — a writing pipeline:

```yaml
---
pattern: fsm
initial: draft
states:
  draft:
    - ready: review
  review:
    - approved: publish
    - revise: draft
  publish: ~
---
```

The draft agent writes something and says "ready." The review agent reads it and says "revise" or "approved." The whole loop is declared in the YAML — no code, no prompt engineering the routing.

Works with Claude and GPT-4. 23 examples: research, writing, debate, stock analysis, customer service. Run locally with just an API key.

GitHub: https://github.com/mlnja/tama

---

## 10. r/compsci

**Title:** Multi-agent control flow as a Mealy machine — LLM outputs (routing word, value), FSM transitions deterministically

**Body:**

Most multi-agent frameworks implement routing as a DAG with either code-defined edge functions (LangGraph) or LLM-driven edge selection (Docker Agent handoffs, CrewAI hierarchical). Both have problems — code coupling for the former, non-determinism for the latter.

tama models agent routing as a Mealy machine:

- **States**: named states in a YAML table, one `.md` file per state containing the LLM system prompt
- **Input**: the routing word (`key`) returned by the current state's LLM call via `finish(key, value)`
- **Output**: the value passed as input to the next state
- **Transition function**: declared in YAML frontmatter — deterministic mapping from (current_state, key) → next_state

```yaml
pattern: fsm
states:
  triage:
    - billing: billing-agent   # δ(triage, "billing") = billing-agent
    - technical: tech-agent    # δ(triage, "technical") = tech-agent
  billing-agent:
    - done: ~
    - escalate: triage         # cycle — well-defined, not a DAG
```

The LLM in each state is constrained to producing a (key, value) pair. It never selects a transition — it only outputs content. The transition function is fully declared and validated before execution begins (`AgentGraph::build()` in Rust).

This composes recursively: an FSM state can itself be a `scatter` node (parallel map), which fans out to workers running `reflexion` loops (cyclic FSM), which each run a `react` loop (tool-use FSM). The full composition graph is parsed and type-checked statically.

Two primitives — FSM and Parallel — cover all 12 patterns. Named patterns (reflexion, critic, debate, etc.) are pre-wired FSM/Parallel compositions. The `fsm` pattern is the escape hatch for arbitrary user-defined machines.

Rust runtime, 23 examples: https://github.com/mlnja/tama
