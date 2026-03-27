---
name: editor
description: FSM — reviewer loops until satisfied; exits preserving the "publish" key.
version: 1.0.0
pattern: fsm
initial: reviewer
states:
  reviewer:
    - revise: reviewer  # reviewer not satisfied → loop with feedback
    - publish: done     # reviewer satisfied → exit, key="publish" propagates up
  done:                 # neutral terminal — passes key and value to workshop
---
