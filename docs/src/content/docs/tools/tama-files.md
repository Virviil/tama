---
title: tama_files
description: Built-in file I/O tools — tama_files_read and tama_files_write.
---

File tools let agents read and write files in the workspace. They must be unlocked via a skill.

## tama_files_write

Writes text content to a file. Creates the file if it does not exist; overwrites if it does.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `path` | string | File path relative to the working directory |
| `content` | string | Content to write |

**Returns** `"written"` on success.

**Example skill**

```markdown
---
name: save-result
description: Save output to a file in the workspace.
tools: [tama_files_write]
---

Use tama_files_write(path, content) to persist results.
Always write to a clearly named file, e.g. `result.md` or `report.md`.
```

## tama_files_read

Reads a file from the workspace.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `path` | string | File path relative to the working directory |

**Returns** the file contents as a string, or an error if the file does not exist.

**Example skill**

```markdown
---
name: read-file
description: Read a file from the workspace.
tools: [tama_files_read]
---

Use tama_files_read(path) to read a file. The full contents are returned as a string.
```

## Common pattern: research + save

Many react agents search the web and save results to a file:

```markdown
---
name: search-web
description: Search the web using DuckDuckGo. No API key required.
tools: [tama_http_get, tama_files_write]
---

Search with tama_http_get, then write findings to a .md file with
tama_files_write before calling finish.
```
