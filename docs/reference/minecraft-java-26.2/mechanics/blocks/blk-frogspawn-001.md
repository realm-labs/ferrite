# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-FROGSPAWN-001` — Frogspawn schedules a no-drop hatch into two to five persistent Tadpoles

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-USE-001`, `ITM-LOOT-001`, `ITM-ANVIL-001`, `ENT-001`,
`ENT-SPAWN-001`, `MOB-001`, `MOB-AI-001`, `MOB-BREED-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`,
`ENV-FIRE-001`, `ENV-LIGHT-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked `FrogspawnBlock`, `PlaceOnWaterBlockItem`,
frog lay-spawn behavior, registration, empty loot, support tags, all
`1,212` decoded templates and exact client resources close property-free
Frogspawn. Its special runtime is source-water placement and survival, a
delayed no-drop hatch into persistent Tadpoles, Falling-Block destruction
and priority-3 pregnant-frog placement beside water.

**Applies when:**

`minecraft:frogspawn` is placed, supported, neighbor-updated, scheduled,
hatched, contacted by a Falling Block, mined, exploded, piston-destroyed,
laid by a pregnant frog, persisted, synchronized or rendered.

**Authoritative state:**

Frogspawn is a property-free `FrogspawnBlock` extending `Block`, has no
block entity and has sole state ID `32084`. Block, block-type and item
protocol IDs are `1181/93/1455`; its stack-64 item is a
`PlaceOnWaterBlockItem`. Tadpole entity type ID is `131`.

Registration fixes `WATER` map color, default `HARP`, zero strength through
`instabreak`, no occlusion, no collision, Frogspawn sounds and piston
reaction `DESTROY`. Friction is `0.6`, speed/jump factors are `1`,
emission is `0`, and no correct tool is required.

The selection shape is `[0,0,0]..[1,1.5/16,1]`. Collision, support, visual
and occlusion shapes are empty. Frogspawn propagates skylight, has light
dampening `0` and shade brightness `1`, supplies no signal or comparator
output, and is not a conductor, view blocker, suffocation state or
spawn-support cube.

The sound profile has volume/pitch `1/1`. Step, break, fall, hatch, hit and
place IDs are `674/675/676/677/678/679`; Frog lay-spawn sound ID is `684`.

**Transition and ordering:**

### Air-use placement and support

`PlaceOnWaterBlockItem.useOn` always returns `PASS`. Air use performs the
shared player-view raycast with fluid mode `SOURCE_ONLY`, moves only the
hit block position upward, constructs `UseOnContext`, and delegates to
ordinary `BlockItem.useOn`.

For Frogspawn at `P`, survival reads `B=P.below()` and requires both:

- the fluid at `B` belongs to fluid tag `supports_frogspawn`, or the block
  at `B` belongs to block tag `supports_frogspawn`; and
- the fluid at `B.above()` is exact Empty.

The locked fluid tag contains only `minecraft:water`, meaning source Water
rather than `flowing_water`; the locked block tag is empty. Source fluid
in a waterlogged support can satisfy the fluid branch. Reload may populate
the block support tag without changing state `32084`.

Generic placement still owns replacement, border, collision, permission,
component and write gates. Every shape update rechecks survival; failure
returns Air, success delegates to the ordinary update. An explicit write
can initially leave unsupported Frogspawn, but a later shape update or its
scheduled tick removes it.

### Scheduled hatch

Every `onPlace` schedules this block after
`nextInt(minHatchTickDelay,maxHatchTickDelay)`. Static defaults are
`3600/12000`, giving `3600..11999` ticks. `setHatchDelay(min,max)` replaces
the process-global bounds without validation; `setDefaultHatchDelay`
restores them. Existing scheduled entries retain their chosen delay.

At the scheduled server tick:

1. Survival is evaluated.
2. Failure calls `Level.destroyBlock(P,false)`, ignores the result and
   returns without hatch effects.
3. Success also calls `destroyBlock(P,false)` and ignores the result.
4. Hatch sound `677` plays at `P`, category `BLOCKS`, volume/pitch `1/1`.
5. Tadpole spawning runs even if destruction failed.

Spawn count is `nextInt(2,6)`, hence `2..5`. Each iteration first calls
`TADPOLE.create(level,BREEDING)`. Null skips the remaining draws and
insertion for that Tadpole. Otherwise two `nextDouble` calls independently
select X/Z offsets using
`clamp(draw,0.20000000298023224,0.7999999970197678)`, and
`nextInt(1,361)` supplies yaw `1..360`. Y is `P.y-0.5`, pitch is `0`.
The Tadpole is snapped there, marked persistence-required, and offered
through `addFreshEntity`; its Boolean result is ignored. Failure does not
roll back destruction, sound or other Tadpoles. The raw doubles are
clamped, not rescaled, concentrating probability at both endpoints.

### Falling-Block contact and loot

When `entityInside` reaches Frogspawn, exact entity type `FALLING_BLOCK`
calls the same ignored-result `destroyBlock(P,false)`. No explicit
logical-side gate exists; client contact can remove local state while the
server remains authoritative. Every other entity has no special effect.

The generated Frogspawn loot table is deliberately pool-free: it contains
only type `block` and random sequence `minecraft:blocks/frogspawn`.
`VanillaBlockLoot` constructs it with `noDrop`. Hand, tools, Silk Touch,
Fortune, explosion, piston reaction, support failure, Falling-Block
contact and hatch therefore emit no Frogspawn item. Zero hardness makes
ordinary breaking immediate but creates no loot.

Frogspawn has no `FireBlock` row, lava-ignition property or fuel time:
encouragement/flammability are `0/0`. It has no random tick, fluid-state,
placement-state, transform, attack, fall, signal, comparator or
block-event override.

### Pregnant-frog placement

`FrogAi` installs `TryLaySpawnOnFluidNearLand.create(FROGSPAWN)` at
priority `3` in activity `LAY_SPAWN`, after priority-2
`TryFindLandNearWater`. The behavior requires absent `ATTACK_TARGET` and
present `WALK_TARGET` and `IS_PREGNANT` memories. It returns `false`
immediately while the frog is in water or not on ground.

Otherwise it starts below the frog and visits the fixed
`Direction.Plane.HORIZONTAL` iterator. For each adjacent support `B`:

1. Reject when `B`'s collision-shape upward face is nonempty.
2. Require its fluid in `supports_frogspawn` or block in the matching
   support tag.
3. Require `B.above()` to be exact Air.
4. Offer state `32084` there with flags `3`, ignoring the write result.
5. Emit `BLOCK_PLACE` with frog/state context.
6. Play lay sound `684`, `BLOCKS`, volume/pitch `1/1`, sourced by the frog.
7. Erase `IS_PREGNANT` and return `true`.

Event, sound and pregnancy erasure occur even when `setBlock` returns
false. If all four candidates reject, the behavior still returns `true`
but preserves pregnancy and emits nothing. Breeding, activity arbitration
and later brain scheduling retain their frog owners.

### Closures, absences and persistence

Frogspawn block and item belong directly to no locked block/item tag, so
both membership closures are empty. `supports_frogspawn` selects the
environment, not Frogspawn: its fluid closure is source Water and its
block closure is empty.

There is no recipe/unlock, Composter entry, fuel row, merchant offer,
fishing/chest/entity loot entry or ordinary survival item source. Natural
Blocks and commands supply the baseline item; pregnant-frog AI supplies
the ordinary world block.

No configured/placed feature, biome, processor or other worldgen data
mentions exact Frogspawn. Decoded and decompressed-string scans of all
`1,212` templates find zero raw cells and zero palette, final-state,
marker, block-entity or entity-NBT occurrences. Exact legacy-fix search
finds no Frogspawn migration.

Chunk palettes and block-update packets preserve only state `32084`;
stacks preserve identity, count and generic component patches. There is no
block-entity payload or Frogspawn-specific item component.

### Client projection

The property-free blockstate selects one unrotated block model. It disables
ambient occlusion and draws a two-sided, untinted horizontal plane at
`0.25/16`, with only up/down faces. Particle and faces use the same
transparent 16×16 `block/frogspawn` texture, SHA-256
`8eb9a395004b5bfee4b2524183fe07b1196b2aeaf7a100703affd6b53053c6bd`.
The plane is lower than the 1.5/16 selection shape.

The item uses one generated flat model with that texture and no predicate,
component branch or tint. The English name is `Frogspawn`. Natural Blocks
publishes it once, after Hanging Roots and before Turtle Egg, Sniffer Egg
and Dried Ghast; no other baseline creative tab contains it.

**Branches and aborts:**

- Direct use-on returns `PASS`; air use must source-raycast and pass
  generic placement plus live support and target-fluid predicates.
- Unsupported ticks destroy without drops and do not hatch.
- Supported ticks destroy before sound/spawn; failure does not abort.
- Null Tadpole construction skips later draws; insertion failure does not
  roll back.
- Only exact Falling-Block contact selects no-drop contact removal.
- Frog laying tests memories, mobility, support face/tag and Air in order;
  selection consumes pregnancy even after write failure, while no
  selection preserves it despite a true behavior result.

**Constants and randomness:**

State/block/block-type/item/Tadpole IDs `32084/1181/93/1455/131`; zero
strength; height `1.5/16`; sound IDs `674..679` and `684`; stack `64`;
default delay `nextInt(3600,12000)`; count `nextInt(2,6)`; X/Z clamp
`0.20000000298023224..0.7999999970197678`; yaw `nextInt(1,361)`; tag
closures `0/0`; structure files/cells `0/0`; render height `0.25/16`.

**Side effects:**

Placement, support removal and tick scheduling; no-drop mining,
piston/explosion/contact/hatch destruction; hatch/lay sounds; two-to-five
entity construction attempts; persistence marking and insertion; frog
game event/memory erasure; state/stack persistence and client projection.

**Gates:**

Use route, source raycast, generic placement, supporting fluid/block and
Empty target fluid; tick survival; destruction, construction and insertion
results; Falling-Block identity; frog activity/memory/water/ground state,
directional collision face, support tags and Air target; registry, reload
and client-resource validity.

**Boundary cases and quirks:**

Source Water in a waterlogged block supports Frogspawn, while the empty
block tag accepts no dry support. `onPlace` always schedules. A rejected
hatch destruction can leave Frogspawn while sound and Tadpoles appear.
Position draws clamp rather than scale. Contact lacks Lily Pad's explicit
server and Boat gates.

**Failure semantics:**

Generic placement/update/break retain their owners. All Frogspawn-specific
destruction ignores result and requests no drops. Hatch proceeds after
supported destruction failure. Null construction and failed insertion are
local to one Tadpole. Frog placement ignores its write result before
event, sound and pregnancy erasure. Reload changes future reads only.

**Client/server authority split:**

The client performs use raycast/prediction and renders the block/item/name.
The server owns tick hatching, authoritative support/destruction, Tadpole
creation, frog placement, events, sound and persistence. The unsided
Falling-Block callback can run in both logical levels.

**Observability:**

Observe IDs, shapes/light/redstone/piston, transformed hit, support reads,
delay/tick/destroy order, every count/position/yaw draw, construction/
persistence/insertion, contact identity, frog memories/direction/read/
write/event/sound order, empty closures, durable identity and projection.

**Persistence and reload:**

Frogspawn saves one identity and has no block entity. Its stack uses
generic components. Support tags, loot and client resources have separate
reload boundaries; registration, scheduling, contact, frog activity and
creative ordering are code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.FrogspawnBlock`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.PlaceOnWaterBlockItem`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.entity.ai.behavior.TryLaySpawnOnFluidNearLand`;
`net.minecraft.world.entity.animal.frog.FrogAi`;
`net.minecraft.data.loot.packs.VanillaBlockLoot`;
`net.minecraft.world.item.CreativeModeTabs`; reports, empty Frogspawn
loot, support tags, all `1,212` templates and exact client resources.
Complete compiled/data/fix/NBT searches find no other exact runtime path.

**Test vectors:**

Run `EXP-BLK-122` across identity, shapes/light/redstone/piston/tool/
explosion/use/support, delay endpoints, supported/unsupported ticks,
destruction, all spawn draws and entity results, client/server
Falling-Block contact, every frog candidate/write result, complete `0/0`
memberships and support closures, data/template/migration absences,
persistence/reload and projection. Assert ordering, constants, RNG
consumption, side effects, absences and vanilla convergence.

**Limits:**

Generic placement/update/mining, tick queues, entity creation, frog
breeding/brain arbitration, packets and rendering retain their owners.
Water, Falling Blocks, frogs and Tadpoles retain their catalog families.
This leaf fixes exact Frogspawn and every direct join selecting it.
