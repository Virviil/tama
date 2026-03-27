---
title: tama add
description: Add a new agent or skill to the current project.
---

`tama add` scaffolds a new agent or skill with the correct file structure and placeholder content.

## Usage

```bash
tama add <pattern> <name>    # add an agent
tama add skill <name>        # add a skill
```

## Adding an agent

```bash
tama add react my-agent
tama add oneshot summarizer
tama add critic essay-writer
tama add scatter deep-research
tama add fsm review-pipeline
tama add reflexion self-improver
tama add constitutional safe-writer
tama add chain-of-verification fact-checker
tama add plan-execute project-builder
tama add debate investment-analyzer
tama add best-of-n headline-generator
tama add parallel due-diligence
tama add human draft-approver
```

Creates `agents/<name>/AGENT.md` (and required step files for multi-step patterns).

## Adding a skill

```bash
tama add skill search-web
tama add skill fetch-url
tama add skill run-python
```

Creates `skills/<name>/SKILL.md`.

## Scaffolded files per pattern

| Pattern | Files created |
|---------|--------------|
| `oneshot` | `AGENT.md` |
| `react` | `AGENT.md` |
| `scatter` | `AGENT.md`, `reduce.md` |
| `parallel` | `AGENT.md` |
| `fsm` | `AGENT.md` |
| `critic` | `AGENT.md`, `draft.md`, `critique.md`, `refine.md` |
| `reflexion` | `AGENT.md`, `reflect.md` |
| `constitutional` | `AGENT.md`, `critique.md`, `revise.md` |
| `chain-of-verification` | `AGENT.md`, `verify.md`, `check.md`, `revise.md` |
| `plan-execute` | `AGENT.md`, `execute.md`, `verify.md` |
| `debate` | `AGENT.md` |
| `best-of-n` | `AGENT.md`, `judge.md` |
| `human` | `AGENT.md`, `resume.md` |

## Naming rules

Agent and skill names must be:
- Lowercase letters, digits, and hyphens only
- Must not already exist in the project
- Must not be empty
