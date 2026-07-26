# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SPONGE-001` — Sponge absorption joins evaporation, monuments and furnace drying

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-005`,
`PLY-006`, `PLY-BREAK-001`, `BLK-003`, `BLK-005`, `BLK-UPDATE-001`, `ITM-003`,
`ITM-004`, `ITM-006`, `ITM-FURNACE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ENT-001`, `ENT-005`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-KNOCKBACK-001`, `MOB-AI-001`, `ENV-001`,
`ENV-002`, `ENV-003`, `ENV-FLUID-001`, `WGEN-003`,
`WGEN-STRUCTURE-OCEAN-MONUMENT-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and reports, exact dry/wet class control flow, the shared
breadth-first kernel and bucket-pickup implementations, exhaustive recipe, advancement, loot, tag,
class-reference and client-resource searches, the complete procedural Ocean Monument owner, and
all 1,212 decoded structure templates exhaust both property-free identities.

**Applies when:**

`minecraft:sponge` or `minecraft:wet_sponge` is placed, receives a neighbor change, absorbs water,
dries under a positional environment attribute, animates, is mined, exploded, smelted, looted from
an Elder Guardian, written by an Ocean Monument, equipped on a Sulfur Cube, persisted, mapped,
sounded or rendered.

**Authoritative state:**

Both are full, property-free blocks without block entities:

| Identity | Runtime class | State | Block protocol ID | Item raw ID |
|---|---|---:|---:|---:|
| Sponge | `SpongeBlock` | `560` | `99` | `220` |
| Wet Sponge | `WetSpongeBlock` | `561` | `100` | `221` |

Both registrations fix `COLOR_YELLOW`, default `HARP`, hardness/resistance `0.6/0.6` and their
named sound type. Every state is a full unit selection/collision/visual/occlusion cube with
emission `0`, light dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors
`1`, restitution `0`, solid redstone conduction, normal piston reaction and full sturdy faces.
Neither adds a block entity, scheduled tick, server random tick, use, attack, contact, signal,
comparator or fluid-state property.

Both directly belong to `mineable/hoe`; neither requires a correct tool or belongs to a
minimum-tier tag. A Hoe therefore supplies the tag-selected mining acceleration, but hand and
wrong-tool removals still admit block loot.

Sound volume/pitch is `1/1`. Break/step/place/hit/fall IDs are:

| Identity | Break | Step | Place | Hit | Fall |
|---|---:|---:|---:|---:|---:|
| Sponge | `1584` | `1588` | `1587` | `1586` | `1585` |
| Wet Sponge | `1766` | `1771` | `1770` | `1769` | `1768` |

Their ordinary block items are common 64-stacks with matching translation/model keys. Both
directly select `sulfur_cube_archetype/fast_flat`.

**Transition and ordering:**

### Dry-Sponge absorption triggers

Dry Sponge invokes absorption on placement unless the old state has the same Sponge block. It also
invokes absorption at the start of every `neighborChanged`, then calls the ordinary superclass
hook. Explicit same-block state writes therefore skip the placement attempt, while any admitted
neighbor notification can retry.

The attempt runs synchronously and consumes no RNG. With no accepted candidate it performs no
identity write or absorb sound, although any adversarial `BucketPickup` callback has already run.
Success first completes every accepted water-removal side effect, then offers Wet Sponge at the
origin with flags `2`, ignoring the write result, and finally plays `block.sponge.absorb` ID
`1589` in `BLOCKS` at volume/pitch `1/1`. The sound is emitted after any nested callbacks caused
by the Wet-Sponge write.

### Exact breadth-first removal

Traversal starts at the Sponge position, maximum graph depth `6`, and accepted-node cap `65`.
The origin is unconditionally accepted and counts toward that cap, so at most 64 water cells are
removed. Accepted nodes enqueue neighbors in exact enum order
`DOWN,UP,NORTH,SOUTH,WEST,EAST`; the queue is FIFO and already-visited positions are skipped.
Only accepted nodes expand, so a skipped water cell, nonwater cell or emptied corridor blocks
access to cells beyond it. Success means the final accepted count is greater than one.

For every nonorigin candidate, read block and fluid state. A fluid outside the live `water` tag is
skipped. A water-tagged candidate then takes the first matching branch:

1. If the block implements `BucketPickup`, call `pickupBlock(null, level, position, state)`. A
   nonempty returned stack is discarded and the position is accepted. Ordinary source Water
   replaces itself with air using flags `11`; a waterlogged simple block clears `waterlogged` with
   flags `3`, can destroy/drop itself when the original state can no longer survive, and returns a
   Water Bucket. An empty result falls through.
2. A remaining `LiquidBlock`, including flowing Water that could not be bucket-picked, is offered
   air with flags `3`; the ignored result does not affect acceptance.
3. Kelp, Kelp Plant, Seagrass or Tall Seagrass first runs generic resource dropping with any block
   entity, then is offered air with flags `3`; the position is accepted.
4. Every other water-bearing state is skipped.

Consequently the cap is on accepted traversal nodes, not successful write Booleans or fluid
volume. BucketPickup and plant callbacks may add their own drops, survival destruction, neighbor
updates and schedules before the Sponge changes identity.

### Wet-Sponge environmental drying

Wet Sponge's placement hook reads positional `gameplay/water_evaporates`, whose default is false
and whose locked Nether dimension value is true. False leaves the placed state unchanged. True
performs these actions even though their write/results are not rolled back:

1. offer dry Sponge at the same position with flags `3`, ignoring the Boolean;
2. emit level event `2009` with data `0`, which clients project as eight zero-velocity cloud
   particles at randomized X/Z and Y plus `1.2`;
3. play `block.wet_sponge.dries` ID `1767` in `BLOCKS`, volume `1`, pitch
   `(1 + nextFloat()*0.2)*0.7`, hence `[0.7,0.84)`.

The dry replacement's placement hook runs before the outer Wet-Sponge call resumes. In an
evaporating environment it can therefore absorb any command/custom water still reachable around
the position. A successful nested absorption writes Wet Sponge, which immediately dries again;
the nested drying event/sound occurs before that absorption's final sound. Locked baseline Nether
water placement normally evaporates earlier, but this callback order is authoritative for
commands, custom data and adversarial write hooks.

### Wet-Sponge display particles

Each admitted client display tick first chooses one of six directions uniformly. `UP` returns
immediately. For another direction, a sturdy neighbor face toward the Sponge suppresses output.
Otherwise one zero-velocity Dripping-Water particle is emitted:

- `DOWN`: position X/Z each add an independent `nextDouble`, and Y is block Y minus `0.05`;
- `EAST/WEST`: Y adds `nextDouble()*0.8`, Z adds `nextDouble`, and X is block X plus
  `1.1/0.05`;
- `SOUTH/NORTH`: Y adds `nextDouble()*0.8`, X adds `nextDouble`, and Z is block Z plus
  `1.1/0.05`.

Thus every non-UP, nonoccluded branch consumes exactly two doubles after the six-way direction
draw. These particles are client-only and do not expose or change stored wetness.

### Harvest and block loot

Each identity has one one-roll self-item entry behind `survives_explosion`, using random sequence
`minecraft:blocks/<identity>`. Because neither registration requires a correct tool, tool
admission never suppresses the table. Silk Touch and Fortune do not change either result; there is
no alternate count, XP or block-specific break hook.

### Furnace drying and bucket conversion

The sole bundled recipe naming either identity is category-block Smelting: one Wet Sponge cooks
for the default `200` ticks into one default Sponge and records experience `0.15`. Its recipe
advancement has one OR requirement: prior knowledge of `minecraft:sponge` or possession of Wet
Sponge; it rewards only that recipe.

On a successful furnace-family completion, if the input is Wet Sponge and fuel slot `1` currently
contains a nonempty Bucket stack, the furnace replaces that entire slot with one Water Bucket
before shrinking the input. Ordinary slot admission permits a Bucket only when that slot does not
already hold one, but command/NBT state can make the whole-stack replacement observable. The
special case does not itself ignite an empty Bucket; a machine must already be burning or
otherwise have acquired burn time. Output admission, timer order, recipe-use counts, slot
insertion/extraction and delayed fractional XP rounding remain with `ITM-FURNACE-001`. Input
components are discarded by the default result.

### Elder Guardian acquisition

The Elder Guardian entity table's third independent one-roll pool is gated only by
`killed_by_player` and emits exactly one Wet Sponge. Looting, fire, smelts-loot and the other table
pools do not modify it. It follows the shard and Cod/crystals/empty pools and precedes the
rare-fish and Tide-template pools under random sequence `minecraft:entities/elder_guardian`.
Generic death admission, table evaluation and world drop placement retain their entity/loot
owners.

No other entity/chest/gameplay loot table, recipe, advancement reward, trade, barter, composting,
fuel or hard-coded mob path creates either item.

### Procedural Ocean-Monument rooms

Wet Sponge is the only scoped state written by Ocean Monument generation. Every simple-top room
placement invocation visits local X/Z `1..6` in X-major then Z order, consuming
`nextInt(3)` for all 36 columns. Zero offers no Sponge. A nonzero result consumes `nextInt(4)`:
zero writes Wet Sponge from Y `2..3`, while a nonzero result writes only Y `3`.

The simple-top fitter admits one-cell rooms with no west/east/north/south/up opening; the number
and positions depend on the generated room graph. Each intersecting processing-chunk invocation
reruns all 36 draws even for clipped cells, and generic structure writes clip the cuboids. Counts
therefore range from zero through 72 raw Wet-Sponge offers per invocation before clipping, without
a fixed Monument total. Exact graph fitting, invocation order, RNG ownership, geometry, clipping,
fluid scheduling and writes remain with `WGEN-STRUCTURE-OCEAN-MONUMENT-001`.

The exhaustive 1,212-template scan finds zero raw Sponge and zero raw Wet Sponge. Procedural
Monument writes are intentionally outside that NBT census.

### Equipment selection

Both items select fast-flat. Its record fixes horizontal/vertical knockback `0.9125/0.09`, push
cooldown `0.9`, impulse threshold `0.03`, additive knockback and explosion-knockback resistance
`-1/-1`, additive bounciness `0.5`, total-multiplied friction
`-0.7999999970197678` and air drag `-0.9900000002235174`, plus its hit/push sounds.
Matching and modifier lifecycle remain with the Sulfur-Cube owners.

**Client projection:**

Both sole blockstates have one unconditional `cube_all` model with the same-named static texture;
each item directly selects its block model. English names are exactly `Sponge` and `Wet Sponge`.
Natural Blocks publishes the pair once, after Dead Horn Coral Fan and before Melon, in dry-then-wet
order. Neither appears in another ordinary tab.

State updates use IDs `560/561`, inventory paths use item IDs `220/221`, maps use
`COLOR_YELLOW`, and sounds use the two profiles and transition events above. No identity adds a
packet field or connection-local state.

**Branches and aborts:**

Dry placement old-same/different and every neighbor retry; BFS depth/count/order, visited state and
four removal branches; zero/one/65 accepted nodes and write failures; wet placement under false/true
evaporation with nested absorption; six display directions and sturdy-face suppression; hand/Hoe,
explosion survival and self loot; Smelting/output/fuel-slot/bucket/XP/unlock paths; Elder Guardian
player-kill gate and surrounding pools; every Monument graph/simple-top/invocation/draw/clip/write
outcome; fast-flat selection, persistence and both client projections are distinct.

**Constants and randomness:**

States/block/item and sound IDs as tabulated; strength `0.6/0.6`; stack `64`; BFS
depth/accepted-cap/water-cap `6/65/64`; direction order `D/U/N/S/W/E`; transition sounds
`1589/1767`; drying event `2009`; drying pitch `[0.7,0.84)`; Smelting
`200 ticks/0.15 XP/1:1`; Elder output `1`; Monument `36` columns with first/second bounds `3/4`
and `0..72` offers per invocation; display direction bound `6`. Absorption and cooking consume no
RNG; drying, client display, Monument placement, loot neighbors and fractional XP retain their
specified streams.

**Side effects:**

Full-block placement/removal and self loot; bucket-pickup, fluid/plant removal, survival
destruction, drops, updates and schedules; dry/wet writes, absorb/dry sounds, event clouds and drip
particles; furnace result, bucket conversion, recipe knowledge/use count and delayed XP; Elder
loot; procedural Monument writes; fast-flat equipment modifiers; ordinary persistence, maps and
block/item projection.

**Gates:**

Placement/neighbor/write authority; water-tag membership, accepted connectivity, depth/count and
branch-specific pickup/removal; positional evaporation snapshot; display-tick direction and
neighbor face; explosion survival; active recipe/advancement/loot/archetype snapshots, furnace
capacity/burn/input/fuel state and player extraction; Elder death/player context; Monument graph,
piece, intersection, clip and write admission; valid registry/map/sound/client-resource context.

**Boundary cases and quirks:**

- The accepted count includes the dry Sponge origin, explaining the source call's cap `65` and
  success test greater than one for a maximum of 64 removed water cells.
- Traversal expands only through accepted cells; after a near cell becomes air it is not a
  traversable corridor for a later attempt.
- A failed removal write still counts as accepted in the LiquidBlock/plant branches, while a
  BucketPickup must return a nonempty stack.
- Absorbed source-water buckets are discarded; waterlogged blocks can instead lose support,
  destroy themselves and drop resources during pickup.
- Wet Sponge drying writes dry Sponge before its event and sound, so dry placement/absorption
  callbacks can nest inside the outer drying transaction.
- An empty Bucket in the furnace fuel slot is a conversion receptacle, not usable fuel.
- Monument sponge fields are procedural, rerolled per processing invocation and absent from every
  NBT template.
- Wetness is block identity, not a Boolean property or stored fluid amount.

**Failure semantics:**

Illegal state patches are rejected by the shared component/state owner. A traversal with no
accepted water performs no identity write or absorb sound. Branch-specific write Booleans are
ignored as stated; exceptions or rejected registry/template lookups fail at their generic owners.
False evaporation preserves Wet Sponge. Recipe mismatch, insufficient output, no burn time or
inactive snapshot prevents completion; nonplayer Elder death omits the Wet-Sponge pool. Monument
intersection/clip/write failure preserves live state. Client resource failure affects projection,
not authoritative identity.

**Client/server authority split:**

The server owns registry identity, absorption/removal, environmental drying, harvest, loot,
Smelting, Monument writes, archetype selection and persistence. Clients project event clouds,
random drip particles, models, names, tabs and playback/rendering of authoritative state and
sounds.

**Observability:**

Commands, fluid/block traces, drops, sounds, events/particles, inventory/furnace/recipe state,
entity loot, structure traces, equipment modifiers, packets, maps, tabs and rendering expose the
listed branches.

**Persistence and reload:**

Placed states persist only the block ID because there is no property or block entity; removed
water/count history is not stored. Item stacks persist ordinary components. Recipe, advancement,
loot, fluid tag, environment attribute and archetype snapshots are reloadable where their owners
specify. Sponge classes and Monument geometry remain hard-coded; existing states do not
retroactively change when snapshots reload.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`; `SpongeBlock#onPlace`, `#neighborChanged`,
`#tryAbsorbWater` and `#removeWaterBreadthFirstSearch`; `WetSpongeBlock#onPlace` and
`#animateTick`; `BlockPos#breadthFirstTraversal`; `SimpleWaterloggedBlock#pickupBlock`;
`LiquidBlock#pickupBlock`; `AbstractFurnaceBlockEntity#burn`;
`OceanMonumentPieces$OceanMonumentSimpleTopRoom#postProcess`; `CreativeModeTabs`; the two
block/item/component/loot/asset reports; the Smelting recipe/advancement, Elder Guardian table,
direct block/item tags, four dimension-type records, all 1,212 NBT templates and exact client
resources. Complete exact-ID data and class-reference searches found no other acquisition,
progression, trade, worldgen or runtime path.

**Test vectors:**

Run `EXP-BLK-098` across both states and IDs; every placement/neighbor/evaporation/nested callback;
BFS shapes at depth/count boundaries through source/flowing/waterlogged/plant/unsupported/failed
writes; all display directions and neighbor faces; hand/Hoe/explosion/loot; every
Smelting/burn/output/bucket/XP/unlock boundary; Elder player/nonplayer death; every Monument
graph/simple-top/invocation/draw/clip/write outcome; all 1,212 templates; fast-flat, persistence,
sounds, events, particles, maps, tabs and models. Assert the exact constants, absence boundaries
and client convergence.

**Limits:**

Generic placement, block notification, traversal storage, bucket/fluid/plant behavior, breaking,
loot, furnace ticking, entity death, Sulfur-Cube behavior, procedural Monument generation, packet
encoding, client level-event handling and rendering remain with their named owners. This leaf
fixes the exact two block identities, custom transitions, joins, absences and projection.
