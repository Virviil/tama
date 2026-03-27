---
name: mem-get
description: Retrieve a value stored by another agent earlier in the pipeline.
tools:
  - tama_mem_get
---

Use `tama_mem_get(key)` to retrieve a value stored earlier with mem-set.
Returns `[no value stored for key '...']` if the key has not been set yet.
