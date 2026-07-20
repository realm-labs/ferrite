# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-AI-001` — Mob AI arbitrates goals, navigation, controls, senses, and memory on entity ticks

**Parent:** `MOB-004`, `MOB-005`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — generic schedulers are located, but per-species memories, sensors,
activities, goals, navigation gates, and permitted route divergence remain unexpanded.

**Applies when:**

A mob is entity-ticking and not in a state that suppresses ordinary AI.

**Authoritative state:**

Goal selectors/flags/priorities, brain activities/memories/sensors, navigation path, move/look/jump
controls, target, attributes, leash and mob-specific timers.

**Transition and ordering:**

Update ambient/mob timers and sensing/brain at their scheduled cadence; stop running goals that
cannot continue; evaluate eligible goals and start only those whose control flags can be acquired
against higher-priority running goals; tick running goals; navigation advances path; controls
translate desired movement/look/jump into entity inputs before travel. Species may use Brain
behaviors, GoalSelector, or both.

**Branches and aborts:**

`NoAI`; dead/removed; inactive chunk; passenger; stunned/sleeping; sensor cadence not due; memory
absent/expired; goal use/continue false; control flag conflict; path unavailable/stuck; target
invalid.

**Constants and randomness:**

Goal priorities are integer and lower numeric value has precedence. Sensor/behavior intervals,
memory expiry and path tolerances are species/source data. Goal reevaluation and many behaviors
consume RNG at species-defined sites; exact cadence is `EXP-MOB-002`.

**Side effects:**

Target/memory/path, movement and rotation, block interaction, item use, attacks, sounds/game events,
breeding/taming state and spawned entities/items.

**Gates:**

Entity ticking, `NoAI`, goals/brain predicates, senses/line of sight/range, mobGriefing where blocks
change, difficulty, time/weather and species state.

**Boundary cases and quirks:**

Navigation computes intent; collision/travel still decides actual motion. Goals with disjoint
control flags can run concurrently. Memory visibility is not equivalent to a live line-of-sight test
every tick.

**Evidence:**

`Confirmed` arbitration structure; species cadences require per-family depth; `OFF-SERVER-001`;
`GoalSelector`/`Brain` class families; `EXP-MOB-002`.

**Test vectors:**

Competing MOVE goals of different priority; concurrent LOOK goal; target disappears; path blocked
after compute; `NoAI` toggle; unload/reload; Brain memory expires at exact tick.
