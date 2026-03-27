---
name: workshop
description: FSM — poet writes, editor reviews. Loops back on revise, exits on publish.
version: 1.0.0
pattern: fsm
initial: poet
states:
  poet: editor          # unconditional — send draft to editor FSM
  editor:
    - publish: approved # editor FSM returned "publish" → done
    - revise: poet      # editor FSM returned "revise" → poet rewrites
  approved:             # terminal — approved poem exits workshop
---
