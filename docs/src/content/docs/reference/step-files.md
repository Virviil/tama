---
title: Step files
description: Optional .md files that provide system prompts for individual steps in multi-step patterns.
---

Multi-step patterns (`critic`, `reflexion`, `constitutional`, `chain-of-verification`, `plan-execute`, `scatter`, `best-of-n`, `human`) load system prompts from separate `.md` files for each step.

## Location

Step files live alongside `AGENT.md` in the agent directory:

```
agents/essay-critic/
├── AGENT.md      ← pattern: critic
├── draft.md      ← step 1: draft system prompt
├── critique.md   ← step 2: critique system prompt
└── refine.md     ← step 3: refine system prompt
```

## Required files per pattern

| Pattern | Required files |
|---------|---------------|
| `scatter` | `reduce.md` |
| `critic` | `draft.md`, `critique.md`, `refine.md` |
| `reflexion` | `act.md`, `reflect.md` |
| `constitutional` | `critique.md`, `revise.md` |
| `chain-of-verification` | `verify.md`, `check.md`, `revise.md` |
| `plan-execute` | `execute.md`, `verify.md` |
| `best-of-n` | `judge.md` |
| `human` | `resume.md` |

## Format

Step files can be plain Markdown (body only) or include optional YAML frontmatter.

### Plain (no frontmatter)

```markdown
You are a meticulous editor. Evaluate the draft for:
1. Factual accuracy
2. Logical flow
3. Writing quality

Provide specific improvement suggestions.
Call finish with key="done" and your critique as value.
```

### With frontmatter

```markdown
---
call:
  model:
    name: anthropic:claude-opus-4-6
    temperature: 0.1
    max_tokens: 2048
  uses:
    - search-web
---

You are a fact-checker with access to web search.
Verify all factual claims in the draft.
Call finish with key="done" and any corrections as value.
```

## Frontmatter fields

The optional frontmatter supports a `pattern:` field and the same `call:` block as `AGENT.md`.

### `pattern`

Controls how the step runs. Defaults to `oneshot` if omitted.

```yaml
pattern: react     # run as a react loop (can call tools, uses finish() for routing)
pattern: oneshot   # single LLM call, no tools (explicit, same as default)
```

Use `pattern: react` when a step needs tools or when its `finish` key is used for routing
(e.g. the reflector in `reflexion`, or the verifier in `plan-execute`).

### `call.model`

Override the model for this specific step:

```yaml
call:
  model:
    role: fast           # use a cheaper/faster model for this step
    temperature: 0.0     # deterministic
    max_tokens: 512
```

Or specify directly:

```yaml
call:
  model:
    name: anthropic:claude-opus-4-6   # use a more capable model for this step
```

### `call.uses`

Add skills specifically for this step:

```yaml
call:
  uses:
    - search-web
    - fetch-url
```

Skills declared in the step file are available only for that step's react loop.

## Inheritance

If a step file has no frontmatter, it inherits the model from the parent `AGENT.md`'s `call.model`. If the parent has no `call.model` either, the default model from `tama.toml` is used.

Step-level `call.uses` are **additive** — they extend (don't replace) any skills already available.

## `tama lint` validation

`tama lint` checks that all required step files exist:

```bash
tama lint
# ✓ agents/essay-critic: ok
# ✗ agents/my-cov: missing required files: check.md, revise.md
```
