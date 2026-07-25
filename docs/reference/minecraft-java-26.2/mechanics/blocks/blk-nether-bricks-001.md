# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-BRICKS-001` — Nether Brick joins smelting and bartering to correct-tool masonry, fortress terrain and protected spawn floors

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`,
`BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`, `PLY-005`,
`PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`, `ITM-001`,
`ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-FURNACE-001`, `ITM-STONECUTTER-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `BLK-RED-NETHER-BRICKS-001`,
`ENT-001`, `ENT-KNOCKBACK-001`, `MOB-001`, `MOB-004`, `MOB-AI-001`,
`MOB-SPAWN-001`, `ENV-001`, `ENV-002`, `ENV-003`, `ENV-FIRE-001`,
`WGEN-PIPELINE-001`, `WGEN-STRUCTURE-FORTRESS-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations/components, three block reports and tables, the complete
thirteen-recipe transform graph and twelve family unlocks, Piglin barter table, hard-coded
fortress/spawn/Delta/Basalt-Columns consumers, all 1,212 structure templates and exact resources
determine every Nether-Brick and three full-block-family branch. Generic processing, masonry
shapes, loot, bartering, spawning, structure, feature, stack and rendering algorithms remain with
the cited owners.

**Applies when:**

A `minecraft:nether_brick` item is smelted, bartered, crafted, moved, renamed, persisted,
synchronized or rendered; or `minecraft:nether_bricks`,
`minecraft:cracked_nether_bricks` or `minecraft:chiseled_nether_bricks` is placed, mined,
exploded, processed, stonecut, equipped on a Sulfur Cube, generated or tested by fortress spawn,
Delta or Basalt-Columns code, persisted or rendered before and after loot, recipe, advancement,
tag, barter, worldgen or resource reload.

**Authoritative state:**

The three property-free ordinary full blocks are:

| Identity | Block ID | Item ID | Sole/default state |
|---|---:|---:|---:|
| Nether Bricks | `381` | `452` | `9334` |
| Cracked Nether Bricks | `942` | `453` | `23094` |
| Chiseled Nether Bricks | `941` | `454` | `23093` |

Each independently registers map color `NETHER`, note instrument `BASEDRUM`,
hardness/resistance `2/6`, `NETHER_BRICKS` sounds and
`requiresCorrectToolForDrops`. Each has no block entity, property, random/scheduled tick,
identity-specific use/contact/signal/comparator hook, fire odds or lava-ignitable property.

Every state is a full selection/collision/visual/occlusion cube with emission zero, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution zero,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. The sole direct block tag for each is `mineable/pickaxe`; no tool-tier tag contains them,
so any pickaxe satisfies the correct-tool gate.

All three common nondamageable block items stack to `64`, use ordinary components and are direct
`sulfur_cube_archetype/slow_bouncy` members. `minecraft:nether_brick` is instead raw item ID
`1275`, a common nondamageable plain `Item`, maximum stack `64`, ordinary components and no direct
tag. It has no food, consumable, remainder, fuel, compost, equipment, durability, repair,
projectile, cooldown, inventory-tick or identity-specific use branch.

**Transition and ordering:**

### Loose Nether Brick acquisition

Recipe `nether_brick` is Furnace smelting only: one exact Netherrack enters the ordinary
category-less record, uses omitted/default cooking time `200`, awards recipe XP `0.1` and emits
one default Nether Brick. Blast Furnace, Smoker and Campfire reject the record. Input component
patches are ignored for exact ingredient matching and are not copied. Machine progress, result
capacity, recipe-use accounting and extraction XP remain `ITM-FURNACE-001`.

Its no-display recipe advancement has one OR requirement containing exact Netherrack possession
and prior `nether_brick` recipe unlock; either criterion grants only that recipe. The widespread
Netherrack terrain/mining lifecycle is an upstream identity boundary, not a second Nether-Brick
algorithm.

Piglin bartering is the only locked path that directly emits loose Nether Brick without first
smelting. One completed Gold-Ingot barter rolls the total-weight-`469` table once. Nether Brick
has weight `40` (`40/469`) and, when selected, receives inclusive uniform count `2..8`. Payment,
admiration, delayed completion, loot invocation and thrown output remain with the Piglin and loot
owners.

No chest, archaeology, fishing, gift, entity-drop, villager, wandering-trader or structure
template directly stores or emits loose Nether Brick. It is not a Piglin-loved or barter-currency
item and has no furnace-fuel role.

### Block and derived masonry recipes

Four loose Nether Bricks in the shaped `2×2` recipe `nether_bricks` produce one default Nether
Bricks block item. There is no reverse/decompression recipe. Nether Brick also joins the
checkerboard `red_nether_bricks` recipe: two loose bricks plus two Nether Wart produce one Red
Nether Bricks. That separate block, its recipes and its Nether-root display remain
`BLK-RED-NETHER-BRICKS-001` and `BLK-NETHER-WART-001`.

The base block participates in ten further family recipes:

| Output | Grid path | Stonecutter path |
|---|---|---|
| Cracked Nether Bricks | Furnace: one base block to one, default `200`, XP `0.1` | none |
| Chiseled Nether Bricks | two Nether-Brick Slabs vertically to one | one base block to one |
| Nether-Brick Fence | `W#W/W#W`: four base blocks plus two loose bricks to six | none |
| Nether-Brick Slab | three base blocks in one row to six | one base block to two |
| Nether-Brick Stairs | six base blocks in stair shape to four | one base block to one |
| Nether-Brick Wall | six base blocks in two rows to six | one base block to one |

Together the family has twelve ordinary recipe advancements: the Netherrack smelt, four-brick
base block, Cracked smelt, two Chiseled paths, Fence, two Slab, two Stairs and two Wall records.
The smelt unlock listens for Netherrack; the base recipe listens for loose Nether Brick; the
shaped Chiseled record listens for Nether-Brick Slab; all other nine derived records listen for
base Nether Bricks. Each pairs its inventory criterion with its own prior recipe unlock in one OR
requirement and grants only that record.

The Red-Nether-Bricks record is a thirteenth recipe consuming this family, but its separate
advancement listens for Nether Wart rather than Nether Brick and belongs to the red/wart owners.
All exact item ingredients ignore component patches and all results are default stacks. Grid
mirroring/offset, machine and Stonecutter selection, atomic input/result handling and recipe-book
publication remain generic.

### Correct-tool loot and slow-bouncy equipment

Ordinary placement always selects the one state for that identity; rotation and mirror are
identity operations. Wrong-tool removal produces no block loot. Any pickaxe admits the
identity's one-roll table, which emits one matching default block item behind
`survives_explosion` and uses `minecraft:blocks/<identity>` as its named random sequence. Silk
Touch and Fortune add no branch. The three tables do not convert among base, cracked or chiseled.

Each block item selects the same locked `slow_bouncy` Sulfur-Cube archetype. It supplies
horizontal/vertical knockback powers `0.4125/0.24`, hit/push sounds, push cooldown `0.5`, impulse
threshold `0.05` and five modifiers: additive knockback/explosion-knockback resistance
`0.4000000059604645` each, additive bounciness `0.6000000238418579`, total-multiplied friction
`-0.699999988079071` and air drag `-0.949999999254942`. Loose Nether Brick does not match.

### Fortress construction, spawn floor and advancement display

The code-built Nether Fortress uses base state `9334`, never cracked or chiseled, throughout its
15 piece families. Piece geometry writes chunk-clipped floors, bridges, walls and downward
supports, alongside separately registered Nether-Brick Fence and Stairs. `fillColumnDown` writes
base Nether Bricks with flags `2` while the current state is air, liquid, Glow Lichen, Seagrass or
Tall Seagrass and world Y remains above `minY+1`. Exact graph, geometry, clipping and write order
remain `WGEN-STRUCTURE-FORTRESS-001`.

Base Nether Bricks also participates in a hard-coded natural-spawn gate. The helper returns false
unless category is exactly `MONSTER`; it then reads the block immediately below the candidate and
returns false unless that block identity is exact base Nether Bricks. Only then does it resolve
the registered Fortress structure and require a valid Fortress start at the candidate position.
Cracked, chiseled, Red Nether Bricks, Fence, Stairs and component/item state cannot substitute.
The admitted spawn-list replacement and remaining density/position/entity gates stay
`MOB-SPAWN-001` and the fortress owner.

`nether/find_fortress` uses a default base Nether Bricks item only as its display icon. Its sole
telemetry-enabled criterion is player location inside `minecraft:fortress`; inventory possession
is irrelevant.

### Delta and Basalt-Columns protection

Base Nether Bricks is hard-coded, rather than tagged, into two feature protection lists:

- Delta clarity rejects a candidate whose block is base Nether Bricks before its ordered
  neighbor-air checks, so neither rim Magma nor contents Lava is offered there;
- Basalt Columns rejects an origin/support/search cell whose block is base Nether Bricks before
  column placement.

Both lists also independently contain Nether-Brick Fence and Stairs. They do not contain Cracked,
Chiseled or Red Nether Bricks, so those states receive no identity-specific protection from these
features. Conditional draws, scans, writes and schedules remain `WGEN-PIPELINE-001`.

An exhaustive decode of all 1,212 locked structure NBT templates finds zero loose, base, cracked
or chiseled family strings/cells. Fortress acquisition is code-built and therefore is not an NBT
exception. No configured/placed-feature data directly produces any of the three full blocks.

**Persistence and reload boundary:**

Chunk palettes persist only the selected property-free state. There is no block entity, fortress
link, spawn eligibility cache, feature-protection bit or pending recipe transform. Stacks persist
identity, count and component patches; machine, knowledge, barter, Sulfur-Cube, structure and
feature state belong to those owners.

Loot/recipe/advancement/tag/barter reload changes future evaluation in its domain without
replaying completed mining, processing, crafting or barters. Fortress construction, exact spawn
floor checks and hard-coded Delta/Basalt-Columns lists change only with code/registry
reconstruction. Existing palettes and chunks are not rewritten. Resource reload independently
changes names, models and textures.

**Wire and client projection:**

Generic publication uses state IDs `9334/23094/23093` and item IDs `452/453/454/1275`; no
family-specific packet exists. English names are `Nether Bricks`, `Cracked Nether Bricks`,
`Chiseled Nether Bricks` and `Nether Brick`.

Each full block and its block item selects one opaque same-named `cube_all` model and texture.
Loose Nether Brick selects its same-named `item/generated` flat model and texture. No tint,
conditional model or special renderer applies. Nether-Bricks material sounds use break/step/
place/hit/fall IDs `1093/1094/1095/1096/1097`.

Building Blocks orders the relevant run Netherrack, Nether Bricks, Cracked Nether Bricks,
Nether-Brick Stairs, Slab, Wall, Fence, Chiseled Nether Bricks, then Red Nether Bricks. Ingredients
orders Bowl, Brick, Nether Brick, Resin Brick. The four identities in this leaf appear nowhere
else in ordinary tabs.

**Branches and aborts:**

Loose/block identity and components; smelting capacity/progress/extraction; barter selection/count;
thirteen recipe matches and twelve family unlocks; wrong/correct pickaxe and explosion survival;
three slow-bouncy selectors; base/cracked/chiseled state; fortress graph/write/spawn-floor/display;
Delta/Basalt protection divergence; zero templates; persistence/reload/wire/client paths are
distinct.

**Constants and randomness:**

Block/item/state IDs `381/452/9334`, `942/453/23094`, `941/454/23093`; loose ID `1275`; block
`2/6`, stack `64`, sounds `1093..1097`; cooking `200`, XP `0.1`; barter weight/total `40/469`,
count `2..8`; recipe counts as tabled; slow-bouncy `0.4125/0.24/0.5/0.05` plus five modifiers;
templates/cells `1212/0`.

**Side effects:**

Machine result/XP and recipe knowledge; Piglin loot output; block/derived crafting and
Stonecutter results; placement/break/self loot; Sulfur-Cube selection; fortress terrain and spawn
list; advancement display; feature write suppression; stack/chunk persistence, synchronization
and exact client projection.

**Gates:**

Exact ingredient/result capacity; Piglin payment/table selection; grid/Stonecutter/knowledge;
world-write/break/correct pickaxe/explosion; live archetype tag; Fortress structure and MONSTER/
below-state spawn test; Delta/Basalt candidate state; registry/chunk/stack decode and client
resources.

**State read/written:**

Reads all gates above and writes only the processing, knowledge, barter, block, loot, archetype,
fortress, spawn, feature-suppression, durable, wire and projection state listed above.

**Failure behavior:**

Wrong machine/input/capacity commits no transform. Unselected barter emits no brick. Wrong grid or
unavailable recipe emits no result. A nonpickaxe removes a block without self loot; failed
explosion survival suppresses it. Failed fortress/spawn/feature gates write or admit nothing.
Reload affects future evaluation only.

**Boundary cases and quirks:**

Four loose bricks compact to a base block with no decompression path. The Fence recipe uniquely
mixes four base blocks with two loose bricks. Chiseled has a slab-grid path and a direct
Stonecutter path; Cracked is Furnace-only. Any pickaxe is correct despite the correct-tool
property. Only base Nether Bricks protects Delta/Basalt cells and enables the Fortress spawn-floor
test; visually related cracked, chiseled and red identities do not.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.NaturalSpawner#isInNetherFortressBounds`;
`net.minecraft.world.level.levelgen.feature.DeltaFeature`;
`net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{nether_bricks,cracked_nether_bricks,chiseled_nether_bricks}`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,worldgen}`;
`reports/minecraft/components/item/{nether_brick,nether_bricks,cracked_nether_bricks,chiseled_nether_bricks}.json`;
`data/minecraft/loot_table/{blocks/{nether_bricks,cracked_nether_bricks,chiseled_nether_bricks},gameplay/piglin_bartering}.json`;
`data/minecraft/recipe/{nether_brick,nether_bricks,cracked_nether_bricks,chiseled_nether_bricks*,nether_brick_{fence,slab*,stairs*,wall*},red_nether_bricks}.json`;
`data/minecraft/advancement/{nether/find_fortress,recipes/{misc/nether_brick,building_blocks/{nether_bricks,cracked_nether_bricks,chiseled_nether_bricks*,nether_brick_{slab*,stairs*}},decorations/nether_brick_{fence,wall*}}}.json`;
`data/minecraft/tags/{block/mineable/pickaxe,item/sulfur_cube_archetype/slow_bouncy}.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{nether_bricks,cracked_nether_bricks,chiseled_nether_bricks}.json`;
`assets/minecraft/models/block/{nether_bricks,cracked_nether_bricks,chiseled_nether_bricks}.json`;
`assets/minecraft/items/{nether_brick,nether_bricks,cracked_nether_bricks,chiseled_nether_bricks}.json`;
`assets/minecraft/models/item/nether_brick.json`;
`WGEN-PIPELINE-001`; `WGEN-STRUCTURE-FORTRESS-001`; `BLK-RED-NETHER-BRICKS-001`;
`EXP-BLK-087`.

**Test vectors:**

Run `EXP-BLK-087` across loose/default/patched stacks, exact and malformed Netherrack smelting,
barter endpoints, all thirteen recipes and twelve family unlock alternatives. Place and break all
three states by hand and every pickaxe across explosion endpoints; select all three slow-bouncy
items and reject loose brick.

Generate every Fortress piece/write and test MONSTER/nonmonster candidates above base and every
lookalike inside/outside valid starts. Run Delta and Basalt Columns on every protected/nonprotected
identity, scan all 1,212 templates, persist/reload/synchronize all owners and assert IDs, names,
sounds, models, textures, icons and tab order.

**Limits:**

Generic machine, crafting, Stonecutter, advancement, loot, Piglin, block lifecycle, Sulfur-Cube,
natural-spawn, Fortress, Delta/Basalt-Columns, packet and renderer control flow remains with the
cited owners. This leaf fixes the exact loose item, three full blocks, transform graph, hard-coded
identity joins, absences and projection.
