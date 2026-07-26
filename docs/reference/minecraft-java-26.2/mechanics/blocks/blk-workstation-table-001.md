# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-WORKSTATION-TABLE-001` — Crafting Table opens 3x3 crafting while Fletching Table remains a POI-only workstation

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`, `PLY-002`, `PLY-005`,
`PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-CONTAINER-001`, `ITM-CONTAINER-MOVE-001`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-FURNACE-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`,
`MOB-001`, `MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FLUID-001`, `ENV-FIRE-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-VILLAGES-001`,
`WGEN-JIGSAW-OUTPOST-001`, `WGEN-JIGSAW-TRAIL-RUINS-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `WGEN-STRUCTURE-IGLOO-001`,
`WGEN-STRUCTURE-SWAMP-HUT-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations distinguish one
`CraftingTableBlock` from one ordinary `Block` despite identical physical
properties. Exact menu/stat, POI/profession, recipe, fuel, loot, world-source
and client-resource sweeps close every identity-specific branch. Exhaustive
decoded-NBT and constant-pool scans of all 1,212 templates find exactly 29
Crafting Table cells and six Fletching Table cells, with no block entity data
or hidden stack reference.

**Applies when:**

`minecraft:crafting_table` or `minecraft:fletching_table` is placed, used,
mined, exploded, burned as fuel, selected as a POI or profession source,
crafted, generated, persisted, synchronized or rendered.

**Authoritative state:**

Both blocks have one property-free state and no block entity:

| Identity | Block ID | State ID | Item ID | Registered implementation |
| --- | ---: | ---: | ---: | --- |
| Crafting Table | `206` | `5310` | `360` | `CraftingTableBlock` |
| Fletching Table | `843` | `20771` | `1389` | ordinary `Block` |

Both register wood map color, Bass note instrument, strength/resistance
`2.5/2.5`, `SoundType.WOOD` and lava ignition. They are opaque full cubes
with friction `0.6`, speed/jump factors `1`, light `0`, normal piston reaction,
ordinary survival and no scheduled or random tick. Neither carries a
direction property: placement, rotation and mirroring preserve the sole state.

Wood sound volume/pitch is `1/1`; break/step/place/hit/fall sound-event IDs
are `1853/1857/1856/1855/1854`.

Both blocks directly belong only to `mineable/axe`. The tag changes mining
speed but neither registration requires a correct tool, so hands and every
tool remain eligible for player harvest.

Each item is a common stack-64 `BlockItem` with empty attribute/enchantment/
lore defaults and only its name and item-model identities. Neither item
belongs to any direct item tag, including a Sulfur Cube archetype.

**Transition and ordering:**

Placement, state updates, breaking and ordinary stack use first retain their
generic owners. The sole block-local transition is Crafting Table's
item-empty interaction override. Fletching Table has no interaction override;
its independent POI registration is consumed asynchronously by Villager AI.

### Crafting Table interaction and menu

After held-item interaction has fallen through to `useWithoutItem`, Crafting
Table returns `SUCCESS` on both sides. The client performs no local open. On
the server it constructs a `SimpleMenuProvider` titled
`container.crafting` (`Crafting`), invokes `ServerPlayer.openMenu`, discards
the returned container ID, then awards
`minecraft:interact_with_crafting_table` custom stat ID `58`.

The provider always creates a `CraftingMenu` using menu type
`minecraft:crafting`, protocol ID `12`, the new container ID, the player's
inventory and access bound to the interacted level/position. Opening first
closes any non-inventory menu, advances the player's container counter, sends
the open-screen packet with that type/title, initializes synchronization and
installs the menu. The stat award remains after that call.

The server menu has 46 slots:

- result slot `0` at `(124,35)`;
- 3x3 input slots `1..9`, laid out from `(30,17)`;
- player main inventory slots `10..36` and hotbar slots `37..45`, laid out
  from `(8,84)`.

It uses the `CRAFTING` recipe-book type. Input changes outside recipe-book
placement recompute slot `0` through `ITM-RECIPE-001`/`ITM-CRAFT-001`; recipe
placement suppresses intermediate recomputation and explicitly finishes with
the selected holder. Quick-moving result slot `0` targets `10..45` in reverse
order and triggers crafted callbacks; player inventory first targets `1..9`,
then swaps main inventory and hotbar ranges. Input/result take, limited
crafting, remainders and recipe knowledge retain their named generic owners.

Server validity requires the live block at the bound position still be exact
Crafting Table and strict eye-to-block-AABB squared distance below
`(player.blockInteractionRange()+4.0)^2`. Closing returns every remaining
input stack through generic clear-container inventory/drop handling; the
computed result is not an independently stored item. Pick-all explicitly
rejects the result container.

Fletching Table instead inherits ordinary `Block.useWithoutItem`, hence
returns `PASS`, opens no menu, awards no interaction stat and has no
Fletching-specific screen or packet.

### Fletching Table POI and profession

`PoiTypes.bootstrap` registers the sole Fletching Table state as
`minecraft:fletcher` point-of-interest type ID `6`, maximum tickets `1` and
valid range `1`. Ordinary section/POI update ownership observes placement and
removal of that exact state.

Villager profession `minecraft:fletcher`, ID `7`, uses exact Fletcher POI
membership for both its held-job-site and acquirable-job-site predicates. It
has empty requested-item and secondary-site block sets, work sound
`minecraft:entity.villager.work_fletcher` ID `1708`, and maps levels `1..5`
to `minecraft:fletcher/level_1` through `level_5` trade sets.

Job claiming, profession assignment/loss, memory, navigation, work cadence,
sound playback, trade materialization and restocking remain generic Villager
owners. This block supplies the exact POI state and profession mapping; using
the block by hand is not a prerequisite and produces no UI.

**Breaking, fire and fuel:**

Each block loot table has one roll and one exact count-one self item guarded
only by `survives_explosion`, with random sequence
`minecraft:blocks/<identity>`. There is no Silk Touch, Fortune, state,
correct-tool or alternate drop branch.

Both registrations set `ignitedByLava`, so `LavaFluid` may treat either state
as a nearby ignition source through `ENV-FLUID-001`. Neither appears in
`FireBlock.bootStrap`; direct ordinary-fire encouragement/flammability odds
are therefore `0/0`.

`FuelValues.vanillaBurnTimes` explicitly adds both block items at
`base*3/2`; with vanilla base `200`, each burns for exactly `300` furnace
ticks. Neither has a Composter entry.

**Crafting and progression joins:**

- Crafting Table uses a 2x2 square of live `#minecraft:planks` and yields one
  default Crafting Table. It fits the player's 2x2 inventory grid and sets
  `show_notification=false`.
- Fletching Table uses two exact Flint in its top row and two full rows of two
  live `#minecraft:planks`, yielding one default table. Its three-row pattern
  requires a 3x3 crafting grid.
- Crafter uses exact Crafting Table in the center of
  `### / #C# / RDR`, surrounded by seven Iron Ingots with Redstone/Dropper in
  the bottom row as encoded, and yields one default Crafter. Crafting Table is
  not an unlock criterion for that recipe; Dropper possession is.

The Crafting Table recipe advancement accepts either existing recipe
knowledge or the unconditional `tick` criterion, unlocks exactly that recipe
and therefore grants it immediately under ordinary advancement processing.
Possessing a Crafting Table separately completes the displayed/no-toast/
no-announcement Story root and sends its telemetry event.

The Fletching Table recipe advancement accepts either existing knowledge or
possession of exact Flint and rewards only that recipe. No other bundled
recipe or advancement directly consumes, produces or tests Fletching Table.

**World sources:**

`WGEN-STRUCTURE-SWAMP-HUT-001` directly offers one default Crafting Table at
local `(3,2,6)` in every retained Swamp Hut piece, after its potted mushroom
and before its Cauldron. This code-built cell is separate from the structure
template census.

All 29 raw Crafting Table template cells are one-cell, property-free palette
entries in 29 files:

- `igloo/top`;
- `pillager_outpost/feature_tent{1,2}`;
- Trail Ruins `buildings/group_upper_{2,3}`,
  `tower/hall_{1,2}` and `tower/large_hall_{1,3,4}`;
- Trial Chambers `intersection/intersection_2`;
- Desert Village `houses/desert_small_house_{5,8}` and both Zombie
  counterparts;
- Savanna Village `houses/savanna_{medium_house_1,medium_house_2,
  small_house_3}` and all three Zombie counterparts;
- Taiga Village `houses/taiga_{medium_house_2,medium_house_3,
  small_house_3,small_house_4}` and all four Zombie counterparts.

All six raw Fletching Table cells are one-cell, property-free palette entries
in:

- `village/desert/houses/desert_fletcher_house_1`;
- normal and Zombie
  `village/plains/**/plains_fletcher_house_1`;
- `village/savanna/houses/savanna_fletcher_house_1`;
- `village/snowy/houses/snowy_fletcher_house_1`;
- `village/taiga/houses/taiga_fletcher_house_1`.

No target cell has block entity data. For each identity, decompressed UTF
occurrences equal its cell count, so there is no hidden item, entity or
storage reference. No configured feature, placed feature, processor list,
loot table or other worldgen record directly names either table. Igloo,
jigsaw, Trail Ruins and Swamp Hut owners retain candidate selection,
processors, transforms, clipping, terrain adaptation, write failure and final
reachability.

**Client projection:**

Each empty blockstate variant selects its same-named fixed `block/cube` model;
the item definition selects that block model directly. Because neither block
has direction state, placement cannot rotate these face assignments:

- Crafting Table uses Oak Planks below, its top texture above, front texture
  north/west and side texture east/south; its particle is the front texture.
- Fletching Table uses Birch Planks below, its top texture above, front
  texture north/south and side texture east/west; its particle is the front
  texture.

All dedicated textures are static 16x16 PNGs without animation metadata.
There is no tint, random variant, multipart state, special renderer or flat
item texture. Names are `Crafting Table` and `Fletching Table`.

The Functional Blocks tab places Crafting Table between Magma Block and
Stonecutter, and Fletching Table between Cartography Table and Smithing Table.
Menu type `12` selects `CraftingScreen`, its crafting recipe-book component
and `textures/gui/container/crafting_table.png`; the title label X is `29`
and recipe-book button offset is `(left+5,height/2-49)`. Fletching Table has no
corresponding menu screen.

**Branches and aborts:**

- Held-item handling can consume interaction before the block-empty path.
- Crafting Table returns `SUCCESS` locally but the server alone opens and
  awards the stat.
- Menu validity fails after block replacement or range failure; close handling
  returns input contents.
- Fletching Table always falls through its block-empty interaction.
- POI claim, release, profession and work branches remain independently
  gated by generic Villager state and ticket ownership.
- Hand or any tool reaches loot; explosion survival can suppress the self
  item.
- Template/piece selection, clipping, processing and write failure can prevent
  any raw world-source cell from becoming final terrain.

**Constants and randomness:**

IDs/states as tabulated; strength/resistance `2.5/2.5`; stack `64`; fuel
`300`; menu/stat IDs `12/58`; POI/profession/work-sound IDs `6/7/1708`;
POI tickets/range `1/1`; 46 menu slots; menu-validity range padding `4.0`;
three direct recipes; 29/6 raw template cells plus one code-built Swamp Hut
Crafting Table. Identity-local block/menu/POI dispatch consumes no RNG.

**Side effects:**

Ordinary placement/breaking and item persistence; menu close/open, container
counter, open-screen and slot synchronization, crafting inputs/results,
recipe/stat/advancement state; POI section/ticket and Villager
memory/profession/work/trade state; furnace fuel timers; structure/template
writes; client screen, model and sound projection.

**Gates:**

Generic interaction ordering; server/client side; exact live Crafting Table
and range for menu validity; recipe input/knowledge/result capacity; exact
Fletching state, POI ticket and Villager job admission; explosion survival;
lava random-fire admission; furnace fuel slot; structure/template selection,
transform, clip, processor and write.

**Boundary cases and quirks:**

Equal physical registration does not imply equal use behavior: one exact
subclass opens a 3x3 menu, while the ordinary table's only special behavior
comes from a registry join outside the block class. Both fixed-direction
textures expose orientation despite having no direction state. The Crafting
Table recipe unlocks from a tick criterion, while possession advances Story
root independently. Lava ignition, zero direct Fire odds and positive furnace
fuel are three distinct mechanisms.

**Failure semantics:**

Failed or preempted interaction opens no new menu. Replacing/ranging away from
a Crafting Table invalidates the server menu; close returns input items rather
than preserving them in the block, which has no entity. Failed recipe, loot,
POI, profession, fuel or world-write gates commit no identity-specific output.
Generic structure placement may retain earlier writes when a later table
write fails; no rollback is added here.

**Client/server authority split:**

The server owns use dispatch, stat, menu and crafting state, POI/profession,
loot/fuel and worldgen. The client returns predicted interaction success,
projects synchronized sole states/stacks, and opens Crafting Screen only from
the authoritative menu packet.

**Observability:**

Observe registry/state IDs, physical values, fixed faces, mining/loot/decay,
lava fire and fuel duration; interaction result, menu type/title/container
ID/slots/validity/close, stat and recipes/advancements; POI tickets,
profession/memories/work sound/trade-set selection; the direct hut cell,
template identities/positions/transforms/final writes; names, models, textures,
tab order and Crafting Screen.

**Persistence and reload:**

Placed blocks persist only identity, with no properties or entity payload.
Stacks use generic item components. An open crafting grid is transient and is
returned on close rather than saved in the block. POI section data and
Villager memories/profession use their generic persistence owners. Tags,
recipes, advancements, trade sets, structures and client resources retain
their reload boundaries; registrations, menu/stat IDs and physical profiles
are code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.CraftingTableBlock#useWithoutItem`;
`net.minecraft.world.level.block.CraftingTableBlock#getMenuProvider`;
`net.minecraft.server.level.ServerPlayer#openMenu`;
`net.minecraft.world.inventory.CraftingMenu`;
`net.minecraft.world.inventory.AbstractContainerMenu#stillValid`;
`net.minecraft.stats.Stats`;
`net.minecraft.world.entity.ai.village.poi.PoiTypes#bootstrap`;
`net.minecraft.world.entity.npc.villager.VillagerProfession#bootstrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.level.material.LavaFluid#randomTick`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece#postProcess`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`net.minecraft.client.gui.screens.MenuScreens`;
`net.minecraft.client.gui.screens.inventory.CraftingScreen`;
block/item/menu/stat/POI/profession/sound reports; item-component reports;
Axe and all item tags; both loot tables; Crafting Table, Fletching
Table and Crafter recipes plus their advancement joins; five Fletcher trade
set keys; all worldgen JSON and 1,212 structures; blockstates, models, item
definitions, textures, GUI texture and language resources. Complete compiled
exact-field and data-reference searches found no other runtime path.

**Test vectors:**

Run `EXP-BLK-112` over both blocks under placement/transform/tool/explosion/
lava/fuel paths; every held-item/client/server/use/menu/stat/slot/recipe/book/
validity/close boundary; all POI ticket/profession/memory/work/trade branches;
Swamp Hut and all 35 template cells under every transform/processor/write
outcome; persistence/reload and exact block/item/screen projection. Assert
IDs, order, constants, absences, census and vanilla convergence.

**Limits:**

Generic interaction, menu synchronization, crafting, recipe, loot, fuel,
fluid-fire, POI/Villager AI, structure/jigsaw/template and rendering kernels
remain with their named owners. Planks, Flint, Crafter, Fletcher trades and
the structures themselves retain their existing catalog families. This leaf
fixes the two table identities and every exact join that selects them.
