# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CARVED-PUMPKIN-001` — Carved Pumpkin is an orientable golem head and visibility-disguising helmet

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`, `PLY-002`, `PLY-005`,
`PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`, `ITM-ANVIL-001`,
`ITM-DISPENSER-001`, `ENT-001`, `ENT-LIFECYCLE-001`, `MOB-AI-001`,
`MOB-SPAWN-001`, `ENV-001`, `ENV-002`, `ENV-003`, `ENV-FIRE-001`,
`ENV-LIGHT-001`, `WGEN-003`, `WGEN-JIGSAW-OUTPOST-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, complete
`CarvedPumpkinBlock` implementation, golem patterns and spawn transactions,
Pumpkin carving, dispenser behavior, equipment consumers, mob AI, recipes,
loot, advancements, tags and client assets fix all four states and every
identity-specific branch. Exhaustive decoded-NBT and constant-pool scans of
all 1,212 structure templates find exactly three raw cells in two templates
and no hidden item reference.

**Applies when:**

`minecraft:carved_pumpkin` is placed, rotated, mirrored, mined, exploded,
carved from Pumpkin, worn, dispensed, selected by Enderman or Sulfur Cube AI,
generated in its two templates, used to assemble a Snow, Iron or Copper Golem,
sheared from a Snow Golem, selected as Halloween mob equipment, composted,
crafted into Jack o'Lantern, used as an advancement unlock/display, persisted,
synchronized or rendered.

**Authoritative state:**

The block is raw block ID `296`; its four canonical states are:

| Facing | State ID |
| --- | ---: |
| north (default) | `7019` |
| south | `7020` |
| west | `7021` |
| east | `7022` |

It has no block entity. Registration constructs `CarvedPumpkinBlock` with
orange map color, HARP note instrument, destroy speed/resistance `1/1`, Wood
sound, full default collision/outline/occlusion, friction `0.6`, speed and jump
factors `1`, light `0`, an always-true entity-spawn predicate and piston
reaction `DESTROY`. Wood break/fall/hit/place/step sound-event protocol IDs are
`1853/1854/1855/1856/1857`, all at volume/pitch `1/1`.

The direct block tags are `enderman_holdable`, `mineable/axe` and
`sword_efficient`; there is no `needs_correct_tool` membership. Its loot is
therefore not tool-gated. The block has no random tick, scheduled-tick,
redstone, comparator, fluid, entity-contact or custom survival hook. It has no
`FireBlock` encouragement/flammability row, lava-ignition property or vanilla
fuel entry.

The item is raw item ID `385`, a common stack-64 `BlockItem`. Its direct tags
are `enchantable/equippable`, `enchantable/vanishing`,
`gaze_disguise_equipment`, `map_invisibility_equipment` and
`sulfur_cube_archetype/fast_flat`.

**Transition and ordering:**

### Ordinary placement and transforms

Item placement chooses the opposite of the placement context's horizontal
direction. The default state is north. Horizontal rotation and mirror use the
ordinary `HorizontalDirectionalBlock` transforms; facing is the only durable
property.

`onPlace` returns immediately when the old state has the same block identity,
so a facing-only update does not retry assembly. Otherwise it calls
`trySpawnGolem`. Replacing Jack o'Lantern with Carved Pumpkin is a different
identity and does retry.

The shared head predicate accepts an exact Carved Pumpkin or Jack o'Lantern at
any facing. Jack o'Lantern is another `CarvedPumpkinBlock` instance with the
same physical registration except light `15`; it can head every pattern, but
retains its own block/item identity and catalog ownership.

### Golem admission

`canSpawnGolem` checks cached one-aisle base patterns in Snow, Iron, Copper
order. The base head cell is deliberately unconstrained:

```text
Snow base       Iron base       Copper base
                    ~ ~
    #               ###              #
    #               ~#~
```

`#` means exact Snow Block for Snow Golem, exact Iron Block for Iron Golem and
membership in block tag `minecraft:copper` for Copper Golem. `~` requires air;
literal spaces accept any state. `BlockPattern.find` owns orientation and the
search volume. Admission succeeds at the first base match.

`trySpawnGolem` searches the full patterns in the same order:

```text
Snow full       Iron full       Copper full
    ^               ~^~              ^
    #               ###              #
    #               ~#~
```

`^` is the shared Carved-Pumpkin-or-Jack-o'Lantern predicate. If the matching
entity type's `create(level, TRIGGERED)` returns null, Snow falls through to
Iron, Iron falls through to Copper, and Copper returns; no pattern cells have
yet changed.

Snow Golem uses matched cell `(0,2,0)` as its spawn cell. Iron Golem uses
`(1,2,0)` and sets `playerCreated=true` before the shared transaction. Copper
Golem uses `(0,0,0)`.

### Shared golem transaction

For every cell of the matched one-aisle full pattern, the server performs these
steps in source order:

1. Cache the original state, call `setBlock(position, Air, 2)` and ignore its
   Boolean result.
2. Emit level event `2001` at that position with the cached original state ID.
3. After all cells, snap the entity to spawn-cell center
   `(x+0.5, y+0.05, z+0.5)` with yaw/pitch `0`.
4. Call `addFreshEntity` and ignore its Boolean result.
5. For every `ServerPlayer` in the entity bounding box inflated by `5`, trigger
   `SUMMONED_ENTITY`.
6. For every matched cell, call `updateNeighborsAt(position, Air)`.

Consequently a failed cell write does not suppress its break event, a failed
entity insertion does not restore the structure or suppress nearby-player
criteria, and neighbor notification happens only after all removal attempts,
events, insertion and criteria.

Copper Golem then performs a second transaction. It reads the matched head's
cached facing and converts the cached copper support at `(0,1,0)` through
`CopperChestBlock.getFromCopperBlock`, writing that chest with flags `2` and
ignoring failure. It derives weather state from the cached original support:
direct `WeatheringCopper` age; otherwise the unwaxed source in
`HoneycombItem.WAX_OFF_BY_BLOCK` when that source is weathering copper;
otherwise unaffected Copper Block age. It finally calls
`CopperGolem.spawn(weatherState)`. Entity insertion and summon criteria
therefore precede both the chest write and weather-state initialization.

### Pumpkin carving

Using exact Shears on an uncarved Pumpkin owns the principal survival producer.
Other items defer to the generic block path. On the client, the Shears branch
returns `SUCCESS` immediately without mutation. On the server:

1. A vertical hit chooses the player's horizontal direction opposite; a
   horizontal hit uses the clicked face.
2. Built-in block-interact loot table `minecraft:carve/pumpkin` evaluates with
   the Pumpkin state, Shears instance and player. It has one unconditional
   result of exactly four Pumpkin Seeds and random sequence
   `minecraft:carve/pumpkin`.
3. Each produced stack becomes an `ItemEntity` at
   `(x+0.5+0.65*dx, y+0.1, z+0.5+0.65*dz)` with velocity
   `(.05*dx+nextDouble*.02, .05, .05*dz+nextDouble*.02)`. Entity insertion
   results are ignored.
4. Play Pumpkin Carve at volume/pitch `1/1`.
5. Write Carved Pumpkin default state with the derived facing and flags `11`;
   ignore the write result.
6. Damage Shears by one in the used-hand equipment slot, emit player game
   event `SHEAR`, award the Shears `ITEM_USED` statistic and return `SUCCESS`.

Loot evaluation or block-write failure does not abort the later sound, tool
damage, event or statistic.

### Dispenser

Only exact Carved Pumpkin has the dedicated optional dispenser behavior; Jack
o'Lantern does not. The target is the block immediately in front of the
dispenser. When that position is empty and the Carved Pumpkin block instance's
base-pattern admission succeeds, the server writes default north state there
with flags `3`, ignores the result and emits `BLOCK_PLACE` with null source.
The behavior then shrinks the stack by one and marks success. The facing is not
derived from dispenser orientation; a successful write's `onPlace` runs the
full assembly transaction.

Otherwise `EquipmentDispenseItemBehavior.dispenseEquipment` tries the ordinary
equipment path and its Boolean result becomes optional-behavior success.
Generic optional behavior owns the resulting success/failure sound and
animation.

**Acquisition, consumption and progression:**

The block loot table has one roll yielding one exact Carved Pumpkin, guarded
only by `survives_explosion`, with random sequence
`minecraft:blocks/carved_pumpkin`. Silk Touch and Fortune add no branch.
Composter bootstrap installs exact chance `0.65`.

Snow Golem shearing is another producer. Interaction returns `SUCCESS` when
the held item is exact Shears and the golem is ready (`hasPumpkin=true`), on
both sides; otherwise it returns `PASS`. The server plays Snow Golem Shear at
volume/pitch `1/1`, clears the pumpkin bit before evaluating built-in
`shearing/snow_golem`, whose sole unconditional result is one Carved Pumpkin,
and spawns output at eye height. It emits player `SHEAR` and damages Shears by
one. A loot failure leaves the pumpkin removed.

The shaped Jack o'Lantern recipe places Carved Pumpkin above Torch and emits
one Jack o'Lantern. Its recipe advancement is an OR between already knowing
the recipe and possessing exact Carved Pumpkin. Pumpkin Pie instead requires
exact uncarved Pumpkin, Sugar and the egg tag: its advancement nevertheless
also accepts Carved Pumpkin as an unlock alternative. Carved Pumpkin can
therefore unlock Pumpkin Pie without satisfying that recipe.

`summon_iron_golem` uses Carved Pumpkin only as its display icon; its criterion
requires a summoned exact Iron Golem and enables telemetry. Natural Blocks
creative order is Melon, Pumpkin, Carved Pumpkin, Jack o'Lantern, Hay Block.

**Equipment and entity consumers:**

The item equips to `HEAD`, has `swappable=false`, and projects camera overlay
`minecraft:misc/pumpkinblur`. It also carries hidden head-only attribute
modifier `minecraft:waypoint_transmit_range_hide`: amount `-1`,
`add_multiplied_total`, on `waypoint_transmit_range`. Absent interaction with
other modifiers, the -100% total multiplier collapses an ordinary wearer's
transmit range to zero. A nonspectator receiver therefore ignores that source
even at zero distance (`distance >= range`); spectator receivers bypass this
ignore test.

For map tracking, after maintaining a carrier's player decoration,
`MapItemSavedData.tickCarriedBy` removes another carrier's named decoration
when any armor-slot stack belongs to `map_invisibility_equipment`. The wearer
is hidden from maps carried by other players, not from the wearer's own
current-map entry.

The `gaze_disguise_equipment` head tag makes
`PLAYER_NOT_WEARING_DISGUISE_ITEM` false for a Player. Enderman stare
evaluation returns false at that first predicate, before its look-vector and
line-of-sight checks. This prevents the gaze-provocation path, not unrelated
Enderman hostility.

Client HUD extraction requires visible HUD, first person and a player who is
not scoping. It scans all equipment slots and emits every matching
`Equippable.cameraOverlay`; a Carved Pumpkin in its declared head slot selects
`textures/misc/pumpkinblur.png` at alpha `1`. The texture metadata enables
blur. Third person, scoping and hidden HUD omit the overlay.

An adult Sulfur Cube with an empty body slot may absorb one Carved Pumpkin
because the item is in the swallowable `fast_flat` archetype. Ground pickup
also requires a live `ItemEntity`, no pickup delay and pickup timer at most
zero; it splits exactly one into BODY, marks guaranteed drop and plays the
absorb sound. Direct equipment rejects babies and an identical existing item,
may drop the old server-side BODY stack, then installs a count-one copy.

`fast_flat` supplies knockback-resistance and explosion-knockback-resistance
additions `-1`, bounciness addition `0.5`, friction total multiplier
`-0.7999999970197678`, air-drag total multiplier
`-0.9900000002235174`, horizontal/vertical knockback `0.9125/0.09`, hit/push
sounds, cooldown `0.9` and impulse threshold `0.03`.

On Halloween, Abstract Skeleton and Zombie finalization may equip the item.
The gate requires an empty head slot, exact local-server-date October 31 and
`nextFloat < 0.25`; a separate `nextFloat < 0.1` chooses Jack o'Lantern,
otherwise Carved Pumpkin. Thus Carved Pumpkin is `90%` conditional on admission
and `22.5%` overall when the date/head gates hold. The head drop chance is
zero. Zombie conversion skips ordinary equipment population but still reaches
this later Halloween gate.

Enderman take-block AI requires no carried block, `MOB_GRIEFING`, its
one-in-reduced-20 admission and a clear outline/no-fluid ray to the sampled
tagged block. It calls `removeBlock(position, false)` without checking success,
emits `BLOCK_DESTROY` for the original state and stores that block's default
state; every carried Carved Pumpkin therefore becomes north.

Leave-block AI requires a carried state, `MOB_GRIEFING`, its
one-in-reduced-2000 admission, air target, a nonair/non-Bedrock/full-collision
support, survival and an empty unit entity box. It updates the carried state
from neighbor shapes, calls flags-3 `setBlock` without checking success, emits
`BLOCK_PLACE` and clears the carried stack. Carved Pumpkin's inherited survival
test imposes no extra condition; a completed placement can trigger golem
assembly.

**World and client projection:**

The exhaustive 1,212-template decoded-NBT census finds exactly three raw
Carved Pumpkin cells:

- `pillager_outpost/feature_targets.nbt` has two west-facing cells at
  `[1,2,1]` and `[1,2,5]` in size `3x3x7`.
- `woodland_mansion/1x1_a2.nbt` has one east-facing cell at `[0,4,3]` in size
  `7x8x7`.

Both templates have DataVersion `4903`. The parallel constant-pool scan finds
only those two files and no UTF-only item/storage reference.

Outpost `features` is a rigid `legacy_single` element with empty processors
and weight one. Six other physical feature elements also weigh one and the
empty element weighs six, so `feature_targets` is `1/13` per draw from that
pool. Its five floor jigsaws target the empty pool. The existing
`WGEN-JIGSAW-OUTPOST-001` owner retains start height, terrain projection,
expansion and generic placement semantics.

First-floor ordinary Mansion 1x1 selection is uniform among
`1x1_a1..1x1_a5`, so `1x1_a2` is `1/5` whenever that eligible room slot is
instantiated. The procedural layout owns how many such slots exist, so this is
not an overall mansion occurrence probability. Mansion placement ignores
entities and only ignores Structure Blocks; placement rotation/mirror
transforms the stored east facing.

Blockstate variants select `block/carved_pumpkin` with no rotation for north
and Y rotations `90/180/270` for east/south/west. The model inherits
`block/orientable`, using `carved_pumpkin` front and shared `pumpkin_side` and
`pumpkin_top`. The item definition directly selects that block model. There is
no tint, random model choice, animation or special renderer.

**Branches and aborts:**

- Same-identity `onPlace` returns before all golem checks.
- Golem base admission does not require a head; the full transaction does.
- Entity creation null falls through without structure mutation.
- Pumpkin interaction without exact Shears remains generic.
- Dispenser assembly failure falls through to equipment dispensing.
- Snow Golem interaction without Shears or without its pumpkin returns `PASS`.
- Sulfur Cube pickup/equip rejects babies, occupied-body pickup and identical
  direct replacement.
- Enderman take/place gates are independent of the block's own placement hook.

**Constants and randomness:**

State IDs `7019..7022`; block/item IDs `296/385`; strength `1/1`; stack `64`;
compost `0.65`; golem snap Y `+0.05`; summon audience inflation `5`; clear
flags `2`; dispenser placement flags `3`; carving flags `11`; four seed output;
seed X/Z offset `0.65`; seed velocity base `0.05` plus
`nextDouble*0.02`; Halloween admission/Jack draws `0.25/0.1`; outpost element
weight `1/13`; eligible first-floor room choice `1/5`.

**Side effects:**

Golem assembly mutates blocks, emits break events, inserts an entity, triggers
criteria and notifies neighbors; Copper additionally writes a chest and
initializes weather. Carving emits loot entities and sound, writes the block,
damages Shears, emits a game event and awards a statistic. Dispenser, Snow
Golem, equipment, Enderman, composting, crafting and advancement paths retain
the ordered effects described above.

**Gates:**

Server authority; same-block identity; block-pattern match; entity-type
creation; dispenser target air; Shears identity; Snow Golem pumpkin bit;
equipment slot/tag; HUD/first-person/not-scoping; spectator receiver;
Sulfur-Cube age/body/item gates; Halloween date/head/RNG; Enderman
`MOB_GRIEFING`, RNG, ray, support/collision and survival; template/pool/room
selection.

**Boundary cases and quirks:**

Jack o'Lantern is accepted as a golem head but lacks Carved Pumpkin's
dispenser registration. Base pattern admission deliberately leaves the head
unconstrained. Golem removals and insertion are not atomic and their Boolean
results are ignored. Summon criteria can fire even when insertion fails.
Copper chest conversion happens after the shared neighbor-update pass while
using cached pre-clear copper/head states. Pumpkin carving commits tool/event/
stat effects even when loot or replacement fails. Enderman take and leave
clear their logical carried state independently of world-write success.

**Failure semantics:**

Ignored golem clear, entity-add, chest-write, carving-write, loot-entity-add,
dispenser-write and Enderman write results do not roll back earlier work or
abort the documented later effects. Explicit admission failures are
side-effect-free except where the owning generic dispenser behavior publishes
its failure event. Reloaded tag changes affect tag-selected equipment, AI,
tool and archetype joins without changing the registered four-state schema.

**Client/server authority split:**

The server owns block writes, golem assembly, carving loot/tool/stat effects,
mob equipment and AI, composting, crafting, advancements and structure
placement. The carving Shears branch predicts only `SUCCESS`. Client state
projects the synchronized facing and equipped stack; the HUD independently
selects the pumpkin-blur overlay from the live first-person/scoping/UI gates.

**Observability:**

Observe canonical state and facing, placement flags/results, pattern match
orientation, every removed/cached cell and event, entity-add result, summon
audience, neighbor notifications, Copper chest/weather state, loot and RNG
cursor, tool damage/stat/game event, equipment modifiers/map/Enderman/HUD
results, Sulfur Cube archetype state, Halloween equipment, Enderman carried
normalization, template transforms and exact block/item models.

**Persistence and reload:**

Only facing persists for the placed block; no block entity exists. Item
components and equipment state persist through generic stack/entity owners.
Golem pattern predicates, dispenser registration, compost chance, Halloween
logic and registration properties are code-built. Block/item/archetype tags,
loot, recipes, advancements, template pools and client resources are
reload-selected; the three raw template cells remain locked payload.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.CarvedPumpkinBlock`;
`net.minecraft.world.level.block.PumpkinBlock#useItemOn`;
`net.minecraft.world.level.block.state.pattern.BlockPattern`;
`net.minecraft.world.level.block.state.pattern.BlockPatternBuilder`;
`net.minecraft.world.level.block.DispenserBlock#registerBehavior`;
`net.minecraft.world.entity.animal.golem.SnowGolem`;
`net.minecraft.world.entity.monster.EnderMan$EndermanTakeBlockGoal`;
`net.minecraft.world.entity.monster.EnderMan$EndermanLeaveBlockGoal`;
`net.minecraft.world.entity.monster.skeleton.AbstractSkeleton#finalizeSpawn`;
`net.minecraft.world.entity.monster.zombie.Zombie#finalizeSpawn`;
`net.minecraft.world.entity.monster.EnderMan#isBeingStaredBy`;
`net.minecraft.world.entity.LivingEntity#PLAYER_NOT_WEARING_DISGUISE_ITEM`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube`;
`net.minecraft.world.entity.SulfurCubeArchetype`;
`net.minecraft.world.level.saveddata.maps.MapItemSavedData#tickCarriedBy`;
`net.minecraft.world.waypoints.Waypoint`;
`net.minecraft.world.waypoints.WaypointTransmitter`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces`;
`net.minecraft.client.gui.Hud#extractCameraOverlays`;
block/item/sound reports and Carved Pumpkin components; direct block/item tags;
`carve/pumpkin`, block and Snow-Golem-shearing loot; Jack-o'Lantern and Pumpkin
Pie recipe/advancements; summon-Iron-Golem advancement; Sulfur-Cube archetype;
all 1,212 templates and relevant Outpost pools; blockstate/model/item/texture/
language resources. Complete compiled exact-field and data-reference searches
found no other runtime path.

**Test vectors:**

Run `EXP-BLK-110` across all facings, placement/transform/same-identity paths,
every base/full pattern orientation and write/create/add/chest failure, carving,
loot, dispenser/equipment, map/gaze/waypoint/HUD, Sulfur Cube, Halloween,
Enderman, recipes/advancements/compost, both templates, persistence/reload and
projection. Assert exact ordering, constants, absences and vanilla convergence.

**Limits:**

Generic placement, state writes, block breaking/explosion, entity insertion,
criteria, equipment, attribute composition, maps, mob AI, crafting,
composting, template placement, packet encoding and rendering remain with
their named owners. Jack o'Lantern, the three golem entities, copper weather/
chest behavior and Pumpkin retain their own leaves or generic families. This
leaf fixes Carved Pumpkin identity, hooks, joins, locked data and projection.
