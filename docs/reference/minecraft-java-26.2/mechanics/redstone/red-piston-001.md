# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-PISTON-001` — A piston resolves a finite move plan before executing its block event

**Parent:** `RED-004`, `RED-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — movement/destruction ordering, sticky branches, and quasi-connectivity
notification cases remain unexpanded.

**Applies when:**

A piston observes a power condition that differs from its extension state.

**Authoritative state:**

Piston facing/extended state, checked power positions, queued block events, resolved push/destroy
lists, block mobility reactions and moving-piston block entities.

**Transition and ordering:**

Neighbor change evaluates piston power; enqueue extend or retract block event; when the event runs,
revalidate power, build the directional move plan, reject if immovable/limit/bounds fail, otherwise
replace destinations/origins with moving states in the required reverse order, create moving block
entities, update piston state and notify affected positions. Sticky retraction conditionally pulls
the front block. Anchors:
`net.minecraft.world.level.block.piston.PistonBaseBlock#triggerEvent(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,int,int)`
and `net.minecraft.world.level.block.piston.PistonStructureResolver#resolve()`.

**Branches and aborts:**

Stale event after power reversal; already correct state; push limit exceeded; immovable/destroy
reaction; build bounds; sticky target cannot be pulled; competing moving piston; plan resolution
false. A failed extension does not partially move the resolved prefix.

**Constants and randomness:**

Maximum push chain is 12 blocks. Movement progress is deterministic per tick; generic resolution
consumes no RNG. Mobility reaction and block-entity eligibility are block-state behavior.

**Side effects:**

Block event queue, moving-piston entities, temporary moving block states, entity displacement,
destroyed blocks/drops, neighbor/shape/comparator updates, sounds and particles.

**Gates:**

Power geometry, facing, event revalidation, push reaction, world bounds, chunk availability,
block-entity/moving restrictions and sticky semantics.

**Boundary cases and quirks:**

The move plan order is observable through updates and entity collision. Retraction can encounter the
head/moving state from a previous transition. Power is not a single adjacent-face query.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; locators above; complex simultaneous piston trace `EXP-RED-003`.

**Test vectors:**

Push 12 and 13 blocks; include destroyable and immovable states; reverse power before the event;
sticky pull versus non-pull; crossed pistons and entities in swept volume.
