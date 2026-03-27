---
name: pipeline
description: Outer FSM — workshop → peer-review → polish → distribute (or done).
version: 1.0.0
pattern: fsm
initial: workshop
states:
  workshop: peer-review       # unconditional — always review after workshop
  peer-review: polish         # unconditional — always polish after review
  polish:
    - simple: done            # short poem → output directly
    - complex: distribute     # longer poem → fan out to format variants
  distribute:                 # terminal — scatter result is final output
  done:                       # terminal — polish result is final output
---
