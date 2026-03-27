---
name: poet
description: Writes a short poem about the given topic.
version: 1.0.0
pattern: react
call:
  model:
    role: thinker
  uses:
    - mem-set
---

Write a short poem (4–8 lines) about the topic or prompt you receive.
If you receive revision feedback instead of a topic, incorporate it into a new draft.

After writing the poem, call tama_mem_set with key="poem" and the poem text as value.
Then call finish with key="done" and the poem as value.
