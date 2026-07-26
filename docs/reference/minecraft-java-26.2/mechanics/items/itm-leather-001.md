# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-LEATHER-001` — Leather joins animal, loot, fishing and barter acquisition to equipment, books, bundles, harnesses, repair and trade

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `PLY-005`, `PLY-006`,
`PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`, `ITM-002`, `ITM-003`,
`ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-RECIPE-SERIALIZER-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-BOOK-FAMILY-001`, `ITM-BUNDLE-001`, `ITM-HARNESS-001`, `ENT-001`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-004`,
`MOB-AI-001`, `WGEN-JIGSAW-BASTION-001`, `WGEN-JIGSAW-VILLAGES-001`,
`WGEN-STRUCTURE-DESERT-PYRAMID-001`, `WGEN-STRUCTURE-JUNGLE-TEMPLE-001`,
`WGEN-STRUCTURE-MINESHAFT-001`, `WGEN-STRUCTURE-SHIPWRECK-001`,
`WGEN-STRUCTURE-STRONGHOLD-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components/tags, all entity, chest, gift, fishing and barter
tables, every recipe/advancement and merchant record, the Piglin and repair consumers, all `1,212`
decoded templates and exact client resources determine every Leather-specific branch. Generic
death, loot, fishing, barter, crafting, equipment, anvil, merchant, persistence, packet and client
algorithms retain their cited owners.

**Applies when:**

`minecraft:leather` is dropped by a mob, selected from a container, gift, fishing or barter table,
crafted from Rabbit Hide, consumed in one of its equipment or utility recipes, used to repair
Leather armor, considered by a Piglin, sold to a Leatherworker, moved, renamed, persisted,
synchronized or rendered before and after data or resource reload.

**Authoritative state:**

Leather is raw item ID `1045`, a common nondamageable plain `Item` with maximum stack `64`. Its two
direct item tags are `repairs_leather_armor` and `ignored_by_piglin_babies`. It has no food,
consumable, remainder, fuel, compost, equippable, durability, projectile, cooldown, inventory-tick
or identity-specific use component.

**Transition and ordering:**

### Entity death acquisition

Eight entity tables have one guaranteed Leather row:

| entity | pool | base count | living-attacker Looting addition |
|---|---:|---:|---:|
| Cow | `0` | `0..2` | `round(LU)` |
| Donkey | `0` | `0..2` | `round(LU)` |
| Horse | `0` | `0..2` | `round(LU)` |
| Llama | `0` | `0..2` | `round(LU)` |
| Mooshroom | `0` | `0..2` | `round(LU)` |
| Mule | `0` | `0..2` | `round(LU)` |
| Trader Llama | `0` | `0..2` | `round(LU)` |
| Hoglin | `1` | `0..1` | `round(LU)` |

Here `L` is the admitted Looting level and each invocation draws independent uniform
`U in [0,1]`. The addition is absent without the living-attacker context. None of these rows has a
killed-by-player gate. Cow/Mooshroom meat and Hoglin pork pools are independent, including their
fire/smelts-loot branches. Entity death admission, attacker context and pool sequencing remain
with the death and loot owners.

### Container, gift, fishing and barter acquisition

Eight baseline chest pools emit loose Leather:

| table / pool | rolls | Leather weight / pool total | count |
|---|---:|---:|---:|
| chests/ancient_city `0` | `5..10` | `2/84` | `1..5` |
| chests/bastion_bridge `2` | `2..4` | `1/5` | `1..3` |
| chests/bastion_hoglin_stable `1` | `3..4` | `1/14` | `1..3` |
| chests/desert_pyramid `0` | `2..4` | `20/247` | `1..5` |
| chests/jungle_temple `0` | `2..6` | `3/89` | `1..5` |
| chests/simple_dungeon `0` | `1..3` | `20/144` | `1..5` |
| chests/stronghold_corridor `0` | `2..3` | `1/101` | `1..5` |
| chests/village/village_tannery `0` | `1..5` | `1/16` | `1..3` |

Trade Rebalance replaces the Ancient-City, Desert-Pyramid and Jungle-Temple tables and removes
their Leather rows. The other five tables remain installed unchanged.

Three other acquisition paths are exact:

- `hero_of_the_village/leatherworker_gift` has one guaranteed one-count Leather row;
- Piglin bartering has one roll and selects Leather at weight `40/469`, count `2..4`; and
- the fishing root must first select its Luck-adjusted junk child; inside junk, Leather has weight
  `10/100` outside Jungle, Sparse Jungle and Bamboo Jungle, or `10/110` where the conditional
  Bamboo row is admitted, count one.

Open-water and Luck-dependent root fishing weights, Piglin barter transaction, gift targeting and
container installation remain with their owners. No archaeology or other bundled table directly
emits loose Leather. An exhaustive decoded scan finds zero exact Leather identities across all
`1,212` structure templates; container sources remain loot-table driven.

### Twenty-six recipe joins and seven direct unlocks

Leather participates in exactly `26` recipes:

- a `2x2` square of four Rabbit Hide emits one Leather;
- Book consumes one Leather plus three Paper;
- Bundle consumes one Leather below one String;
- Item Frame consumes one Leather surrounded by eight Sticks;
- Boots, Chestplate, Helmet and Leggings consume `4/8/5/7` Leather;
- Leather Horse Armor consumes seven Leather in rows `X X / XXX / X X`;
- Saddle consumes three Leather around one Iron Ingot in rows ` X  / X#X`; and
- each of the `16` colored Harness recipes consumes three Leather over two Glass and its exact Wool.

Every record has one advancement. Direct Leather possession can satisfy the inventory alternative
for the four humanoid armor pieces, Leather Horse Armor, Saddle and Item Frame (`7` direct
unlocks). Book uses Paper, Bundle uses String, Harnesses use their exact Wool and Leather itself
uses Rabbit Hide for their inventory alternatives. Recipe knowledge is an OR alternative in every
listener.

All outputs are default stacks. Leather armor dyeing, equipment/freeze behavior and cauldron
washing remain with armor/dye/component owners; Book, Bundle and Harness runtime remains with
their named leaves. Grid matching, capacity, atomic consumption and knowledge publication remain
generic.

### Repair, Piglin and merchant joins

The live `repairs_leather_armor` tag is the repair ingredient for the four humanoid Leather armor
pieces. One admitted Leather restores one quarter of the target maximum durability through the
generic anvil path. Leather Horse Armor has no repairable component and rejects Leather despite its
material and name. Combination, cost, rename, cap and commit remain `ITM-ANVIL-001`.

`PiglinAi#wantsToPickup` first rejects a stack in `ignored_by_piglin_babies` when the Piglin is a
baby. Thus default Leather cannot enter the later loved-item, food or inventory pickup branches for
babies; adults do not take this early tag rejection and continue through the generic predicate
chain. Tag reload changes future consideration only.

Baseline Leatherworker level one selects two of three candidates without replacement. The exact
Leather purchase therefore has inclusion probability `2/3`: six Leather gives one Emerald, maximum
uses `16`, Villager XP `2`, reputation discount `0.05`. Trade Rebalance does not replace this
record or set. Generic current-price adjustment, transaction, exhaustion and restock remain
merchant-owned.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. Entities, containers, fishing hooks, Piglins,
knowledge, anvils, offers and recipe outputs persist with their owners. Loot, recipe, advancement,
tag and trade reload changes only future evaluation; completed deaths, loot, fishing, barter,
crafts, repairs, pickup decisions and trades are not replayed or rewritten. Existing offers retain
their constructed costs/results. Resource reload independently changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `1045`; no Leather-specific packet exists. English name is
`Leather`. The item definition selects one untinted `item/generated` flat using
`minecraft:item/leather`. There is no condition, tint, animation or special renderer.

Ingredients orders Blue Egg, Leather, Rabbit Hide. Combat and utility outputs select their own
component/model owners; this leaf does not make the raw material wearable or dyeable.

**Branches and aborts:**

Default/patched Leather; eight entity base/Looting rows; eight baseline chests with three removed
overlays; gift, barter and biome-sensitive nested fishing; 26 recipes/listeners and seven direct
unlocks; four repairs versus Horse-Armor rejection; baby/adult Piglin; selected/unselected
Leatherworker offer; zero templates; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Item ID `1045`; stack `64`; entity tables `8`, base `0..2` except Hoglin `0..1`; baseline direct
chests `8`, rebalanced removals `3`; barter `40/469`, count `2..4`; nested junk `10/100` or
`10/110`; recipes/listeners/direct unlocks `26/26/7`; repair targets `4`; Leatherworker inclusion
`2/3`, exchange `6:1`, uses/XP/discount `16/2/0.05`; templates/matches `1212/0`.

**Side effects:**

Nineteen direct table outputs across death, chest, gift, barter and fishing domains; crafted
Leather/equipment/utility outputs and knowledge; repaired armor; Piglin pickup admission; merchant
input/output; durable stack/container/entity state, synchronization and exact client projection.

**Gates:**

Death/attacker/Looting context; installed loot overlay, roll/condition/weight/count; fishing
open-water/Luck/biome child selection; barter/gift admission; exact grid/result capacity and
knowledge; target repairable/tag/cost; Piglin age/live tag and later pickup predicates;
profession/level/trade set; registry/stack decode and client resources.

**Boundary cases and quirks:**

Hoglin's Leather base is `0..1`, unlike the seven `0..2` rows. Trade Rebalance removes three loose
chest sources rather than merely changing their denominators. Fishing's nested denominator changes
when the conditional Bamboo entry is eligible. Leather repairs four humanoid armor items but not
Leather Horse Armor. The same ordinary material is explicitly ignored by baby Piglins before the
generic pickup tests, while adults skip that early rejection.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#wantsToPickup`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{leather,leather_helmet,leather_horse_armor}.json`;
`data/minecraft/tags/item/{ignored_by_piglin_babies,repairs_leather_armor}.json`;
`data/minecraft/loot_table/{entities/{cow,donkey,horse,llama,mooshroom,mule,trader_llama,hoglin},chests/{ancient_city,bastion_bridge,bastion_hoglin_stable,desert_pyramid,jungle_temple,simple_dungeon,stronghold_corridor,village/village_tannery},gameplay/{piglin_bartering,fishing,fishing/junk,hero_of_the_village/leatherworker_gift}}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/{ancient_city,desert_pyramid,jungle_temple}.json`;
`data/minecraft/recipe/{leather,book,bundle,item_frame,leather_*,saddle,*_harness}.json`;
`data/minecraft/advancement/recipes/**/*.json`;
`data/minecraft/{villager_trade/leatherworker/1/leather_emerald,tags/villager_trade/leatherworker/level_1,trade_set/leatherworker/level_1}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/leather.*`;
`assets/minecraft/lang/en_us.json`;
`ITM-BOOK-FAMILY-001`; `ITM-BUNDLE-001`; `ITM-HARNESS-001`;
`ITM-RECIPE-SERIALIZER-001`; `EXP-ITM-091`.

**Test vectors:**

Run `EXP-ITM-091` across default/patched Leather, every entity base/Looting branch, all eight
baseline chest rows and three rebalanced removals, gift, Piglin barter and every fishing
Luck/open-water/biome denominator. Execute all 26 recipes/listeners, four anvil repairs and Horse
Armor rejection, baby/adult Piglin pickup and selected/unselected Leatherworker offers under
independent tag/data reload. Scan every template, persist/reload/synchronize all owners and assert
ID, name, generated model, texture and Ingredients order.

**Limits:**

Generic death, loot, fishing, barter, crafting, armor/equipment, anvil, Piglin AI, merchant, packet
and renderer control flow remains with cited owners. Leather armor, Horse Armor, Book, Bundle,
Harness, Saddle and Item Frame outputs retain their dedicated owners. This leaf fixes the exact
raw material, acquisition/sink joins, absences and projection.
