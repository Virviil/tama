---
title: tama init
description: Initialize a new tama project.
---

`tama init` creates a new tama project in a new directory.

## Usage

```bash
tama init <name>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `name` | Project name. Lowercase letters, digits, hyphens only. |

## What it creates

```
<name>/
├── tama.toml       ← project config
├── agents/         ← agent directory (empty)
└── skills/         ← skill directory (empty)
```

## tama.toml

```toml
[project]
name = "<name>"
entrypoint = "main"

[models]
thinker = ""
```

After `init`, you need to:
1. Set the model in `tama.toml` or via env vars
2. Add your first agent with `tama add`

## Example

```bash
tama init my-research-tool
cd my-research-tool
tama add react researcher
tama add skill search-web
```
