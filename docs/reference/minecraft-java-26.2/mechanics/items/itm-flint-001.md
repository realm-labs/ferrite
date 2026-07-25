# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-FLINT-001` — Flint joins Gravel Fortune loot and two structure chests to three crafts and five villager offers

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`, `WGEN-PIPELINE-001`,
`WGEN-STRUCTURE-RUINED-PORTAL-001`, `WGEN-JIGSAW-VILLAGES-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked Flint and Gravel identities, the complete Gravel loot tree, exact-item
recipe/advancement/loot/trade searches, all `1,212` templates and exact client resources determine
every Flint-specific branch. Generic falling-block, breaking, explosion, loot, crafting, merchant,
structure, persistence, packet and rendering algorithms retain the cited owners.

**Applies when:**

`minecraft:flint` is selected from Gravel or structure loot, bought from or sold to a Villager,
consumed by Arrow, Fletching Table or Flint-and-Steel crafting, moved, renamed, persisted,
synchronized or rendered before and after loot, recipe, advancement, trade or resource reload.

**Authoritative state:**

Flint is raw item ID `1010`, a common nondamageable plain `Item` with maximum stack `64` and no
direct item tag. Its default component map has no food, consumable, remainder, fuel, compost,
equipment, durability, projectile, cooldown, trim, repair, inventory-tick or identity-specific use
branch.

Its renewable block source is Gravel: block ID `40`, item ID `90`, sole/default state `124`.
Gravel is a property-free `ColoredFallingBlock` with falling-dust color `#807C7B`, Stone map color,
Snare note instrument, strength `0.6` and Gravel sounds. Its direct block tags include
`mineable/shovel`; that tag changes mining speed but is not a correct-tool loot gate. Gravel
falling, landing, support, worldgen and projection remain block/world owners rather than Flint
state.

**Transition and ordering:**

### Gravel loot

`minecraft:blocks/gravel` makes one outer alternatives roll under the identically named random
sequence:

1. A tool with Silk Touch level at least one selects and emits one Gravel.
2. Otherwise an inner alternatives entry first applies `survives_explosion` to the whole
   Flint-or-Gravel choice. Failure emits nothing. Survival tests Fortune's `table_bonus`; a pass
   emits one Flint and a failure emits one Gravel.

The Fortune chances for levels `0`, `1`, `2` and `>=3` are respectively `0.1`,
`0.14285715`, `0.25` and `1.0`. The final array entry is reused above level three. A shovel is
not required to obtain either result, and this table emits no XP.

The outer Silk branch precedes and bypasses the explosion-survival condition. Ordinary
player-mined Silk therefore returns Gravel; a synthetic loot context containing both Silk and an
explosion also takes that first branch. Without Silk, explosion survival is decided once before
the Fortune draw, rather than by applying independent explosion decay to a selected stack.

### Structure/container acquisition

Exactly two bundled chest tables directly select Flint:

| Table / pool | rolls | Flint weight / pool total | count |
|---|---:|---:|---:|
| chests/ruined_portal `0` | `4..8` | `40/398 = 20/199` | `1..4` |
| chests/village/village_fletcher `0` | `1..5` | `6/23` | `1..3` |

Each roll is independent and may select Flint repeatedly. The Ruined-Portal denominator includes
the three entries whose omitted weight defaults to one; the Village-Fletcher denominator likewise
includes its default-weight Emerald. Container placement, loot seeds, named sequences, capacity
and commit remain with the structure/loot owners.

Together with Gravel these are exactly three bundled Flint-emitting loot tables. No entity,
fishing, archaeology, gift, barter, raid or other chest/block table directly emits Flint. An exact
UTF scan finds zero Flint identity strings across all `1,212` structure templates; the two chest
sources are therefore loot-table joins, not stored template stacks.

### Three recipe joins and progression

Flint participates in exactly three bundled recipes:

- Arrow is a shaped vertical Flint/Stick/Feather column, movable to any grid column; it consumes
  one of each and emits four default Arrows.
- Fletching Table is shaped as two Flint across the top row over two rows of two live `planks`;
  the `2x3` pattern can move within the grid and consumes two Flint plus four Planks for one
  default Fletching Table.
- Flint and Steel is shapeless and consumes one exact Iron Ingot and Flint for one default Flint
  and Steel.

Each has one recipe advancement. Their requirement arrays make prior recipe knowledge an
alternative to inventory criteria. Flint possession is itself an `inventory_changed` alternative
for all three; Arrow additionally listens for Feather, while Flint and Steel additionally listens
for Obsidian. Thus the counts for recipes, listeners and direct Flint unlocks are `3/3/3`.
Default results do not copy Flint patches. Pattern normalization, tag lookup, shapeless matching,
result capacity, atomic consumption and knowledge publication remain generic.

### Five baseline villager offers

Baseline trade sets select amount two without replacement from their resolved tags. Flint appears
in five predicate-free records:

| profession / level | exchange | max uses | XP | discount | candidates / inclusion |
|---|---|---:|---:|---:|---:|
| Fletcher `1` | `10` Gravel + `1` Emerald -> `10` Flint | `12` | `1` | `0.05` | `3`, `2/3` |
| Fletcher `2` | `26` Flint -> `1` Emerald | `12` | `10` | `0.05` | `2`, `1` |
| Leatherworker `2` | `26` Flint -> `1` Emerald | `12` | `10` | `0.05` | `3`, `2/3` |
| Toolsmith `3` | `30` Flint -> `1` Emerald | `12` | `20` | `0.05` | `5`, `2/5` |
| Weaponsmith `3` | `24` Flint -> `1` Emerald | `12` | `20` | `0.05` | `2`, `1` |

The level-one Fletcher record omits `xp`, so the trade codec supplies its default `1`; it is the
only merchant acquisition record. The other four consume Flint. Baseline
`common_smith/level_3` is empty, yielding the candidate counts above. None has a merchant
predicate or result modifier; only the Gravel exchange has a second cost.

Trade Rebalance overrides neither these profession sets nor the five records, so the table remains
unchanged when that pack is enabled. Offer generation consumes nothing. A successful generic
trade validates and consumes the current adjusted cost or costs, transfers the result, increments
uses and applies merchant/player effects atomically. Named trade-set sequences, demand,
reputation/special-price adjustment, exhaustion, restock and menu synchronization remain
merchant-owned.

**Persistence and reload boundary:**

Stacks persist identity, count and arbitrary patches; block/chest containers, recipe knowledge and
merchant offers persist with their owners. Loot, recipe, advancement, tag and trade reload changes
future evaluation or offer construction only. It does not replay completed breaks/loot/crafts or
rewrite existing offers. Existing merchant offers retain the costs and results constructed when
selected. Resource reload independently changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `1010`; no Flint-specific packet exists. The English name
is `Flint`. It selects one untinted `item/generated` flat with texture `item/flint`, without a
conditional model, tint, animation, explicit display transform or special renderer.

Ingredients orders Stick, Flint, Wheat, Bone and Bone Meal. Flint appears once and in no other
ordinary tab.

**Branches and aborts:**

Default/patched stack; Gravel Silk versus explosion-survival and four Fortune brackets; two chest
pools; three recipes/listeners; five merchant records with selected/unselected, buy/sell and
adjusted-price paths; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Flint ID `1010`, stack `64`; Gravel block/item/state `40/90/124`, strength `0.6`, dust
`#807C7B`; Gravel rolls `1`, Fortune chances `0.1/0.14285715/0.25/1.0`; chest rows `2`,
emitting loot tables `3`; recipes/listeners/direct unlocks `3/3/3`; trade records `5`, inclusion
`2/3,1,2/3,2/5,1`; templates/matches `1212/0`.

**Side effects:**

Gravel or chest loot output; crafted Arrow, Fletching Table or Flint and Steel; recipe knowledge;
merchant Gravel/Emerald/Flint consumption and output; offer uses/XP/economy effects; durable stack,
container and offer state; synchronization and exact client projection.

**Gates:**

Loot context/tool/Silk/Fortune/explosion; table roll/weight/count; exact grid and live Planks tag;
result capacity; advancement inventory/knowledge state; profession/level/trade-set/current adjusted
cost; registry/stack decode and client resources.

**State read/written:**

Reads all gates above and writes only the loot, crafting, advancement, offer, durable, wire and
projection state listed above.

**Failure behavior:**

Failed explosion survival or table selection emits no Flint. Silk selects Gravel before Fortune.
Wrong or insufficient recipe inputs emit no result. Unselected, rejected or exhausted merchant
offers consume nothing. Reload affects future evaluation only; decode failure follows generic
stack policy.

**Boundary cases and quirks:**

Fortune III makes the non-Silk, explosion-surviving branch certain to choose Flint, while Fortune
levels above three do not exceed one. Silk precedes explosion survival. The three recipe
advancements all listen directly for Flint even though Arrow has another material listener and
Flint and Steel has Obsidian. The story advancement icon and Nether-Fortress chest references to
the distinct `minecraft:flint_and_steel` identity are not Flint sources or sinks. Likewise,
`creeper_igniters` and `enchantable/durability` tag membership belongs only to Flint and Steel.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.ColoredFallingBlock`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:gravel`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/flint.json`;
`data/minecraft/tags/block/{azalea_root_replaceable,enderman_holdable,goats_spawnable_on,lush_ground_replaceable,mineable/shovel,overworld_carver_replaceables,sculk_replaceable,supports_bamboo,trail_ruins_replaceable}.json`;
`data/minecraft/loot_table/{blocks/gravel,chests/{ruined_portal,village/village_fletcher}}.json`;
`data/minecraft/recipe/{arrow,fletching_table,flint_and_steel}.json`;
`data/minecraft/advancement/recipes/{combat/arrow,decorations/fletching_table,tools/flint_and_steel}.json`;
`data/minecraft/{villager_trade/{fletcher/{1/gravel_and_emerald_flint,2/flint_emerald},leatherworker/2/flint_emerald,toolsmith/3/flint_emerald,weaponsmith/3/flint_emerald},tags/villager_trade/{fletcher/{level_1,level_2},leatherworker/level_2,toolsmith/level_3,weaponsmith/level_3,common_smith/level_3},trade_set/{fletcher/{level_1,level_2},leatherworker/level_2,toolsmith/level_3,weaponsmith/level_3}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/flint.*`;
`assets/minecraft/lang/en_us.json`;
`WGEN-STRUCTURE-RUINED-PORTAL-001`; `WGEN-JIGSAW-VILLAGES-001`;
`EXP-ITM-087`.

**Test vectors:**

Run `EXP-ITM-087` across default/patched Flint, every Gravel tool/Silk/Fortune/explosion branch,
both chest rows, all three recipes/listeners, every five-offer selection and transaction branch,
every template, all reload domains, persisted/synchronized owners and exact ID/name/model/tab
projection.

**Limits:**

Generic falling-block, tool speed, breaking, explosion survival, loot, structure, crafting,
advancement, merchant, stack codec, packet and renderer control flow remains with cited owners.
Gravel worldgen and block behavior, Arrow, Fletching Table, Flint and Steel and Emerald retain
their own owners. This leaf fixes exact Flint identity, source/sink joins, absences and projection.
