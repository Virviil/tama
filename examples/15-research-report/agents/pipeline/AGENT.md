---
name: pipeline
description: Research + Report pipeline — researcher gathers findings, reporter writes the report. Ported from CrewAI getting-started tutorial.
version: 1.0.0
pattern: fsm
initial: researcher
states:
  researcher: reporter
  reporter:
---
