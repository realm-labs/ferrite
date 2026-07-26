# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SNIFFER-EGG-001` — Sniffer Eggs crack through three scheduled stages and hatch baby Sniffers

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INTERACT-001`,
`PLY-BREAK-001`, `PLY-COLLISION-001`, `PLY-AUTOJUMP-001`,
`RED-001`, `RED-UPDATE-001`, `RED-COMPARATOR-001`, `ITM-003`,
`ITM-004`, `ITM-006`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-SPAWN-001`, `MOB-001`,
`MOB-BREED-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FLUID-001`, `ENV-FIRE-001`, `ENV-LIGHT-001`,
`WGEN-003`, `WGEN-PIPELINE-001`, `WGEN-STRUCTURE-OCEAN-RUIN-001`,
`BLK-BRUSHABLE-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked `SnifferEggBlock`, Sniffer breeding,
registration, loot, advancement, hatch-boost tag, Ocean-Ruin archaeology,
all `1,212` templates and exact client resources close the three-state
Sniffer Egg. Each placement/state transition schedules the next third of
its hatch, Moss halves only the newly scheduled interval, and final hatch
destroys without drops before independently creating a baby Sniffer.

**Applies when:**

`minecraft:sniffer_egg` is placed, state-written, scheduled, cracked,
hatched, mined, exploded, piston-moved, bred, brushed from Warm Ocean Ruin
Suspicious Sand, persisted, synchronized or rendered.

**Authoritative state:**

Sniffer Egg is a `SnifferEggBlock` with integer property `hatch=0..2`, no
block entity and state IDs `15102/15103/15104`; `0` is default. Block,
block-type and item IDs are `746/191/675`, and Sniffer entity type ID is
`119`. The item is an ordinary stack-64 uncommon `BlockItem`.

Registration fixes `COLOR_RED`, default `HARP`, strength/resistance
`0.5/0.5`, Metal sounds, no occlusion and default piston reaction
`NORMAL`. Friction is `0.6`, speed/jump factors are `1`, emission is `0`,
and no correct tool is required.

Selection, collision, support and visual geometry is the full-height
inset box `[1/16,0,2/16]..[15/16,1,14/16]`. No-occlusion prevents an
ordinary opaque occlusion cube. The state propagates skylight, has
dampening `0`, supplies no signal/comparator output and explicitly returns
false for every `PathComputationType`.

Metal profile volume/pitch is `1/1.5`; break/fall/hit/place/step IDs are
`975/976/977/978/981`. Dedicated plop/crack/hatch IDs are
`1564/1565/1566`.

**Transition and ordering:**

### Placement and interval scheduling

Sniffer Egg overrides no placement-state or survival predicate. Generic
placement writes default `hatch=0`; any replaceable target and admitted
collision/border/permission/component transaction can place it. It
survives without support and contains Empty fluid.

Every `onPlace(state,level,P,old,moving)` call:

1. tests whether `P.below()` belongs to block tag
   `sniffer_egg_hatch_boost`;
2. only on a non-client level and a true boost emits level event `3009`
   at `P` with data `0`;
3. selects total target time `12000` when boosted or `24000` otherwise,
   divides by three to obtain base interval `4000` or `8000`;
4. emits game event `BLOCK_PLACE` at `P` with state-only context; and
5. schedules this block after `base + nextInt(300)`, hence
   `4000..4299` or `8000..8299` ticks.

The locked boost tag contains exactly `minecraft:moss_block` and no
parent. It is sampled only when an interval is scheduled. Adding/removing
Moss does not alter an already queued delay, while the next successful
state transition samples the then-current support.

Accepted same-block property writes invoke `onPlace`; therefore each
accepted crack transition re-emits the placement event and schedules the
next third. With unchanged support and all writes admitted, placement to
final hatch takes `12000..12897` ticks on Moss or `24000..24897`
otherwise. Mixed support yields the sum of the three independently chosen
intervals. External state writes also invoke the callback and can add or
replace scheduled work according to the generic tick queue owner.

On the client, level event `3009` creates `EGG_CRACK` particles on block
faces with uniform per-face count `3..6`. The server sends it only for a
boosted `onPlace`; the event does not itself mutate hatch state.

### Crack ticks

At scheduled server tick, the passed state decides the branch. For
`hatch=0` or `1`:

1. play crack sound `1565`, category `BLOCKS`, volume `0.7`, pitch
   `0.9 + nextFloat()*0.2`;
2. offer the same state with `hatch+1` using flags `2`; and
3. ignore the Boolean write result.

An accepted write re-enters `onPlace` synchronously and schedules the next
interval using current support. Rejected write leaves the old stage and
does not obtain that re-arming callback; the already played sound and RNG
draw remain committed.

### Final hatch

For `hatch=2`, ordering is:

1. play hatch sound `1566` with the same category, volume and randomized
   pitch formula;
2. call `destroyBlock(P,false)` and ignore its result;
3. call `EntityTypes.SNIFFER.create(level,BREEDING)`;
4. if nonnull, mark it baby, snap it to block center
   `(P.x+0.5,P.y+0.5,P.z+0.5)` with pitch `0` and yaw
   `wrapDegrees(level.random.nextFloat()*360)`; and
5. offer it through `addFreshEntity`, ignoring the result.

Creation failure skips center/yaw work and insertion. Destruction failure
does not prevent creation, and insertion failure does not restore the Egg.
Unlike Frogspawn Tadpoles, the Snifflet is not explicitly marked
persistence-required by this block.

### Mining, explosion, piston and fire

Any tool can harvest. The one-roll self table emits one Sniffer Egg behind
`survives_explosion`, using random sequence
`minecraft:blocks/sniffer_egg`; Silk Touch and Fortune do not change it.
Explosion decay may suppress the item. All three hatch states share the
same item result.

Default `NORMAL` piston reaction permits movement rather than forced
destruction. Destination placement re-enters `onPlace`, samples support,
emits its game event and schedules an interval; the generic piston/tick
owners decide stale-source scheduling and state preservation.

There is no Fire bootstrap row, lava-ignition property or fuel time:
encouragement/flammability are `0/0`. There is no random tick, update,
fluid-state, transform, entity-contact, attack, fall, signal, comparator
or block-event override.

### Sniffer breeding and acquisition

`Sniffer.spawnChildFromBreeding` does not create a child mob. It constructs
one default Sniffer Egg stack and `ItemEntity` at the breeding Sniffer's
exact position, sets default pickup delay, then calls generic
`finalizeSpawnChildFromBreeding(level,mate,null)`. It next plays plop sound
`1564` at volume `1` and pitch
`(nextFloat()-nextFloat())*0.2+0.5`, and finally offers the item entity;
the insertion result is ignored. Generic love reset, breeder statistics,
criteria and experience retain `MOB-BREED-001`.

Warm Ocean Ruin archaeology is the nonrenewable data path. Its one-roll
table has total weight `15`; Sniffer Egg has weight `1`, hence conditional
probability `1/15` per generated/brushed Suspicious Sand loot container.
The code-built Warm processor changes surviving Sand cells to Suspicious
Sand, attaches this table plus a position-seeded loot seed, and globally
caps actual replacements at five per piece. Integrity, cap shuffle,
placement, brushing, item reveal and failure retain
`WGEN-STRUCTURE-OCEAN-RUIN-001` and `BLK-BRUSHABLE-001`.

The hidden `husbandry/obtain_sniffer_egg` advancement is an
`inventory_changed` criterion for this item, uses it as icon, sends
telemetry and parents `feed_snifflet`. Warm archaeology generation also
satisfies the `ocean_ruin_warm` loot-table criterion of `salvage_sherd`,
but that advancement independently requires a decorated-pot sherd in
inventory; the Egg itself is not in that item tag.

There is no recipe, merchant, Composter, fishing, ordinary chest or entity
death table for the item. Breeding, warm archaeology, self loot, Creative
and commands are the baseline sources.

### Tags, worldgen census and persistence

The Egg block/item has no direct locked block/item tag, so membership
closures are `0/0`. The one-entry `sniffer_egg_hatch_boost` tag selects
Moss support, not the Egg. Reload affects only future interval scheduling.

No feature/biome record directly writes Sniffer Egg. Exhaustive decoded
and string scans of all `1,212` structure templates find zero raw Egg
cells and zero palette/final-state/marker/block-entity/entity-NBT
occurrences. Warm archaeology produces the item indirectly through a
processor-created Brushable block. Complete legacy-fix search finds no
exact migration.

Chunk palettes and block packets persist the property state ID; queued
ticks persist through the scheduled-tick owner. Stacks retain identity,
count and generic patches. The block has no block entity or
Egg-specific component.

### Client projection

`hatch=0/1/2` select unrotated
`sniffer_egg_not_cracked/slightly_cracked/very_cracked` models. Each
inherits the same six-face cuboid `[1,0,2]..[15,16,14]`, matching
authoritative geometry; top/down faces declare their corresponding
cullface. The three stages select six distinct untinted 16×16 textures
each. Particle texture follows the north face.

The item uses a separate 16×16 `item/sniffer_egg` texture in an ordinary
generated flat model, with no predicate, component branch or tint.
English name is `Sniffer Egg`. Natural Blocks publishes it once after
Turtle Egg and before Dried Ghast; no other baseline tab contains it.

**Branches and aborts:**

- Each `onPlace` independently samples the live Moss tag before event,
  game event, RNG and scheduling.
- Crack sound precedes the ignored state write; only an accepted write
  re-arms through `onPlace`.
- Final sound/destruction precedes construction; destruction, creation
  and insertion failures are independent.
- Self loot is tool-independent but explosion-conditioned.
- Breeding finalizes with null child, plays plop before ignored item
  insertion, and produces no direct Snifflet.
- Warm archaeology is `1/15` only after its structure/processor/brush
  gates admit a loot container.

**Constants and randomness:**

States `15102..15104`; block/block-type/item/Sniffer IDs
`746/191/675/119`; strength `0.5`; geometry `14×16×12` pixels; Metal IDs
`975/976/977/978/981`; plop/crack/hatch `1564/1565/1566`; intervals
`4000|8000 + nextInt(300)` three times; crack/hatch volume `0.7`, pitch
`0.9+0.2r`; Snifflet yaw `wrapDegrees(360r)`; plop pitch
`0.5+0.2(r1-r2)`; boost closure `1`; memberships `0/0`; archaeology
weight `1/15`; templates/cells `0/0`; models/textures `3/19`.

**Side effects:**

State placement and scheduled ticks; level/game events and particles;
crack/hatch/plop sounds; no-drop final destruction; baby-Sniffer and
breeding-item insertion; mining/explosion/piston movement and loot;
advancement/telemetry; archaeology reveal; persistence and projection.

**Gates:**

Generic placement/write; live Moss tag and logical side; tick stage and
write admission; destroy/create/insert results; tool/explosion/piston;
breeding transaction; Ocean-Ruin integrity/cap/write/brush/loot draw;
inventory criterion; registry/reload/resource validity.

**Boundary cases and quirks:**

Moss changes only intervals scheduled after it is observed. A rejected
final destruction can leave an Egg and still spawn a Snifflet. A rejected
crack write emits sound but does not naturally re-arm. Piston movement
replays placement scheduling. Breeding makes an item Egg rather than a
child. Render and collision share a tall inset cuboid, unlike the thin
water-surface eggs nearby in creative ordering.

**Failure semantics:**

Generic placement, piston, tick queue, loot, breeding, archaeology and
advancement owners retain commit semantics. All Egg tick writes/
destruction and entity/item insertions ignore results. Earlier sound,
event, memory/stat and RNG effects do not roll back. Structure and brush
transactions can partially commit.

**Client/server authority split:**

The server owns state, scheduling, cracking/hatching, breeding,
archaeology, loot, advancement and persistence. The client predicts
placement, handles event-3009 particles and renders stage/model/item/name.
Authoritative blocks, entities, stacks, sounds and events synchronize.

**Observability:**

Observe IDs/property, shapes/path/light/redstone/piston, every `onPlace`
support/event/RNG/schedule, crack/final order and results, Snifflet fields,
breeding item/finalizer/plop order, archaeology weight and cap, complete
closures/census, durable queued state and exact client stage projection.

**Persistence and reload:**

The three states and queued ticks persist through their owners; no block
entity exists. Stack components are generic. Tag, loot, advancement,
worldgen and client resources reload independently. Registration, block
control flow, Sniffer breeding and tab ordering are code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SnifferEggBlock`;
`net.minecraft.world.entity.animal.sniffer.Sniffer#spawnChildFromBreeding`;
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces`;
`net.minecraft.client.renderer.LevelEventHandler`;
`net.minecraft.world.item.CreativeModeTabs`; reports; self/archaeology
loot; boost tag; both advancements; Warm Ocean Ruin records; all `1,212`
templates; blockstate, three block models, item model, 19 textures and
language resources. Complete compiled/data/fix/NBT searches find no other
exact runtime path.

**Test vectors:**

Run `EXP-BLK-123` across every state/ID/shape/path/light/redstone/piston/
tool/explosion branch; placement and external writes above Moss/control
with all three interval endpoints and support changes; every accepted/
rejected crack, destroy, creation and insertion result; breeding order and
plop draws; Warm Ocean Ruin processor/brush/`1/15` loot; advancement,
closures, template/fix absences, persistence/reload and all models.

**Limits:**

Generic placement/mining/piston/tick queues, Sniffer lifecycle/breeding,
Brushable blocks, Ocean Ruins, advancements, packets and rendering retain
their owners. Moss and Sniffers retain their catalog families. This leaf
fixes exact Sniffer Egg and every direct join selecting it.
