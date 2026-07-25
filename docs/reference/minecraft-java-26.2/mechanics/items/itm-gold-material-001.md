# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-GOLD-MATERIAL-001` — Raw Gold, Gold Ingots and Gold Nuggets join ore, loot and trade acquisition to Piglin barter, crafting, repair, Beacon payment and armor trim

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-CONTAINER-MOVE-001`,
`ITM-CONTAINER-CLOSE-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-FURNACE-001`, `ITM-SMITHING-001`, `ITM-RECIPE-SERIALIZER-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`MOB-001`, `MOB-004`, `MOB-AI-001`, `BLK-BEACON-001`,
`BLK-BREAK-HOOK-001`, `BLK-BRUSHABLE-001`, `BLK-RAW-STORAGE-001`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-BASTION-001`,
`WGEN-STRUCTURE-IGLOO-001`, `WGEN-STRUCTURE-OCEAN-RUIN-001`,
`WGEN-JIGSAW-TRAIL-RUINS-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations/components, seven direct tag roles, exact Piglin
currency/pickup code, 54 recipes and their advancement joins, 36 loot rows, two Cleric purchase
forms, Gold trim and tool/armor materials, five placed ore features, 100 template Gilded-Blackstone
cells and the one embedded Igloo offer determine every Raw-Gold, Gold-Ingot and Gold-Nugget-specific
branch. Generic stack, Piglin AI, Beacon, recipe, cooking, anvil, smithing, merchant, loot,
worldgen and rendering algorithms remain with the cited owners.

**Applies when:**

A `minecraft:raw_gold`, `minecraft:gold_ingot` or `minecraft:gold_nugget` stack is created,
matched, cooked, crafted, repaired with, offered to a Piglin, used as Beacon or trim material,
moved, traded, persisted, synchronized or rendered; or when Gold Ore, Deepslate Gold Ore, Nether
Gold Ore, Gilded Blackstone, a container, archaeology site or Zombified Piglin is evaluated as one
of the family's acquisition sources before and after recipe, tag, advancement, loot, trade,
trim-material or resource reload.

**Authoritative state:**

Raw Gold, Gold Ingot and Gold Nugget have raw item IDs `935`, `936` and `1147`. All are common
nondamageable plain `Item` instances with maximum stack `64`. Their defaults contain the common
empty modifiers, enchantments and lore, item-break sound, translated name, direct item-model key,
repair cost, swing animation, tooltip display and use effects.

Gold Ingot additionally has
`minecraft:provides_trim_material=minecraft:gold`. The other two do not. None has food,
consumable, remainder, durability, equipment, tool, projectile, cooldown, inventory-tick or
identity-specific air use. Arbitrary valid ordinary patches persist through generic stack owners.
Except for trim assembly, exact-item and tag tests below do not require the default component map.

The complete direct memberships are:

| Item | Direct tags |
|---|---|
| Raw Gold | `piglin_loved` |
| Gold Ingot | `beacon_payment_items`, `gold_tool_materials`, `piglin_loved`, `repairs_gold_armor`, `trim_materials` |
| Gold Nugget | `metal_nuggets` |

The two Gold material/repair tags each contain only Gold Ingot.
`beacon_payment_items` has five identities, `trim_materials` eleven and `metal_nuggets` Copper,
Iron and Gold Nuggets. `piglin_loved` has 26 direct values plus nested `gold_ores`; it controls
admiration and player-held sensing, not barter currency. Exact currency is independently fixed to
Gold Ingot in `PiglinAi`.

**Transition and ordering:**

Prototype stacks return generic `PASS` in air and use ordinary block-first interaction. Operational
behavior enters only through exact identity, tag, component, recipe, loot, merchant, Piglin and
generation joins.

### Material repair, Beacon and trim

`ToolMaterial.GOLD` uses `incorrect_for_gold_tool`, durability `32`, mining speed `12.0`,
attack-damage bonus `0.0`, enchantment value `22` and `gold_tool_materials`. Golden Pickaxe,
Shovel, Axe, Hoe, Sword and Spear therefore store a live repair set whose locked sole member is
Gold Ingot.

`ArmorMaterials.GOLD` has durability multiplier `7`, defense
Boots/Leggings/Chestplate/Helmet/Body `1/3/5/2/7`, enchantment value `25`, Gold equip sound, zero
toughness and knockback resistance, and `repairs_gold_armor`. Its four humanoid armor pieces admit
Gold Ingot. Golden Horse and Nautilus Armor have no repairable component despite both recycling.
Anvil pricing, damage removal and commit remain `ITM-ANVIL-001`; this leaf fixes the ten admitted
and two deliberately rejected targets.

Beacon payment slot zero admits any live `beacon_payment_items` member and caps at one. Gold Ingot
qualifies regardless of ordinary patches or trim component. Direct placement, exact-count-one
quick move, valid-effect removal and close-time return/drop otherwise remain `BLK-BEACON-001` and
the container owners.

Eighteen Smithing-Trim records—Bolt, Coast, Dune, Eye, Flow, Host, Raiser, Rib, Sentry, Shaper,
Silence, Snout, Spire, Tide, Vex, Ward, Wayfinder and Wild—require their exact template, live
`trimmable_armor` base and live `trim_materials` addition. Default Gold Ingot passes the tag and
supplies the `minecraft:gold` holder. Replacing that component selects another valid holder;
removing it makes assembly empty after recipe admission. An identical existing trim also fails.

The locked material asset is `gold`, description color `#DEB12D`, translation
`trim_material.minecraft.gold` (`Gold Material`) and Gold-equipment override `gold_darker`; other
equipment uses `gold`. Admission tag, actual stack component and material/resource record are
independent.

### Piglin identity dispatch and barter

Raw Gold and Gold Ingot are loved. Subject to the generic reachability, repellent, activity,
inventory and admiration gates, item-entity pickup removes exactly one of either identity, moves
it to the off hand, erases `TIME_TRYING_TO_REACH_ADMIRE_ITEM` and installs
`ADMIRING_ITEM=true` for `119` ticks. Any previous offhand stack is dropped first. A player holding
either identity also satisfies the loved-item sensor/look boundary.

Raw Gold is not currency. When an adult finishes admiring it, the Piglin attempts equipment
replacement and then inventory storage; a plain Raw-Gold stack reaches storage, with overflow
thrown. It never invokes barter loot.

Gold Ingot is the exact currency. Direct player interaction succeeds only for an adult Piglin that
is not already admiring and lacks `ADMIRING_DISABLED`. It consumes/returns one held Ingot through
living-entity semantics, moves that one to the off hand, installs the same `119`-tick memory, stops
walking and returns `SUCCESS`; all rejected cases return `PASS`. Ordinary patches do not affect
the exact identity test.

Normal adult holding completion removes the Ingot and takes one roll from
`gameplay/piglin_bartering`, total weight `469`:

| Result | Weight | Count/component |
|---|---:|---|
| Soul-Speed Enchanted Book | `5` | one; randomly enchanted from `soul_speed` |
| Soul-Speed Iron Boots | `8` | one; randomly enchanted from `soul_speed` |
| Fire-Resistance Potion | `8` | one |
| Fire-Resistance Splash Potion | `8` | one |
| Water Potion | `10` | one |
| Iron Nugget | `10` | `10..36` |
| Ender Pearl | `10` | `2..4` |
| Dried Ghast | `10` | one |
| String | `20` | `3..9` |
| Quartz | `20` | `5..12` |
| Obsidian | `40` | one |
| Crying Obsidian | `40` | `1..3` |
| Fire Charge | `40` | one |
| Leather | `40` | `2..4` |
| Soul Sand | `40` | `2..8` |
| Nether Brick | `40` | `2..8` |
| Spectral Arrow | `40` | `6..12` |
| Gravel | `40` | `8..16` |
| Blackstone | `40` | `8..16` |

The one selected output stack is thrown toward the current wanted player when present, otherwise
toward a random position. If hurt forces offhand finalization with barter disabled, an adult exact
currency stack is cleared without barter, equipment or inventory fallback; the paid Ingot is
lost. Baby finalization retains its separate equipment/main-hand policy. Piglin brain scheduling,
throw target, table mechanics and equipment/inventory behavior remain `MOB-AI-001` and
`ITM-LOOT-001`.

Gold Nugget is neither loved nor currency. Its hard-coded pickup exception takes the entire Nugget
item-entity stack rather than one, discards the entity, and then attempts equipment/inventory
handling without admiration. Pickup admission still requires inventory capacity. Thus the three
loose identities enter three distinct Piglin paths.

The `nether/distract_piglin` advancement has one OR requirement. Its thrown criterion accepts a
live `piglin_loved` item picked up by an adult Piglin; its direct-interaction criterion accepts
exact Gold Ingot. Both require the source player to wear no `piglin_safe_armor` member in head,
chest, legs or feet. Raw Gold can satisfy only the thrown route, Gold Ingot either route, and Gold
Nugget neither. Bartering itself is not gated by this armor predicate.

### Cooking and material conversion

Eight Gold-Ingot cooking records pair Gold Ore, Deepslate Gold Ore, Nether Gold Ore and Raw Gold:
Furnace emits one Ingot after default `200` ticks; Blast Furnace after default `100`. Every record
has recipe XP `1.0` and group `gold_ingot`. Smoker and Campfire reject them.

Two recycling records accept exactly six Golden tools, four humanoid Golden armor pieces, Golden
Horse Armor and Golden Nautilus Armor. Furnace emits one Gold Nugget after `200` ticks for XP
`0.1`; Blast Furnace after `100` for `0.1`. Damage, enchantments, trim and other patches do not
affect identity matching and are discarded. Every cooking output is default and leaves no
remainder.

Six exact compact/decompression records are:

- nine Gold Ingots to one Gold Block and one Gold Block to nine Ingots;
- nine Gold Nuggets to one Gold Ingot and one Gold Ingot to nine Nuggets;
- nine Raw Gold to one Raw Gold Block and one Raw Gold Block to nine Raw Gold.

The Raw-Gold pair is also fixed by `BLK-RAW-STORAGE-001`.

### Remaining crafting

Six shaped tool recipes use live `gold_tool_materials` at `X` and Stick at `#`: Axe
`XX/X#/ #`, Hoe `XX/ #/ #`, Pickaxe `XXX/ # / # `, Shovel `X/#/#`, Spear `  X/ # /#  ` and
Sword `X/X/#`. Four armor grids use exact Ingots: Boots `X X/X X`, Chestplate
`X X/XXX/XXX`, Helmet `XXX/X X`, and Leggings `XXX/X X/X X`.

Ten additional records are exact:

| Result | Ingredients/pattern and output |
|---|---|
| Clock | four Ingots around Redstone; one |
| Glistering Melon Slice | eight Nuggets around Melon Slice; one |
| Golden Apple | eight Ingots around Apple; one |
| Golden Carrot | eight Nuggets around Carrot; one |
| Golden Dandelion | eight Nuggets around Dandelion; one |
| Light Weighted Pressure Plate | two horizontal Ingots; one |
| Name Tag | diagonal live `metal_nuggets` plus Paper; one |
| Netherite Ingot | shapeless four Netherite Scraps plus four Gold Ingots; one |
| Powered Rail | `X X/X#X/XRX`; `X` Ingot, `#` Stick, `R` Redstone; `6` |
| Firework Star | special recipe below; one |

The Firework-Star serializer requires exactly one Gunpowder and at least one component-bearing
live `dyes` member, permits at most one shape, Diamond trail and Glowstone-Dust twinkle input, and
rejects all others. Gold Nugget is the exact `star` shape input; when present it sets
`FIREWORK_EXPLOSION.shape=star`. Assembly preserves row-major dye colors, empty fade colors and the
optional trail/twinkle flags on a default Firework Star. This special recipe is always available
and has no recipe advancement.

All 26 crafting records emit default outputs except that component constructed by the special
recipe. Inputs otherwise copy no arbitrary patch and leave no remainder. Tag reload can broaden
tool and Name-Tag inputs but not exact Gold positions. Shape translation/mirroring and ordinary
grid consumption remain `ITM-CRAFT-001`.

### Recipe progression

Each ordinary recipe advancement has one OR requirement with its known-recipe criterion. Gold
Ingot possession unlocks Gold Block, Gold Nugget, Golden Apple, all four humanoid Golden armor
pieces and Light Weighted Pressure Plate; any live `gold_tool_materials` member unlocks each six
tools. Gold Nugget unlocks Gold Ingot from Nuggets, Golden Carrot and Golden Dandelion and, through
`metal_nuggets`, Name Tag. Raw Gold unlocks both of its cooking records and Raw Gold Block.

Exact Gold/Deepslate/Nether Gold Ore unlocks its corresponding Furnace and Blast-Furnace pair.
Gold Block unlocks decompression, Raw Gold Block unlocks Raw Gold, and any of the twelve recyclable
gear identities unlocks each Nugget cooking record.

Clock instead unlocks from Redstone, Glistering Melon from Melon Slice, Netherite Ingot from
Netherite Scrap and Powered Rail from Rail. Golden Dandelion also has Dandelion as an alternative;
Name Tag also has Paper and Name-Tag alternatives. Every trim recipe unlocks from its exact
template. Firework Star has no listener. Possessing Gold Ingot therefore does not alone unlock
Clock, Glistering Melon, Golden Carrot/Dandelion, Netherite Ingot, Powered Rail, cooking, recycling,
Firework Star or trim.

### Ore and Gilded-Blackstone acquisition

Gold Ore, Deepslate Gold Ore and Nether Gold Ore are property-free `DropExperienceBlock` states
`129`, `130` and `135`. The Overworld pair requires an iron-tier correct pickaxe and has XP `0`.
Nether Gold Ore is pickaxe-mineable and has uniform XP `0..1`; exact correct-tool admission remains
with the break owner.

Each table tests Silk Touch first and emits its own ore block. Non-Silk Overworld ore emits one Raw
Gold, then applies Fortune's `ore_drops` multiplier and per-unit explosion decay. At positive
Fortune `L`, multiplier `M=max(1,nextInt(L+2))`; one has probability `2/(L+2)`, and each
`2..L+1` probability `1/(L+2)`. At zero it stays one without a draw.

Non-Silk Nether Gold instead first draws inclusive `C in 2..6`, multiplies by the same `M`, then
applies per-unit explosion decay to Gold Nuggets. Its admitted non-Silk break separately draws XP
`0..1`; Silk suppresses XP. The three named sequences match their table IDs.

Gilded Blackstone has a nested one-roll table. Silk emits its block. Otherwise the entire fallback
first passes `survives_explosion`; failure emits nothing. On survival, `table_bonus` tests Gold
Nugget chances `0.1`, `0.14285715`, `0.25`, and `1.0` at Fortune levels `0`, `1`, `2`, and `>=3`.
Success emits uniform `2..5` Nuggets. Failure emits one Gilded Blackstone. There is no Fortune
count multiplier or per-unit explosion decay. Its named sequence is
`minecraft:blocks/gilded_blackstone`.

### Chest and archaeology acquisition

For each row, the selected pool takes the inclusive roll count, then each roll independently
selects by the stated weight/total; a selected entry sets the inclusive count:

| Table | Rolls | Item | Weight/total | Count |
|---|---:|---|---:|---:|
| `abandoned_mineshaft` | `2..4` | Ingot | `5/98` | `1..3` |
| `bastion_bridge` | `1..2` | Ingot | `1/13` | `4..9` |
| `bastion_bridge` | `2..4` | Nugget | `1/5` | `2..6` |
| `bastion_hoglin_stable` | `3..4` | Nugget | `1/14` | `2..8` |
| `bastion_other` | `2` | Ingot | `2/20` | `1..6` |
| `bastion_other` | `3..4` | Nugget | `1/13` | `2..8` |
| `bastion_treasure` | `3..4` | Ingot | `1/9` | `3..9` |
| `buried_treasure` | `5..8` | Ingot | `10/35` | `1..4` |
| `desert_pyramid` | `2..4` | Ingot | `15/247` | `2..7` |
| `end_city_treasure` | `2..6` | Ingot | `15/89` | `2..7` |
| `igloo_chest` | `2..8` | Nugget | `10/63` | `1..3` |
| `jungle_temple` | `2..6` | Ingot | `15/89` | `2..7` |
| `nether_bridge` | `2..4` | Ingot | `15/78` | `1..3` |
| `ruined_portal` | `4..8` | Nugget | `15/398` | `4..24` |
| `ruined_portal` | `4..8` | Ingot | `5/398` | `2..8` |
| `shipwreck_treasure` | `3..6` | Ingot | `10/150` | `1..5` |
| `shipwreck_treasure` | `2..5` | Nugget | `10/80` | `1..10` |
| `simple_dungeon` | `1..4` | Ingot | `5/125` | `1..4` |
| `stronghold_corridor` | `2..3` | Ingot | `5/101` | `1..3` |
| `stronghold_crossing` | `1..4` | Ingot | `5/62` | `1..3` |
| `underwater_ruin_big` | `2..8` | Nugget | `10/33` | `1..3` |
| `village_plains_house` | `3..8` | Nugget | `1/43` | `1..3` |
| `village_savanna_house` | `3..8` | Nugget | `1/46` | `1..3` |
| `village_temple` | `3..8` | Ingot | `1/19` | `1..4` |
| `village_toolsmith` | `3..8` | Ingot | `1/53` | `1..3` |
| `village_weaponsmith` | `3..8` | Ingot | `5/107` | `1..3` |
| `woodland_mansion` | `1..4` | Ingot | `5/175` | `1..4` |

Trade Rebalance replaces Abandoned Mineshaft, Desert Pyramid and Jungle Temple. Their rows, rolls,
weights and counts remain; only Desert Pyramid's relevant total changes from `247` to `237`.

Cold and Warm Ocean-Ruin archaeology each takes one roll and emits one Gold Nugget at weight
`2/15`; Trail-Ruins Common emits one at `1/45`. Suspicious-block installation, brushing and pickup
remain with the brushable and structure owners.

### Zombified-Piglin acquisition and Cleric sinks

Zombified Piglin's second pool creates Gold Nugget, sets base `B` to uniform integer `0..1`, then
with positive living-attacker Looting `L` adds `round(L*U)` for a fresh float `U`. Only positive
effective counts emit; there is no player-kill gate. Its third pool requires player kill and a
chance below `0.025` without positive Looting or `0.025+0.01L` at positive `L`; success emits one
Gold Ingot. Both use sequence `minecraft:entities/zombified_piglin`.

Reloadable Cleric level two has two candidates and amount two, so its exact Gold purchase is
guaranteed. It wants three Gold Ingots, gives one Emerald, has uses `12`, Villager XP `10` and
discount `0.05`. Trade Rebalance does not replace it.

`structure/igloo/bottom.nbt` separately embeds one fixed Plains-Cleric offer: it wants nine Gold
Ingots, gives one Emerald, begins at uses `0`, has max uses `7`, and stores `rewardExp=true`.
This is a persisted template offer, not the reloadable trade record; trade/tag reload does not
rewrite it. Generic price adjustment, transaction, exhaustion and restock remain merchant-owned.
Both forms accept exact Gold-Ingot identity with arbitrary ordinary patches.

### Generation and absence boundary

Every one of the `55` Overworld biomes schedules:

- `ore_gold`: buried size `9`, air-exposure discard `0.5`, count `4`, trapezoid `-64..32`;
- `ore_gold_lower`: the same configured feature, count uniformly `0..1`, uniform `-64..-48`.

Badlands, Eroded Badlands and Wooded Badlands additionally schedule `ore_gold_extra`: nonburied
size `9`, discard `0`, count `50`, uniform `32..256`.

Nether Gold uses size `10`, Netherrack target and discard `0`. Crimson Forest, Nether Wastes,
Soul Sand Valley and Warped Forest schedule count `10`; Basalt Deltas substitutes count `20`.
Both use in-square and uniform above-bottom `10` through below-top `10`.

The 1,212-template census finds exactly `100` live Gilded-Blackstone cells in 26 templates, all
feeding its break table only after ordinary Bastion placement gates. It finds no Raw-Gold or
Gold-Nugget identity string and one Gold-Ingot string: the embedded Igloo offer above, not a loose
stack. There is no Gold ore-vein variant and no direct fishing, cat-gift, compost, fuel, brewing
fuel or dispenser branch for the three loose identities.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. They do not own Piglin memories/offhand/inventory,
Beacon state, recipe progress/XP, knowledge, anvil/Smithing preview, loot cursor, merchant offer,
brush state or worldgen state. Recipe/tag/advancement/loot/trade/trim reload changes only future
evaluation in its domain. Exact Piglin currency, Nugget bulk-pickup and the embedded Igloo offer do
not follow tag/trade reload. Completed work, barters, offers, loot and chunks are not replayed.
Resource reload independently changes names/models/textures/palettes.

**Wire and client projection:**

Generic stack codecs publish raw IDs `935/936/1147`, count and patches. No family packet exists.
Ingredients orders `Raw Copper, Raw Iron, Raw Gold`, later
`Copper Nugget, Iron Nugget, Gold Nugget`, then
`Copper Ingot, Iron Ingot, Gold Ingot`; each appears once and nowhere else in ordinary tabs.

English names are `Raw Gold`, `Gold Ingot`, `Gold Nugget`. Each selects a like-named
`item/generated` model and texture. Gold trim uses `gold`/`gold_darker`, not the loose Ingot
texture.

**Branches and aborts:**

Three identities/seven direct roles; loved/nonloved and currency/noncurrency Piglin paths; normal
versus interrupted admiration; Beacon/repair/trim component states; ten cooking, 26 crafting and
18 trim records; three ore and Gilded tables; 27 chest and three archaeology rows; two Zombified
pools; reloadable versus embedded Cleric offers; five placed features, 100 Gilded cells and one
offer string; persistence/reload/wire/client branches are distinct.

**Constants and randomness:**

IDs `935/936/1147`; stack `64`; Gold tool `32/12/0/22`; armor `7/25`; admiration `119`; barter
total `469`; compacting `9:1`; cooking `200/100`, XP `1/0.1`; ore states `129/130/135`, base
`1` or `2..6`, Fortune multiplier above, XP `0/0/0..1`; Gilded chances
`0.1/0.14285715/0.25/1`, count `2..5`; Zombified Nugget `B+round(L*U)`, Ingot chance
`0.025+0.01L`; Cleric costs `3/9`, uses `12/7`; feature values above; trim `#DEB12D`.

**Side effects:**

Piglin pickup, admiration, currency consumption and barter output; Beacon payment; cooking,
crafting, repair, trim and knowledge; block/container/entity/archaeology loot; merchant offers;
generated ore/Gilded terrain; persistence, synchronization and projection.

**Gates:**

Identity/components/tags; Piglin age/memories/inventory/reachability; Beacon menu; machine/grid/
special-recipe inputs; repair set; Smithing roles/component/existing trim; knowledge; correct tool,
Silk, Fortune, explosion; table/pool/pack/death context; merchant/structure state; biome/feature/
template admission; client resources.

**State read/written:**

Reads all gates above and writes only the Piglin, payment, processing, result, repair, trim,
knowledge, loot, offer, generated-terrain, stack, wire and client state listed above.

**Failure behavior:**

Rejected Piglin/Beacon interaction spends nothing; interrupted currency can vanish without barter.
Wrong machines/grids/materials, full results, invalid/unchanged trim and rejected repair do not
commit. Wrong-tool ore emits nothing; Silk bypasses loose material; failed explosion/chance/weight/
merchant/worldgen gates emit/write nothing. Reload affects future work only.

**Boundary cases and quirks:**

Love and currency deliberately diverge: Raw Gold admires without bartering, Gold Ingot does both,
and Gold Nugget bulk-pickups without admiration. Gold Ingot repairs only humanoid Gold equipment
although Horse/Nautilus armor recycle. Gold Nugget selects Firework-Star shape but the recipe has
no advancement. Gilded Fortune `>=3` guarantees its Nugget branch only after explosion survival.
All Overworld biomes receive two Gold placements, Badlands three; Basalt Deltas doubles Nether
attempts. Igloo holds a fixed nine-Ingot offer, not loose Gold.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.ToolMaterial`;
`net.minecraft.world.item.equipment.ArmorMaterials`;
`net.minecraft.world.entity.monster.piglin.Piglin#mobInteract`;
`net.minecraft.world.entity.monster.piglin.PiglinAi`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#mobInteract`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#canAdmire`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isLovedItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isBarterCurrency`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#wantsToPickup`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#pickUpItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#stopHoldingOffHandItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#getBarterResponseItems`;
`net.minecraft.world.inventory.BeaconMenu$PaymentSlot`;
`net.minecraft.world.item.crafting.FireworkStarRecipe`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe`;
`net.minecraft.world.entity.npc.villager.AbstractVillager`;
`net.minecraft.world.item.trading.VillagerTrade`;
`net.minecraft.world.item.trading.TradeSet`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set,trim_material,worldgen}`;
`reports/blocks.json#minecraft:{gold_ore,deepslate_gold_ore,nether_gold_ore,gilded_blackstone}`;
`reports/minecraft/components/item/{raw_gold,gold_ingot,gold_nugget,golden_pickaxe,golden_helmet,golden_horse_armor,golden_nautilus_armor}.json`;
`data/minecraft/tags/item/{beacon_payment_items,gold_tool_materials,piglin_loved,repairs_gold_armor,metal_nuggets,trim_materials}.json`;
`data/minecraft/trim_material/gold.json`;
`data/minecraft/recipe/{gold_*,golden_*,raw_gold,raw_gold_block,clock,firework_star,glistering_melon_slice,light_weighted_pressure_plate,name_tag,netherite_ingot,powered_rail,*_armor_trim_smithing_template_smithing_trim}.json`;
`data/minecraft/advancement/{nether/distract_piglin,recipes/**/*.json}`;
`data/minecraft/loot_table/{blocks/{gold_ore,deepslate_gold_ore,nether_gold_ore,gilded_blackstone,raw_gold_block},entities/zombified_piglin,chests/**/*.json,archaeology/*.json,gameplay/piglin_bartering}.json`;
`data/minecraft/{villager_trade/cleric/2/gold_ingot_emerald,tags/villager_trade/cleric/level_2,trade_set/cleric/level_2}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/{abandoned_mineshaft,desert_pyramid,jungle_temple}.json`;
`data/minecraft/worldgen/{configured_feature/{ore_gold,ore_gold_buried,ore_nether_gold},placed_feature/{ore_gold,ore_gold_lower,ore_gold_extra,ore_gold_nether,ore_gold_deltas},biome/*.json}`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item}/{raw_gold,gold_ingot,gold_nugget}.json`;
`assets/minecraft/textures/item/{raw_gold,gold_ingot,gold_nugget}.png`;
`assets/minecraft/equipment/gold.json`;
`assets/minecraft/textures/trims/color_palettes/{gold,gold_darker}.png`;
`EXP-ITM-079`.

**Test vectors:**

Run `EXP-ITM-079` across all identity/component/tag variants; direct/thrown Piglin barter,
admiration completion/interruption, loved sensing and Nugget bulk pickup; every Beacon, repair and
trim branch; all ten cooking, 26 crafting, 18 trim and progression records. Break every ore and
Gilded state across tool/Silk/Fortune/explosion/XP; exhaust all 27 chest, three archaeology and two
Zombified rows under both pack states.

Generate/transact the reloadable and embedded Cleric offers; run all five placed features in every
scheduled biome and all 100 Gilded template cells; scan all 1,212 templates. Persist/reload/
synchronize owners and assert IDs, names, generated models, Ingredients ordering and
gold/gold-darker projection.

**Limits:**

Generic stack/use, Piglin brain arbitration, Beacon control, cooking/crafting/special serializers,
advancements, anvil/Smithing commit, loot/brushing, merchant economy, feature/template placement,
wire and rendering remain with the cited owners.
