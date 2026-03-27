---
name: pipeline
description: Stock analysis pipeline — 4 specialist analysts run in parallel, then a portfolio manager synthesizes the recommendation. Ported from CrewAI stock analysis tutorial.
version: 1.0.0
pattern: fsm
initial: analysts
states:
  analysts: portfolio-manager
  portfolio-manager: _end
  _end:
---
