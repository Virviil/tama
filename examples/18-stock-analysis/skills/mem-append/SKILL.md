---
name: mem-append
description: Append an item to a shared array in pipeline memory. Creates the array if it does not exist yet.
tools:
  - tama_mem_append
---

Use `tama_mem_append(key, item)` to append an item to a JSON array stored under `key`.

- `key` — the array name, e.g. `"pipeline_errors"` or `"layers"`
- `item` — a JSON-encoded value to append. May be an object, string, or number.
  Examples:
  - `'{"phase": "fixer", "error": "build failed", "attempt": 1}'`
  - `'"some string"'`

If the key does not exist yet, a new array is created automatically.
The current array contents are returned after the append.

Other agents can read the full array with `tama_mem_get(key)`.
