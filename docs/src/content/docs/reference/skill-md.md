---
title: SKILL.md format
description: Complete reference for SKILL.md frontmatter fields and body.
---

`SKILL.md` defines a tool that agents can use. Skills follow the [Anthropic Agent Skills specification](https://agentskills.io/specification).

## File location

```
skills/{skill-name}/SKILL.md
```

The directory name must match the `name` field. Only lowercase letters, digits, and hyphens.

## Full example

```markdown
---
name: search-web
description: Search the web using DuckDuckGo. Use when the user asks to find information online.
license: MIT
tama:
  version: "1.0.0"
  depends:
    apt:
      - duckduckgo-cli
    bins:
      - duckduckgo-search
---

Use bash to run `duckduckgo-search "$QUERY"` and return the top results.

If the query is ambiguous, search multiple variations and combine results.

## Examples

Input: "latest LLM benchmarks"
→ Run: `duckduckgo-search "LLM benchmarks 2025"`
→ Return top 5 results with title, URL, and snippet
```

## Frontmatter fields

### `name` (required)

```yaml
name: search-web
```

- Lowercase letters, digits, and hyphens only
- Must match the directory name
- Referenced in agent `call.uses` lists

### `description` (required)

```yaml
description: Search the web using DuckDuckGo.
```

This is the **Level 1** disclosure — injected into the agent's system prompt at startup. It should tell the agent:
- What the skill does
- When to use it
- Any important caveats

Keep it under ~200 characters for token efficiency.

### `license` (optional)

```yaml
license: MIT
```

License name or path to a license file.

### `tama` (optional)

tama-specific metadata, stored under the `tama:` key to stay compatible with the Anthropic spec:

```yaml
tama:
  version: "1.0.0"
  depends:
    apt:
      - poppler-utils       # apt packages
    uv:
      - pypdf2>=3.0.1       # Python packages (via uv)
    bins:
      - pdftotext           # required binaries (checked after install)
```

`tama brew` uses the `depends` block to install dependencies into the Docker image.

### `tools` (optional)

```yaml
tools:
  - tama_mem_get
```

Built-in runtime tools this skill unlocks when loaded. The agent calls `read_skill("skill-name")` to load the skill, and these tools become available for subsequent LLM calls.

## Body

The text after `---` is the skill's **Level 2** disclosure — full instructions that the agent receives when it calls `read_skill("skill-name")`.

This should include:
- Step-by-step instructions for using the tool
- Edge cases and error handling
- Examples of input and expected output

## Two kinds of skill

### Instruction skills

Teach the agent to use existing always-on tools (like bash):

```markdown
---
name: search-web
description: Search the web using DuckDuckGo.
---

Use bash to run `duckduckgo-search "$QUERY"`.
Return the top 5 results with title, URL, and snippet.
```

### Tool-unlock skills

Gate access to a built-in runtime tool:

```markdown
---
name: mem-set
description: Store a value that can be retrieved later by this agent or others.
tools:
  - tama_mem_set
---

Use `tama_mem_set(key, value)` to store a value for later retrieval.
Keys are strings. Values are strings. Overwrites existing values.
```

## Progressive disclosure

Skills follow a two-level disclosure model to keep context windows lean:

```
Startup: agent sees name + description for all declared skills
            ↓
Agent calls read_skill("search-web")
            ↓
Full body injected as tool result
Any declared tools unlocked for subsequent calls
```

## Directory structure

```
skills/
  search-web/
    SKILL.md              # required
    scripts/              # optional: executable scripts
      search.py
    references/           # optional: additional docs
    assets/               # optional: templates, data
```
