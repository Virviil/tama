---
title: Skills Overview
description: What skills are, how they work, and the two kinds of skill.
---

A **skill** is a tool that agents can use. Skills are `.md` files — human-readable, git-diffable, editable in any editor.

```
skills/
  search-web/SKILL.md     ← search DuckDuckGo
  fetch-url/SKILL.md      ← HTTP GET
  run-python/SKILL.md     ← execute Python code
  mem-get/SKILL.md        ← retrieve shared memory
```

## Why Markdown?

A skill is documentation that runs. The same file that explains how to use a tool *is* the tool. No separate YAML schema, no code generator, no executor to maintain.

This follows the [Anthropic Agent Skills specification](https://agentskills.io/specification).

## Two kinds of skill

### Instruction skills

Teach the agent to use tools already available in its environment (bash, file system, etc.):

```markdown
---
name: search-web
description: Search the web using DuckDuckGo.
---

Use bash to run `duckduckgo-search "$QUERY"`.
Return the top 5 results with title, URL, and snippet.
```

The agent reads these instructions and uses its existing bash access to run the command.

### Tool-unlock skills

Gate access to a built-in runtime tool that isn't available by default:

```markdown
---
name: mem-get
description: Retrieve a value stored by another agent earlier in the pipeline.
tools:
  - tama_mem_get
---

Use `tama_mem_get(key)` to retrieve a stored value.
Returns `[no value for key '...']` if not set.
```

When the agent calls `read_skill("mem-get")`, the `tama_mem_get` tool is unlocked and added to the available tools for subsequent calls.

## Progressive disclosure

Skills follow a two-level disclosure model:

**Level 1 — always on:** The skill's `name` and `description` are injected into every agent's system prompt that declares the skill in `uses:`. The agent knows what tools are available.

**Level 2 — on demand:** The agent calls `read_skill("skill-name")` to load the full instructions. This also unlocks any `tools:` declared in the skill's frontmatter.

```
Agent system prompt:
  "Available skills:
   - search-web: Search the web using DuckDuckGo
   - fetch-url: Fetch the content of a URL"

Agent calls: read_skill("search-web")
  → Full SKILL.md body returned as tool result
  → Agent now has complete instructions

Agent calls: duckduckgo-search "Rust 2025 trends"
  → Results returned
```

This pattern keeps context windows lean — agents only pay the token cost for skills they actually use.

## Declaring skills in agents

```yaml
# agents/researcher/AGENT.md
call:
  uses:
    - search-web
    - fetch-url
```

Skills in `uses:` are available to the agent. Their descriptions appear in the system prompt. Full instructions loaded on demand.

## Dependencies

Skills can declare system dependencies that `tama brew` will install:

```yaml
tama:
  depends:
    apt:
      - duckduckgo-cli
    uv:
      - requests>=2.31.0
    bins:
      - duckduckgo-search
```

- `apt` — system packages
- `uv` — Python packages (installed via `uv`)
- `bins` — binary executables (verified after install)

## Next steps

- [Writing a Skill](/skills/writing) — step-by-step guide
- [SKILL.md reference](/reference/skill-md) — complete format documentation
