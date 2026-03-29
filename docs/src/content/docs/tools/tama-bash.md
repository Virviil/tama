---
title: tama_bash
description: Built-in shell execution tool — tama_bash.
---

`tama_bash` runs arbitrary shell commands and returns their output. It must be unlocked via a skill.

## tama_bash

Executes a shell command via `sh -c`.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `command` | string | Shell command to run |

**Returns** stdout, and stderr (prefixed with `[stderr]`) if non-empty. Exit code is appended as `[exit N]` on failure.

**Example skill**

```markdown
---
name: bash
description: Run shell commands. Use for file operations, running scripts, or any CLI task.
tools: [tama_bash]
---

Use tama_bash(command) to run any shell command.

## Safety
- Prefer read-only commands unless the task explicitly requires writes.
- Always check the exit code in the response (`[exit N]`).

## Examples
- tama_bash("ls -la")
- tama_bash("python3 script.py --input data.json")
- tama_bash("cat result.md")
- tama_bash("wc -l *.txt")
```

## Common uses

- Running Python or Node scripts
- File system inspection (`ls`, `find`, `cat`)
- Data processing (`jq`, `awk`, `sed`)
- Build and test commands

## Security note

`tama_bash` has full access to the system the agent runs on. Only expose it in agents that require it, and only to tasks you trust.
