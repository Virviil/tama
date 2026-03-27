---
title: tama lint
description: Validate all agents and skills in the current project.
---

`tama lint` validates the project structure — checks that all agents have their required step files and that all referenced skills exist.

## Usage

```bash
tama lint
```

Run from the project root (where `tama.toml` lives).

## What it checks

- All `AGENT.md` files parse correctly (valid YAML frontmatter, required fields present)
- All required step files exist for each pattern (e.g., `reduce.md` for scatter, `draft.md`/`critique.md`/`refine.md` for critic)
- All skills referenced in `call.uses` lists exist in `skills/`

## Example output

```
✓ agents/researcher: ok
✓ agents/summarizer: ok
✗ agents/essay-critic: missing required files: critique.md, refine.md
✗ agents/fact-checker: skill 'search-web' not found in skills/
```

## Exit code

- `0` — all checks pass
- `1` — one or more checks failed

Use in CI:

```yaml
# .github/workflows/ci.yml
- name: Lint tama project
  run: tama lint
```
