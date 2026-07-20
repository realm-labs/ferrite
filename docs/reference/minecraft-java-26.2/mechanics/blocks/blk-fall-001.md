# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-FALL-001` — A falling block transfers block state into an entity and back

**Parent:** `BLK-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes the generic entity transaction, all registered subtype
overrides, persistence boundary, constants, RNG and failure behavior; `EXP-BLK-003` remains a
conformance probe.

**Applies when:**

Any of the 26 audited falling/stability blocks (sand, red sand, gravel, sixteen concrete powders,
three anvils, dragon egg, suspicious sand/gravel and scaffolding) schedules or starts its fall, and
while the sole `minecraft:falling_block` entity type is active.

**Authoritative state:**

The server owns the origin and destination states/fluids, entity carried state, position/previous
position/velocity, integer `time`, start position, fall distance, `dropItem`, `cancelDrop`, hurt
parameters, optional carried block-entity tag, destination survival/replaceability, chunk
persistence and `doEntityDrops`. A newly constructed entity defaults to sand, time `0`, item drops
enabled, cancel disabled, damage disabled, amount `0`, maximum `40`; load instead defaults damage
enabled when its carried state is in `#anvil`.

**Transition and ordering:**

Start and generic motion: `FallingBlock#onPlace` and every shape update schedule the same block
after `getDelayAfterPlace`: normally `2` ticks and `5` for dragon egg. On its tick, it starts only
when the block below is air, in `#fire`, liquid, or replaceable and origin `y >= minY`.
`FallingBlockEntity#fall` creates the entity at `(x+0.5,y,z+0.5)` with zero velocity, clears a
present `waterlogged` property, records the start position, replaces the origin with that original
state's fluid legacy block using flags `3`, and then requests entity admission. The removal precedes
admission and its boolean is ignored, so rejected admission does not restore the block. Generic
creation does **not** copy an origin block entity.

Each entity tick discards immediately when its carried state is air. Otherwise it increments `time`,
adds vertical acceleration `-0.04`, moves as `MoverType.SELF`, applies block effects/portal
handling, runs the server landing/timeout branch, and finally multiplies all three velocity
components by `0.98`. An unloaded entity is absent from the ticking list: chunk unload stores it and
removes its callback, pending load restores it only after the persistent manager's UUID guard
succeeds, and neither `time` nor motion advances for unloaded wall time. End-dimension teleport
additionally permits the removed original to finish its current landing/timeout branch once,
preventing that mid-tick duplication path from skipping cleanup.

**Landing transaction:**

Ordinary landing begins when `onGround`; concrete powder also begins when it contacts water.
Velocity is first multiplied by `(0.7,-0.5,0.7)`. A `moving_piston` target defers all placement,
drop and removal for that tick. Otherwise `cancelDrop` discards and calls the subtype broken hook
without an item. The entity then evaluates target replacement with a downward
`DirectionalPlaceContext`, support via `canSurvive`, and whether the block below remains free. When
replaceable, supported and no longer falling, it copies destination water into a present
`waterlogged` property, attempts `setBlock(...,3)`, and on success sends the tracking block update,
discards, then calls `onLand`. Only an explicitly serialized `TileEntityData` is overlaid onto the
new block entity (save without metadata, copy every carried entry, load with components, mark
changed); ordinary falling never supplied such data.

If successful placement is unavailable because replacement/survival fails, the entity discards and,
only when `dropItem && doEntityDrops`, calls the broken hook and spawns the carried block item. If
the eligibility checks passed but `setBlock` itself returns false, that same drop/discard branch
runs only when item drops are enabled; otherwise the entity remains to retry. Away from a landing,
timeout occurs at strict `time>100` when `y<=minY || y>maxY`, or unconditionally at strict
`time>600`; it optionally spawns the item under the same `dropItem && doEntityDrops` gate and then
discards. Timeout does not consult `cancelDrop`.

**Subtype branches:**

- Anvil start configures damage amount `2.0` and maximum `40`. On impact let
  `i=ceil(fallDistance-1)`; `i<0` causes no damage processing. Every alive
  non-creative/non-spectator living entity intersecting its box receives `min(floor(2i),40)` anvil
  damage. When that amount is positive, consume exactly one entity RNG float; `<0.05+0.05i` advances
  anvil to chipped to damaged while preserving facing, and a damaged anvil sets `cancelDrop`.
  Landing emits level event `1031`, broken impact `1029`, unless silent.
- Concrete powder is already replaced by its paired concrete during placement or a shape update when
  its position contains water or a neighboring water fluid has a non-sturdy face toward it. During
  fast fall (`velocity.lengthSqr()>1`) it collider-raycasts previous to current position with source
  fluids only; the first water hit becomes the landing position and bypasses the still-falling-below
  test. After powder placement, `onLand` replaces it with paired concrete when the prelanding target
  met the same solidification predicate.
- Suspicious sand/gravel are `BrushableBlock`, not `FallingBlock`: placement/shape updates schedule
  after `2`; the tick first resets its block entity if needed, then invokes generic creation and
  sets `cancelDrop`. No archaeology block-entity tag is captured. Successful landing creates a fresh
  block entity; a failed landing emits destroy event `2001` plus `BLOCK_DESTROY` and no item, but
  the generic timeout quirk may still drop its item because timeout ignores `cancelDrop`.
- Scaffolding schedules after `1`. Distance is `0` above a sturdy top face, exactly the below
  scaffolding distance for a vertical column, otherwise one plus the minimum horizontal scaffolding
  distance, capped at `7`; `bottom` is true exactly when distance is positive and below is not
  scaffolding. A tick recomputes both: newly reaching `7` destroys with drops, remaining at `7`
  spawns a generic falling entity, and a supported changed state writes flags `3`. Landing survives
  only where recomputation is below `7`.
- Dragon egg interaction/attack teleport is separate from its delayed fall. It tries at most `1,000`
  candidates, consuming six `nextInt` calls per attempt: paired `16` draws for triangular `dx,dz`
  and paired `8` draws for `dy`. The first air candidate above nonair, inside build height and world
  border is written with flags `2`, then the origin is removed without drops; both booleans are
  ignored. The client instead emits `128` portal particles. No valid candidate changes nothing.

**Branches and aborts:**

Supported source, below-minimum source, air carried state, entity admission rejection, moving
piston, cancel, target replacement/survival, failed state write, water contact, timeout, gamerule
and subtype transformation branch exactly as ordered above. Generic landing has no world-border
gate. The entity is non-attackable; hurt merely records the hit and returns false.

**Constants and randomness:**

Delays `1/2/5`; gravity `0.04`; drag `0.98`; landing velocity multipliers `(0.7,-0.5,0.7)`; time
boundaries `100/600`; anvil formula and one-float degradation above; dragon-egg attempts/draw
bounds/particles above. Generic transition and concrete-powder raycast consume no RNG. Falling-block
ambient dust always consumes `nextInt(16)` and emits only on zero.

**Side effects:**

Scheduled ticks, origin fluid replacement and neighbor/client work, entity
admission/movement/portal/persistence/removal, destination state and block-entity writes, tracking
update, item entity, damage, level/game events and particles in the stated order.

**Gates:**

Free-below/min-height start, active entity ticking, destination replaceability/survival,
`doEntityDrops`, subtype state/tag, water collision, scaffolding distance, dragon-egg build
height/world border, and chunk UUID admission. Difficulty does not alter this rule.

**Boundary cases and quirks:**

Source removal can outlive failed spawn admission. A failed eligible `setBlock` with drops disabled
leaves a landed entity retrying. `cancelDrop` suppresses landing drops but not timeout drops.
Vertical scaffolding does not add distance. Generic falling preserves state, not implicit source
block-entity data. Piston targets pause rather than break.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.FallingBlock`,
`net.minecraft.world.entity.item.FallingBlockEntity`, `net.minecraft.world.level.block.AnvilBlock`,
`net.minecraft.world.level.block.ConcretePowderBlock`,
`net.minecraft.world.level.block.DragonEggBlock`, `net.minecraft.world.level.block.BrushableBlock`,
`net.minecraft.world.level.block.ScaffoldingBlock`,
`net.minecraft.world.level.entity.PersistentEntitySectionManager`.

**Test vectors:**

(1) For every source subtype, assert schedule delay and the origin-fluid-before-admission
transition, including rejected admission. (2) Land on replaceable/supported, nonreplaceable,
unsupported and moving-piston targets with every drop/cancel/gamerule combination; force `setBlock`
false. (3) Test exact timeout inequalities and unload/reload across each boundary. (4) Exercise
slow/fast concrete powder water contact. (5) Sweep anvil fall distances across ceil/floor/cap and
degradation thresholds. (6) Preserve and lose brushable data exactly as specified, including landing
versus timeout. (7) Cover scaffolding vertical/horizontal distances and old/new distance-7 branches.
(8) Exhaust dragon-egg valid/invalid candidates while tracing draw count. Run `EXP-BLK-003` as the
executable matrix.
