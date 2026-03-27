---
title: Patterns Overview
description: All 13 tama patterns at a glance — when to use each one.
---

tama has 13 built-in patterns. Every pattern is a pre-wired composition of LLM calls; you declare the pattern and write the prompts — tama handles the control flow.

## Quick reference

| Pattern | Complexity | Required files | Best for |
|---------|-----------|----------------|---------|
| [`oneshot`](/patterns/oneshot) | ⬤ | _(none)_ | Simple transformations, classification, formatting |
| [`react`](/patterns/react) | ⬤⬤ | _(none)_ | Tool use, research, multi-step reasoning |
| [`scatter`](/patterns/scatter) | ⬤⬤⬤ | `reduce.md` | Parallel research, batch processing |
| [`parallel`](/patterns/parallel) | ⬤⬤ | _(none)_ | Independent analyses, voting |
| [`fsm`](/patterns/fsm) | ⬤⬤⬤ | _(none)_ | Custom workflows, conditional routing |
| [`critic`](/patterns/critic) | ⬤⬤ | `draft.md`, `critique.md`, `refine.md` | Writing quality, iterative improvement |
| [`reflexion`](/patterns/reflexion) | ⬤⬤⬤ | `act.md`, `reflect.md` | Self-correcting agents, performance improvement |
| [`constitutional`](/patterns/constitutional) | ⬤⬤ | `critique.md`, `revise.md` | Safe generation, principle-adherent output |
| [`chain-of-verification`](/patterns/chain-of-verification) | ⬤⬤⬤ | `verify.md`, `check.md`, `revise.md` | Factual accuracy, claim verification |
| [`plan-execute`](/patterns/plan-execute) | ⬤⬤⬤ | `execute.md`, `verify.md` | Complex tasks with verifiable sub-steps |
| [`debate`](/patterns/debate) | ⬤⬤⬤ | _(none — uses named agents)_ | Balanced analysis, decision-making |
| [`best-of-n`](/patterns/best-of-n) | ⬤⬤⬤ | `judge.md` | High-quality generation through selection |
| [`human`](/patterns/human) | ⬤⬤⬤ | `resume.md` | Human-in-the-loop approval, interactive tasks |

## How patterns are built

Under the hood, all patterns are combinations of two primitives:

```
FSM      — sequential steps with routing
Parallel — concurrent steps with collection
```

The named patterns are pre-wired compositions so you don't have to write the FSM YAML yourself:

```
critic     = FSM: draft → critique → refine
scatter    = FSM(map) → Parallel(workers) → FSM(reduce)
best-of-n  = Parallel(N variants) → FSM(judge)
debate     = FSM: position-a → position-b → judge
```

Use `fsm` directly when none of the named patterns fit your workflow.

## Choosing a pattern

**Start simple.** Most tasks fit `oneshot` or `react`:

- Need just an LLM response? → `oneshot`
- Need to use tools or reason in multiple steps? → `react`
- Need to improve quality iteratively? → `critic` or `reflexion`
- Need to process many items in parallel? → `scatter`
- Need factual accuracy? → `chain-of-verification`
- Need a complex custom workflow? → `fsm`

## Step files

Patterns with multiple steps load system prompts from separate `.md` files. These files live alongside `AGENT.md` in the agent directory and can optionally include their own `call:` frontmatter to override the model or tools for that specific step:

```
agents/
  essay-critic/
    AGENT.md        ← pattern: critic, shared config
    draft.md        ← system prompt for the draft step
    critique.md     ← system prompt for the critique step
    refine.md       ← system prompt for the refine step
```

Each step file can optionally include frontmatter:

```markdown
---
pattern: react          # run as a react loop; defaults to oneshot
call:
  model:
    name: anthropic:claude-opus-4-6
    temperature: 0.2
---

You are a meticulous critic. Identify the three most significant weaknesses...
```

See [Step files reference](/reference/step-files) for details.
