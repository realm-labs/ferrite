# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-LAPIS-BLOCK-001` — Lapis block joins compacting, slow-bouncy selection and mansion decoration

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENT-005`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, reports, complete direct data-reference search,
sulfur-cube data and consumer, all 1,212 decoded structure templates, mansion selector and placement
code, and client assets exhaust the lapis-block identity's behavior and projection.

**Applies when:**

`minecraft:lapis_block` is placed, written, mined, exploded, compacted or decompressed, used below a
note block, selected as sulfur-cube equipment or food, emitted by the woodland-mansion room graph,
serialized or rendered.

**Authoritative state:**

Lapis block is a property-free ordinary `Block` with no block entity and sole state `565`.
Registration selects map color `LAPIS`, requires a correct tool for drops and sets destroy
speed/explosion resistance to `3.0/3.0`. It does not override the ordinary properties constructor,
so the state retains `HARP`, `STONE` sound, unit selection/collision/occlusion shapes, emission zero,
light dampening 15, friction `0.6`, speed/jump factors `1`, solid redstone conduction, normal piston
reaction, full sturdy faces, no random ticking and no block entity.

The direct block tags are exactly `mineable/pickaxe` and `needs_stone_tool`. The matching ordinary
block item is common, stacks to 64 and has no nondefault component beyond the standard block-item
identity/model/name set.

**Transition and ordering:**

#### Placement, breaking and projection

Ordinary placement or an authoritative component/template write commits state `565`. Rotation and
mirror are identity operations because the state has no property. The generic player break
transaction admits the matching self drop only with a correct stone-tier-or-better pickaxe. The
block loot table then performs one roll, returns one `lapis_block` behind `survives_explosion`, and
uses random sequence `minecraft:blocks/lapis_block`. An incorrect tool can remove the state but
does not reach the correct-tool self-drop path; an admitted explosion can suppress the one entry.

A powered note block above state `565` reads the default `HARP` instrument. Power/attack admission,
pitch, note-block state, block-event delivery and sound/particle projection remain with the
note-block and redstone owners.

The blockstate's sole variant selects `minecraft:block/lapis_block`; that `cube_all` model uses the
matching texture on every face. The item definition directly selects the same block model without
a condition or transform.

#### Compacting, decompression and exact-item boundaries

The reloadable processing graph names the block in exactly two recipes:

- shaped building-category recipe `lapis_block` consumes a full 3-by-3 grid of nine
  `lapis_lazuli` items and returns one block;
- shapeless recipe `lapis_lazuli` consumes one block and returns nine `lapis_lazuli`.

Neither recipe declares a group. Each matching advancement has one OR requirement containing its
own `recipe_unlocked` criterion and possession of the recipe input, then grants only its matching
recipe. Recipe grant and inventory discovery are therefore alternative unlock paths.

The output items become ordinary lapis-lazuli stacks after the decompression transaction. The
storage block itself is not a lapis-lazuli substitute: the enchantment-menu reagent slot and
quick-move branch compare the exact `Items.LAPIS_LAZULI`, while `trim_materials` names only the
individual item. Enchanting and trim application remain independently owned.

No other locked recipe, advancement, trade, non-block loot table or optional built-in-pack JSON
record names `minecraft:lapis_block`.

#### Slow-bouncy sulfur-cube selection

The block item is a direct `sulfur_cube_archetype/slow_bouncy` member. A sulfur cube that equips or
swallows the item can therefore select that archetype through `matchingArchetypes`. The locked
archetype supplies horizontal/vertical knockback powers `0.4125/0.24`, hit/push sounds
`minecraft:entity.sulfur_cube.slow_bouncy.{hit,push}`, push-sound cooldown `0.5` and impulse
threshold `0.05`.

Its five attribute modifiers add knockback and explosion-knockback resistance
`0.4000000059604645`, add bounciness `0.6000000238418579`, and apply multiplied-total friction
`-0.699999988079071` and air drag `-0.9499999992549419`. Multiple-match ordering, admission,
equipment replacement, component application, contact handling and effect projection remain with
the sulfur-cube and knockback owners.

#### Woodland-mansion generation

An exhaustive decompression scan of all 1,212 locked structure NBT files finds exactly one
lapis-block palette occurrence, and it is live rather than palette-only:
`woodland_mansion/2x2_a3.nbt`. That one-palette `15x8x15` template contains one state at local
position `(7,5,10)`, palette index `7`; it has no properties, block-entity NBT or template entity.

For an ordinary first-floor 2-by-2 room, `FirstFloorRoomCollection#get2x2` consumes
`nextInt(4)+1`; result `3` selects `2x2_a3`. The independently specified room graph must first
produce and admit that room. `addRoom2x2` then applies the selected doorway/clockwise-side offset,
mirror and quarter-turn plus the mansion rotation. Template placement transforms the local lapis
cell, writes it with the other explicit non-structure-block cells and leaves later clipping and
foundation behavior to the mansion owner.

The remaining 1,211 templates contain no lapis-block string, and no template contains an additional
palette-only or jigsaw `final_state` occurrence. The complete server-class constant-pool sweep finds
no current gameplay consumer beyond registration and the generic data-selected joins described
above; other hits are item/creative registration, data generators or legacy data-fix mappings.

**Client projection:**

Chunk and block-update paths publish exact state `565`. The client resolves one opaque matching
`cube_all` model and the item resolves the same model directly. Ordinary full-cube face culling,
stone break sounds/particles, map color and inventory rendering consume the selected state/assets.
Note, sulfur-cube and mansion effects use their existing generic packet/effect families; this leaf
adds no packet layout or connection state.

**Branches and aborts:**

Ordinary versus component/template placement; correct versus incorrect tool; survived versus
suppressed explosion loot; compacting versus decompression; recipe-unlocked versus inventory
discovery; block versus exact lapis-lazuli item gates; slow-bouncy match versus no match; mansion
graph omission versus an admitted first-floor 2-by-2 room; `nextInt(4)` values with only value `2`
selecting suffix `a3`; eight valid 2-by-2 transforms and clipping; server state versus block/item
projection are distinct.

**Constants and randomness:**

State `565`; strength `3.0/3.0`; emission `0`; dampening `15`; friction `0.6`; speed/jump `1`;
stack `64`; compression `9:1`; decompression `1:9`; slow-bouncy powers `0.4125/0.24`, cooldown
`0.5`, threshold `0.05` and five attribute amounts as listed; template size `15x8x15`, palette
index `7`, local position `(7,5,10)` and one live cell; room selector `nextInt(4)+1`. The block
itself consumes no RNG. Generic explosion/loot, sulfur-cube and mansion owners retain their
documented streams.

**Side effects:**

Generic state placement/removal; conditional matching self loot; two recipe outputs and two recipe
grants; slow-bouncy equipment/archetype state; one transformed mansion-template block write;
ordinary persistence, map shading and opaque block/item projection.

**Gates:**

Block versus item identity; placement/write authority; correct harvest tool; explosion context;
active recipe, advancement, loot, tag and archetype snapshots; exact lapis-lazuli comparison;
sulfur-cube admission/equipment state; mansion structure admission, first-floor ordinary 2-by-2
room existence, selector draw, transform and processing box; client state/model context.

**Boundary cases and quirks:**

Lapis block retains the default harp instrument rather than the bass drum used by many mineral
storage blocks. Its stone-tier harvest requirement and map color do not make it a beacon base.
Compacting does not let the block enter the exact enchantment reagent slot or lapis trim-material
tag. The mansion occurrence is one live decorative cell, not loot, a hidden block entity, a
palette artifact or an unconditional placement: the graph and a one-of-four ordinary first-floor
room draw must select `2x2_a3`.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties`;
`net.minecraft.world.inventory.EnchantmentMenu$3#mayPlace(net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.inventory.EnchantmentMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes(net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces$FirstFloorRoomCollection#get2x2(net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces$MansionPiecePlacer#addRoom2x2`;
`net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces$WoodlandMansionPiece`;
`net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#postProcess`;
`reports/blocks.json#minecraft:lapis_block`;
`reports/minecraft/components/item/lapis_block.json`;
`data/minecraft/tags/block/{mineable/pickaxe,needs_stone_tool}.json`;
`data/minecraft/tags/item/{sulfur_cube_archetype/slow_bouncy,trim_materials}.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/loot_table/blocks/lapis_block.json`;
`data/minecraft/recipe/{lapis_block,lapis_lazuli}.json`;
`data/minecraft/advancement/recipes/{building_blocks/lapis_block,misc/lapis_lazuli}.json`;
`data/minecraft/structure/**/*.nbt`;
`data/minecraft/structure/woodland_mansion/2x2_a3.nbt`;
`assets/minecraft/blockstates/lapis_block.json`;
`assets/minecraft/models/block/lapis_block.json`;
`assets/minecraft/items/lapis_block.json`.

**Test vectors:**

Run `EXP-BLK-050` across state `565` under ordinary/component/template writes; correct/incorrect
tools and explosion survival; both recipes, both OR-unlocks and data reload; note substrate;
enchantment-slot/quick-move and trim-material exclusion; slow-bouncy matching and all installed
values; every first-floor 2-by-2 selector draw, valid room transform and processing-box boundary;
save/reload and block/item models. Assert exact state, constants, tool/drop result, outputs,
item-identity rejection, archetype values, transformed sole mansion cell and projection. Re-run
the complete data/class/template reference sweeps and assert the documented absence boundaries.

**Limits:**

Generic placement/breaking, note-block runtime, recipes/advancements/loot, enchanting, trim
application, sulfur-cube runtime, woodland-mansion graph/placement and client rendering remain with
`BLK-PLACE-001`, `BLK-BREAK-001`, `ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`, `ITM-LOOT-001`,
`ITM-ENCHANT-001`, `ENT-KNOCKBACK-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001` and `CLI-006`.
Lapis ore, lapis-lazuli acquisition/use, beacon storage blocks and other mineral/storage identities
retain their separate owners or `Unreviewed` status.
