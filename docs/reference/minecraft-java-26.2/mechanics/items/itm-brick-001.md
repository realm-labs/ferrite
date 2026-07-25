# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BRICK-001` — Brick joins Clay-Ball smelting, masonry and pot recipes, archaeology, Mason selling and blank decorated-pot faces

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ENT-001`,
`BLK-BRUSHABLE-001`, `BLK-BRICKS-001`, `BLK-DECORATED-POT-001`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-TRAIL-RUINS-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, one smelting record, four crafting records,
four recipe advancements, two archaeology rows, one guaranteed Mason offer, the decorated-pot
ingredient/recovery mapping and direct client resources determine every Brick-specific branch.
Generic furnace, crafting, archaeology, merchant, decorated-pot, stack and client algorithms
remain with the cited owners.

**Applies when:**

A `brick` stack is smelted, brushed from archaeology, bought from a Mason, recovered from a
cracked decorated pot, used in a Bricks/Flower-Pot/decorated-pot recipe, moved, renamed, persisted,
synchronized or rendered before and after recipe, item-tag, loot, trade, advancement or resource
reload.

**Authoritative state:**

`minecraft:brick` is raw item ID `1054`. It is a common nondamageable plain `Item` with maximum
stack `64`. Its default components are the common empty modifiers/enchantments/lore, item-break
sound, translated name, direct item-model key, repair cost, swing animation, tooltip display and
use effects. It has no food, consumable, use remainder, cooldown, durability, equipment, tool,
projectile or other identity-specific hook.

Brick is the sole direct identity before nested `#minecraft:decorated_pot_sherds` in
`#minecraft:decorated_pot_ingredients`. It is not itself a pottery sherd and maps to the blank
decorated-pot face. It has no other direct item tag.

**Transition and ordering:**

Clay-Ball smelting:

The sole cooking record `minecraft:brick` is type `smelting`. One Clay Ball produces one default
Brick after the omitted/default `200` cooking ticks and records `0.3` recipe experience. It is
accepted by a Furnace, but its type does not match Blast Furnace, Smoker or Campfire recipe maps.
Input component patches do not propagate and there is no remainder.

The no-display `recipes/misc/brick` advancement has one OR requirement: exact Clay-Ball possession
or already knowing `minecraft:brick` grants that recipe. Brick possession does not unlock its own
smelting recipe. Furnace fuel admission, slot progress/reset, result stacking, recipe-used
accounting, player extraction, fractional XP and `recipe_crafted`/unlock work remain
`ITM-FURNACE-001` and `ITM-ADVANCEMENT-001`.

Crafting and recipe progression:

Brick is consumed by four locked crafting records:

- `bricks` is a shaped `2×2` square of exactly four Bricks and emits one default Bricks block.
  It fits the inventory grid or any admitted offset in a `3×3` grid.
- `flower_pot` is the two-row shape `# #` over ` # `, exactly three Bricks, and emits one default
  Flower Pot. Width three excludes the `2×2` inventory grid; vertical placement in `3×3` is
  otherwise generic.
- `decorated_pot_simple` is the exact `3×3` cardinal cross with Brick at top, left, right and
  bottom center. It emits one default Decorated Pot whose encoded component is four Brick
  identities, operationally `PotDecorations.EMPTY`.
- the special `decorated_pot` recipe also requires a `3×3` grid with exactly four nonempty
  cardinal-cross cells. Each independently accepts the live `decorated_pot_ingredients` tag.
  A Brick in top/left/right/bottom becomes an empty back/left/right/front face respectively,
  while a sherd becomes its named face. Mixed inputs therefore emit one component-patched pot.

All shapes reject extra, missing or misplaced nonempty inputs. The all-Brick cross resolves to the
simple shaped recipe and default all-blank result; the special recipe is a non-placeable,
notification-free custom recipe rather than recipe-book content. Every successful transaction
consumes one from each occupied input, copies no arbitrary Brick patches and leaves no remainder.

The Bricks and Flower-Pot advancements each use one OR requirement: exact Brick possession or the
corresponding known recipe grants that recipe. `decorated_pot_simple` instead accepts possession
of any live `decorated_pot_ingredients` member—including Brick or any current sherd—or knowledge
of only the simple recipe. Thus a sherd can unlock the Brick-only simple recipe.

`adventure/craft_decorated_pot_using_only_sherds` requires special recipe ID `decorated_pot` plus
four independent live `decorated_pot_sherds` ingredient predicates. Any Brick face makes that
criterion fail even though the mixed recipe succeeds. The display icon itself contains two blank
Brick faces but grants no semantic authority.

Archaeology acquisition:

Both archaeology tables make one roll and emit one default Brick when its entry is selected:

- `archaeology/desert_well` has eligible total weight `8`; Brick weight `1` gives probability
  `1/8` under random sequence `minecraft:archaeology/desert_well`;
- `archaeology/trail_ruins_common` has fourteen weight-`2` and seventeen weight-`1` entries,
  total `45`; Brick weight `2` gives probability `2/45` under sequence
  `minecraft:archaeology/trail_ruins_common`.

The admitted Desert-Well feature assigns its table and position-derived seed to two suspicious
Sand choices with replacement among five fixed well-floor positions. Trail-Ruins processors can
install at most six common suspicious Gravel blocks per house, two per road or two per tower top,
subject to their full processor/write gates. These are table opportunities, not guaranteed Brick
outputs.

Ten accepted brushing strokes, block/entity survival, table invocation, directional item
exposure, later pickup and failed-output behavior remain with `BLK-BRUSHABLE-001`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-TRAIL-RUINS-001` and `ITM-LOOT-001`.

Mason sale:

Mason level one has exactly two predicate-free records and selects amount `2` without duplicates,
so `mason/1/emerald_brick` is guaranteed once in every default fresh level-one set. It consumes
one Emerald matching an empty component predicate and gives ten default Bricks, with maximum uses
`16`, villager XP `1` and reputation discount `0.05`.

Arbitrary Emerald patches satisfy the unconstrained input predicate. There is no second cost,
predicate modifier, output modifier or double-price enchantment. Trade Rebalance does not replace
the record. Offer creation/economy, demand, reputation, restocking and menu commit remain
merchant-owned.

Decorated-pot blank-face and recovery mapping:

`PotDecorations` stores back/left/right/front as optional item identities. Decoding Brick in any
position converts it to an empty optional; encoding, tooltip substitution and ordered recovery
convert every empty optional back to Brick. The default decorated-pot item therefore encodes four
Bricks but shows no decoration tooltip and renders four blank side sprites.

Cracking a decorated pot selects the dynamic `minecraft:sherds` drop and emits four unit ingredient
stacks in stored back/left/right/front order. Every empty/Brick face emits one default Brick, so
the Brick result count is exactly the number of blank faces from `0..4`; named faces return their
sherds. The four outputs are separate unit stacks before generic world insertion. An uncracked
break instead emits one decorated-pot item preserving the face component and no Brick stacks.

Tool/enchantment/projectile cracking admission, content ejection, block loot, item-entity
creation, pot component persistence, tooltip and world/item rendering remain
`BLK-DECORATED-POT-001` and loot/client owners.

**Persistence and reload boundary:**

Brick stacks persist identity, count and arbitrary ordinary patches. They store no furnace
progress, recipe knowledge, archaeology cursor, merchant offer, pot face, crack state or
advancement progress; those values persist with their owners.

Recipe reload changes future smelting/crafting matching and output. Tag reload changes future
special-pot ingredients, simple-recipe possession unlocks and all-sherd criterion admission.
Loot reload changes future archaeology evaluation. Trade and advancement reload change future
offers/listeners. Existing offers, stacks, pots and completed work are not replayed or rewritten.
Resource reload independently controls name, model and texture.

**Client and wire projection:**

Generic stack encoding projects raw ID `1054` plus patches. The locked English name is `Brick`;
it is common with no forced glint or subtype tooltip. Its direct item definition selects the
ordinary generated `item/brick` model and same-named texture.

Brick appears exactly once and only in Ingredients, ordered Bowl, Brick, Nether Brick, Resin
Brick. It adds no packet layout; its appearance in pot components uses the generic item-registry
mapping and the pot's component/renderer owners.

**Branches and aborts:**

Identity/components/tag; smelting/unlock/extraction; Bricks, Flower-Pot, simple/special pot
crafting; all-sherd criterion; two archaeology tables and installation owners; Mason offer;
cracked/uncracked pot and `0..4` blank faces; persistence/reload/wire; name/model/tab.

**Constants and randomness:**

Raw ID `1054`; max `64`; smelting `1→1`, `200` ticks, XP `0.3`; crafting `4→1` Bricks,
`3→1` Flower Pot, `4→1` Decorated Pot; archaeology `1/8` and `2/45`, count `1`; Mason
`1→10`, uses/XP `16/1`; pot recovery `0..4` separate unit Bricks. Only furnace XP,
archaeology selection and their owners consume relevant randomness.

**Side effects:**

Furnace input/fuel/progress/result/recipe/XP; crafting inputs/results/knowledge/criteria;
archaeology cursor/brushable output; Mason offer/economy; cracked-pot dynamic drops; ordinary
stack persistence/wire and direct client projection.

**Gates:**

Exact smelting type/input and Furnace admission; exact grid/recipe/live tag; advancement
listeners; generated suspicious block/table and brush completion; level-one Mason/offer validity;
pot face/crack/drop admission; registry/decode; client language/model/tab bootstrap.

**State read/written:**

Reads Brick stack/components/tag, furnace/recipe/fuel, grid/knowledge/progression,
worldgen/brushable/loot, merchant, pot and client state. Writes only the cooking, crafting,
archaeology, trade, pot-drop, progression, stack and projection state listed above.

**Failure behavior:**

Wrong cooking machine or missing/replaced recipe does not smelt. Invalid grids do not craft.
Brick-containing special pots fail the all-sherd criterion. Unselected archaeology entries emit
alternatives; failed suspicious-block installation/brushing emits no Brick. Invalid/exhausted
Mason offers commit nothing. Uncracked pot destruction returns the pot rather than face
ingredients. Reloaded data changes only future evaluation; missing resources cannot grant
authority.

**Boundary cases and quirks:**

Brick possession unlocks three output recipes but not its own smelting recipe. Any sherd can
unlock the Brick-only simple-pot recipe. Brick is a real tag ingredient and encoded component
identity but collapses to an empty pot face; the default pot therefore encodes four Bricks while
showing no face tooltip. An all-Brick cross has both a custom semantic interpretation and a shaped
simple recipe, with the simple default result winning. Cracked blank pots recover up to four
Bricks as separate stacks.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.crafting.CustomRecipe#isSpecial`;
`net.minecraft.world.item.crafting.DecoratedPotRecipe#matches`;
`net.minecraft.world.item.crafting.DecoratedPotRecipe#assemble`;
`net.minecraft.world.level.block.entity.PotDecorations`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.loot.packs.VanillaArchaeologyLoot`;
`net.minecraft.data.tags.VanillaItemTagsProvider`;
`net.minecraft.data.tags.VillagerTradesTagsProvider`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{brick,decorated_pot}.json`;
`data/minecraft/recipe/{brick,bricks,flower_pot,decorated_pot_simple,decorated_pot}.json`;
`data/minecraft/advancement/recipes/{misc/brick,building_blocks/bricks,decorations/{flower_pot,decorated_pot_simple}}.json`;
`data/minecraft/advancement/adventure/craft_decorated_pot_using_only_sherds.json`;
`data/minecraft/tags/item/{decorated_pot_ingredients,decorated_pot_sherds}.json`;
`data/minecraft/loot_table/archaeology/{desert_well,trail_ruins_common}.json`;
`data/minecraft/{villager_trade/mason/1/emerald_brick,tags/villager_trade/mason/level_1,trade_set/mason/level_1}.json`;
`data/minecraft/loot_table/blocks/decorated_pot.json`;
`assets/minecraft/{items,models/item,textures/item}/brick.*`;
`ITM-FURNACE-001`; `ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `BLK-BRUSHABLE-001`; `BLK-BRICKS-001`;
`BLK-DECORATED-POT-001`; `WGEN-PIPELINE-001`; `WGEN-JIGSAW-TRAIL-RUINS-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-075`.

**Test vectors:**

Smelt default and patched Clay Balls with every cooking-machine type, recipe/fuel/slot boundary,
batch count and fractional-XP result. Exercise each output-recipe grid/offset and every unlock
route before/after tag/recipe/advancement reload, including all Brick/sherd face permutations and
the all-Brick recipe collision.

Force both archaeology entries and alternatives across exact structure/processor/brush
boundaries. Generate, transact, exhaust and restock fresh level-one Mason sets with default and
patched Emeralds. Crack and uncrack pots with every `0..4` blank-face count/order and trace four
separate outputs. Persist/synchronize all stacks/pots and verify raw ID, name, ordinary
model/texture and exact Ingredients neighborhood.
