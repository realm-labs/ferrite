# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-STICK-001` — Stick joins foliage, Dead Bush, Witch, fishing, archaeology and chest acquisition to fuel, 111 crafts and Fletcher trade

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-FURNACE-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-KNOCKBACK-001`, `MOB-001`,
`MOB-004`, `MOB-AI-001`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `WGEN-JIGSAW-VILLAGES-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item registration/components, the complete hardcoded fuel entry, all
exact-item loot, recipe, advancement and trade records, all `1,212` decoded templates and exact
client resources determine every Stick-specific branch. Generic breaking, loot, fishing,
archaeology, crafting, furnace, merchant, persistence, packet and rendering algorithms retain
their cited owners.

**Applies when:**

`minecraft:stick` is crafted, selected from block/entity/container/fishing/archaeology loot,
bought by a Fletcher, consumed as furnace fuel or one of 109 recipe inputs, moved, renamed,
persisted, synchronized or rendered before and after loot, recipe, advancement, trade or resource
reload.

**Authoritative state:**

Stick is raw item ID `974`, a common nondamageable plain `Item` with maximum stack `64` and no
direct item tag. Its default component map has no food, consumable, remainder, compost,
equipment, durability, projectile, cooldown, trim, repair, inventory-tick or intrinsic use
branch. Exact recipe predicates, the fuel table and Fletcher record select its identity directly
rather than through a tag.

**Transition and ordering:**

### Eleven foliage tables and Dead Bush

The Acacia, Azalea, Birch, Cherry, Dark-Oak, Flowering-Azalea, Jungle, Mangrove, Oak, Pale-Oak
and Spruce Leaves tables share one independent Stick pool. That pool is disabled when the tool is
exact Shears or has Silk Touch level at least one. Otherwise its Fortune table-bonus chance is
`0.02`, `0.022222223`, `0.025`, `0.033333335` or `0.1` at levels `0`, `1`, `2`, `3` or
`>=4`. Success sets count uniformly to `1..2`, then applies per-unit explosion decay.

This pool is separate from the leaves/self versus sapling alternatives and, for the three
Apple-bearing leaf types, their later Apple pool. A successful sapling or Apple draw neither
suppresses nor guarantees the Stick roll. Shears or Silk Touch suppresses the Stick pool as a
whole before its Fortune draw.

Dead Bush uses one alternatives entry. Exact Shears emits the Bush and stops the alternative.
Every other tool, including no tool, selects a Stick stack with uniform count `0..2` followed by
per-unit explosion decay. Zero count or total decay emits nothing. None of these twelve block
tables emits XP.

### Witch, fishing and archaeology

The Witch's first pool makes uniform `1..3` independent rolls. Stick has weight `2` while each of
the five other entries has weight `1`, hence `2/7` selection per roll. Selection sets base count
uniformly to `0..2` and, only with a living attacker, adds Looting `round(LU)` for independent
uniform `U in [0,1]`. The second Redstone pool is independent.

The parent fishing table makes one weighted quality-adjusted choice among junk `10/-2`, open-water
treasure `5/+2` and fish `85/-1`; generic fishing owns quality resolution. Within the selected
junk table, Stick has weight `5`. The unconditional entries total `100`, giving `1/20` outside
Jungle-family biomes; Jungle, Sparse Jungle and Bamboo Jungle admit the weight-`10` Bamboo entry,
giving `1/22`. Selection emits one default Stick.

Desert-Well archaeology makes one roll over total weight `8`: its two pottery sherds have weight
`2` each and Brick, Emerald, Stick and Suspicious Stew have weight `1` each. Stick therefore
emits one default stack with probability `1/8` when this table is resolved. Brush cadence,
suspicious-block completion and well placement remain their existing owners.

### Five chest tables

Five chest pools directly select Stick:

| Table / pool | rolls | Stick weight / pool total | count |
|---|---:|---:|---:|
| Spawn Bonus Chest `3` | `4` | `10/41` | `1..12` |
| Trial Chambers Entrance `0` | `2..3` | `5/36` | `2..5` |
| Village Cartographer `0` | `1..5` | `5/50 = 1/10` | `1..2` |
| Village Fletcher `0` | `1..5` | `6/23` | `1..3` |
| Village Toolsmith `0` | `3..8` | `20/53` | `1..3` |

Rolls are independent and can select Stick repeatedly. Together, the eleven Leaves, Dead Bush,
Witch, junk fishing, Desert-Well archaeology and five chests are exactly `20` bundled tables with
a direct Stick entry. The parent fishing table is an indirect router. No other bundled block,
entity, chest, gift, barter, raid or archaeology table directly emits Stick.

An exhaustive decoded scan finds zero exact Stick identities across all `1,212` structure
templates. Chest and suspicious-block sources are loot-table joins, not stored item stacks.

### Two producers and 109 crafting sinks

Exactly `111` shaped recipes reference Stick, and all `111` have recipe advancements:

- two vertical live `planks` emit four Sticks, while two vertical exact Bamboo emit one Stick;
- twelve wood families each provide a Fence, Fence Gate and Sign sink (`36`): corresponding
  Planks surround/alternate with Sticks and emit `3/1/3`;
- all sixteen colors consume one Stick below six matching Wool for one Banner;
- Wooden, Stone, Copper, Iron, Golden and Diamond material families each provide Axe, Hoe,
  Pickaxe, Shovel, Spear and Sword sinks (`36`). The first five patterns consume two Sticks and
  Sword consumes one;
- the remaining `21` sinks are Activator Rail, Armor Stand, Arrow, Bow, Brush, Campfire, Copper
  Torch, Crossbow, Fishing Rod, Grindstone, Item Frame, Ladder, Lever, Painting, Powered Rail,
  Rail, Redstone Torch, Soul Campfire, Soul Torch, Torch and Tripwire Hook.

The two producing recipes listen for live Planks and exact Bamboo respectively; obtaining a Stick
does not unlock either producer. Direct Stick possession is an OR alternative to prior knowledge
for Wooden Axe, Hoe, Pickaxe, Shovel, Spear and Sword, Campfire and Ladder (`8` direct Stick
unlocks). Every other advancement uses its own material criteria or prior knowledge.

All results are fixed default stacks and never copy Stick patches. Pattern normalization,
mirroring, tag lookup, result capacity, atomic ingredient consumption, remainder placement and
knowledge publication remain generic.

### Furnace fuel and Fletcher trade

`FuelValues#vanillaBurnTimes` inserts Stick directly at `200 / 2 = 100` burn ticks. Furnace-family
admission consumes one Stick and installs that time with no remainder. Recipe cooking duration,
already-burning state, output capacity and progress preservation remain `ITM-FURNACE-001`; the
same `100` ticks may represent half of a default `200`-tick cook or all of a `100`-tick cook.

Baseline Fletcher level one selects amount two without replacement from three candidates. Its
`32` Sticks to one Emerald offer therefore has inclusion probability `2/3`, maximum uses `16`, XP
`2` and reputation discount `0.05`. Trade Rebalance does not replace this record, tag or set.
Offer construction consumes nothing; a successful generic trade validates the current adjusted
cost, consumes Sticks, transfers the Emerald, increments uses and applies merchant/player effects
atomically.

**Persistence and reload boundary:**

Stacks, containers, recipe knowledge, furnace state and merchant offers persist with their owners.
Loot, recipe, advancement and trade reload changes future evaluation or offer construction only;
it does not replay completed breaks, deaths, fishing, archaeology, crafts, burns or trades.
Already-burning fuel time and existing offers retain their constructed state. Resource reload
independently changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `974`; no Stick-specific packet exists. English name is
`Stick`. The item definition selects one untinted `item/handheld` model using
`minecraft:item/stick`, with no condition, animation, tint or special renderer.

Ingredients orders Netherite Ingot, Stick, Flint, Wheat and Bone. Stick appears once and in no
other ordinary creative tab. Debug Stick, Carrot on a Stick, Warped Fungus on a Stick and Sticky
Piston are distinct identities whose substring does not create a Stick source or sink.

**Branches and aborts:**

Default/patched Stick; eleven Leaves across Shears/Silk/Fortune/explosion; Dead Bush Shears versus
`0..2`; Witch selection/Looting; parent fishing and biome-sensitive junk; archaeology; five chest
pools; two producing and 109 consuming recipes with eight direct unlocks; fuel admission; Fletcher
selected/unselected/current-cost; zero templates; persistence/reload/wire/client branches are
distinct.

**Constants and randomness:**

Item ID `974`; stack `64`; Leaves tables `11`, chance
`0.02/0.022222223/0.025/0.033333335/0.1`, count `1..2`; Dead Bush `0..2`;
Witch rolls `1..3`, selection `2/7`, base `0..2` plus `round(LU)`; junk conditional selection
`1/20` or `1/22`; archaeology `1/8`; chest rows `5`; direct tables `20`;
recipes/producers/sinks/advancement grants/direct unlocks `111/2/109/111/8`; fuel `100`; Fletcher
inclusion `2/3`, exchange `32:1`, uses/XP/discount `16/2/0.05`; templates/matches `1212/0`.

**Side effects:**

Block/entity/container/fishing/archaeology loot; crafted Stick or 109 sink results and knowledge;
furnace burn state; Fletcher input/output, uses, XP and economy effects; durable stack/container/
machine/offer state; synchronization and exact client projection.

**Gates:**

Tool/Shears/Silk/Fortune/explosion; attacker/Looting; fishing quality/open-water/biome; loot
table roll/weight/count; brush completion; exact shaped grid/live tags/result capacity;
advancement inventory/knowledge; furnace slot/burn/result; profession/level/set/current cost;
registry/stack decode and client resources.

**Boundary cases and quirks:**

Leaf Sticks use a second independent pool, so a sapling or Apple can coexist with them; Shears or
Silk Touch suppresses the entire pool. Dead Bush can select a zero-count Stick before explosion
decay. Fishing's Jungle-only Bamboo candidate changes the junk-table denominator from `100` to
`110`. The two Stick-producing recipes are not unlocked by Stick itself, while only eight of 109
sink advancements listen directly for Stick. A single Stick burns for only half a normal
`200`-tick furnace recipe.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/stick.json`;
`data/minecraft/loot_table/{blocks/{acacia_leaves,azalea_leaves,birch_leaves,cherry_leaves,dark_oak_leaves,dead_bush,flowering_azalea_leaves,jungle_leaves,mangrove_leaves,oak_leaves,pale_oak_leaves,spruce_leaves},entities/witch,gameplay/{fishing,fishing/junk},archaeology/desert_well,chests/{spawn_bonus_chest,trial_chambers/entrance,village/{village_cartographer,village_fletcher,village_toolsmith}}}.json`;
`data/minecraft/recipe/**/*.json`;
`data/minecraft/advancement/recipes/**/*.json`;
`data/minecraft/{villager_trade/fletcher/1/stick_emerald,tags/villager_trade/fletcher/level_1,trade_set/fletcher/level_1}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/stick.*`;
`assets/minecraft/lang/en_us.json`;
`ITM-RECIPE-SERIALIZER-001`; `ITM-FURNACE-001`; `EXP-ITM-094`.

**Test vectors:**

Run `EXP-ITM-094` across default/patched Stick, every Leaves and Dead-Bush tool/Fortune/explosion
branch, Witch selection/Looting, fishing open-water/quality/biome paths, Desert-Well archaeology,
all five chest rows, all 111 recipes/advancements, fuel admission and Fletcher selection/trade.
Scan every template, persist/reload/synchronize owners and assert ID, name, handheld model,
texture and Ingredients position.

**Limits:**

Generic block breaking, death, loot, fishing, archaeology, crafting, furnace, merchant, packet and
renderer control flow remains with cited owners. Leaves, Dead Bush, Witch, fishing, Desert Well,
the five structures and every crafted result retain their dedicated owners. This leaf fixes the
exact loose item, acquisition/sink joins, absences and projection.
