# SKILL.md format

One-to-one with the [Anthropic Agent Skills specification](https://agentskills.io/specification).

---

## Markdown as the only format

**Decision:** all skills are `.md` files with YAML frontmatter.

**Why:** human-readable, git-diffable, editable in any editor,
parseable in any language. A skill is documentation that runs.

**❌ Anti-pattern:** separate YAML/TOML for metadata and a separate `.sh` for code.
That's two files instead of one, and the document loses its unity.

---

## Directory structure

```
skill-name/
├── SKILL.md          # required: metadata + instructions
├── scripts/          # optional: executable code (Python, Bash, JS)
├── references/       # optional: additional documentation
└── assets/           # optional: templates, resources
```

---

## Frontmatter fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Max 64 chars. Lowercase letters, digits, and hyphens only. Must match the directory name. |
| `description` | Yes | Max 1024 chars. What it does and when to use it. |
| `license` | No | License name or path to license file. |
| `tools` | No | Built-in runtime tools this skill unlocks when loaded (see progressive disclosure). |
| `tama` | No | tama-specific fields (depends, etc.). |

---

## Progressive disclosure

In the react pattern, skills follow a two-level disclosure model — same as Claude Code:

**Level 1** (always): `name` + `description` of each declared skill is injected into the system prompt.
The agent sees what's available and decides when to use each skill.

**Level 2** (on demand): the agent calls `read_skill(name)` to load the full SKILL.md body.
This also unlocks any built-in runtime tools listed in the skill's `tools:` field —
making them available in subsequent LLM calls.

```
System prompt = AGENT.md body + skill descriptions (Level 1)

Built-in tools always available:
  start, finish, read_skill

Agent calls read_skill("mem-get"):
  → full skill body injected as tool result
  → tama_mem_get tool unlocked and added to the tool list
  → agent can now call tama_mem_get(key) directly
```

1. **Metadata** (~100 tokens): `name` + `description` loaded at startup for all declared skills
2. **Instructions + tools** (<5000 tokens): full SKILL.md body loaded when the agent calls `read_skill`; declared tools are unlocked at this point

This is the only way to bring a tool to an agent's context — through a skill.
Tools are never given to agents by default.

**❌ Anti-pattern:** converting SKILL.md to a JSON tool schema at load time.
That requires an executor for every skill and couples the runtime to skill internals.
Progressive disclosure keeps the runtime simple.

---

## Two kinds of skill

**Instruction skills** — teach the agent to use existing always-on tools:

```markdown
---
name: search-web
description: Search the web using DuckDuckGo.
---

Use bash to run `duckduckgo-search "$QUERY"` and return the top results.
```

**Tool-unlock skills** — gate access to a built-in runtime tool:

```markdown
---
name: mem-get
description: Retrieve a value stored by another agent earlier in the pipeline.
tools:
  - tama_mem_get
---

Use `tama_mem_get(key)` to retrieve a value stored earlier with mem-set.
Returns `[no value stored for key '...']` if the key has not been set yet.
```

---

## Example: full skill

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

If the query is ambiguous, search for multiple variations and combine results.

## Examples

Input: "latest LLM benchmarks"
→ Run: `duckduckgo-search "LLM benchmarks 2025"`
→ Return top 5 results with title and URL
```

---

## tama-specific fields

tama stores its metadata in the `tama:` block to stay compatible with the Anthropic spec:

```yaml
tama:
  depends:
    apt:
      - poppler-utils
    uv:
      - pypdf2>=3.0.1
    bins:
      - pdftotext    # compiler checks the binary exists after install
```

---

## Naming

**Kebab-case everywhere**: skill and agent names use hyphens.
This follows the Anthropic spec: the `name` field allows only lowercase letters, digits, and hyphens.
Agents follow the same rule for consistency.

✅ `search-web`, `pdf-processing`, `react-search`, `essay-critic`
❌ `search_web`, `reactSearch`, `SearchWeb`
