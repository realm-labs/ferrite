# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BRUSHABLE-001` — Brushable blocks serialize ten accepted strokes into one exposed archaeology item

**Parent:** `SIM-003`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`,
`BLK-007`, `PLY-005`, `ITM-001`, `ITM-003`, `ITM-006`, `ENT-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client classes, block/item/registry reports and the brush item
model fix the complete use cadence, shared block-entity cooldown, dust stages, delayed regression,
loot materialization, completion, persistence and visible item projection for suspicious sand and
suspicious gravel. The selected archaeology loot table and generic item durability, item-entity,
scheduled-tick, falling-block and effect-delivery transactions retain their existing owners; this
rule fixes the brushable subtype's inputs and ordering at every join.

**Applies when:**

A player starts or continues using a brush, a brush pulse targets any block, a matching brushable
block entity accepts or rejects a pulse, a suspicious block receives a scheduled tick or loses
support, archaeology loot is materialized, the block completes or regresses, its state is saved or
loaded, or the client renders the hidden item and completion effect.

**Authoritative state:**

Suspicious sand and suspicious gravel each have only integer `dusted=0..3`, default `0`. Their
locked state IDs are sand `119..122` and gravel `125..128` in dust order. Sand turns into ordinary
sand and uses `item.brush.brushing.sand`; gravel turns into ordinary gravel and uses
`item.brush.brushing.gravel`. Within each registration, ordinary strokes and completion use the
same resource, strength `0.25`, destroy piston reaction, and a scheduled delay of two ticks.

The block entity starts with internal brush count `0`, reset deadline `0`, cooldown deadline `0`,
empty materialized item, null hit direction, null loot-table key and seed `0`. The block entity type
is protocol ID `40`. The brush is a singleton-stack item with 64 durability, `BRUSH` use animation
and use duration 200 ticks.

**Transition and ordering:**

Use admission starts a continuous reraycast. Each residue-five pulse predicts dust and sound before
the server checks the matching entity and its shared cooldown. An admitted pulse materializes loot,
increments count, publishes a changed dust stage and schedules reset work, or on count ten ejects
content, sends completion event 3008, replaces the block and damages the brush. Due block work
regresses count before testing support and possibly entering the separately owned falling pipeline.

**Starting and continuing brush use:**

`useOn` obtains the player. With a player, it independently raycasts along the view vector through
the player's current block-interaction range using pickable entities as interceptors; only a block
result starts use in the requested hand. A null player, miss or entity hit does not start use.
Every branch nevertheless returns `CONSUME`.

While use continues, negative remaining time or a non-player user immediately releases the item.
Every tick reruns the same view raycast; a non-block result also releases immediately. Let
`u=200-remaining+1`. Work occurs only when `u mod 10 = 5`, so the normal pulse sequence is use ticks
5, 15, 25, and so on. Other ticks do not read the target state or produce brush effects.

An effect pulse reads the current hit position and state, derives the visual arm from used hand and
main arm, optionally creates block dust, then plays one sound at volume/pitch 1 in `BLOCKS`, with
the brushing player as the excluded/predicting entity. A `BrushableBlock` selects its registered
stroke sound; every other block uses `item.brush.brush_generic`. This audiovisual pulse happens on
the client path as well as before the server's matching-entity check. Therefore an ordinary block,
a missing/wrong block entity, or a server-side cooldown rejection still has a predicted stroke but
does not advance archaeology state.

**Dust particles:**

Particles require `shouldSpawnTerrainParticles` and a non-invisible render shape. Each pulse first
draws a uniform count in `7..11`, then emits that many `BLOCK` particles carrying the target state
at the exact hit location. West and north hits subtract `1e-6` from X or Z respectively to keep the
particles on the visible face. Each particle consumes independent X and Z doubles. Horizontal
faces use fixed tangential/outward vectors with components `1` and `0.1`; top/bottom derive their
horizontal vector from `(viewZ, -viewX)`. The selected arm multiplies both horizontal velocities by
`+1` for right and `-1` for left, then by `3` and the independent draw. Vertical velocity is zero.

**Server admission, shared cooldown, and dust stages:**

Only a `ServerLevel` plus a `BrushableBlockEntity` at the reraycast position calls `brush`. The
first call with null hit direction stores the current hit face. Every call, including a cooldown
rejection, first sets `resetAt=gameTime+40`. A call with `gameTime<cooldownEndsAt` then returns false
without loot resolution, count change, scheduling, state publication or durability damage. Thus
the cooldown is block-entity-global across players, while rejected competing pulses can postpone
regression. Equality is admitted.

An admitted pulse sets `cooldownEndsAt=gameTime+10`, materializes pending loot, records the old
completion stage, increments the count, and completes at count 10. Counts below completion map to
stages exactly as follows:

| Internal count | `dusted` |
|---:|---:|
| `0` | `0` |
| `1..2` | `1` |
| `3..5` | `2` |
| `6..9` | `3` |

Every admitted nonfinal pulse schedules this same block after two ticks. It writes flags `3` only
when the internal stage changed; write success is ignored. The server returns false for all nine
nonfinal accepted pulses and true only on completion. `BrushItem` consequently damages the used
brush by one only after the tenth accepted pulse; generic durability owns creative, unbreaking and
break handling. Continuous single-player pulses land exactly at the equality boundary and need 95
use ticks from the first local tick to the tenth effect pulse.

**Loot materialization:**

The first admitted pulse with a pending table resolves it before count increment. A server player
first triggers `GENERATE_LOOT` for that table. The `ARCHAEOLOGY` context contains origin at block
center, the living brusher and its luck, and the exact used brush stack as tool. Evaluation uses the
stored loot seed. Zero results select empty, one selects it, and more than one logs a warning then
retains only the first result. The table key is cleared and the entity marked changed. No later
pulse reevaluates it.

Specific structure/worldgen leaves own which archaeology table and seed are installed. This leaf
owns their common runtime interpretation and deliberately admits any table that decodes through the
resource-key codec. The normal locked archaeology tables produce at most one stack, but the
multi-result warning/first-item branch remains observable for custom data.

**Regression and scheduled work:**

A suspicious block schedules itself after two ticks on placement and after every shape update.
Every due tick first calls `checkReset` on a matching entity, then independently checks falling.
If count is nonzero and `gameTime>=resetAt`, regression subtracts two with floor zero, writes flags
`3` only if the mapped dust stage changes, and sets the next reset deadline to `gameTime+4`.
Nonzero count then schedules another two-tick check. Reaching zero clears hit direction and both
deadlines; a zero count also clears those fields whenever a scheduled check happens. Before the
40-tick deadline, the due check preserves count and schedules another check after two ticks.

Regression therefore walks count `9→7→5→3→1→0` at four-tick deadline intervals, with visible
stages `3→3→2→2→1→0`; scheduled callbacks still occur at the intermediate unchanged stages. A
cooldown-rejected stroke changes only the first deadline and its predicted effects.

**Completion, item ejection, and replacement:**

The tenth accepted pulse first drops the materialized item. Nonempty content chooses the hit
direction, defaulting null to up, and centers an item entity in the adjacent block at
`adjacentY+0.5+itemEntityHeight/2`. Its stack is a split of uniform `10..30` units using level RNG,
its velocity is exactly zero, and insertion success is ignored. The block entity then clears its
item unconditionally, so an oversized remainder or a rejected item-entity insertion is lost.

Next the server sends level event `3008` at the source position with the pre-replacement block-state
ID. It finally writes the registered `turns_into` block's default state with flags `3`; a mismatched
non-`BrushableBlock` host instead turns into air. The completion path does not schedule another
reset tick and ignores replacement success. The event precedes replacement, while the item entity
precedes both.

The client decodes event `3008`'s state ID. If that old state belongs to a brushable block, it plays
the registered completion sound in `PLAYERS` at volume/pitch 1, then always adds the ordinary
destroy-block effect for the decoded old state. A nonbrushable state skips only the sound.

**Falling join:**

After reset processing, a due tick starts a generic fall exactly when the below state is free and
source Y is at least the level minimum. It immediately disables the falling entity's drop. As
specified by `BLK-FALL-001`, the carried block state retains `dusted`, but no loot key, materialized
item, hit direction, internal count or deadlines are copied. A successful landing creates a fresh
empty entity; a failed landing emits event `2001` and `BLOCK_DESTROY` without an ordinary item,
subject to the separately owned generic timeout quirk.

The client ambient path consumes `nextInt(16)` every animation tick. Only zero plus a free below
state emits one `FALLING_DUST` particle at random X/Z within the block and Y-0.05 with zero velocity.

**Persistence and synchronization:**

Full save writes either `LootTable` plus optional nonzero `LootTableSeed`, or a nonempty `item`,
never both. It does not write hit direction. Load gives a decoded loot table priority and clears
the live item; otherwise absent/invalid item is empty, and it also accepts optional
`hit_direction` through the legacy direction-ID codec because update tags share this load path.
Count, reset deadline and cooldown deadline are never serialized. `setLootTable` itself only stores
key and seed; it does not dirty or publish the entity.

The update packet is the ordinary block-entity data packet. Its tag contains nullable
`hit_direction` and a nonempty materialized `item`, but never the pending table/seed, count or
deadlines. Consequently hidden loot is not revealed to a client before the first admitted stroke.
Across ordinary reload, the block's persisted `dusted` state can coexist with reset internal count
zero and null direction. The first accepted pulse captures its new face and recomputes from count
zero; the first scheduled reset check keeps direction null. An update-tag load can instead install
the synchronized direction without making it durable. This divergence is version-locked rather
than normalized.

**Client item projection:**

The block-entity renderer submits nothing unless `dusted>0`, hit direction is nonnull and the
materialized item render state is nonempty. It resolves the stack in `FIXED` context and samples
light from the adjacent hit-face position. Let `f=0.075*dusted`. After the common center offset,
the face coordinates are east `x=.73+f`, west `x=.25-f`, up `y=.75+f`, down `y=.27-f`, north
`z=.25-f`, or south `z=.73+f`, with the other two coordinates `.5`. The renderer rotates Y by
75 degrees, then by 101 degrees for east/west or 11 degrees otherwise, scales uniformly by 0.5,
and submits with no overlay.

The brush item model is a ten-tick `use_cycle` range dispatch scaled by 0.1, with thresholds
0.25/0.5/0.75 selecting the three brushing models and the ordinary brush as fallback. Generic
first-person `BRUSH` animation remains client-owned; this rule fixes the locked item-model selector.

**Branches and aborts:**

Null/no-block start; use release; every pulse residue; invisible/nonparticle state; ordinary versus
brushable sound; client/server side; missing/wrong entity; six hit faces and both arms; first versus
retained direction; cooldown below/equal/above; counts 0..10; every dust threshold; pending/absent
loot and zero/one/many results; nonplayer/player brusher; empty/nonempty/oversized content; entity
admission and replacement failure; every regression boundary; supported/free/min-Y fall; save,
load, update and restart divergence; six rendered faces and hidden/materialized item.

**Constants and randomness:**

Use duration 200; pulse period 10 at residue 5; entity cooldown 10; reset delay 40 then decay by two
every four ticks; completion at ten; dust stages above; scheduled delay two; block strength 0.25;
brush durability 64; particle count `7..11`; particle speed factors `3`, `1` and `0.1`; item split
`10..30`; events 3008 and 2001; renderer progress factor `0.075`, scale 0.5 and Y rotations 75 plus
101/11 degrees. Loot uses its stored seed; each effect pulse and content split use the described
level/client RNG streams. No count, cooldown or regression branch consumes RNG.

**Side effects:**

Continuous-use state and release; predicted sounds/particles; loot criterion and evaluation;
block-entity dirtying; scheduled ticks; flags-3 dust/replacement writes; brush durability; item
entity construction/insertion; completion and falling level/game events; block replacement;
persistence/update tags; ambient dust and block-entity/item-model rendering.

**Gates:**

Current raycast/range and pickable-entity interception; logical side; exact block-entity type;
global entity cooldown; reset/count deadlines; loot/resource validity; item-entity admission;
free-below/min-Y fall; render state/item/direction; client sound/particle/resource settings through
their generic owners. No difficulty or game rule directly gates brushing.

**Boundary cases and quirks:**

`useOn` consumes even when it cannot start use. Nonbrushable targets still show each predicted
stroke. A competing rejected stroke postpones regression. Direction is fixed by the first call,
not the completing player. Brush durability decreases once per completed block, not once per
stroke. Completion discards an oversized remainder and ignores entity/replacement failure. Falling
preserves dust state but destroys archaeology data. Reload preserves dust while resetting
direction, the internal count and both timers; an update tag alone carries direction.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.item.BrushItem`, `net.minecraft.world.item.BrushItem$DustParticlesDelta`,
`net.minecraft.world.level.block.BrushableBlock`,
`net.minecraft.world.level.block.entity.BrushableBlockEntity`,
`net.minecraft.client.renderer.blockentity.BrushableBlockRenderer`,
`net.minecraft.client.renderer.blockentity.state.BrushableBlockRenderState`,
`net.minecraft.client.renderer.LevelEventHandler`,
`net.minecraft.client.renderer.item.properties.numeric.UseCycle`; locked block/item/registry
reports and `assets/minecraft/items/brush.json`; `EXP-BLK-019`.

**Test vectors:**

Exhaust both blocks and all eight state IDs; use start/release and every pulse residue; six faces,
both hands/arms and intercepted raycasts; ordinary/missing/wrong targets; shared-player cooldown
interleavings; all counts, dust transitions and rejected writes; reset deadlines before/at/after
with postponed and reloaded state; every loot result count and context/seed; empty/unit/oversized
content plus failed insertion/replacement; completion order/durability/event; support/fall/landing/
timeout joins; full-save versus update-tag fields; item-model cycle plus every face renderer
transform. Run
`EXP-BLK-019` as the executable matrix.
