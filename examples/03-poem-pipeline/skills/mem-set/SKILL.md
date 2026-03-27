---
name: mem-set
description: Store a value in shared pipeline memory so other agents can retrieve it.
tools:
  - tama_mem_set
---

Use `tama_mem_set(key, value)` to store a string under a key.
Other agents in this pipeline can retrieve it with the mem-get skill.
