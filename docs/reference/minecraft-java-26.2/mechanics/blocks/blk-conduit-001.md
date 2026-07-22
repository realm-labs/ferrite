# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CONDUIT-001` — Conduits scan water and frame state before powering players or attacking one wet enemy

**Parent:** `SIM-003`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`,
`BLK-007`, `ENV-001`, `ENV-002`, `ENV-003`, `ENT-005`, `ENT-006`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server and client classes, block/registry/item reports, bundled block,
item and loot assets fix the water gate, 42-position frame scan, effect radius, target retention,
damage, sound, particle, persistence and renderer branches. Generic fluid scheduling, mob-effect
merge, magic-damage defenses and sound/particle audiences remain owned by their parent rules.

**Applies when:**

A conduit is placed, waterlogged, block-entity ticked, activated, deactivated, saved, loaded,
synchronized, rendered or used as an item model; or its active scan applies Conduit Power and its
full-frame scan selects, retains or attacks an enemy.

**Authoritative state:**

The block has two locked states: state ID `15276` is the default `waterlogged=true`, and state ID
`15277` is `waterlogged=false`. Placement replaces that default with true only when the placement
cell's fluid is tagged water and is full. A waterlogged state exposes a source-water fluid state;
neighbor shape updates schedule the ordinary water-fluid tick.

The block entity begins with `tickCount=0`, `activeRotation=0`, inactive, not hunting, an empty
effect-block list, no destroy-target reference and `nextAmbientSoundActivation=0`. Active, hunting,
frame positions, counters, rotation and ambient deadline are runtime-only. The optional destroy
target is the sole custom saved and synchronized field, encoded under `Target` as an entity UUID.
Its block-entity protocol ID is `25`.

The block emits light level `15`, has diamond map color, hardness and blast resistance `3`, no
occlusion, forced solid behavior, hat note-block instrument and a centered `6×6×6`-pixel shape from
coordinates 5 through 11 on each axis. It is never pathfindable. Its unconditional loot table drops
one conduit, and the block belongs directly to `mineable/pickaxe`. The item stacks to 64, has
uncommon rarity and uses the conduit special item renderer.

**Transition and ordering:**

Each admitted block-entity tick advances local animation time. At global 40-tick boundaries the
water scan precedes frame collection, active-change sound and state installation, hunting
derivation, player-effect refresh, then target resolution, attack and target-change projection.
Ordinary active ambient clocks run afterward on every server tick. Client ticks independently
derive structure state, emit frame/target particles and advance active rotation before render
frames interpolate the stored state. Save/update paths expose only the optional target UUID.

**Tick domains and refresh cadence:**

`ConduitBlock#getTicker` installs `clientTick` on a client level and `serverTick` otherwise, only
for the conduit block-entity type. Both increment their local `tickCount` every admitted
block-entity tick. Frame and water state are recomputed only when the local level game time is
divisible by 40; this is a global phase, not a per-entity countdown.

On each server refresh, the implementation computes the new active value, emits activate or
deactivate sound if it differs from the prior value, stores it, derives hunting from the refreshed
frame count, and, only when active, refreshes player effects then target/attack state. On each client
refresh, it independently recomputes active and hunting from client world state. No active or
hunting Boolean arrives in the update tag.

A newly constructed or reloaded entity therefore remains locally inactive until the next
`gameTime % 40 == 0` refresh even when its surrounding structure is already valid. A block or fluid
change between refreshes leaves the previous active/hunting result in effect until that boundary.

**Water volume and frame scan:**

Every refresh first clears the remembered effect-block list. It then checks all 27 positions in
the centered offsets `[-1,1]³`, including the conduit itself, in X/Y/Z nested order. The first
position for which `Level.isWaterAt` is false aborts immediately as inactive. Consequently the
conduit must itself be waterlogged and the other 26 cells must contain water; a dry conduit cannot
activate even if every neighboring cell is water.

If the water volume passes, the scan visits offsets `[-2,2]³` and considers exactly 42 frame
positions: the union of the three centered 5×5 axial rings, excluding the inner `3×3×3` cube and
deduplicating their intersections by the single coordinate traversal. A candidate contributes one
position when its block is prismarine, prismarine bricks, sea lantern or dark prismarine. Other
blocks are ignored; the valid positions need not form a contiguous ring.

At least 16 collected positions makes the conduit active. Hunting requires at least 42, and 42 is
the complete candidate set, so hunting means every candidate is one of the four valid blocks. The
ordered collected list is also the client particle source.

**Player effect refresh:**

For an active frame with `n` valid positions, the effect radius is
`16 × floor(n / 7)` blocks. Thus the reachable tiers are 32 blocks for `n=16..20`, 48 for
`21..27`, 64 for `28..34`, 80 for `35..41`, and 96 for the complete 42-position frame.

The server queries players in an inflated conduit-cell box, then admits a player only when the
conduit's integer block position is strictly closer than the radius to the player's block position
and `Player.isInWaterOrRain` is true. Each admitted player receives Conduit Power amplifier 0 for
260 ticks with ambient and visible flags true. The refresh repeats every 40 game ticks, so an
uninterrupted admitted player normally retains the effect with a 220-tick margin. Effect merge,
immunity, removal and client effect projection remain owned by `ENT-EFFECT-001`.

**Hunting target state machine:**

Target work runs only on an active server refresh. With fewer than 42 frame blocks it returns null,
clearing any old reference. With a complete frame and no reference, it queries living entities in
the conduit block AABB inflated by 8 on every axis, filters to `Enemy` instances currently in water
or rain, and chooses one uniformly with the level RNG. A newly chosen target is eligible for the
same refresh's attack.

With an existing reference, UUID resolution must produce a living entity that is alive and whose
integer block position is strictly within distance 8 of the conduit block position. Failure returns
null and does not choose a replacement during that refresh; the next 40-tick refresh may select
one. A retained target is not rechecked for the `Enemy` marker or water/rain. Initial selection uses
the inflated AABB rather than the later strict block-position sphere, so a corner candidate outside
the strict distance may be selected and attacked once, then cleared at the next refresh.

If the returned reference resolves to a living entity, the server first emits conduit attack sound
at that entity's current coordinates, then calls `hurtServer` with magic damage `4.0`; the Boolean
damage result is ignored. Rejected or fully absorbed damage therefore does not retract the sound or
target. Generic magic immunity, armor/effect reduction, cooldown, lethal transition and knockback
semantics remain owned by the entity damage leaves.

Reference equality is UUID equality. Whenever the returned reference differs from the stored one,
the entity replaces it and calls `sendBlockUpdated(position,state,state,2)`, which projects the
new optional `Target` tag. It does not call `setChanged`. Target acquisition/clearing therefore
updates visible clients without independently marking the chunk dirty.

Complete loss of activation is a special residue: the inactive branch skips target work entirely.
It sets hunting false but retains and continues saving/synchronizing the old target reference. An
active partial frame clears that reference on its next refresh because target work runs with
hunting false. A later full-frame reactivation may instead reuse a still-alive retained reference
if it remains strictly within eight blocks, regardless of its current wetness.

**Server sounds:**

Activation and deactivation sounds occur only at a 40-tick refresh where active changes. While the
stored active flag is true, the ambient sound plays whenever game time is divisible by 80. The
short ambient sound plays on the first active tick whose game time is greater than the stored
deadline; before emitting it, the conduit sets the next deadline to current game time plus
`60 + nextInt(40)`. The strict greater-than comparison makes subsequent intervals 61 through 100
ticks. These ambient checks run every block-entity tick, not only on frame refreshes, and a refresh
that deactivates suppresses both active ambient branches in that same tick.

The initial deadline is zero. Activation at positive game time can therefore emit the short sound
in the activation tick; activation at game time zero first becomes eligible at time one. The
80-tick ambient sound and short sound are independent and may both emit on one tick.

**Client particles and rendering:**

Every client tick calls the animation path, even while inactive. Each remembered valid frame
position independently passes a `nextInt(50) == 0` gate and, on success, consumes three more floats
to emit one nautilus particle from the conduit toward an offset based on that frame block. Because
the list refreshes only every 40 client game ticks, it may transiently describe an earlier frame.

If the synchronized target UUID resolves to an entity, every client tick consumes three floats and
emits one additional nautilus particle from the target's eye position toward a randomized offset.
This branch does not test active or hunting. It therefore continues for a retained target while the
client renders an inactive conduit, until a later target-clearing update, entity loss or unload.

While inactive, the block-entity renderer draws only the base shell. While active it draws the
bobbing/rotating cage, two wind layers and a camera-facing eye; hunting selects the open-eye texture
and nonhunting selects the closed eye. The wind orientation/texture phase is
`floor(tickCount / 66) mod 3`. Active rotation advances by one on each active client tick and is
projected with partial-tick interpolation; it stops advancing while inactive.

The bundled block model supplies only the particle texture, leaving visible world geometry to the
block-entity renderer. The item JSON instead selects a special conduit model with a half-block
translation; `ConduitSpecialRenderer` always draws the base shell and has no active, eye, wind or
target state.

**Persistence and synchronization:**

Full save and the ordinary block-entity update tag write `Target` only when a reference exists.
Missing or malformed `Target` loads as null. A valid UUID loads as an unresolved reference and is
resolved through the level's living-entity UUID lookup when consumed. The update packet is the
ordinary block-entity-data packet for protocol type 25.

Load does not reconstruct active, hunting, effect blocks, counters, rotation or ambient deadline;
all remain their constructor defaults until client/server ticking derives or advances them. It
does not dirty the block or force an immediate shape scan. A saved target may therefore project
before the first post-load frame refresh, and target particles may begin as soon as that UUID can
resolve even while active remains false.

Because target changes do not mark dirty, durability of a newly acquired or cleared UUID depends
on some admitted chunk save path already considering the block entity/chunk dirty. The semantic
contract is nevertheless exact when a full save is taken: the then-current optional UUID survives,
while every other conduit runtime field resets as described.

**Branches and aborts:**

Both waterlogged states; every failing cell in the ordered 27-water scan; all 0..42 valid frame
counts and four valid block types; every radius tier and strict distance edge; wet/dry players;
client/server refresh phases; active changes and retained phases; hunting false/true; empty,
unresolved, removed, dead, near/far, wet/dry and nonenemy retained targets; empty/one/many selection
sets including AABB corners; accepted/rejected damage; equal/different UUID references; ambient
deadline boundaries; particle RNG outcomes; full save/update tags with missing/malformed/valid
Target; reload before and after the first 40-tick refresh.

**Constants and randomness:**

State IDs `15276..15277`; block-entity protocol ID `25`; refresh period `40`; ambient period `80`;
short-sound deadline increment `60 + nextInt(40)`; 27 required water cells; 42 frame candidates;
active threshold `16`; hunting threshold `42`; effect radius `16 × floor(n/7)`; effect duration
`260`; target inflation/retention distance `8`; magic damage `4.0`; frame-particle probability
`1/50` per remembered block per client tick; wind phase length `66`; active rotation scale
`-0.0375`; light `15`; shape `5..11`; strength `3`.

Server RNG is consumed by each nonempty new-target selection and each short-ambient deadline.
Client RNG is independent: every remembered frame position consumes one bounded draw per client
tick and each success consumes three floats; every resolved target consumes three floats per client
tick. Frame/water scans, player effects, target retention, damage and rendering consume no server
RNG beyond the stated selection/deadline draws.

**Side effects:**

Fluid tick scheduling; active/hunting/frame/target runtime mutation; activate, deactivate, ambient,
short-ambient and attack sounds; Conduit Power refresh; magic damage; block-entity-data projection;
nautilus particles; active/inactive block-entity rendering and special item rendering. The conduit
has no menu, direct interaction transaction, redstone output, comparator output, game event,
inventory mutation or self-created scheduled block tick.

**Gates:**

Matching block-entity type for ticking; the 27-cell water volume; frame count 16 for activation and
42 for hunting; player strict-radius plus water/rain; target selection type plus water/rain;
retained target alive plus strict block-position distance; ordinary damage defenses. Difficulty
and game rules do not gate conduit activation, effect refresh, target selection or the attack call.

**Boundary cases and quirks:**

The registered default is waterlogged even though ordinary dry placement overrides it. Activation
lags structural changes until the shared 40-tick phase. Valid frame blocks need not be contiguous.
Selection is box-based while retention is strict spherical block-position distance. Retention does
not recheck wetness. An invalid old reference creates one empty refresh before reselection. Total
deactivation retains the target, whereas an active partial frame clears it. Target projection does
not dirty. Client target particles ignore active/hunting and can outlive the active render. The two
ambient sounds can coincide.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`. Anchors:
`net.minecraft.world.level.block.ConduitBlock#getTicker`,
`net.minecraft.world.level.block.ConduitBlock#getFluidState`,
`net.minecraft.world.level.block.ConduitBlock#updateShape`,
`net.minecraft.world.level.block.ConduitBlock#getShape`,
`net.minecraft.world.level.block.ConduitBlock#getStateForPlacement`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity#clientTick`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity#serverTick`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity#updateShape`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity#applyEffects`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity#updateAndAttackTarget`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity#updateDestroyTarget`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity#selectNewTarget`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity#animationTick`,
`net.minecraft.world.entity.EntityReference#read`,
`net.minecraft.world.entity.EntityReference#store`,
`net.minecraft.world.entity.EntityReference#getLivingEntity`,
`net.minecraft.client.renderer.blockentity.ConduitRenderer#extractRenderState`,
`net.minecraft.client.renderer.blockentity.ConduitRenderer#submit`,
`net.minecraft.client.renderer.special.ConduitSpecialRenderer#submit`; locked block, block-entity and item
reports; bundled blockstate/model/item/loot/tag assets; `EXP-BLK-023`.

**Test vectors:**

Exhaust water/state placement and each ordered water abort; enumerate all 42 frame positions and
0..42 valid counts; cross every radius tier with wetness and strict distance. Drive active/hunting
refreshes, sound boundaries, full target selection/retention/clear/damage branches and exact RNG
cursors, including corner selection, one-refresh reselection delay and total-deactivation residue.
Save/load/update every target form around dirty and first-refresh boundaries, then compare client
frame/target particles, shell/cage/wind/eye phases and special item rendering. Run `EXP-BLK-023` as
the executable matrix.
