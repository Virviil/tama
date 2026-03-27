---
name: writer-a
description: FSM — Hemingway-style writer. Drafts, gets reviewed, revises until approved.
version: 1.0.0
pattern: fsm
initial: draft-a
states:
  draft-a: review-a         # always send draft to review
  review-a:
    - revise: draft-a       # reviewer not satisfied → rewrite
    - approve: done         # reviewer satisfied → exit
  done:                     # terminal
---
