# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-PUMPKIN-001` — Pumpkin grows as stem fruit and carves into a golem-capable head

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`BLK-STEM-CROP-001`, `BLK-CARVED-PUMPKIN-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`PLY-BREAK-001`, `PLY-COLLISION-001`, `PLY-AUTOJUMP-001`,
`RED-001`, `RED-UPDATE-001`, `RED-COMPARATOR-001`, `ITM-003`,
`ITM-004`, `ITM-006`, `ITM-USE-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ITM-DISPENSER-001`, `ITM-PUMPKIN-PIE-001`,
`ENT-001`, `ENT-KNOCKBACK-001`, `MOB-001`, `MOB-AI-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`,
`ENV-FIRE-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-OUTPOST-001`,
`WGEN-JIGSAW-VILLAGES-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration and `PumpkinBlock` bytecode,
connection and chunk-upgrade exact-identity branches, complete loot,
recipe, advancement, trade and tag data, code-built Composter and merchant
records, every worldgen reference, all `1,212` decoded templates, contextual
legacy fixes and exact client resources close the property-free Pumpkin
block and item. Its special runtime is Shears carving, explicit connection
rejection, stem-fruit production, three direct tag joins, fast-flat
equipment, exact natural/village/structure acquisition and migration.

**Applies when:**

`minecraft:pumpkin` is placed, used by Shears, queried by a fence, pane or
wall, mined, exploded, composted, crafted, traded, carried by an Enderman,
selected as Sulfur-Cube equipment, grown from a stem, generated, migrated,
persisted, synchronized or rendered.

**Authoritative state:**

Pumpkin is a property-free `PumpkinBlock` with no block entity and sole
block-state ID `8332`. Its block protocol ID is `360`; its ordinary
block-item raw ID is `384`.

Registration selects map color `COLOR_ORANGE`, note instrument
`DIDGERIDOO`, hardness/resistance `1/1`, Wood sounds, friction `0.6`,
speed/jump factors `1`, emission `0`, light dampening `15`, piston reaction
`DESTROY` and no correct-tool requirement. Outline, collision, visual,
support and occlusion shapes are full unit cubes. Every face is sturdy;
shade brightness is `0.2`; the state is an ordinary redstone conductor,
view blocker, suffocation state and valid-spawn support.

It has no placement-state, survival, random/scheduled tick, shape-update,
neighbor, attack, entity-contact, fall, signal, comparator, fluid or
block-event override. Its sole callback is exact-Shears item use. Land,
water and air pathfinding return false through the ordinary full-solid
path. Rotation and mirror preserve state `8332`.

Wood break/fall/hit/place/step sound-event IDs are
`1853/1854/1855/1856/1857`, with profile volume/pitch `1/1`. The common
stack-64 `BlockItem` has ordinary generic components.

**Transition and ordering:**

### Placement, connections, mining and self loot

Ordinary item placement, component placement and command writes select
state `8332` without a support predicate.

Despite full sturdy faces, `Block.isExceptionForConnection` returns true
for exact Pumpkin. A neighboring ordinary Fence, Iron Bars/pane or Wall
therefore cannot use Pumpkin's sturdy face as its generic connection
support. Same-family and gate/tag alternatives remain with those block
owners. This exception changes their connection properties and shapes; it
does not remove Pumpkin's sturdy-face support for unrelated consumers.

The direct `mineable/axe` and `sword_efficient` tags select their generic
tool-speed rules. Neither establishes loot correctness. Hand, Axe, Sword,
every other tool, Silk Touch and Fortune all reach the same one-roll block
table, which emits one Pumpkin behind `survives_explosion` and uses random
sequence `minecraft:blocks/pumpkin`. Explosion decay can suppress the
result; no tool/enchantment changes its count or survival probability.

Piston handling destroys rather than moves the state. Pumpkin has no
`FireBlock.bootStrap` row, lava-ignition property or fuel time: direct fire
encouragement/flammability are `0/0`.

### Exact-Shears carving transaction

An item other than exact Shears delegates to the ordinary block-use path.
The Shears branch returns `SUCCESS` immediately on the client without a
mutation. On the server, it commits in this order:

1. A vertical hit chooses the opposite of the player's horizontal
   direction; a horizontal hit uses the clicked face.
2. Built-in block-interact table `minecraft:carve/pumpkin` evaluates with
   the live Pumpkin state, Shears instance and player. Its sole
   unconditional result is exactly four Pumpkin Seeds and its random
   sequence is `minecraft:carve/pumpkin`.
3. The output stack becomes an `ItemEntity` at
   `(x+0.5+0.65*dx,y+0.1,z+0.5+0.65*dz)`. Its velocity is
   `(.05*dx+nextDouble*.02,.05,.05*dz+nextDouble*.02)`;
   entity insertion is ignored.
4. Pumpkin Carve sound ID `1348` plays at volume/pitch `1/1`.
5. The server offers default Carved Pumpkin with the derived facing at the
   original position, flags `11`, and ignores the Boolean result.
6. It damages Shears by one in the used-hand slot, emits player `SHEAR`,
   awards the Shears `ITEM_USED` statistic and returns `SUCCESS`.

Loot evaluation or rejected entity/block writes do not abort later sound,
durability, event or statistic side effects. A successful Carved-Pumpkin
write invokes that block's `onPlace`; when the Pumpkin occupied the head of
a complete Snow/Iron/Copper-Golem pattern, carving can therefore run the
golem transaction owned by `BLK-CARVED-PUMPKIN-001`.

### Recipes and progression

Two shapeless recipes consume exact Pumpkin:

- one Pumpkin produces four Pumpkin Seeds; and
- Pumpkin plus Sugar plus one member of item tag `eggs` produces one
  Pumpkin Pie.

Extra occupied slots reject either match. Ingredient component patches are
ignored and no Pumpkin components transfer to either output.

The seed-recipe advancement has one OR requirement: exact Pumpkin
possession or existing knowledge of `pumpkin_seeds`. The Pie advancement
has one OR requirement across existing Pie-recipe knowledge, exact Pumpkin
possession and exact Carved-Pumpkin possession. Carved Pumpkin can therefore
unlock the Pie recipe but cannot satisfy its actual Pumpkin ingredient.
Pumpkin-Pie components, eating, Balanced Diet, chest/trade/gift sources and
composting retain `ITM-PUMPKIN-PIE-001`; seed planting, crop growth, animal
food and independent seed sources retain `BLK-STEM-CROP-001`.

No locked recipe produces Pumpkin.

### Composter, chest and merchant acquisition

Composter bootstrap registers the exact Pumpkin item at Java float chance
`0.65f`. At level zero an admitted player or automated insertion succeeds
without RNG; levels `1..6` test strict `nextDouble() < 0.65f` widened to
double. Level-seven extraction, delayed conversion, item/stat/event order
and failed-attempt behavior remain with the Composter owner.

The first pool of `chests/shipwreck_supply` makes uniformly `3..10` rolls
with replacement across total weight `84`. Pumpkin has weight `2`, hence
probability `1/42` per roll, and emits a uniformly integral count `1..3`.
The table uses random sequence `minecraft:chests/shipwreck_supply`.

Farmer level two has three predicate-free records and selects two distinct
offers through `minecraft:trade_set/farmer/level_2`, so
`farmer/2/pumpkin_emerald` has inclusion probability `2/3`. It consumes six
Pumpkins, gives one Emerald, permits `12` uses, grants villager XP `10` and
uses reputation discount `0.05`.

Wandering-Trader `emerald_pumpkin` consumes one Emerald for one Pumpkin,
permits `4` uses, inherits XP `1` and uses reputation discount `0.05`. It is
one of `76` common records; that set selects five distinct offers through
`minecraft:trade_set/wandering_trader/common`, giving exact inclusion
probability `5/76`. Offer construction, ordering, stock, pricing, demand,
reputation and transactions retain the merchant owner.

Complete baseline loot and merchant searches find no other direct Pumpkin
source. Stem growth, self loot, shipwreck supply, the Wandering Trader,
natural/village/structure generation, creative publication and commands
are its acquisition paths; the Farmer record is a sink.

### Complete block/item tag closures

Pumpkin belongs directly, and only, to block tags `enderman_holdable`,
`mineable/axe` and `sword_efficient`; none has a locked ancestor, giving a
complete three-tag block closure.

An empty-handed Enderman under `mobGriefing` can select a live Pumpkin,
remove it without block loot, emit `BLOCK_DESTROY` and carry state `8332`.
Its placement goal can later offer that carried state after its target,
support, obstruction and game-rule gates. Removal/placement writes,
particles/events and failure behavior remain with the Enderman owner.

The item directly belongs to `sulfur_cube_archetype/fast_flat`, nested by
`sulfur_cube_swallowable`; that is the complete two-tag item closure. The
non-buoyant fast-flat archetype fixes horizontal/vertical knockback
`0.9125/0.09`, additive knockback and explosion-knockback resistance
`-1/-1`, additive bounciness `0.5`, total-multiplied friction/air drag
`-0.7999999970197678/-0.9900000002235174`, hit/push sound IDs
`1945/1946`, push cooldown `0.9` and impulse threshold `0.03`. It supplies
no contact damage or explosion.

An accepting adult Sulfur Cube can install one Pumpkin in empty BODY
equipment. The swallowable parent also admits the generic dispenser search
and first-accepting-cube consumption; otherwise protected default ejection
runs. Matching order, equipment replacement, modifiers, collision,
knockback, sound and residue retain `ENT-KNOCKBACK-001` and
`ITM-DISPENSER-001`.

### Mature-stem production and chunk repair

An admitted age-seven Pumpkin-Stem fruit trial chooses one horizontal
direction, requires Air at the target and a live
`supports_pumpkin_stem_fruit` block below, resolves both holders, then
offers default Pumpkin at the target before offering the source's attached
stem. Both `setBlockAndUpdate` results are ignored and neither failure rolls
back the other. Brightness, crop-speed calculation, growth draw, direction,
holder lookup and support closure retain `BLK-STEM-CROP-001`.

During old-chunk `UpgradeData` shape repair, a mature age-seven Pumpkin Stem
whose supplied horizontal neighbor is exact Pumpkin becomes default
Attached Pumpkin Stem facing that supplied direction. Other ages, other
neighbors and Melon stems return unchanged. This repair updates the stem,
not the Pumpkin.

### Natural Pumpkin patch

Configured feature `pumpkin` is `simple_block` with a simple provider for
state `8332`. Its placed wrapper runs these modifiers in exact order:

1. rarity `300`;
2. in-square X/Z;
3. `MOTION_BLOCKING` heightmap;
4. biome filter;
5. count `96`;
6. trapezoid random offset X/Z `-7..7`, Y `-3..3`; and
7. an all-of predicate requiring the target in block tag `air` and exact
   Grass Block one below.

Each surviving candidate calls the provider, passes Pumpkin's unconditional
survival check, offers the state with flags `2`, ignores write failure and
returns true through the generic simple-block transaction. A failed rarity,
biome or placement predicate makes no offer.

Exactly 46 biome records schedule `patch_pumpkin`:

- Badlands, Eroded Badlands and Wooded Badlands;
- Bamboo Jungle, Jungle and Sparse Jungle;
- Birch Forest, Old Growth Birch Forest, Dark Forest, Flower Forest,
  Forest and Pale Garden;
- Old Growth Pine Taiga, Old Growth Spruce Taiga, Snowy Taiga and Taiga;
- Plains, Sunflower Plains, Savanna, Savanna Plateau and Windswept Savanna;
- Windswept Forest, Windswept Gravelly Hills, Windswept Hills, Grove,
  Snowy Slopes and Ice Spikes;
- Beach, Snowy Beach, Stony Shore, Desert, Snowy Plains, Swamp, River and
  Frozen River;
- Ocean, Cold Ocean, Frozen Ocean, Lukewarm Ocean and Warm Ocean;
- Deep Ocean, Deep Cold Ocean, Deep Frozen Ocean and Deep Lukewarm Ocean;
  and
- Deep Dark and Dripstone Caves.

Biome-generation step/order and modifier algorithms retain
`WGEN-PIPELINE-001`; the record list does not guarantee a patch or write in
every generated chunk.

### Taiga-village Pumpkin piles

Configured `pile_pumpkin` uses the audited block-pile algorithm with a
weighted provider: Pumpkin weight `19`, north-facing Jack o'Lantern weight
`1`, total `20`. Its placed wrapper has no modifiers.

The feature is a rigid feature-pool element of weight `2` in ordinary Taiga
village decor, total weight `39`, and weight `2` in zombie Taiga decor,
total weight `26`. After feature selection, block-pile minimum-height,
radii `2..3`, radial admission, empty/support tests, provider draws and
flags-`260` ignored writes retain `WGEN-PIPELINE-001`. Each admitted
provider call selects Pumpkin with probability `19/20`; a chosen pile can
still write none.

### Structure-template census

An exhaustive decoded scan of all `1,212` bundled templates finds exactly
41 raw state-`8332` cells in six files, with no target-cell block NBT:

- Pillager Outpost `feature_tent2` has `4`;
- Taiga-village `houses/taiga_large_farm_1` has `7`,
  `taiga_large_farm_2` has `5`, and `taiga_small_farm_1` has `4`;
- zombie Taiga `houses/taiga_large_farm_2` has `5`; and
- Woodland Mansion `1x2_a8` has `16`.

Outpost `feature_tent2` is a weight-one empty-processor rigid element in
the 13-weight features pool, whose Empty entry has weight `6`.

The ordinary Taiga houses pool totals `76`: large farms `1/2` each have
weight `6`, and small farm `1` has weight `1`. Its large farms use
`farm_taiga`, whose Wheat rules do not match raw Pumpkin; the small farm's
`mossify_10_percent` likewise does not. The zombie houses pool totals `74`:
normal large farm `1`, zombie large farm `2` and normal small farm `1`
have weights `6/6/1`. `zombie_taiga` has no Pumpkin predicate or output, so
raw Pumpkin cells pass unchanged when their element survives all other
processing.

Woodland Mansion room selection can choose `1x2_a8`; its structure-template
transaction ignores structure blocks but has no Pumpkin-specific processor.
Exact decompressed-string scanning finds only the six palette names: no
extra Jigsaw `final_state`, block/entity NBT or marker names exact Pumpkin.
Pool/room reachability, shuffling, transforms, processors, clipping,
placement and partial failure retain the Outpost, Village and Mansion
owners; raw cells are not guaranteed final-world writes.

### Persistence and contextual legacy migration

Ordinary chunk palettes and block-update packets preserve only state
`8332`; stacks preserve identity, count and generic component patches.
There is no direct pre-flattening numeric block/item state that maps
unconditionally to modern uncarved Pumpkin:

- old numeric block `86` states `1376..1379` and old
  `minecraft:pumpkin` facing aliases flatten to Carved Pumpkin; remaining
  metadata uses that old block's Carved-Pumpkin default;
- numeric item `86` first names `minecraft:pumpkin`, but
  `ItemStackTheFlatteningFix` maps damage-qualified
  `minecraft:pumpkin.0` to Carved Pumpkin.

Two contextual fixes deliberately create modern Pumpkin. During paletted
chunk conversion, a flagged Carved Pumpkin with exact Grass Block or Dirt
immediately below is rewritten property-free to Pumpkin, preserving
naturally generated old pumpkins while leaving other carved heads. Separately,
`VillagerTradeFix` visits `buy`, `buyB` and `sell` stacks and rewrites exact
Carved-Pumpkin item identities to Pumpkin, preserving historical trades
after the identity split.

`SavedDataFeaturePoolElementFix` also recognizes the old Pumpkin/Jack
weighted pile and reconstructs `pile_pumpkin`; later chunk shape upgrade
performs the mature-stem repair described above. Complete data-fix search
finds no other Pumpkin-specific migration.

### Client projection

The sole blockstate variant selects `minecraft:block/pumpkin`. Its
`cube_column` model maps the four sides to `block/pumpkin_side` and up/down
to `block/pumpkin_top`; both are static, untinted, fully opaque 16×16
textures. The item definition points directly to that block model.

Its English name is `Pumpkin`. Natural Blocks publishes it exactly once,
after Melon and before Carved Pumpkin, in local order Melon, Pumpkin,
Carved Pumpkin, Jack o'Lantern, Hay Block. It appears in no other baseline
creative tab.

**Branches and aborts:**

- Placement is unconditional, but neighboring fences/panes/walls explicitly
  reject sturdy-face connection to Pumpkin.
- Exact Shears select the client-success/server-transaction carving branch;
  every output/write failure preserves later side effects.
- Every tool reaches self loot; only explosion survival can suppress it.
- Recipes consume Pumpkin but do not produce it; their unlock alternatives
  are not identical.
- Composter, shipwreck, Farmer and Wandering-Trader paths have independent
  probability/economy gates.
- Live tag reload can change tool, Enderman and Sulfur-Cube selection
  without mutating an existing state or stack.
- Stem, patch, pile and structure gates can reject every offered Pumpkin.
- Legacy numeric inputs normally become Carved Pumpkin; only contextual
  chunk/trade fixes create uncarved Pumpkin.

**Constants and randomness:**

State/block/item IDs `8332/360/384`; strength `1/1`; friction `0.6`;
sound IDs break/fall/hit/place/step/carve
`1853/1854/1855/1856/1857/1348`; stack `64`; carving seeds/doubles
`4/2`; tag closures `3/2`; Composter `0.65f`; shipwreck rolls/weight/count
`3..10`, `2/84`, `1..3`; Farmer selection/input/output/uses/XP
`2 of 3`, `6/1`, `12/10`; trader selection/input/output/uses
`5 of 76`, `1/1`, `4`; fast-flat constants as listed; patch rarity/count/
offsets/biomes `300/96/±7,±3/46`; pile provider `19/20`, decor weights
`2/39` and `2/26`; structure files/cells `6/41`.

**Side effects:**

Placement, connection shapes, mining/loot/piston destruction, seed entities,
sound, Carved-Pumpkin write, Shears damage, game event/stat and possible
golem assembly; crafting/results/knowledge; Composter, chest and merchant
state; Enderman relocation and Sulfur-Cube equipment/dispenser consumption;
stem/feature/structure writes; chunk/stack persistence and migration; exact
model, textures, name and creative projection.

**Gates:**

World-write and break authority; neighbor identity/face; exact Shears and
logical side; block-interact loot and Carved write; explosion survival;
recipe/grid/knowledge; Composter level/draw; chest rolls and merchant set/
profession/economy; live block/item tags; Enderman game rule/AI and
Sulfur-Cube age/equipment; stem brightness/growth/direction/target/support;
patch/pile modifiers/providers/RNG; structure reachability/transform/
processor/clip/write; registry, reload, migration and client-resource
validity.

**Boundary cases and quirks:**

Pumpkin is a full sturdy cube but an explicit connection exception. Its
note instrument is Didgeridoo, unlike Carved Pumpkin's Harp registration.
Carving commits seed drops and later tool/event/stat effects even when the
state write fails; a successful write can immediately assemble a golem.
The Farmer buys six Pumpkins, whereas the Wandering Trader sells one.
Old numeric Pumpkins ordinarily become Carved Pumpkins; terrain and trade
context are what recover the modern uncarved identity.

**Failure semantics:**

Generic placement, break, crafting, loot, trade, Composter, equipment,
feature and structure transactions retain their owners' commit semantics.
The carving callback explicitly ignores loot-output entity insertion and
Carved-state write results. Stem fruit writes do not roll each other back.
Feature and structure writes can partially commit. Reload changes future
reads only; migration applies only while its owning fix is active.

**Client/server authority split:**

The server owns connection state, placement, carving, break/loot, recipes/
knowledge, composting, chest/trades, tags/mobs/equipment, stem/worldgen/
structures and persistence/migration. The client predicts ordinary
placement and returns immediate carving success, then consumes synchronized
state, entity, sound, inventory, offer and statistic outputs and renders
the model, textures, name and tab entry.

**Observability:**

Observe state/block/item/sound IDs; every shape/support/light/path/redstone/
connection read; carving face, two-double cursor, seed entity, ignored
writes and side-effect order; mining/loot/explosion; recipes/knowledge,
Composter/chest/trade results; complete tag closures/consumers; every stem/
feature/structure read, draw and write; exact 41-cell census; contextual
legacy conversion, durable/wire identity and exact client projection.

**Persistence and reload:**

Pumpkin saves one property-free identity and has no block entity. Its stack
uses generic components. Tags, loot, recipes, advancements, trades,
worldgen and client resources have independent reload boundaries.
Registration, connection exception, carving, Composter entry, merchant
defaults, chunk/stem repair, contextual data fixes and creative ordering
are code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.PumpkinBlock`;
`net.minecraft.world.level.block.Block#isExceptionForConnection`;
`net.minecraft.world.level.block.ComposterBlock`;
`net.minecraft.world.level.chunk.UpgradeData$BlockFixers$5`;
`net.minecraft.util.datafix.fixes.BlockStateData`;
`net.minecraft.util.datafix.fixes.ItemIdFix`;
`net.minecraft.util.datafix.fixes.ItemStackTheFlatteningFix`;
`net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix$UpgradeChunk`;
`net.minecraft.util.datafix.fixes.VillagerTradeFix`;
`net.minecraft.util.datafix.fixes.SavedDataFeaturePoolElementFix`;
`net.minecraft.world.entity.SulfurCubeArchetypes`;
`net.minecraft.world.item.trading.VillagerTrades`;
`net.minecraft.world.item.CreativeModeTabs`; block/item/sound/component
reports; block/carve/chest loot, two recipes/advancements, both merchant
records/tags/sets; complete block/item tags and fast-flat archetype;
configured/placed patch and pile, all 46 biome and four pool records,
processors; all `1,212` decoded templates and decompressed strings; exact
blockstate/model/item/texture/language resources. Complete compiled
exact-field, data, legacy-fix and decoded-NBT searches find no other
identity-specific runtime path.

**Test vectors:**

Run `EXP-BLK-119` across state/registry identity, every placement/shape/
support/path/redstone/connection/tool/explosion branch, client/server
Shears carving with every face and rejected output/write, both recipes/
unlocks, all Composter/chest/Farmer/trader boundaries, complete `3/2` tag
closures and Enderman/fast-flat equipment/dispenser consumers, stem and
old-chunk repair, all 46 patch schedules, both pile pools, all 41 raw
template cells, every legacy/contextual fix, persistence/reload and exact
client projection. Assert IDs, ordering, constants, absences, census and
vanilla convergence.

**Limits:**

Generic block lifecycle/mining, fence/pane/wall connection, loot,
crafting/progression, Composter, merchant, Enderman, Sulfur-Cube, stem,
feature/Jigsaw/Mansion, packet and rendering algorithms retain their named
owners. Carved Pumpkin, Pumpkin Seeds, Pumpkin Pie, Jack o'Lantern and all
structures retain their catalog families. This leaf fixes exact uncarved
Pumpkin and every direct join that selects it.
