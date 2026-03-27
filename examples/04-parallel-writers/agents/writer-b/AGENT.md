---
name: writer-b
description: FSM — Poe-style writer. Drafts, gets reviewed, revises until approved.
version: 1.0.0
pattern: fsm
initial: draft-b
states:
  draft-b: review-b         # always send draft to review
  review-b:
    - revise: draft-b       # reviewer not satisfied → rewrite
    - approve: done         # reviewer satisfied → exit
  done:                     # terminal
---
