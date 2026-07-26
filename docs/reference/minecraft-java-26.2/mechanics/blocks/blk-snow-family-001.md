# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SNOW-FAMILY-001` — Snow layers accumulate and melt while Powder Snow traps, freezes and buckets entities

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`BLK-CARVED-PUMPKIN-001`, `BLK-LAVA-CAULDRON-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-MOVE-001`,
`PLY-MOVE-SPECIAL-001`, `PLY-COLLISION-001`, `PLY-AUTOJUMP-001`,
`RED-001`, `RED-UPDATE-001`, `RED-COMPARATOR-001`,
`ITM-003`, `ITM-004`, `ITM-006`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-DISPENSER-001`, `ENT-001`, `ENT-DAMAGE-001`,
`ENT-EFFECT-001`, `ENT-LIFECYCLE-001`, `ENT-KNOCKBACK-001`,
`MOB-001`, `MOB-AI-001`, `MOB-SPAWN-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`,
`ENV-FIRE-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-VILLAGES-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `WGEN-STRUCTURE-ANCIENT-CITY-001`,
`WGEN-STRUCTURE-IGLOO-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, source, data and client inspection
close all three block identities and their items: eight-layer geometry,
support, stacking and melting; Powder Snow collision, contact, freezing,
bucket and cauldron transactions; mob and weather joins; loot, recipes,
progression, legacy migration and projection. An exhaustive decoded scan of
all 1,212 templates fixes 2,525 raw cells and eight executable Snow Block
Jigsaw final states.

**Applies when:**

`minecraft:snow`, `minecraft:snow_block` or `minecraft:powder_snow` is
placed, stacked, updated, random-ticked, collided with, walked on, fallen
onto, bucketed, mined, generated, migrated, persisted, synchronized or
rendered; it also fixes the matching Snow/Snow Block items and Powder Snow
Bucket.

**Authoritative state:**

None has a block entity. Exact registrations are:

| Identity | Block ID | State IDs/default | Item ID/form | Implementation | Strength/resistance | Map/sound |
| --- | ---: | --- | --- | --- | --- | --- |
| Snow | `276` | `6919..6926`, `layers=1..8`; default `6919` | `365`, stack-64 `BlockItem` | `SnowLayerBlock`; type `minecraft:snow_layer` | `0.1/0.1`, correct tool | Snow/Snow |
| Snow Block | `278` | `6928` | `367`, stack-64 `BlockItem` | ordinary `Block` | `0.2/0.2`, correct tool | Snow/Snow |
| Powder Snow | `1027` | `27162` | no ordinary item; Powder Snow Bucket `1043`, stack `1`, `SolidBucketItem` | `PowderSnowBlock`; type `minecraft:powder_snow` | `0.25/0.25` | Snow/Powder Snow |

Snow is replaceable, random-ticking, force-solid-off and piston-destroying.
Its view-blocking predicate is true only at eight layers. Snow Block is an
ordinary full cube. Powder Snow is dynamic-shape, nonoccluding and never a
redstone conductor. All retain Harp instrument, friction `0.6`, speed/jump
factors `1`, light `0`, no signal/comparator output and no scheduled-tick
loop.

Snow break/fall/hit/place/step sound-event IDs are
`1568/1569/1575/1576/1577`; Powder Snow uses
`1337/1338/1339/1340/1341`. Both sound profiles have volume/pitch `1/1`.
Powder Snow Bucket empty/fill sounds are IDs `227/234`.

All three stacks have common rarity, empty attributes/enchantments/lore and
ordinary generic components. Snow Block alone directly belongs to item tag
`sulfur_cube_archetype/fast_sliding`; Snow and Powder Snow Bucket have no
direct item tag.

**Transition and ordering:**

### Snow layers: geometry, support and stacking

For layer count `L`:

- outline, visual and support shape are `16×(2L)×16`;
- collision shape is `16×(2L-2)×16`, so layer one has no collision and
  layer eight still collides only through Y `14/16`;
- land pathfinding is true only for `L<5`; water and air pathfinding are
  false;
- shade brightness is `0.2` at `L=8` and `1` otherwise.

Snow uses its shape for light occlusion. Its survival test reads the block
below in this strict order:

1. membership in live `cannot_support_snow_layer` rejects;
2. membership in live `support_override_snow_layer` accepts;
3. otherwise accept when the below collision shape has a full `UP` face, or
   when below is exact Snow at eight layers.

Baseline `cannot_support_snow_layer` is Ice, Packed Ice and Barrier.
Baseline `support_override_snow_layer` is Honey Block, Soul Sand and Mud.
Every shape update rechecks survival; failure immediately returns Air and
drops nothing, while success returns the superclass update. No tick is
scheduled.

With a held exact Snow item and `L<8`, replacement is admitted when the
context is not replacing the clicked block, or when it is and the clicked
face is `UP`. For every other held item, only `L=1` is replaceable. Placement
on exact Snow increments `L` by one, capped at eight; fresh placement uses
one layer. Each accepted ordinary item placement consumes one Snow item.

On a selected server random tick, block light at the Snow position greater
than `11` first calls entityless `dropResources`, then
`removeBlock(position,false)`; both outcomes are ignored. Because the loot
pool requires `THIS_ENTITY`, this melt path produces no item. Light `<=11`
does nothing.

### Powder Snow collision and contact

Powder Snow suppresses the shared face between two exact Powder Snow states
and has an empty visual shape. Its collision query is:

1. placement context, non-entity context or null entity: empty;
2. entity fall distance `>2.5`: full X/Z and Y `0..0.9`
   (`14.4/16`);
3. Falling Block, or an entity that can walk on Powder Snow, is above the
   full-block shape and is not descending: superclass collision, here a
   full cube;
4. otherwise empty.

An entity can walk when its type belongs to live
`powder_snow_walkable_mobs`, or it is living and wears exact Leather Boots
in `FEET`. Baseline walkable types are Rabbit, Endermite, Silverfish and
Fox. The entity-inside collision shape returns the context collision shape,
except that an empty result becomes a full cube so inside traversal remains
observable.

`entityInside` skips only the slowdown/particle section when a living
entity's current in-block state is not this exact Powder Snow implementation.
Every other entity is stuck with multiplier `(0.9,1.5,0.9)`. On the client,
horizontal movement consumes one `nextBoolean`; true emits one Snowflake at
`(entityX, blockY+1, entityZ)` with X/Z velocity independently sampled in
`[-1,1]/12` and Y velocity `0.05`.

The contact then always queues its conditional destroy consumer before
adding `FREEZE` and `EXTINGUISH`. During step flush, freeze applies first;
the consumer then runs just before extinguishing. If the entity was on fire,
the server's `mobGriefing` rule is true or the entity is a Player, and
`mayInteract(server,position)` is true, it calls
`destroyBlock(position,false)` and ignores the result. Thus an admitted
burning entity can erase Powder Snow without loot before its fire is
cleared. Step effect types are deduplicated, consumers are not, and
remaining effects execute only while the entity is alive.

Powder Snow overrides `fallOn` without calling the superclass. A living
entity with fall distance at least `4` plays its small fall sound below `7`
and big fall sound at or above `7`, at volume/pitch `1/1`. No entity receives
the ordinary block-mediated fall-damage call from this block.

The block reports pathfindable for every computation type. Generic walking
evaluation nevertheless recognizes exact Powder Snow as
`PathType.POWDER_SNOW`; eligible walkable mobs can run
`ClimbOnTopOfPowderSnowGoal`. That goal is active when the mob was or is in
Powder Snow, its type is in the walkable tag, and the block above is exact
Powder Snow or exposes the canonical empty collision shape; it owns only
`JUMP`, requires every tick and calls `JumpControl.jump`.

Generic player travel retains its `PLY-MOVE-001` join: after movement, a
horizontal collision or jump while previously in Powder Snow and eligible
to walk replaces only velocity Y with `0.2`.

### Freezing, damage and mob conversions

The `FREEZE` contact marks `isInPowderSnow=true`. When `canFreeze`, it
increments synchronized `ticksFrozen` by one, capped at required freeze
ticks `140`. A living entity not in Powder Snow or no longer eligible decays
that counter by two per tick to zero.

Living `canFreeze` rejects spectators and any live
`freeze_immune_wearables` member in the four humanoid armor slots, then
delegates to the entity-type immunity test. The baseline wearable tag is all
four Leather armor pieces plus Leather Horse Armor; the latter is in BODY
and is not visited by this humanoid-ARMOR loop. Baseline
`freeze_immune_entity_types` is Stray, Polar Bear, Snow Golem and Wither.

Each living server tick removes the old transient
`minecraft:powder_snow` movement-speed modifier. When the movement-affecting
block below is nonair and `ticksFrozen>0`, it adds an `ADD_VALUE` modifier
of `-0.05*percentFrozen`. At full freeze, every tick whose
`tickCount % 40 == 0` offers `1` Freeze damage. The result is ignored;
members of `freeze_hurts_extra_types` — Strider, Blaze and Magma Cube —
receive the generic fivefold freeze transform, hence `5`. Ignition clears
the freeze counter.

`ticksFrozen` is entity metadata, saves as `TicksFrozen` only when positive
and loads with default zero. `isInPowderSnow` is transient. The client uses
the synchronized percentage for the frozen overlay.

Skeleton overrides `canFreeze` false but has a separate server-only,
alive, AI-enabled conversion:

- while in Powder Snow and not converting, increment `inPowderSnowTime`;
  reaching `140` starts a `300`-tick conversion;
- while converting, decrement first and convert to Stray when the result is
  negative;
- outside Powder Snow, reset `inPowderSnowTime=-1` and clear converting.

Conversion uses single-entity parameters
`(keepEquipment,preserveCanPickUpLoot)=(true,true)`. A nonsilent source emits
level event `1048` in its
after-conversion callback. Only the remaining conversion timer persists as
`StrayConversionTime`; `-1` means inactive.

Stray natural spawning climbs from the candidate's cell above through every
exact Powder Snow block. It then applies ordinary Monster spawn rules; a
non-spawner reason additionally requires sky visibility at the cell below
the first non-Powder cell. Stray itself is freeze- and Powder-Snow-danger
immune.

Entity-type block danger treats Powder Snow as dangerous unless the live
type-specific immunity tag contains it. The direct baseline tags make Polar
Bear, Snow Golem and Stray immune. Fox pounce landing on exact Snow can set
pitch `60`, clear its target and mark it faceplanted; generic Fox goal
admission retains ownership.

Snow Golem's server AI samples four foot cells:
`floor(x+((i%2)*2-1)*0.25)`, `floor(y)`,
`floor(z+(((i/2)%2)*2-1)*0.25)` for `i=0..3`. With
`mobGriefing=true`, each air cell where default one-layer Snow survives is
offered through `setBlockAndUpdate`; the result is ignored and
`BLOCK_PLACE` is emitted after the offered write. Its environmental melt
damage and the two-Snow-Block construction pattern remain owned by
`ENV-003` and `BLK-CARVED-PUMPKIN-001`.

### Buckets and cauldrons

Powder Snow has no ordinary block item, but its Solid Bucket registration
maps the block back to Powder Snow Bucket, so clone/pick returns that stack.
Player `useOn` delegates to ordinary BlockItem placement; a consuming result
replaces the held stack with the generic empty-Bucket success result.
Placement sound is Bucket Empty Powder Snow.

The dispenser/container `emptyContents` path succeeds only in world bounds
and on an empty cell. The server offers default Powder Snow with flags `3`
and ignores the result; the client skips that write. Both sides then play
the empty sound at `1/1`, emit `FLUID_PLACE`, and return true. A successful
dispenser placement consumes the filled bucket and retains Bucket as
remainder; failure uses protected default ejection.

`pickupBlock` offers Air with flags `11`, ignores the result, emits server
level event `2001` with original state ID `27162`, and returns Powder Snow
Bucket. Its pickup sound is Bucket Fill Powder Snow. The generic empty
Bucket player path and dispenser Bucket path both use this interface;
successful dispenser pickup emits `FLUID_PICKUP` and replaces one Bucket
with the filled result.

Cauldron joins retain `BLK-LAVA-CAULDRON-001` ownership:

- empty-Cauldron snow precipitation uses `nextFloat()<0.1` to offer default
  level-one Powder Snow Cauldron, then emits `BLOCK_CHANGE` even after a
  rejected write;
- a nonfull Powder Snow Cauldron selected by snow precipitation uses the
  same threshold to increment its level and likewise ignores the write
  before the event;
- an empty Bucket fills only from level three, producing Powder Snow Bucket,
  awarding `USE_CAULDRON` and Bucket-used stats, offering empty Cauldron,
  then playing fill sound and emitting `FLUID_PICKUP`;
- Powder Snow Bucket is installed in every cauldron dispatcher and empties
  to level three, returning Bucket, awarding `FILL_CAULDRON` and item-used,
  offering the state, then playing empty sound and emitting `FLUID_PLACE`.

Cauldron state-write results are ignored; client interactions return success
without authoritative mutation. A burning entity inside a Powder Snow
Cauldron first changes it to same-level Water Cauldron, then lowers that
level, producing Water level `L-1` or empty at `L=1`.

### Loot, crafting, progression and acquisition

Snow and Snow Block require the generic correct-tool harvest gate and are
directly `mineable/shovel`; every vanilla Shovel qualifies. A wrong tool or
hand breaks without player-harvest loot.

Snow's one loot pool additionally requires an available `THIS_ENTITY`.
Without Silk Touch, admitted layer `L` yields exactly `L` Snowballs. With
Silk Touch, layers `1..7` yield exactly `L` Snow items and layer `8` yields
one Snow Block. No branch has explosion decay. Consequently entityless
melting/support removal and a source-less explosion yield nothing; an
explosion whose direct source entity is present passes the entity condition
and retains the full nonsilk Snowball count despite an explosion-radius
parameter.

Snow Block yields itself with Silk Touch; otherwise it yields four Snowballs
with explosion decay. Powder Snow has no block loot table, so ordinary
mining produces no stack.

Two shaped recipes form a reversible but lossy item graph:

- one horizontal row of three Snow Blocks produces six Snow items;
- a `2×2` square of Snowballs produces one Snow Block.

Both recipe advancements use exact Snowball inventory and recipe-unlocked
criteria in one OR requirement group, then reward their recipe. The first
therefore unlocks from Snowballs despite consuming Snow Blocks.

The `Light as a Rabbit` advancement, after `adventure/sleep_in_bed`, uses a
location trigger requiring exact Leather Boots in feet and stepping on exact
Powder Snow; it sends telemetry. Powder Snow Bucket is acquired through
pickup/cauldron paths or creative/command, not a recipe, merchant or loot
record.

Snow Block is a weight-`4` entry among total weight `53` in each of `3..8`
first-pool rolls of the snowy-village-house chest table. Snowballs have
weight `10` and count `1..7` there. No other non-block loot table or merchant
offer directly produces these three matching block items.

Snow Block's direct fast-sliding tag lets an adult Sulfur Cube with empty
BODY equipment select that archetype when it accepts the item. The
archetype fixes horizontal/vertical knockback powers `0.6625/0.09`;
additive knockback and explosion-knockback resistance `0.5/0.5`; additive
bounciness `0.10000000149011612`; total-multiplied friction and air drag
`-0.9499999992549419/-0.9900000002235174`; fast-sliding hit/push sounds
`1949/1950`, push cooldown `1`, impulse threshold `0.05`, and no contact
damage or explosion.
Admission, multi-match order and knockback calculation retain
`ENT-KNOCKBACK-001`.

None of the blocks/items is registered as fire fuel or compostable, and
direct fire encouragement/flammability are `0/0`.

### Tags, weather and generation

All three blocks directly belong to live `snow`. Snow and Snow Block are
direct Fox/Goat/Rabbit/Wolf spawn substrates. Additional direct joins are:

- Snow: `combination_step_sound_blocks`,
  `mangrove_roots_can_grow_through`, `replaceable` and `mineable/shovel`;
- Snow Block: `azalea_grows_on`, `azalea_root_replaceable`,
  `ice_spike_replaceable` and `mineable/shovel`;
- Powder Snow: `azalea_grows_on`, `azalea_root_replaceable`,
  `inside_step_sound_blocks`, `polar_bear_immune_to`,
  `snow_golem_immune_to` and `stray_immune_to`.

Nested `snow` makes all three Overworld-carver replaceables. Dirt snowy-state
maintenance and sound/spawn/plant/tag consumers retain their named owners;
reload changes subsequent membership tests.

During weather precipitation at `MOTION_BLOCKING` height, when raining,
`max_snow_accumulation_height>0` and the biome says Snow:

- exact Snow at `L<min(gameRule,8)` calls
  `pushEntitiesUp(old,L+1,...)` for its entity effects, discards the returned
  state, then offers `L+1`;
- every other target offers default one-layer Snow.

Write results are ignored. The rule defaults to `1` and is clamped `0..8`.
Biome snow requires snow precipitation, valid build height, block light
below `10`, a target that is air or exact Snow, and successful default-Snow
survival. The precipitation callback on the block below follows afterward.

The existing `WGEN-PIPELINE-001` owner fixes all procedural consumers:
freeze-top-layer writes default one-layer Snow with flags `2`; `pile_snow`
uses the same provider; surface rules select Snow Block and noise-gated
Powder Snow; Iceberg may write Snow Block; ice-spike/ice-patch, frozen
springs, pine/spruce-on-snow and root-system records use the exact states or
tags documented there. The `snowy_kingdom` flat preset displays Snow and
stacks Bedrock `1`, Stone `59`, Dirt `3`, Grass Block `1`, Snow `1`, with
features/lakes false and Villages/Igloos enabled.

An exhaustive decoded scan of all 1,212 bundled templates finds:

| Identity | Raw files/cells | State split | Root groups | Extra exact strings |
| --- | --- | --- | --- | --- |
| Snow | `43/1,479` | layers `1..8 = 1204/74/53/70/44/19/6/9` | Ancient City `1/30`; Village `42/1,449` | five villager `type=minecraft:snow` values, not blocks |
| Snow Block | `23/651` | property-free | Igloo `1/94`; Village `22/557` | eight Village Jigsaw `final_state=minecraft:snow_block` |
| Powder Snow | `1/395` | property-free | Trial Chambers `1/395` | none |

No raw target cell has block NBT. Snow uses 123 distinct palette entries;
its five nonpalette UTF matches are three ordinary and two zombie Snow
Villager entity-data values. Snow Block has 23 palette strings plus the
eight executable connector values. Powder Snow has exactly one palette
string. Template transforms, processors, clipping and failed writes retain
their structure owners.

### Persistence migration and client projection

Current state persistence and terrain packets preserve Snow layers exactly;
the other two states preserve identity only. Legacy numeric states
`1248..1255` map to Snow layers `1..8`; invalid metadata states
`1256..1263` fill from the block default, layer `1`. Legacy `1280..1295`
map to Snow Block. Old block aliases are `minecraft:snow_layer` for Snow and
`minecraft:snow` for Snow Block. Item ID/name pairs
`78/minecraft:snow_layer.0` and `80/minecraft:snow.0` flatten to current Snow
and Snow Block; Snowball item ID `332` remains Snowball. Powder Snow has no
pre-flattening numeric mapping.

Snow blockstates select `snow_height2/4/6/8/10/12/14` for layers `1..7`
and the full `snow_block` model for layer `8`. The layer models have the
matching `2..14` model-unit height and Snow texture; the Snow item always
selects `snow_height2`. Snow Block is an opaque `cube_all` Snow texture for
both block and item.

Powder Snow selects a six-element boundary-shell model. Each opposing face
is represented twice in a shell only `0.002` model units thick at
`0/0.002` or `15.998/16`; all use the Powder Snow texture and matching
cullface. Powder Snow Bucket uses its generated item model. Snow, Powder
Snow and Bucket textures are static 16×16 PNGs; the frozen outline is a
static 256×256 PNG. No block tint applies. Names are `Snow`, `Snow Block`
and `Powder Snow Bucket`.

Camera probing exact Powder Snow selects `FogType.POWDER_SNOW`. Its fog base
color is ARGB `0xFF9FBBCC`; nonspectators use environmental start/end
`0/2`, spectators `-8/renderDistance*0.5`, and sky/cloud end equal the
environmental end. A local player with positive frozen ticks renders
`textures/misc/powder_snow_outline.png` at alpha `percentFrozen`.

Natural Blocks orders the run Ice, Packed Ice, Blue Ice, Snow Block, Snow,
Moss Block. Powder Snow has no block item there; Tools & Utilities orders
Lava Bucket, Powder Snow Bucket, Milk Bucket.

**Branches and aborts:**

- Snow support rejects the cannot tag before consulting the override tag;
  failed updates become Air immediately.
- Stacking stops at eight and clicked-block replacement requires the top
  face.
- Snow melt is strict light `>11` and its entityless loot context fails.
- Powder collision stops at context, fall-distance, falling/walkable/above/
  descending gates; most entities receive no collision.
- Contact destruction requires fire, griefing-or-player and interaction
  permission, but freeze and extinguish are still queued.
- Bucket empty/pickup, cauldron, weather, Snow-Golem and worldgen writes
  deliberately ignore low-level Boolean results.
- Freezing stops at spectator, wearable and entity-type immunity; extra
  damage is a later generic transform.
- Skeleton conversion and ordinary freezing are disjoint because Skeleton
  hardcodes `canFreeze=false`.
- Loot stops at correct-tool player harvest and, for Snow, presence of
  `THIS_ENTITY`.

**Constants and randomness:**

Layers `1..8`; outline height `2L/16`; collision `(2L-2)/16`; land path
limit `5`; melt light `>11`; Powder fall collision `>2.5`, height `0.9`;
slowdown `(0.9,1.5,0.9)`; particle Boolean and X/Z `[-1,1]/12`, Y `0.05`;
fall sounds `4/7`; freeze cap `140`, decay `2`, damage cadence `40`, base
damage `1`, speed coefficient `-0.05`; Skeleton exposure/conversion
`140/300`; player escape Y `0.2`; precipitation chance `0.1`, game-rule
default/range `1/0..8`; raw cells `1,479/651/395`; Jigsaw final states `8`.

**Side effects:**

Layer writes/removal and entity displacement; loot and item consumption;
Powder slowdown, particles, destruction, freeze/extinguish, sounds and
damage; entity metadata/modifiers and Skeleton conversion; bucket/cauldron
stacks, stats, sounds, events and level event; Snow-Golem trail; weather,
feature, surface and structure writes; recipe/progression; client fog,
overlay and models.

**Gates:**

Live support/tool/entity/item/block tags; face/context/layer; block light;
collision context/entity/fall/descending; side, movement and random Boolean;
fire, game rule and interaction permission; spectator/equipment/type/timer;
server/client; world bounds/empty target/container state; cauldron level and
precipitation draw; correct tool/entity loot/Silk/explosion; recipe and
advancement predicates; biome/weather/game rule/light/support; feature,
surface, template and processor selection; data version and resource reload.

**Boundary cases and quirks:**

Eight Snow layers render as a full cube but collide only to `14/16`.
Support override cannot rescue a member of the earlier cannot-support tag.
Powder Snow returns a full entity-inside trigger shape precisely when its
ordinary collision is empty. Its fall callback suppresses ordinary
block-mediated fall damage even for entities that receive no replacement
sound. Fire destruction is queued before extinguishing but flushed after
freeze. Leather Boots simultaneously enable walking and the advancement;
any one of four Leather armor pieces prevents freezing, while tagged Leather
Horse Armor in BODY is not read by the humanoid armor loop. Snow's entity
loot condition makes source-bearing and source-less explosions diverge.
Weather stacking discards `pushEntitiesUp`'s returned state before writing
the independently constructed next layer.

**Failure semantics:**

Every direct state write described as offered ignores its Boolean result and
has no rollback. Snow-Golem, precipitation and cauldron events can survive a
rejected write. Powder Bucket empty can report success and emit sound/event
after a rejected server write; pickup can return the filled bucket after a
rejected Air write. Earlier contact effects/consumers and worldgen/template
writes remain committed when a later action fails.

**Client/server authority split:**

The server owns support/random ticks, collision-authoritative movement,
freeze/damage/conversion, bucket/cauldron inventory, loot, progression,
weather, generation and migration. The client predicts placement/cauldron
results, runs the movement particle branch, and renders synchronized states,
frozen metadata, models, fog, overlay, names and tab contents.

**Observability:**

Observe registry/state/item/sound IDs, layer geometry and support order,
replacement/placement/melt transitions, every Powder collision/contact
gate and effect order, freeze metadata/modifiers/damage/conversion, bucket
stacks/stats/sounds/events, cauldron/weather/Snow-Golem writes, loot counts,
recipe/progression/archetype/tag joins, exact template census and Jigsaw
strings, legacy inputs, persisted/wire states and complete client projection.

**Persistence and reload:**

Snow persists `layers`; Snow Block and Powder Snow persist identity. No block
entity exists. Entity `TicksFrozen` and active Skeleton conversion timer
persist as described; transient contact flags do not. Stacks use generic
components. Tags, loot, recipes, advancements, worldgen and client resources
retain independent reload boundaries. Registrations, physical profiles,
bucket behavior, entity algorithms and data-fix mappings are code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SnowLayerBlock`;
`net.minecraft.world.level.block.PowderSnowBlock`;
`net.minecraft.world.item.SolidBucketItem`;
`net.minecraft.world.item.BucketItem`;
`net.minecraft.core.dispenser.DispenseItemBehavior`;
`net.minecraft.core.cauldron.CauldronInteractions`;
`net.minecraft.world.level.block.CauldronBlock`;
`net.minecraft.world.level.block.LayeredCauldronBlock`;
`net.minecraft.world.entity.InsideBlockEffectApplier`;
`net.minecraft.world.entity.Entity`;
`net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.monster.skeleton.Skeleton`;
`net.minecraft.world.entity.monster.skeleton.Stray`;
`net.minecraft.world.entity.ai.goal.ClimbOnTopOfPowderSnowGoal`;
`net.minecraft.world.entity.animal.golem.SnowGolem`;
`net.minecraft.world.entity.animal.feline.Fox`;
`net.minecraft.world.entity.SulfurCubeArchetypes`;
`net.minecraft.server.level.ServerLevel#tickPrecipitation`;
`net.minecraft.world.level.biome.Biome#shouldSnow`;
`net.minecraft.util.datafix.fixes.BlockStateData`;
`net.minecraft.util.datafix.fixes.ItemIdFix`;
`net.minecraft.util.datafix.fixes.ItemStackTheFlatteningFix`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.client.Camera`;
`net.minecraft.client.renderer.fog.environment.PowderedSnowFogEnvironment`;
`net.minecraft.client.gui.Hud`; block/item/sound/component reports; all
direct/composed tags; both loot tables, recipes and recipe advancements;
walking advancement; snowy-village chest table; all relevant worldgen
records; all 1,212 decoded structures and decompressed strings; blockstates,
models, item definitions, textures and language resources. Complete compiled
exact-field, data and decoded-NBT searches found no other identity-specific
runtime path.

**Test vectors:**

Run `EXP-BLK-115` across all ten Snow-family states, every support/tag/light/
placement/tool/loot branch, every Powder context/entity/equipment/fire/
permission/fall/freeze/damage/conversion path, bucket/dispenser/cauldron
transactions including rejected writes, Snow-Golem/weather/worldgen joins,
2,525 raw cells and eight Jigsaw final states, legacy migration,
persistence/reload and exact client projection. Assert IDs, order, constants,
absences, census and vanilla convergence.

**Limits:**

Generic placement/break, collision/travel, damage, container remainder,
loot, recipe/progression, mob AI/spawning, weather, feature/surface/Jigsaw,
data-fix dispatch and rendering retain their named owners. Snowball,
Leather equipment, cauldron identities, mobs and structures retain their
catalog families. This leaf fixes the three block identities, their matching
item forms and every exact join that selects them.
