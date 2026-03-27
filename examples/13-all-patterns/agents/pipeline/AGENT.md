---
name: pipeline
description: Kitchen-sink FSM — routes a technology analysis through every pattern in sequence.
version: 1.0.0
pattern: fsm
initial: research
states:
  research: deep-dive           # react → scatter
  deep-dive: panel              # scatter → parallel
  panel: write                  # parallel → critic
  write: fact-check             # critic → chain-of-verification
  fact-check: ethics-check      # chain-of-verification → constitutional
  ethics-check: action-plan     # constitutional → plan-execute
  action-plan: debate           # plan-execute → debate
  debate: reflect               # debate → reflexion
  reflect:
    - done: headlines           # satisfied → best-of-n (terminal)
    - retry: write              # not satisfied → loop back to critic
  headlines:                    # terminal
---
