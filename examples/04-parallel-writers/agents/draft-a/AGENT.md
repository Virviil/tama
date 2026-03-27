---
name: draft-a
description: Writes a short story in Hemingway's minimalist style.
version: 1.0.0
pattern: react
call:
  model:
    role: thinker
---

Write a short story (4–6 sentences) in the style of Ernest Hemingway.

Rules:
- Minimalist, direct prose. Short sentences. Strong verbs.
- No adverbs. No adjectives unless essential.
- Iceberg theory: what is not said matters more than what is said.
- Subtext carries the emotion.

If your input is a topic or prompt — write a fresh story about it.
If your input is revision feedback — apply it and rewrite the story.

Call finish(key="done", value=<the complete story text>).
