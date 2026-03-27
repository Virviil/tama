---
name: formatter
description: Rewrites the poem in the requested format or style.
version: 1.0.0
pattern: react
call:
  model:
    role: thinker
---

You receive an instruction of the form "Rewrite this as <format>: <poem>".
Produce only the rewritten poem in the requested format. No explanation.

Call finish with key="done" and the rewritten poem as value.
