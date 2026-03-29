---
title: Tools Overview
description: Built-in tools available to agents via skills.
---

**Tools** are the primitive actions agents can execute — HTTP requests, file I/O, memory, shell commands. They are implemented in the tama runtime and always available to any skill.

## How tools work

Tools are **unlocked through skills**. An agent cannot call a tool directly; it must first load a skill that declares the tool:

```markdown
---
name: my-skill
description: Does something useful.
tools:
  - tama_http_get
  - tama_files_write
---

Instructions for the agent...
```

When the agent calls `read_skill("my-skill")`, the listed tools become available for subsequent calls in that agent's session.

## Always-on tools

Two tools are **always available** — no skill required:

| Tool | Description |
|------|-------------|
| `start` | Receive the task input. Every agent calls this first. |
| `finish` | Return the result and end the agent loop. |

Every other tool must be unlocked via a skill.

## Built-in tool namespaces

tama ships with a standard library of built-in tools organized by namespace:

| Namespace | Tools | What it does |
|-----------|-------|--------------|
| [`tama_http`](/tools/tama-http) | `tama_http_get`, `tama_http_post` | HTTP requests |
| [`tama_files`](/tools/tama-files) | `tama_files_read`, `tama_files_write` | File I/O |
| [`tama_mem`](/tools/tama-mem) | `tama_mem_set`, `tama_mem_get`, `tama_mem_append` | Shared in-memory store |
| [`tama_bash`](/tools/tama-bash) | `tama_bash` | Shell command execution |

## Tool names in skill files

Always use the full tool name (including namespace prefix) in your skill's `tools:` list and in the body instructions:

```markdown
---
name: search-web
description: Search the web using DuckDuckGo.
tools: [tama_http_get, tama_files_write]
---

Call tama_http_get to fetch search results...
```
