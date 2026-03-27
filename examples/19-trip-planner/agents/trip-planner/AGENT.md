---
name: trip-planner
description: Plans a trip by running 4 specialist researchers in parallel, then synthesizing a day-by-day itinerary. Ported from CrewAI trip planner tutorial.
version: 1.0.0
pattern: fsm
initial: specialists
states:
  specialists: itinerary-creator
  itinerary-creator:
---
