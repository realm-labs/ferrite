# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BELL-001` — Bells separate immediate ring ingress from queued hearing, shake, resonance, glow, and particles

**Parent:** `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `BLK-007`, `PLY-005`,
`RED-001`, `ITM-007`, `ENT-006`, `MOB-004`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client source, generated block/item reports, the six-member raider
tag and bell loot table fix all 32 states, placement/support transforms, hit geometry,
redstone/projectile/explosion ingress, block-event/cache ordering, hearing memory, shake/resonance
clocks, glow, renderer motion and particle arithmetic. Generic block-event admission, entity-query
ordering, effect merging and sound/particle resource policy remain under their shared owners; this
rule fixes the bell's exact caller inputs and subtype transaction.

**Applies when:**

A bell is placed or receives support/power changes, main-hand block use or a projectile/explosion
reaches its ring hook, block event 1 executes, its server/client block-entity ticker runs, or its
transient state is rendered.

**Authoritative state:**

The block has horizontal `facing` × `attachment={floor,ceiling,single_wall,double_wall}` ×
`powered`, exactly 32 states; default is north/floor/false. It is gold-colored, forced-solid,
strength 5, anvil-sounding, destroyed by pistons and never pathfindable. Its block entity has
transient `lastRingTimestamp=0`, `ticks=0`, `shaking=false`, nullable click direction and
nearby-living list, `resonating=false`, and `resonationTicks=0`; none is persisted or transferred as
an item component.

**Transition and ordering:**

Placement/support determines the block tuple. An admitted ring first mutates the server block entity
and queues event `(1,directionId)`, then immediately emits the ordinary bell sound/game event and
optional stat. Only when the queued event later executes does each side refresh/reuse its own
living-entity cache, update server hearing memories, reset synchronized shake/resonance counters and
begin the ticker transaction. Glow and particles occur only after the full resonance clock below.

**Placement, support, shape, and updates:**

A vertical click tries only floor for an upward face or ceiling for a downward face, with player
horizontal facing, and returns null if that candidate cannot survive. A horizontal click faces back
toward the clicked support. It chooses double-wall only when both opposing neighbors along that axis
expose the required sturdy faces, otherwise single-wall; after survival failure it falls back to
floor when the below top face is sturdy, otherwise ceiling, retaining that horizontal facing, and
returns null if the fallback fails. Every placement candidate starts unpowered. Floor and wall use
ordinary sturdy-face attachment. Ceiling requires center support on the block above's downward face
and rejects the locked `unstable_bottom_center` tag, whose only expansion is all fence gates.

On a support-side shape update, a nonsurviving floor/ceiling/single wall becomes air. Double wall
skips that immediate removal: loss of either axis neighbor's tested sturdy face downgrades to single
wall and flips facing toward the other side; a single wall upgrades to double when its opposite axis
neighbor gains the tested sturdy face. Later loss of the retained single support can remove it.
Rotation/mirror change only facing. Collision and outline are identical: the centered body is the
union of X/Z `5..11`, Y `6..13` and X/Z `4..12`, Y `4..6`; ceiling adds a centered 2-wide column Y
`13..16`, floor adds a centered `16×16×8` prism rotated by facing axis, double wall adds a rotated
2-by-16 column Y `13..15`, and single wall adds the facing-rotated 2-wide bracket from local Z
`0..13`, Y `13..15`.

**Ring admission and hit geometry:**

Main-hand block use reaches `useWithoutItem` after the base block's try-empty-hand result, including
when a main-hand item is present; generic secondary-use bypass with either hand nonempty skips it,
and offhand alone does not enter this empty-hand fallback. Projectile impact uses the projectile
owner only when it is a player. Both require a proper hit: vertical faces fail; local hit Y strictly
above `0.8124F` fails, equality passes; a floor accepts horizontal faces whose axis equals facing
axis, either wall attachment accepts the perpendicular axis, and ceiling accepts every horizontal
face. A proper client hit returns success without ringing locally. A proper server hit returns true
even if the matching block entity is missing, but stat award requires actual ring success. Player
direct use or player-owned projectile awards `bell_ring` once on success; nonplayer projectiles do
not. Public callers that set `requireHitFromCorrectSide=false` bypass only this geometry.

`attemptToRing` succeeds only server-side with the matching block entity. A null direction becomes
current facing. `onHit` stores direction, resets `ticks` only when already shaking or otherwise
starts shaking, and queues the exact block event. The caller then plays `BELL_BLOCK` in `BLOCKS` at
volume 2/pitch 1 and emits source-context `BLOCK_CHANGE`. Multiple successful attempts therefore
each emit sound/event/stat even when identical queued event records deduplicate; different direction
parameters remain distinct records. A neighbor callback samples `hasNeighborSignal`; only a
false-to-true edge attempts a facing-directed ring, then both edges offer the captured state with
`powered=signal`, flags 3, regardless of ring success. A trigger-capable explosion attempts a
facing-directed source-null ring before generic explosion handling.

**Queued event and living cache:**

Event ID 1 invokes `updateEntities`, resets `resonationTicks`, stores
`Direction.from3DDataValue(parameter)`, resets `ticks`, sets shaking and returns true for broadcast;
other IDs delegate. Cache refresh occurs only when the list is null or current game time is strictly
greater than `lastRingTimestamp+60`; equality reuses it. Refresh snapshots every living entity whose
box intersects the block AABB inflated 48 and records the current time. On the server event, each
cached entity that is alive, not removed and at strict center distance below 32 receives brain
memory `HEARD_BELL_TIME=currentGameTime`, regardless of raider membership. Client event execution
builds only its independent cache.

**Shake and resonance clock:**

Each admitted side-specific block-entity tick first increments `ticks` while shaking. At
`ticks>=50`, it clears shaking and resets ticks to zero before later checks. At any still-shaking
tick `>=5`, if `resonationTicks==0` and a cached alive, nonremoved `#raiders` entity is currently at
strict center distance below 32, set resonating and call `BELL_RESONATE` at volume/pitch 1. The tag
is exactly evoker, pillager, ravager, vindicator, illusioner and witch. The effective audible call
originates on the server; the client ticker's null-excluded `ClientLevel` call does not play for an
existing local player. A resonance tick increments while below 40; on the following tick with value
40 it runs the side-specific end action and clears resonating without resetting the value. Resonance
may continue after shaking stops.

A new event always resets `resonationTicks` but deliberately does not clear `resonating`. Thus a
rerung active resonance restarts its 40-tick duration immediately, continues incrementing even
during the new shake's first four ticks, and emits no second resonance sound. A completed resonance
cannot restart from a later cached raider until a new block event resets its counter. When no raider
qualifies at tick 5, the test repeats on every shaking tick through 49 against current positions in
the same cached list.

**Server glow and client particles:**

The server end action applies a default amplifier-zero Glowing instance for 60 ticks to every cached
entity that is then alive, not removed, tagged raider and at strict center distance below 48;
generic effect merge owns the result against an existing instance. The client end action first
counts cached living entities whose current center is below 48 without testing liveness, removal or
raider tag. Let `n=clamp((count-21)/-2,3,15)` using Java integer division. In cached-list order,
each currently eligible raider below 48 receives `n` deterministic `ENTITY_EFFECT` particles. A
single mutable color begins at `16700985` and adds 5 before every particle across all raiders.
Particle Y is `bellY+0.5`; X/Z is bell center plus a unit horizontal vector computed from the entity
to the bell's integer X/Z corner. Zero horizontal distance therefore feeds nonfinite division
results to the generic particle admission path. Velocity is zero and this transaction consumes no
RNG.

**Rendering, persistence, item, and loot:**

While shaking, render time is `ticks+partialTicks` and
`baseRot=sin(renderTime/PI)/(4+renderTime/3)`. North/south apply negative/positive X rotation;
east/west apply negative/positive Z rotation; otherwise both are zero. Removal or chunk reload
discards all shake, resonance, cache and timestamp state because the subtype has no
save/load/component fields; already-applied memories/effects remain entity-owned. The bell item has
max stack 64 and no special default component. The locked block loot yields one bell only through
`survives_explosion` and copies no transient state.

**Branches and aborts:**

Every placement face/support and upgrade/downgrade; powered equality/rising/falling; main/offhand
and secondary-use routing; proper-hit Y/axis/attachment; player/nonplayer projectile; client/wrong
BE; null direction; duplicate/different block events; trigger-capable explosion;
null/stale/refreshed caches; shake/resonance boundaries; raider tag/liveness/distance; effect merge;
renderer direction; unload and explosion loot.

**Constants and randomness:**

Hit Y `0.8124F`; ordinary/resonance sound volume `2/1`, pitch 1; event ID 1; cache/search/highlight
radii `48`, hearing/resonance radius 32; strict cache interval 60; shake 50; resonance delay 5 and
duration 40; glow duration 60; particle color/count/position formula above; state flags 3. No
bell-owned transaction consumes RNG; generic sound seeds, entity-query order, effect merge and
particle admission are shared-owner boundaries.

**Side effects:**

Block state/support writes, queued/deduplicated block event and broadcast, two sounds,
source-context game event, bell-ring stat, living brain memories, transient server/client state,
glowing effect, client render rotation and particles, plus generic explosion/block loot.

**Gates:**

Placement replaceability/survival, sturdy support, current state/subtype, interaction priority, hit
geometry, logical side, neighbor signal edge, explosion interaction mode, block-event
activity/current block, block-entity ticking, cache timing, living state/tag/current distance,
client local-player exclusion, effect and particle settings. Difficulty and raid membership beyond
the raider tag do not gate this rule.

**Boundary cases and quirks:**

A geometrically proper use succeeds on the client or with a missing server block entity even though
no ring occurs. Identical queued events can deduplicate while every ingress already emitted its
immediate sound/event/stat. Cache equality at 60 ticks is stale. Hearing applies to all living
entities inside 32; glow/particles use raiders inside 48. Particle density is based on all cached
living entities currently inside 48, not raider count. Re-ringing an active resonance restarts its
end clock without a new resonance sound. None of the visible motion survives reload.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.BellBlock`,
`net.minecraft.world.level.block.entity.BellBlockEntity#onHit`,
`net.minecraft.world.level.block.entity.BellBlockEntity#triggerEvent`,
`net.minecraft.world.level.block.entity.BellBlockEntity#serverTick`,
`net.minecraft.world.level.block.entity.BellBlockEntity#clientTick`,
`net.minecraft.world.level.block.entity.BellBlockEntity#updateEntities`,
`net.minecraft.world.level.block.entity.BellBlockEntity#showBellParticles`,
`net.minecraft.world.level.block.entity.BellBlockEntity#makeRaidersGlow`,
`net.minecraft.client.renderer.blockentity.BellRenderer`,
`net.minecraft.client.model.object.bell.BellModel`; locked block/item reports,
`data/minecraft/tags/entity_type/raiders.json`,
`data/minecraft/tags/block/unstable_bottom_center.json`, and
`data/minecraft/loot_table/blocks/bell.json`; `EXP-BLK-009`.

**Test vectors:**

Exhaust 32 states and every placement/support/update shape; proper-hit axes and Y below/equal/above
threshold under main/offhand, item/empty and secondary use; player/nonplayer projectiles, both power
edges, wrong/missing BE and every explosion interaction; identical/different queued event records;
cache null and times 59/60/61 with moved/dead/removed tagged and ordinary living entities at strict
32/48 boundaries; shake 0/4/5/49/50, late resonance and all 40 end ticks; rerings before event,
during shaking, active resonance and after completion; server glow merges; client all-living count
formula, list order, zero horizontal distance, colors and exact animation axes; unload/reload and
loot admission. Run `EXP-BLK-009` as the executable matrix.
