# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BUBBLE-COLUMN-001` — Bubble columns propagate a water-filled direction state into entity motion, boats, particles and sound

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`PLY-MOVE-001`, `PLY-MOVE-SPECIAL-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `ENT-001`, `ENT-EFFECT-001`, `MOB-AI-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`, `ENV-LIGHT-001`,
`CLI-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, complete server/client class-reference and
method-override sweeps, reports, block/fluid tags, generated client assets and exhaustive scans of
all 1,212 decoded structure templates fix both states, every creation/update route, entity
specialization, fluid/container behavior and client output. No item, block entity, loot, recipe,
worldgen record or raw/UTF-only template occurrence exists.

**Applies when:**

`minecraft:bubble_column` is created above a valid base, propagated or removed, receives neighbor
or fluid updates, is picked up with a bucket, intersects an entity, affects breathing, movement,
pathfinding, enchanting transmission or fluid placement, animates, persists, synchronizes or is
rendered.

**Authoritative state:**

Bubble Column is a `BubbleColumnBlock` with Boolean property `drag`, no block entity and exactly two
states. The default `drag=true` state is ID `15294`; `drag=false` is ID `15295`. Protocol block ID
is `796`. `drag=true` means downward whirlpool and `drag=false` means upward current. There is no
`minecraft:bubble_column` item registry entry, block item, component report, creative-tab entry,
loot table or ordinary placement stack.

Registration supplies water map color, replaceable, no collision, no loot table, piston reaction
`DESTROY`, liquid, empty sound type and no occlusion. Unset defaults are destroy time/resistance
`0`, friction `0.6`, speed/jump factors `1`, restitution and light emission `0`, and harp note
instrument. The selection, collision, occlusion, support, interaction and visual shapes are empty;
render shape is `INVISIBLE`, shade brightness is `1.0`, and it is not a redstone conductor,
view-blocking or suffocating surface. Its embedded full-source water makes skylight propagation
false and light dampening `1`. It is LAND-pathfindable because its collision shape is not full and
WATER-pathfindable because its fluid is tagged water.

`getFluidState` always returns nonfalling source water with amount `8`, independent of `drag`.
Block-level explosion resistance remains `0`, but the generic explosion calculation takes the
maximum of block and fluid resistance, so the effective resistance input at this cell is water's
`100`.

The only direct block tag is `replaceable`. It is a transitive member of
`enchantment_power_transmitter` because that tag contains `#minecraft:replaceable`; the sole fluid
admission tag `bubble_column_can_occupy` contains `minecraft:water`.

**Transition and ordering:**

Bubble state is derived from the live block directly below. Creation may begin from an eligible
full source-water `LiquidBlock` scheduled for `20` ticks or an existing Bubble Column scheduled for
`5`; the due callback calls `BubbleColumnBlock.updateColumn`. Each Bubble Column update also
schedules its embedded water's fluid tick at the water tick delay.

#### Column admission and propagation

A Bubble Column survives when the below block is another Bubble Column or belongs to
`enables_bubble_column_push_up` or `enables_bubble_column_drag_down`. A target cell can be occupied
when it is already a Bubble Column, or when all of these are true: its fluid is in
`bubble_column_can_occupy`, the block is a `LiquidBlock`, and the fluid is source with amount at
least `8`. A waterlogged non-liquid block and flowing water therefore fail even though their fluid
can be water.

The selected target state is ordered:

1. Below Bubble Column preserves the below state exactly, propagating its `drag` value.
2. A below `enables_bubble_column_push_up` member selects default state `15295`
   (`drag=false`).
3. A below `enables_bubble_column_drag_down` member selects default state `15294`
   (`drag=true`).
4. Otherwise an old Bubble Column becomes default water.
5. Otherwise the old target state is preserved.

`updateColumn` first rejects a nonoccupiable initial target. Otherwise it writes the selected first
state with flags `2` and ignores that result, then walks upward while each next cell is occupiable.
Each upper write uses flags `2`; the first rejected upper write aborts the walk. Thus a rejected
first write does not prevent attempted upper propagation, whereas a rejected later write does.
Every upper cell derives from the state being propagated, not by rereading a base tag.

`updateShape` schedules a Bubble Column block tick after `5` when the current state no longer
survives, for every downward neighbor update, or for an upward update whose new upper neighbor is
not already Bubble Column but is occupiable. It does not immediately replace the state and then
returns the base update result. `tick` rereads the current below state before updating the column.

Eligible full-source water `LiquidBlock` placement or neighbor change checks the live below-base
tags and schedules its block callback after `20`; its downward shape-update route does the same.
Ordinary `FlowingFluid` spreading explicitly excludes Bubble Column from its fluid-container
predicate, so this embedded-water block does not become a general container for a second fluid.

#### Bucket pickup

As a `BucketPickup`, Bubble Column attempts to write air at its position with flags `11`, ignores
the write result and returns exactly one Water Bucket. The pickup sound delegates to water and is
`item.bucket.fill` (sound-event protocol ID `230`). There is no Bubble Column item result and no
state-dependent branch.

#### Entity contact

The block effect runs only for a current bounding-box intersection accepted by the generic
inside-block traversal, not for a merely swept contact. The block reads the state immediately
above. When that block's collision shape is empty and its fluid is empty, the entity receives the
above-column handler; otherwise it receives the inside-column handler.

For generic `Entity`, the handlers preserve X/Z velocity:

| Contact | `drag=true` Y | `drag=false` Y | Other effect |
|---|---:|---:|---|
| open surface above | `max(-0.9, oldY - 0.03)` | `min(1.8, oldY + 0.1)` | server surface particles |
| inside/capped column | `max(-0.3, oldY - 0.03)` | `min(0.7, oldY + 0.06)` | reset fall distance |

The surface helper runs only in a `ServerLevel`. It loops twice and each iteration emits one
`SPLASH` and one `BUBBLE` at independently randomized X/Z coordinates on `y + 1`: splash has zero
delta and speed `1`, bubble has delta `(0,0.01,0)` and speed `0.2`. The two iterations therefore
emit two particles of each type and consume four X/Z pairs.

Concrete entity overrides retain these differences:

- A `Player` skips both handlers completely while flying; otherwise it uses the generic clamped
  behavior.
- A `Projectile` adds rather than clamps Y: surface `-0.03`/`+0.1`, inside
  `-0.03`/`+0.06`; inside also resets fall distance and surface emits the same particles.
- An `AbstractArrow` skips both handlers while in ground.
- A thrown Ender Pearl deliberately calls the generic `Entity` handlers, restoring the clamped
  behavior instead of Projectile's additive behavior.
- On its first tick, a `ThrowableProjectile` scans every block cell overlapping its box and calls
  Bubble Column contact with the intersection flag forced true before gravity/inertia, so a newly
  spawned throwable can receive this effect before normal block-effect traversal.

An `AbstractBoat` overrides the open-surface handler. On the server it marks itself above a column,
records the direction, and initializes a zero synchronized bubble timer to `60`. When not
underwater, a `nextInt(100)==0` branch plays `entity.generic.splash` at volume `1` and pitch
`0.8 + 0.4*nextFloat`, emits one splash at randomized X/Z and `y+0.7`, and fires a `SPLASH` game
event with the controlling passenger. Capped/inside boat contact still uses the generic clamped
handler.

Each server boat tick resets the timer to `0` if no open-surface contact was marked; otherwise it
decrements the timer. At expiry a downward column adds `-0.7` Y velocity and ejects all passengers.
An upward column replaces Y velocity with `2.7` when any passenger is a Player, otherwise `0.6`;
X/Z are preserved. The contact mark clears each tick. Client bubble multiplier changes by `+0.05`
while timer is positive and `-0.1` otherwise, clamped to `0..1`; rendered angle is
`10*sin(0.5*tickCount)*multiplier`.

#### Breathing, movement and other identity consumers

`LivingEntity#baseTick` treats eyes in water whose exact eye block is Bubble Column as the
out-of-water air-refill branch: air increases by `4` per tick, capped at maximum, rather than
entering the drowning branch. `BreathAirGoal#givesAir` first admits empty fluid or exact Bubble
Column and then requires LAND pathfindability; the empty collider makes Bubble Column pass.

`Entity#getBlockSpeedFactor` returns the current block's factor immediately for exact water or
Bubble Column. Bubble Column therefore returns its own `1` instead of inheriting a slowing factor
from the block below. It also transmits enchanting power through an intermediary position because
of transitive `enchantment_power_transmitter` membership; bookshelf provider distance and all
other enchanting gates remain with `BLK-ENCHANTING-TABLE-001`.

The legacy `ChunkProtoTickListFix` includes Bubble Column in its always-waterlogged identity set
when migrating old `LiquidTicks` to `fluid_ticks`. This migration role does not add a current
block-state property or persistence payload. The block itself has no random tick, use, attack,
fall, comparator, signal, block-event or server particle/sound callback beyond the paths above.

**Client projection:**

The generated blockstate has one unconditional empty variant referencing
`minecraft:block/water`; model generation aliases Bubble Column to Water. There is no dedicated
block model or item definition/model. `INVISIBLE` suppresses ordinary block geometry and the
embedded source-water fluid supplies the visible surface, tinted by biome water color. The block
color registration uses the water-particle tint source: direct block color is `-1`, while terrain
particles use the biome's average water color.

Each client animation tick emits direction-specific particles and may emit an ambient sound:

- `drag=true` always creates one `current_down` particle at `(x+0.5,y+0.8,z)` with zero input
  velocity, then `nextInt(200)==0` plays `block.bubble_column.whirlpool_ambient` at the block,
  volume `0.2 + 0.2*nextFloat` and pitch `0.9 + 0.15*nextFloat`, without distance delay.
- `drag=false` creates one `bubble_column_up` at `(x+0.5,y,z+0.5)` and a second at independently
  randomized X/Y/Z, both with input velocity `(0,0.04,0)`, before the same one-in-200 sound draw.
  Success plays `block.bubble_column.upwards_ambient` with the same volume/pitch ranges.

The downward particle starts at Y velocity `-0.05`, lives `30 + floor(nextFloat*60)` ticks, steers
horizontally by a rotating angle and removes when outside water or on ground. The upward particle
applies per-axis noise to scaled input velocity, uses gravity `-0.125`, friction `0.85`, lifetime
`floor(40/(0.2+0.8*nextFloat))`, and removes when outside the water fluid tag. Both use the
`minecraft:bubble` texture and opaque render type.

The client ambient handler searches loaded states intersecting the player box inflated by
`(0,-0.4000000059604645,0)` and then deflated by `0.000001`. On the first found Bubble Column after
a tick with none, except the handler's own first tick and while not spectator, it plays
`whirlpool_inside` for `drag=true` or `upwards_inside` for false at volume/pitch `1`. Finding a
column sets the latch even on first tick or while spectator; finding none clears it.

Sound-event IDs are Bubble Pop `217`, Upwards Ambient `218`, Upwards Inside `219`, Whirlpool
Ambient `220`, Whirlpool Inside `221`, and UI HUD Bubble Pop `222`. This block emits IDs
`218..221` as specified above. Exact consumer search finds no Bubble Column path that emits
registered block Bubble Pop `217`; HUD air-bubble output is separately owned.

**Branches and aborts:**

Base identity/tag order; old-column versus full-source-water admission; first versus upper write
failure; neighbor direction and survival scheduling; live below-state reread; bucket write result;
surface versus capped contact; generic/player/projectile/arrow/pearl/boat dispatch; flying,
in-ground, underwater and passenger gates; drowning/air-goal/speed-factor identity checks; client
direction, sound draw, first-tick, spectator and transition latch; fluid/tag reload and persistence
are distinct.

**Constants and randomness:**

States `15294`/`15295`, block ID `796`, source amount `8`; schedules `5`/`20`; writes `2` and `11`;
block/fluid explosion inputs `0`/`100`; dampening `1`; friction `0.6`; generic velocity clamps and
increments in the table; two Splash plus two Bubble surface particles; boat timer `60`, expiry
velocities `-0.7`/`0.6`/`2.7`, client increments `+0.05`/`-0.1`, splash chance `1/100`; animation
sound chance `1/200`, volume `[0.2,0.4)`, pitch `[0.9,1.05)`; ambient search inflation
`-0.4000000059604645` and deflation `0.000001`; air refill `4`; particle constants above.

**Side effects:**

Scheduled block/fluid ticks; flags-`2` propagation or water fallback; flags-`11` air write and Water
Bucket result; entity velocity/fall-distance/passenger state; server particles and game event;
boat synchronized timer and client wobble; breathing/path/speed/enchanting decisions; legacy fluid
tick migration; client fluid tint, particles and transition/ambient sounds.

**Gates:**

Loaded state and state-write authority; live block/fluid tags; exact source amount and `LiquidBlock`
class; survival and neighbor direction; current intersection and above collision/fluid emptiness;
entity subtype/state; server/client side, boat water/passenger/timer state; eye block and
pathfinding type; animation RNG and ambient latch; client render/fluid context.

**Boundary cases and quirks:**

The property name `drag` is true for downward motion. Existing column state outranks both base tags.
A rejected first propagation write does not abort the upward walk, but a rejected upper write does.
The block contains full source water yet rejects ordinary fluid-container spreading, does not
propagate skylight and lets breathing AI treat it as air. Open-surface detection requires both
empty collision and empty fluid above. Ender Pearls intentionally avoid the Projectile additive
override. The sound registry contains Bubble Pop, but the block never emits it. It is a replaceable
block with no obtainable item and no raw structure-template occurrence.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.BubbleColumnBlock`,
`net.minecraft.world.level.block.BubbleColumnBlock#updateColumn`,
`net.minecraft.world.level.block.BubbleColumnBlock#getColumnState`,
`net.minecraft.world.level.block.BubbleColumnBlock#canOccupy`,
`net.minecraft.world.level.block.BubbleColumnBlock#canSurvive`,
`net.minecraft.world.level.block.BubbleColumnBlock#updateShape`,
`net.minecraft.world.level.block.BubbleColumnBlock#tick`,
`net.minecraft.world.level.block.BubbleColumnBlock#entityInside`,
`net.minecraft.world.level.block.BubbleColumnBlock#pickupBlock`,
`net.minecraft.world.level.block.BubbleColumnBlock#animateTick`,
`net.minecraft.world.level.block.LiquidBlock#tryScheduleBubbleBlockColumn`,
`net.minecraft.world.level.material.FlowingFluid#canHoldAnyFluid`,
`net.minecraft.world.entity.Entity#onAboveBubbleColumn`,
`net.minecraft.world.entity.Entity#onInsideBubbleColumn`,
`net.minecraft.world.entity.Entity#sendBubbleColumnParticles`,
`net.minecraft.world.entity.Entity#getBlockSpeedFactor`,
`net.minecraft.world.entity.LivingEntity#baseTick`,
`net.minecraft.world.entity.player.Player#onAboveBubbleColumn`,
`net.minecraft.world.entity.player.Player#onInsideBubbleColumn`,
`net.minecraft.world.entity.projectile.Projectile#onAboveBubbleColumn`,
`net.minecraft.world.entity.projectile.Projectile#onInsideBubbleColumn`,
`net.minecraft.world.entity.projectile.arrow.AbstractArrow#onAboveBubbleColumn`,
`net.minecraft.world.entity.projectile.arrow.AbstractArrow#onInsideBubbleColumn`,
`net.minecraft.world.entity.projectile.throwableitemprojectile.ThrownEnderpearl#onAboveBubbleColumn`,
`net.minecraft.world.entity.projectile.throwableitemprojectile.ThrownEnderpearl#onInsideBubbleColumn`,
`net.minecraft.world.entity.projectile.ThrowableProjectile#tick`,
`net.minecraft.world.entity.vehicle.boat.AbstractBoat#onAboveBubbleColumn`,
`net.minecraft.world.entity.vehicle.boat.AbstractBoat#tickBubbleColumn`,
`net.minecraft.world.entity.ai.goal.BreathAirGoal#givesAir`,
`net.minecraft.world.level.block.EnchantingTableBlock#isValidBookShelf`,
`net.minecraft.util.datafix.fixes.ChunkProtoTickListFix`,
`net.minecraft.client.color.block.BlockColors`,
`net.minecraft.client.data.models.BlockModelGenerators`,
`net.minecraft.client.resources.sounds.BubbleColumnAmbientSoundHandler#tick`,
`net.minecraft.client.particle.WaterCurrentDownParticle`,
`net.minecraft.client.particle.BubbleColumnUpParticle`;
`reports/blocks.json#minecraft:bubble_column`,
`reports/registries.json#minecraft:{block,sound_event,particle_type}`,
`data/minecraft/tags/block/{replaceable,enchantment_power_transmitter}.json`,
`data/minecraft/tags/fluid/bubble_column_can_occupy.json`,
`data/minecraft/structure/**/*.nbt`,
`assets/minecraft/blockstates/bubble_column.json`,
`assets/minecraft/particles/{current_down,bubble_column_up}.json`,
`assets/minecraft/lang/en_us.json`.

**Test vectors:**

Run `EXP-BLK-107` across both state/ID tuples; every valid/invalid base, source/flowing/waterlogged
target, neighbor direction, schedule, first/upper write result and tag reload; bucket pickup;
surface/capped contact for generic, flying Player, Projectile, in-ground Arrow, Ender Pearl,
first-tick Throwable and Boat with all timer/passenger/water branches; breathing, air-goal,
speed-factor, fluid-container, enchanting and legacy-datafix consumers; client tint/model, both
particle branches, sound boundaries and ambient latch; all 1,212 templates and persistence. Assert
exact states, read/draw/write order, velocities, particles, sounds, absence claims and vanilla
convergence.

**Limits:**

This leaf does not re-specify generic state writes/scheduling, water flow, collision traversal,
entity movement/damage, drowning, path search, enchanting-table geometry, boat networking, particle
engine integration, sound playback, chunk data fixing, state packets or model loading. Those
remain with `BLK-UPDATE-001`, `ENV-FLUID-001`, player/entity/AI owners, `BLK-ENCHANTING-TABLE-001`,
`CLI-EFFECT-001`, persistence owners and `CLI-006`.
