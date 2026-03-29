---
title: tama_mem
description: Built-in shared memory tools — tama_mem_set, tama_mem_get, tama_mem_append.
---

Memory tools provide a shared key-value store accessible to all agents within a single run. Use them to pass data between agents in a pipeline without writing to disk.

Memory is **in-process and ephemeral** — it does not persist across runs.

## tama_mem_set

Stores a string value under a key.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `key` | string | Storage key, e.g. `"summary"` |
| `value` | string | Value to store |

**Example skill**

```markdown
---
name: mem-set
description: Store a value in shared pipeline memory so other agents can retrieve it.
tools: [tama_mem_set]
---

Use tama_mem_set(key, value) to store a string under a key.
Other agents in this pipeline can retrieve it with mem-get.
```

## tama_mem_get

Retrieves a value stored by a previous agent.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `key` | string | Storage key to retrieve |

**Returns** the stored value, or `[no value stored for key '...']` if not set.

**Example skill**

```markdown
---
name: mem-get
description: Retrieve a value stored by another agent earlier in the pipeline.
tools: [tama_mem_get]
---

Use tama_mem_get(key) to retrieve a value stored with mem-set.
```

## tama_mem_append

Appends an item to a JSON array stored under a key. Creates the array if it doesn't exist.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `key` | string | Array key, e.g. `"results"` |
| `item` | string | JSON-encoded value to append |

**Returns** the full array after appending.

**Example skill**

```markdown
---
name: mem-append
description: Append an item to a shared array in pipeline memory.
tools: [tama_mem_append]
---

Use tama_mem_append(key, item) where item is a JSON-encoded value.
Examples:
- tama_mem_append("errors", '"something failed"')
- tama_mem_append("results", '{"agent": "analyst", "score": 0.9}')

Read the full array with tama_mem_get(key).
```

## Pipeline pattern

A typical multi-agent pipeline uses mem to pass state:

```
agent-a  → tama_mem_set("report", <text>)
agent-b  → tama_mem_get("report") → processes it
agent-c  → tama_mem_append("findings", <json>)
reporter → tama_mem_get("findings") → synthesizes
```
