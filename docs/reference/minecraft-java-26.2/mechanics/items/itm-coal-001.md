# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-COAL-001` — Coal and Charcoal join code-built fuel, shared recipes and minecart propulsion to distinct acquisition paths

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ENT-001`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-VEHICLE-001`, `MOB-001`,
`MOB-004`, `MOB-AI-001`, `MOB-SPAWN-001`, `BLK-BREAK-HOOK-001`,
`BLK-BRUSHABLE-001`, `ENV-FIRE-001`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-ANCIENT-CITY-001`, `WGEN-STRUCTURE-IGLOO-001`,
`WGEN-STRUCTURE-SHIPWRECK-001`, `WGEN-STRUCTURE-OCEAN-RUIN-001`,
`WGEN-STRUCTURE-STRONGHOLD-001`, `WGEN-STRUCTURE-MINESHAFT-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `WGEN-JIGSAW-VILLAGES-001`,
`WGEN-JIGSAW-TRAIL-RUINS-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked plain-item registrations, the code-built fuel table, two direct tags,
twelve recipe records, twelve recipe advancements, block/entity/chest/archaeology loot, five
profession offer sets, ore/fossil generation joins and direct client resources determine every
Coal- and Charcoal-specific branch. Generic stack, furnace, crafting, loot, merchant, vehicle,
archaeology, structure, worldgen and client algorithms remain with the cited owners.

**Applies when:**

A `minecraft:coal` or `minecraft:charcoal` stack is tested or spent as Furnace or Furnace-Minecart
fuel; matched by a recipe or recipe-unlock predicate; produced by cooking, block/entity/chest/
archaeology loot or a trade; moved, renamed, persisted, synchronized or rendered before and after
recipe, loot, tag, trade, advancement or resource reload.

**Authoritative state:**

Coal and Charcoal are raw item IDs `924` and `925`. Each is a common nondamageable plain `Item`
with maximum stack `64`. Their default components are the common empty modifiers, enchantments and
lore, item-break sound, translated name, direct item-model key, repair cost, swing animation,
tooltip display and use effects. Neither has food, consumable, remainder, durability, equipment,
tool, projectile, cooldown, inventory tick or identity-specific use behavior.

Both—and only these two identities—are direct members of `#minecraft:coals` and
`#minecraft:furnace_minecart_fuel`. Arbitrary ordinary component patches preserve the item
identity and satisfy the empty component predicates used by fuel, recipes and trades. Neither item
has another direct item-tag membership.

**Transition and ordering:**

Code-built Furnace fuel:

`FuelValues.vanillaBurnTimes` uses standard unit `200` and directly maps Coal and Charcoal to
`8 * 200 = 1600` burn ticks apiece. `isFuel`/`burnDuration` select only `stack.getItem()`, so
ordinary component patches do not affect admission or duration. Starting a fuel transaction
consumes one item and leaves no remainder under `ITM-FURNACE-001`; one item can cover eight
default-`200`-tick cooks, with unused burn time retained by the owning furnace state.

The compacted Coal Block is a separate code-built fuel worth `8 * 10 * 200 = 16000` ticks.
Consequently nine individual Coal items provide `14400` ticks, while their one crafted block
provides `16000`. Charcoal cannot enter that compacting recipe. Coal-Block fire admission
(`5/5`) and block behavior remain `ENV-FIRE-001` and the block owners.

Furnace-Minecart fuel:

Both identities are the complete locked `furnace_minecart_fuel` tag. Furnace-Minecart interaction
always returns success. A live member adds `3600` fuel ticks only when the result is at most
`32000`, sets horizontal push to cart position minus player position and consumes one through
living-entity item semantics. A wrong identity or an over-cap offer still consumes the action but
does not mutate stack, fuel or push. Propulsion, speed, smoke, persistence and lit-block projection
remain `ITM-MINECART-001` and `ENT-VEHICLE-001`.

Coal/Charcoal production and compacting:

- shaped `coal_block` is an exact full `3×3` grid of nine Coal and emits one default Coal Block;
  Charcoal is rejected;
- shapeless `coal` consumes one Coal Block and emits nine default Coal;
- `coal_from_smelting_coal_ore` and its Deepslate variant are Furnace-only, each consuming the
  exact named ore and emitting one default Coal after `200` ticks for recipe XP `0.1`;
- the two parallel `coal_from_blasting_*` records are Blast-Furnace-only, take the same exact ore
  inputs, emit one Coal after `100` ticks and record XP `0.1`;
- Furnace-only `charcoal` accepts any live `#minecraft:logs_that_burn` member, emits one default
  Charcoal after `200` ticks and records XP `0.15`. The locked tag expands nine wood-family tags
  with four identities each, `36` inputs; Crimson and Warped stems are deliberately excluded.

Smoker and Campfire recipe maps reject all five cooking outputs. Furnace rejects the two blasting
records; Blast Furnace rejects the three smelting records. Successful processing copies no
arbitrary input patches and leaves no remainder. Fuel/progress/reset/result capacity, recipe-used
accounting, extraction and fractional experience remain `ITM-FURNACE-001`.

Shared ingredient recipes:

Five crafting records consume exactly one Coal-or-Charcoal position:

- shaped `torch` is Coal/Charcoal over Stick and emits four Torches; its `1×2` pattern fits the
  `2×2` inventory grid and admitted `3×3` offsets;
- shaped `soul_torch` is Coal/Charcoal over Stick over a live
  `#minecraft:soul_fire_base_blocks` member and emits four Soul Torches; height three requires the
  `3×3` grid;
- shaped `copper_torch` is Copper Nugget over Coal/Charcoal over Stick and emits four Copper
  Torches; height three likewise requires `3×3`;
- shapeless `fire_charge` combines one Gunpowder, one Blaze Powder and one Coal/Charcoal and emits
  three Fire Charges in either crafting-grid size;
- shaped `campfire` is Stick at top center, Stick/`#coals`/Stick on the middle row and three live
  `#minecraft:logs` members on the bottom row, emitting one Campfire in `3×3`.

Torch, Soul-Torch, Copper-Torch and Fire-Charge records encode an inline two-identity ingredient,
so changing `#coals` does not alter them. Campfire matching uses the live tag. Extra, missing or
misplaced inputs fail; successful results are default stacks, copy no input patches and leave no
remainder.

Recipe progression:

The locked recipe advancements have these OR groups:

- exact Coal possession or known `coal_block` grants the compacting recipe;
- exact Coal-Block possession or known `coal` grants decompression;
- exact matching Coal-Ore item possession or the corresponding known recipe grants each of the
  four smelting/blasting records independently;
- any live `logs_that_burn` possession or known `charcoal` grants Charcoal smelting;
- any live `coals` member, Stick possession or known `campfire` grants the Campfire recipe.

Coal/Charcoal possession therefore grants Campfire, and Coal alone grants Coal Block. Neither item
possession unlocks Torch, Soul Torch, Copper Torch, Fire Charge or Charcoal: those advancements
instead use Stone Pickaxe, soul-fire substrate, Copper Nugget, Blaze Powder and burnable-log
criteria respectively, each OR its known recipe. Listener registration, knowledge persistence and
craft criteria remain `ITM-ADVANCEMENT-001`.

Coal-Ore break acquisition:

Coal Ore and Deepslate Coal Ore are property-free `DropExperienceBlock` states `133/134` and
require a correct tool for drops. After the break owner admits loot, each one-roll table first
tests Silk Touch level at least one and emits its own default ore block. Otherwise it emits one
default Coal, applies Fortune's `ore_drops` formula and then explosion decay.

At Fortune level zero, count is one without a bonus draw. At positive level `L`, it draws
`D = nextInt(L+2)` and produces `max(1,D)`: count one has probability `2/(L+2)`, and each
`2..L+1` has probability `1/(L+2)`. Explosion decay independently retains each post-Fortune item.
Fortune is bypassed by Silk. An admitted non-Silk correct-tool break separately draws uniform
experience `0..2`; wrong-tool and Silk branches suppress that experience under
`BLK-BREAK-HOOK-001`.

The ordinary Coal Ore alone, not Deepslate Coal Ore, is a direct `snaps_goat_horn` member. When the
Goat ram owner reaches its front-position or above-position tag test, that identity can select the
horn-drop/sound/finish path. This membership does not itself schedule a ram or alter the ore.

Campfire break acquisition:

`blocks/campfire` first tests Silk Touch level at least one and emits one default Campfire on that
branch. Otherwise it creates exactly two default Charcoal and applies `survives_explosion` to the
entry: a nonexplosive admitted break always emits both, while an explosion admits the whole
two-item entry with probability `1/radius` or emits neither.
This is one all-or-nothing draw, not per-item explosion decay. Fortune is inert, arbitrary
Campfire state is not copied and Soul Campfire uses another table. The named sequence is
`minecraft:blocks/campfire`; Silk context and block removal remain with the block-break and loot
owners.

Wither-Skeleton death acquisition:

`entities/wither_skeleton` evaluates its Coal pool before Bone and the player-gated skull pool. The
one Coal entry creates a default stack, replaces count with uniform integer `B in -1..1`, then,
with a living attacking entity and Looting level `L>0`, spends a fresh float `U` and adds
`round(L*U)`. Effective count is `B + round(L*U)` and only a positive result emits; Looting can
therefore revive a zero or sufficiently offset a negative base. No killed-by-player condition
gates Coal. The named sequence is `minecraft:entities/wither_skeleton`; other pools retain order
on the same cursor.

Chest acquisition:

Every admitted row emits default Coal, applies the listed integer count and permits replacement
selection. Probability is per roll:

| Table and pool | Rolls | Coal weight / total | Count |
|---|---:|---:|---:|
| `chests/abandoned_mineshaft`, pool 1 | uniform `2..4` | `10/98` | uniform `3..8` |
| `chests/ancient_city`, pool 0 | uniform `5..10` | `7/84` | uniform `6..15` |
| `chests/igloo_chest`, pool 0 | uniform `2..8` | `15/63` | uniform `1..4` |
| `chests/shipwreck_supply`, pool 0 | uniform `3..10` | `6/84` | uniform `2..8` |
| `chests/simple_dungeon`, pool 1 | uniform `1..4` | `15/125` | uniform `1..4` |
| `chests/stronghold_crossing`, pool 0 | uniform `1..4` | `10/62` | uniform `3..8` |
| `chests/underwater_ruin_big`, pool 0 | uniform `2..8` | `10/33` | uniform `1..4` |
| `chests/underwater_ruin_small`, pool 0 | uniform `2..8` | `10/30` | uniform `1..4` |
| `chests/woodland_mansion`, pool 1 | uniform `1..4` | `15/175` | uniform `1..4` |
| `chests/village/village_butcher`, pool 0 | uniform `1..5` | `3/28` | uniform `1..3` |
| `chests/village/village_fisher`, pool 0 | uniform `1..5` | `2/11` | uniform `1..3` |
| `chests/village/village_snowy_house`, pool 0 | uniform `3..8` | `5/53` | uniform `1..4` |
| `chests/village/village_toolsmith`, pool 0 | uniform `3..8` | `1/53` | uniform `1..3` |

Each table uses its own matching namespaced random sequence. Other pools advance the same table
cursor as specified by their owner. Trade Rebalance replaces the Abandoned-Mineshaft and
Ancient-City table files but preserves these Coal pools, denominators and rows exactly.

Archaeology acquisition:

Three archaeology tables take one roll. Warm and Cold Ocean Ruins each give Coal weight `2` of
total `15`, probability `2/15`; Trail-Ruins Common gives Coal weight `1` of total `45`,
probability `1/45`. A selection emits one default Coal under the table's matching random sequence.
Ocean-Ruin processing globally caps successful suspicious Sand/Gravel replacements at five per
piece invocation. Trail-Ruins processors can install at most six common suspicious Gravel blocks
per house, two per road and two per tower top. These are opportunities subject to the owning
processor/write gates, not guaranteed Coal. Ten brushing strokes, table/seed state, item exposure
and pickup remain `BLK-BRUSHABLE-001` and the structure owners.

Villager Coal purchases:

Three predicate-free records purchase exact Coal and give one default Emerald:

| Record | Cost | Uses | Villager XP | Discount |
|---|---:|---:|---:|---:|
| `fisherman/1/coal_emerald` | `10` Coal | `16` | `2` | `0.05` |
| `butcher/2/coal_emerald` | `15` Coal | `16` | `2` | `0.05` |
| `smith/1/coal_emerald` | `15` Coal | `16` | `2` | `0.05` |

The Fisherman level-one set selects two without duplicates from four predicate-free candidates, so
Coal appears with probability `1/2`. Butcher level two selects two of three, probability `2/3`.
The Smith record is shared: Armorer and Toolsmith level one each select two of five, probability
`2/5`, while Weaponsmith selects two of three, probability `2/3`. Under Trade Rebalance, Armorer
level one replaces its candidates with exactly Coal and Iron-Ingot purchases while retaining
amount two, so Coal becomes guaranteed there; the other four sets are unchanged.

Arbitrary Coal patches satisfy the empty input predicate. No record accepts Charcoal, has a second
cost, predicate/output modifier or double-price enchantment. Offer construction, demand,
reputation, restocking and menu commit remain merchant-owned.

Ore and fossil generation join:

Normal Coal acquisition composes with the already audited ore/fossil pipeline:

- `ore_coal` and `ore_coal_buried` have size `17`, ordered Stone/Deepslate targets and air-exposure
  discard `0/0.5`; upper placement uses count `30` and uniform height absolute `136` through
  below-top `0`, while lower uses count `20` and trapezoid absolute `0..192`;
- both placed features occur in all `55` locked Overworld biomes;
- `fossil_coal` is rarity `1/64`, in-square, uniform absolute zero through below-top zero and
  occurs in Desert, Mangrove Swamp and Swamp. Its eight paired overlay templates contain exactly
  `574` raw Coal-Ore cells, no entities, before integrity-`0.1`, protection, transform, clipping
  and write behavior.

Generation writes ore blocks, not Coal items. A later admitted non-Silk correct-tool break is still
required to reach the Coal loot branch. Exact candidate scans, ore geometry, fossil
rotation/index/burial/processor order, failed writes and biome scheduling remain
`WGEN-PIPELINE-001`.

**Persistence and reload boundary:**

Coal/Charcoal stacks persist identity, count and arbitrary valid patches. They store no furnace
burn/progress, minecart fuel/push, recipe/knowledge, loot cursor, merchant offer, archaeology or
worldgen state; those values persist with their owners. Remaining furnace burn time and
Furnace-Minecart fuel survive only through those owners, not the consumed stack.

Recipe reload changes future cooking/crafting matches and output. Loot reload changes future
block/entity/chest/archaeology evaluation. Tag reload changes future Campfire and
Furnace-Minecart admission plus Campfire possession criteria; it does not change the code-built
Furnace duration or four inline two-identity recipes. Trade and advancement reload change future
offers/listeners. Existing stacks, offers, completed work and generated chunks are not replayed or
rewritten. Resource reload independently controls names, models and textures.

**Client and wire projection:**

Generic stack encoding projects raw IDs `924/925` plus patches. Locked English names are `Coal`
and `Charcoal`; both are common, have no forced glint and use ordinary generated
`minecraft:item/{coal,charcoal}` models and same-named textures.

Ingredients begins with Coal then Charcoal, followed by Raw Copper, Raw Iron and Raw Gold. Each
identity appears exactly once and in no other vanilla creative tab. This leaf adds no packet field,
acknowledgement or connection-local state.

**Branches and aborts:**

Identity/components; Furnace fuel available/unavailable and remaining burn; Furnace-Minecart
valid/invalid/over-cap fuel; seven production/compacting and five shared-input recipes across
machine/grid/result boundaries; twelve unlock records; correct/wrong-tool, Silk/Fortune/explosion/
XP ore loot; Campfire Silk/explosion loot; Goat tag; Wither-Skeleton death/Looting; thirteen
chests under both pack states;
three archaeology tables/installers; five profession sets under both pack states; two ore and one
fossil generation families; persistence/reload/wire; name/model/tab.

**Constants and randomness:**

Raw IDs `924/925`; max `64`; Furnace duration `1600`; Coal Block duration `16000`;
Furnace-Minecart increment/cap `3600/32000`; compact/decompact `9→1/1→9`; cooking
times/XP `200/0.1`, `100/0.1`, `200/0.15`; shared outputs `4/4/4/3/1`; ore Fortune
`max(1,nextInt(L+2))`, explosion decay and XP `0..2`; Campfire non-Silk count `2` with
all-or-nothing explosion survival; Wither-Skeleton base `-1..1` plus `round(L*U)`; chest,
archaeology, trade and generation values as listed above.

**Side effects:**

Fuel-stack consumption and furnace/minecart timers; crafting/cooking inputs, results, knowledge and
XP; ore/Wither/chest/archaeology output; merchant offer/economy; Goat horn selector admission;
ore/fossil palette writes; ordinary stack persistence/wire and direct client projection.

**Gates:**

Exact identity or live tag; furnace/machine recipe and fuel admission; output capacity; valid
grid; advancement listener; correct-tool/Silk/Fortune/explosion/death/attacker/Looting contexts;
generated chest/suspicious-block/table and brushing; profession/level/candidate/offer validity;
Goat ram owner; structure/feature/processor/write admission; registry/decode; client language/
model/tab bootstrap.

**State read/written:**

Reads stacks/components/tags, furnace/minecart, recipe/grid/knowledge, block/death/loot,
archaeology/structure, merchant, Goat, worldgen and client state. Writes only the fuel, processing,
progression, loot, merchant, mob, worldgen, stack and projection state listed above.

**Failure behavior:**

Invalid/over-cap fuel mutates nothing beyond the owning Furnace-Minecart interaction result.
Wrong machine, missing/replaced recipe, invalid grid or full result rejects processing. Wrong-tool
ore breaks emit no loot/XP; Silk replaces Coal with ore and suppresses XP. Unselected loot or
offer candidates emit alternatives/nothing. Failed suspicious-block, brushing, trade, Goat,
structure or feature gates do not gain authority from identity membership. Reload changes only
future evaluation; missing client resources cannot grant server behavior.

**Boundary cases and quirks:**

Coal and Charcoal are equal `1600`-tick Furnace fuels and interchangeable in five output recipes,
but only Coal compacts, appears in all non-Campfire acquisition/trade records and can be generated
through ores. Only Charcoal is the non-Silk Campfire drop. Campfire recipe matching and its
possession unlock use live `coals`; four other recipes hard-code the pair. Coal/Charcoal possession
does not unlock Torch variants or Fire Charge. Coal Block gains `1600` burn ticks versus its nine
inputs. Furnace-Minecart fuel is `3600` per item rather than Furnace burn duration.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.entity.vehicle.minecart.MinecartFurnace#addFuel`;
`net.minecraft.world.entity.ai.behavior.RamTarget#hasRammedHornBreakingBlock`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{coal,charcoal}.json`;
`reports/blocks.json#minecraft:{coal_ore,deepslate_coal_ore}`;
`data/minecraft/recipe/{coal,charcoal,coal_block,coal_from_{smelting,blasting}_{coal_ore,deepslate_coal_ore},torch,soul_torch,copper_torch,fire_charge,campfire}.json`;
`data/minecraft/advancement/recipes/{building_blocks/coal_block,decorations/{campfire,torch,soul_torch,copper_torch},misc/{coal,charcoal,coal_from_{smelting,blasting}_{coal_ore,deepslate_coal_ore},fire_charge}}.json`;
`data/minecraft/tags/item/{coals,furnace_minecart_fuel,logs_that_burn}.json`;
`data/minecraft/loot_table/blocks/{coal_ore,deepslate_coal_ore,campfire}.json`;
`data/minecraft/loot_table/entities/wither_skeleton.json`;
`data/minecraft/loot_table/chests/{abandoned_mineshaft,ancient_city,igloo_chest,shipwreck_supply,simple_dungeon,stronghold_crossing,underwater_ruin_{big,small},woodland_mansion,village/{village_butcher,village_fisher,village_snowy_house,village_toolsmith}}.json`;
`data/minecraft/loot_table/archaeology/{ocean_ruin_cold,ocean_ruin_warm,trail_ruins_common}.json`;
`data/minecraft/{villager_trade/{butcher/2/coal_emerald,fisherman/1/coal_emerald,smith/1/coal_emerald},tags/villager_trade/{butcher/level_2,fisherman/level_1,common_smith/level_1,armorer/level_1,toolsmith/level_1,weaponsmith/level_1},trade_set/{butcher/level_2,fisherman/level_1,armorer/level_1,toolsmith/level_1,weaponsmith/level_1}}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/{loot_table/chests/{abandoned_mineshaft,ancient_city},tags/villager_trade/armorer/level_1}.json`;
`data/minecraft/worldgen/{configured_feature/{ore_coal,ore_coal_buried,fossil_coal},placed_feature/{ore_coal_upper,ore_coal_lower,fossil_upper},processor_list/fossil_coal,biome/*.json}`;
`data/minecraft/structure/fossil/*_coal.nbt`;
`assets/minecraft/{items,models/item}/{coal,charcoal}.json`;
`assets/minecraft/textures/item/{coal,charcoal}.png`;
`ITM-FURNACE-001`; `ITM-RECIPE-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-MINECART-001`; `BLK-BREAK-HOOK-001`;
`BLK-BRUSHABLE-001`; `WGEN-PIPELINE-001`; `WGEN-STRUCTURE-OCEAN-RUIN-001`;
`WGEN-JIGSAW-TRAIL-RUINS-001`; `CLI-EFFECT-001`; `EXP-ITM-076`.

**Test vectors:**

Run `EXP-ITM-076` with default and patched Coal/Charcoal through every Furnace start/residual-burn
boundary and Furnace-Minecart invalid/valid/cap boundary. Exercise every production and
shared-input recipe, near-miss, machine/grid, output capacity and unlock route before/after
recipe/tag/advancement reload.

Break both ores through wrong/correct tools, Silk, all Fortune levels, explosion radii and XP
draws; break Campfire through Silk/non-Silk and nonexplosive/explosive all-or-nothing branches;
evaluate Wither-Skeleton bases/Looting, all thirteen chest rows under both pack states, three
archaeology tables and installation/brush owners, and all five profession sets under both trade
states. Run both ore placements and upper fossil selection/processor/template/write boundaries;
assert 55/55/3 biome references and eight-file/574-cell overlay census. Persist, synchronize and
verify exact IDs, names, generated models/textures and Ingredients prefix.

**Limits:**

Generic stack/use, furnace timers/XP, crafting, advancements, block/death/chest/archaeology loot,
merchant economy, Furnace-Minecart propulsion, Goat AI/horn state, structure/worldgen algorithms,
packet encoding and client rendering remain with `ITM-001`, `ITM-FURNACE-001`,
`ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`, `ITM-LOOT-001`, `ITM-MINECART-001`,
`ENT-VEHICLE-001`, `MOB-AI-001`, `BLK-BRUSHABLE-001`, `WGEN-PIPELINE-001`,
the cited structure owners, `PROTO-PLAY-CLIENTBOUND-CONTAINER-001`,
`PROTO-PLAY-CLIENTBOUND-ENTITY-001` and `CLI-006`.
