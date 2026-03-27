---
name: distribute
description: Scatter — fans the poem out to multiple format variants, then picks the best.
version: 1.0.0
pattern: scatter
worker: formatter
call:
  model:
    role: thinker
  uses:
    - mem-get
---

You receive a finished poem (or call tama_mem_get with key="poem" if it wasn't passed directly).

Decide on exactly three format variants to produce. For each variant, write a string
that gives the formatter both the poem and the target format, like:

  "Rewrite this as a haiku: <poem text here>"
  "Rewrite this for a greeting card: <poem text here>"
  "Rewrite this as a limerick: <poem text here>"

Call finish with key="parallel" and a JSON array of those three strings as value.
