# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-PISTON-001` — A piston resolves a finite move plan before executing its block event

**Parent:** `RED-004`, `RED-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked source fixes power sampling, plan construction, event revalidation,
overwrite-safe writes, sticky retraction, moving-entity progress and collision displacement.

**Applies when:**

A piston observes a power condition that differs from its extension state.

**Authoritative state:**

Piston facing/extended state, checked power positions, queued block events, resolved push/destroy
lists, block mobility reactions and moving-piston block entities.

**Transition and ordering:**

Power sampling first checks all six adjacent positions except the piston-facing position, then the
piston position itself toward down, then all positions adjacent to the block above except below.
Each uses ordinary/conductor signal. Powering an unextended piston resolves an extension plan before
queueing event 0; failure queues nothing. Unpowering an extended piston queues event 1, except an
extending moving piston two blocks ahead selects event 2 when its progress is below 0.5, it ticked
this game time, or the server is currently handling a tick.

At event execution the server rechecks power. Power restored before retract event 1/2 writes the
extended state with flag 2 and cancels. Power lost before extend event 0 cancels without a write.
Successful extension executes the plan, writes extended base with flags 67, plays pitch
`randomFloat*0.25+0.6` at volume 0.5 and emits block-activate. Retraction first finalizes any moving
head directly ahead, replaces the base itself with a retracting source moving-piston entity (flags
276), immediately updates its neighbors/shapes, performs sticky/default head handling, then plays
pitch `randomFloat*0.15+0.6` and emits block-deactivate.

The resolver's direction is piston facing for extension and its opposite for retraction; starts one
block ahead when extending and two blocks ahead when retracting. Air succeeds. Initial unpushable
state succeeds only for extension plus `DESTROY`, adding that one position to `toDestroy`.
Pushability requires world Y/border bounds; excludes obsidian, crying obsidian, respawn anchor,
reinforced deepslate, unbreakable states, outward movement at min/max Y, extended pistons and every
block entity. Push reactions block, destroy only when caller permits it, and push-only only when
movement equals connection direction.

Line construction first walks backward through mutually sticky blocks, stopping at air,
non-stickability, unpushability or the piston, and fails above 12 total. Honey and slime never stick
to each other; either sticky block sticks to ordinary neighbors. It appends that backward run from
farthest to start, then walks forward. Air ends successfully; unpushable/piston fails; `DESTROY`
adds one destroy position and succeeds; total 12 is the limit. Collision with an existing push-list
position reorders the newly appended run ahead of the collided suffix and rechecks sticky branches.
Every sticky member recursively scans all directions whose axis differs from push direction, in
`Direction.values()` order. Any failing line/branch rejects the entire plan before world mutation.

Execution snapshots `toPush` states and destroys `toDestroy` in reverse list order: drop resources,
write air with flag 18 and emit block-destroy. It then visits `toPush` in reverse, writes each
destination moving-piston state with flag 324 and installs a moving entity carrying its snapshot.
Extension installs an analogous moving piston-head at the arm. Any original source position not
overwritten by a destination becomes air with flag 82. It applies source/air indirect shape updates
for those clears, then in reverse destroy order performs removal hooks, indirect shapes and
orientation-aware neighbor updates, then in reverse push order notifies original positions, and
finally notifies the head position for extension.

Sticky retraction first finalizes a compatible still-extending moving piece two blocks ahead. If no
such piece, event 1 pulls only non-air state that is pull-pushable toward the piston and has normal
reaction (piston blocks are also admitted); otherwise it simply removes the head position. Event 2
never starts a fresh pull. Nonsticky retraction always removes the head position. Anchors:
`net.minecraft.world.level.block.piston.PistonBaseBlock#triggerEvent(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,int,int)`
and `net.minecraft.world.level.block.piston.PistonStructureResolver#resolve()`.

Each moving block advances progress by 0.5 per block-entity tick, moving collided entities over the
new sweep by the minimum separation capped to delta plus 0.01. `IGNORE` entities are skipped. A
moving slime block sets the movement-axis velocity component to ±1 for non-server-player entities;
honey moving horizontally carries on-ground normal-reaction entities on its top by the exact 0.5
delta. Retraction source motion additionally ejects entities intersecting the piston base. At the
tick after progress reaches 1, the server replaces moving state with neighbor-shape-adjusted carried
state, clears waterlogged if necessary, and sends the neighbor callback; clients retain a completed
entity for up to five extra ticks. Forced `finalTick` installs air for source pistons and adjusted
carried state otherwise.

**Branches and aborts:**

Stale event after power reversal; already correct state; push limit exceeded; excluded/immovable
state, block entity, bounds or border; destructive reaction in a nonpermitting phase; sticky branch
overflow; target cannot be pulled; compatible moving piece finalization; and plan resolution false.
A failed extension does not partially move or destroy the resolved prefix.

**Constants and randomness:**

Maximum `toPush` is 12. Progress steps are 0→0.5→1.0 and collision padding is 0.01; sticky top AABB
extends to `1.5000010000000001`; movement constant is 0.51. Resolution/movement consumes no RNG.
Only successful extend/retract sound pitch consumes one level random float after writes.

**Side effects:**

Block event queue, moving-piston entities, temporary moving block states, entity displacement,
destroyed blocks/drops, neighbor/shape/comparator updates, sounds and particles.

**Gates:**

Power geometry, facing, event revalidation, push reaction, world bounds, chunk availability,
block-entity/moving restrictions and sticky semantics.

**Boundary cases and quirks:**

The plan, reverse write and reverse notification orders are distinct and observable. `PUSH_ONLY`
depends on the caller's connection direction, not only physical movement. Moving carried blocks
temporarily exist as invisible moving-piston states with block entities and dynamic collision.
Breaking a moving state finalizes/removes the related extended base as specified by its block hooks.
Power around the position above is sampled only when `checkIfExtend` is invoked; power alone does not
manufacture a missing neighbor callback.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `PistonBaseBlock`, `PistonStructureResolver`,
`PistonMovingBlockEntity`, `MovingPistonBlock`; `EXP-RED-003`.

**Test vectors:**

Every direct/above power position and missing-update case; reverse power before events 0/1/2; all
push reactions, excluded states, min/max Y, border, extended piston and block entity; sticky backward,
branch, honey/slime exclusion, collision reorder and 12/13 limits; reverse destroy/move/clear/update
trace; normal/sticky fast retraction; entity `IGNORE`/normal, player/nonplayer slime, honey support,
base ejection, progress/finalization and waterlogged carried state.
