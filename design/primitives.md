# Runtime Primitives

## The two fundamental entities

The entire system is built from exactly two kinds of thing.

### 1. React — the atom

A **react agent** is the irreducible execution unit. It cannot be decomposed further.

It is a loop: call LLM → execute tool → call LLM → ... → `finish(key, value)`.
The loop is driven by the model's tool calls. It terminates when the model calls `finish`.

**Every LLM call in the system happens inside a react atom. There is no other way to invoke a model.**

A react agent with no tools (`uses: []`) degenerates to a single LLM call — but it is still a react atom, not a separate primitive.

### 2. Composition — FSM and Parallel

All other patterns are compositions of react atoms (or of other compositions).
There are exactly two composition primitives:

| Primitive    | Semantics |
|-------------|-----------|
| **FSM**      | Sequential: steps run one after another; routing is determined by the `key` in each `finish(key, value)` call |
| **Parallel** | Concurrent: steps run simultaneously; results are collected before the next step |

These two primitives are sufficient to express any agentic workflow.

---

## How every named pattern maps to the two primitives

The named patterns are not new primitives — they are pre-wired compositions with
fixed structure and named prompt files, so users don't have to write FSM YAML by hand.

| Pattern               | Composition                                                      |
|-----------------------|------------------------------------------------------------------|
| `react`               | **atom** (no composition)                                        |
| `human`               | FSM: react(phase-1) → [human pause] → react(phase-2)            |
| `critic`              | FSM: react(draft) → react(critique) → react(refine)             |
| `reflexion`           | FSM loop: react(act) → react(evaluate) → react(reflect) → loop  |
| `constitutional`      | FSM: react(generate) → react(critique) → react(revise)          |
| `chain-of-verification` | FSM: react(generate) → react(verify) → react(check) → react(revise) |
| `plan-execute`        | FSM loop: react(plan) → react(execute) → react(verify) → loop   |
| `debate`              | FSM: react(position-a) → react(position-b) → react(judge)       |
| `parallel`            | Parallel: fixed list of react agents, same input                 |
| `best-of-n`           | Parallel (N react variants) → FSM: react(judge)                 |
| `scatter`             | FSM: react(map) → Parallel: react(worker)×N → FSM: react(reduce)|
| `fsm`                 | User-defined FSM over named agents                               |

Each "step" inside a named pattern (draft, critique, refine, etc.) is a react atom
running with a specific system prompt loaded from a `.md` file.

**❌ Anti-pattern:** thinking of `critic` as a different kind of thing from `fsm`.
`critic` is a FSM with three fixed states and no routing choices. `fsm` is the
escape hatch when the fixed structure doesn't fit.

---

## Skills and Agents — two distinct concepts

**`SKILL.md`** — a tool. Does one thing. Deterministic. No LLM.
**`AGENT.md`** — an execution unit. Has a pattern + system prompt. LLM-driven.

```
skills/
  search-web/SKILL.md      ← tool: duckduckgo
  fetch-url/SKILL.md       ← tool: http

agents/
  critic/AGENT.md          ← pattern: critic  (draft → critique → refine)
  react-search/AGENT.md    ← pattern: react,  uses: [search-web]
  deep-research/AGENT.md   ← pattern: scatter, worker: react-search
```

---

## Data flow model

Agents are **stateless**. Data flows only through `finish` and `start`.

**Three universal tools available to every react agent:**

```
start()              → returns the task assigned to this agent
finish(key, value)   → signals completion; value is passed to the next step
read_skill(name)     → loads skill body + unlocks its declared tools
```

What `start()` returns depends on where the agent sits:

| Agent position | `start()` returns |
|----------------|-------------------|
| Entry agent | CLI input from the user |
| FSM non-initial state | `value` from the previous agent's `finish` |
| Scatter worker | one item from the map phase's `finish(key="parallel", value='[...]')` |
| Parallel worker | the same input all workers received |

**`finish(key, value)`:**
- `key` — routing word used by FSM to select the next state; ignored in terminal states and parallel workers
- `value` — data passed to the next agent via `start()`

**❌ Anti-pattern:** passing data between agents as user messages or implicit parameters.
Data flows through `finish`/`start` only.

---

## FSM

FSM is the sequential composition primitive. The user-facing `fsm` pattern is the
escape hatch when no named pattern fits — but every named sequential pattern
(`critic`, `reflexion`, `constitutional`, etc.) is a FSM under the hood.

### Syntax

```yaml
tama:
  pattern: fsm
  initial: draft
  states:
    agent-a: agent-b          # unconditional transition
    agent-b:
      - yes: agent-c          # conditional: key "yes" → agent-c
      - no: agent-a           # conditional: key "no"  → loop back
      - "*": error-handler    # catch-all for unknown keys
    agent-c:                  # terminal state — no transitions
    error-handler:
```

### Rules

- **Unconditional** (`agent: next`) — ignores key, always proceeds
- **Conditional** — first match wins
- **`"*"` catch-all** — matches any key with no explicit transition
- **Terminal** — state with no transitions; `value` from `finish` becomes the run output
- **Error** — key didn't match and no `"*"` defined

The agent doesn't know about the FSM structure — it only knows its own system prompt.
The system prompt should tell the agent which keys are valid:
```
Call finish with key="approve" if the result is good, or key="revise" if it needs work.
```

### Example: critic with early exit

The `critic` named pattern has no early exit. FSM can add one:

```yaml
tama:
  pattern: fsm
  initial: draft
  states:
    draft:
      - good-enough: done     # ← impossible in pattern: critic
      - needs-work: critique
    critique: refine
    refine:
      - good-enough: done
      - needs-work: critique  # another round
    done:
```

### Nested FSMs and key propagation

A terminal state is a **transparent pass-through**: it returns the full `AgentOutput`
(key + value) from the last agent that ran, unchanged.

This is what makes nested FSMs work. When an inner FSM is used as a state in an outer FSM,
the outer FSM branches on the key that the inner FSM's last agent produced:

```
reviewer calls finish(key="publish", value="poem")
  → inner FSM routes to terminal `done`
  → done returns AgentOutput { key="publish", value="poem" }   ← key preserved
  → outer FSM receives key="publish" and routes accordingly
```

**Design rule:** every key that an outer FSM needs to branch on must be routable
to a terminal state inside the inner FSM. The terminal's name is irrelevant —
only the fact that it has no transitions matters. A single terminal (`done`) can
serve all exit keys, since it just passes whichever key reached it.

```yaml
# inner FSM — two possible exits, one terminal handles both
states:
  reviewer:
    - publish: done     # exits with key="publish"
    - escalate: done    # exits with key="escalate"
    - revise: reviewer  # loops
  done:                 # terminal — passes key through unchanged

# outer FSM — branches on inner FSM's exit key
states:
  editor:               # ← this is the inner FSM agent
    - publish: approved
    - escalate: human-review
    - revise: poet
  approved:
  human-review:
```

---

## Parallel

Parallel is the concurrent composition primitive. The user-facing `parallel` pattern
runs a fixed heterogeneous list of agents with the same input simultaneously.
`scatter` and `best-of-n` use Parallel internally with a dynamic or homogeneous list.

```yaml
tama:
  pattern: parallel
  workers:
    - finance-agent
    - reputation-agent
```

- All workers receive the same input via `start()`
- Runtime waits for **all** workers to finish before the next step
- Results are collected into a map keyed by agent name:

```json
{
  "finance-agent": "financial analysis...",
  "reputation-agent": "reputation analysis..."
}
```

**`parallel` vs `scatter`:**
- `parallel` — different agents, same input, static list decided at design time
- `scatter` — same agent, different inputs, dynamic list decided by the map react at runtime

---

## Scatter — dynamic parallelism

Scatter is a three-phase composition: FSM(map) → Parallel(workers) → FSM(reduce).

```yaml
tama:
  pattern: scatter
  worker: summarize-page
  uses: [search-web]
```

**Phase 1 — map (react atom):**
System prompt from `AGENT.md` body. The agent analyses the task and calls:
```
finish(key="parallel", value='["item1", "item2", ...]')
```
If `key` is anything other than `"parallel"`, the result is returned directly — no fan-out.

**Phase 2 — parallel:**
The runtime spawns one worker per item. Each worker is a full `run_node` call
on the agent named in `worker:`. Workers run concurrently.

**Phase 3 — reduce (react atom):**
System prompt from `reduce.md`. Receives all worker results and synthesises a final answer.

**Required files:** `reduce.md`

**❌ Anti-pattern (old design):** `parallel_run` as a tool the LLM calls.
The LLM should not name workers at runtime — worker identity is a design decision,
not a runtime decision. The map phase only decides *what* to fan out, not *who* handles it.

---

## Human — human-in-the-loop pause

`human` is a two-phase react composition with a pause between phases.

```yaml
tama:
  pattern: human
  uses: [search-web]
```

Required files: `AGENT.md` body (phase 1), `resume.md` (phase 2).

```
Phase 1: react loop (AGENT.md body)
  → agent prepares context, formulates question
  → finish("channel-id", "Here is the draft, please review: ...")

          ── PAUSE ──  runtime opens channel "channel-id"
          [human responds via stdin (dev) or HTTP POST (prod)]

Phase 2: react loop (resume.md)
  → start() returns the human's response
  → finish("done", "final result")
```

The `key` in phase 1's `finish` is a **channel ID**. Unique IDs enable parallel waits;
repeated IDs queue responses.

---

## Required files per pattern

| Pattern | Required files (beyond AGENT.md) |
|---------|----------------------------------|
| `react` | _(none)_ |
| `scatter` | `reduce.md` |
| `parallel` | _(none)_ |
| `fsm` | _(none)_ |
| `human` | `resume.md` |
| `critic` | `draft.md`, `critique.md`, `refine.md` |
| `reflexion` | `reflect.md` |
| `constitutional` | `critique.md`, `revise.md` |
| `chain-of-verification` | `verify.md`, `check.md`, `revise.md` |
| `plan-execute` | `execute.md`, `verify.md` |
| `debate` | `position-a.md`, `position-b.md`, `judge.md` |
| `best-of-n` | `judge.md` |

---

## Debugger — consequences of the two-entity model

The retry unit in the debugger is the **agent** (react atom or composition root),
not a single LLM call.

| Hook | When | Purpose |
|------|------|---------|
| `before_call` | before each LLM call inside a react atom | step through; optionally edit system prompt |
| `after_call` | after each LLM call | observation only; press Enter to continue |
| `after_agent` | after the full react loop produces `finish()` | decide: proceed or retry the whole atom |

On retry, the react loop restarts from scratch. `before_call` fires again for the
first LLM call — that is the natural place to edit the system prompt.

Parallel workers all run truly concurrently and independently hit `before_call`.
The CLI debugger serialises terminal interaction through a queue — one worker
at a time on screen, the rest waiting — exactly like goroutines in `dlv`.
`join_all` ensures reduce/merge never starts until every worker has been approved.
