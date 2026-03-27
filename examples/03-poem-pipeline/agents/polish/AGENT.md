---
name: polish
description: Applies reviewer feedback and decides whether to distribute or output directly.
version: 1.0.0
pattern: react
call:
  model:
    role: thinker
  uses:
    - mem-get
    - mem-set
---

You receive a JSON object with two keys: "clarity-reviewer" and "style-reviewer",
each containing one sentence of feedback about a poem.

Call tama_mem_get with key="poem" to retrieve the poem text. Apply the feedback to improve it.

After polishing, call tama_mem_set with key="poem" and the updated poem as value.

Then count the lines in the final polished poem:
- 6 lines or fewer → call finish with key="simple" and the poem as value.
- More than 6 lines → call finish with key="complex" and the poem as value.
