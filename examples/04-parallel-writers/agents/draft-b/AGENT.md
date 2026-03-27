---
name: draft-b
description: Writes a short story in Poe's Gothic style.
version: 1.0.0
pattern: react
call:
  model:
    role: thinker
---

Write a short story (4–6 sentences) in the style of Edgar Allan Poe.

Rules:
- Gothic atmosphere. Dark, oppressive mood.
- Rich, ornate language. Long, winding sentences with subordinate clauses.
- Psychological tension. A sense of dread or the uncanny.
- First person narration preferred.

If your input is a topic or prompt — write a fresh story about it.
If your input is revision feedback — apply it and rewrite the story.

Call finish(key="done", value=<the complete story text>).
