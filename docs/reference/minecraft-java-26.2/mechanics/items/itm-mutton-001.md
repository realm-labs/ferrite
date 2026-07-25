# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-MUTTON-001` — Raw and Cooked Mutton join Sheep fire-converting death loot to cooking, village loot, Butcher trade, hero gifts and Wolf feeding

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`MOB-AI-001`, `MOB-BREED-001`, `MOB-RAID-001`,
`WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, Sheep's ordered meat/wool table, live
fire-smelting join, three recipes/unlocks, one village-chest row, guaranteed level-three Butcher
purchase, Butcher hero gift, Wolf tag closure, Balanced Diet and direct client resources determine
every Mutton-specific branch. Generic use, death, loot, cooking-machine, structure, merchant,
Villager AI, Wolf, progression, stack and client behavior remains with the cited owners.

**Applies when:**

A `mutton` or `cooked_mutton` stack is eaten, emitted by Sheep death, village chest or Butcher
gift, cooked, sold to a Butcher, offered to a Wolf, moved, renamed, persisted, synchronized or
rendered before and after component, tag, recipe, advancement, loot, trade or resource reload.

**Authoritative state:**

| Item | Raw ID | Food | Other state |
|---|---:|---|---|
| `minecraft:mutton` | `1294` | nutrition `2`, saturation `1.2` | common, maximum `64`, ordinary `32`-tick eat |
| `minecraft:cooked_mutton` | `1295` | nutrition `6`, saturation `9.6` | common, maximum `64`, ordinary `32`-tick eat |

Both are nondamageable plain `Item` instances. Each has the empty consumable with no consume
effects and `can_always_eat=false`. Common remaining components are empty attribute modifiers,
enchantments and lore, item-break sound, translated name, direct item-model key, repair cost,
swing animation, tooltip display and use effects.

Both directly belong to `meat`; `wolf_food` includes the live `meat` tag. They have no direct
`wolf_food` membership.

**Transition and ordering:**

Player consumption:

In-air use enters generic consumption only below full hunger or when player ability permits
eating at full hunger. Block interaction remains block-first. Interruption, release or live-hand/
component replacement before completion commits no statistic, criterion, food, event or shrink.

Successful server completion emits final eat effects, awards the item-used statistic, triggers
`consume_item` against the pre-shrink stack, applies food, runs the empty consume-effect list,
emits the `EAT` game event and shrinks one unless materials are infinite. Default Raw Mutton adds
`2` food and `1.2` saturation; Cooked Mutton adds `6` and `9.6`, subject to the hunger owner's
clamps. Neither spends effect RNG or has a remainder.

Raw and Cooked Mutton are independent requirements of telemetry-enabled
`husbandry/balanced_diet`. Completion advances the matching pre-shrink identity even when food
clamps. The full advancement requires `40` foods and awards `100` experience.

Sheep death acquisition:

An admitted adult Sheep table evaluates its Mutton pool first and its wool alternatives pool
second under sequence `minecraft:entities/sheep`. Mutton creates a default Raw-Mutton stack,
replaces count with uniform integer `1..2`, conditionally runs `furnace_smelt`, then applies
Looting count increase.

The smelt function runs when the Sheep is on fire or the direct attacker's main hand carries an
enchantment in `smelts_loot`. It resolves the live Raw-Mutton smelting recipe during loot
evaluation. The locked recipe converts the entire base stack to Cooked Mutton before Looting. A
missing recipe leaves it Raw; replacement recipe output controls future converted drops. Fire and
enchantment conditions are ORed and cannot double-smelt.

For a living attacking entity and Looting level `L>0`, the bonus spends one float `U` and adds
`round(L*U)` after possible conversion. Absent/nonliving attacker or level zero skips that draw.
Final locked output count is therefore `B + round(L*U)` for `B in 1..2`.

The later alternatives pool inspects all `16` Sheep colors in fixed order and delegates to the
matching one-wool table only when the Sheep is unsheared. Color and sheared state therefore
control later wool, not the earlier Mutton pool. Baby/death-rule admission, wool output, equipment,
XP and all table mechanics remain with the entity and loot owners.

Cooking and recipe progression:

Three exact recipes consume one Raw Mutton and emit one default Cooked Mutton:

| Recipe | Domain | Time | Recipe XP |
|---|---|---:|---:|
| `cooked_mutton` | Furnace | `200` ticks | `0.35` |
| `cooked_mutton_from_smoking` | Smoker | `100` ticks | `0.35` |
| `cooked_mutton_from_campfire_cooking` | Campfire | `600` ticks | `0.35` |

Each no-display advancement uses one OR requirement: exact Raw-Mutton possession or matching
recipe unlock grants that recipe. Cooked-Mutton possession does not satisfy it. Furnace and
Smoker accumulate `0.35` XP per completion; player extraction resolves fractional XP and awards
recipe/smelt criteria under the machine owner. Campfire re-resolves on completion and awards
neither recipe XP nor unlock. Input component patches do not propagate.

Village Butcher-chest acquisition:

`chests/village/village_butcher` makes uniform `1..5` rolls. Each roll chooses among Emerald
weight `1`; Porkchop, Wheat, Beef and Mutton weight `6` each; and Coal weight `3`, for total `28`.
Mutton therefore has per-roll probability `6/28 = 3/14`, then replaces count with uniform integer
`1..3`. Repeated selections are permitted and use sequence
`minecraft:chests/village/village_butcher`.

Village template placement, marker/materialization, container seed, all rolls and insertion remain
with `WGEN-JIGSAW-VILLAGES-001` and `ITM-LOOT-001`. This table emits no Cooked Mutton.

Guaranteed level-three Butcher sink:

The base level-three Butcher tag contains exactly Mutton and Beef purchase records. Its set asks
for two distinct offers, so both predicate-free records are guaranteed and only their order varies
under sequence `minecraft:trade_set/butcher/level_3`.

`butcher/3/mutton_emerald` wants seven matching Raw Mutton and gives one default Emerald. It has
maximum uses `16`, Villager XP `20`, reputation discount coefficient `0.05`, no second cost,
predicate or modifier. Empty component matching accepts ordinary patches. Generic demand,
special price, reputation, restock, use and XP remain with merchant/Villager owners. Trade
Rebalance does not replace this tag or record.

Butcher hero-gift acquisition:

An adult Butcher's Hero behavior can evaluate
`gameplay/hero_of_the_village/butcher_gift` after acquiring an admitted visible Hero. The
one-roll table chooses uniformly among Cooked Rabbit, Cooked Chicken, Cooked Porkchop, Steak and
Cooked Mutton. Cooked Mutton therefore has conditional probability `1/5`, count one and named
sequence `minecraft:gameplay/hero_of_the_village/butcher_gift`.

The initial eligible cooldown is `600` ticks and later cooldown is `600 + nextInt(6001)`. The
behavior requires the Hero within five blocks, lasts at most `100` ticks and throws only after
elapsed time exceeds `20`. Age/profession, memories, navigation/look, table context, throw motion
and cleanup remain with `MOB-RAID-001`, `MOB-AI-001` and the loot owner.

Wolf feeding:

The live `meat -> wolf_food` closure admits both identities. Taming remains exact Bone. For a
tamed injured Wolf, the server heals twice the live food nutrition before generic feeding:
default Raw Mutton heals `4`, Cooked Mutton heals `12`, and a removed food component falls back
to `2`. The consume helper spends one unless materials are infinite, plays the eat sound and
returns success.

Other admitted states reach generic feeding: age-locked babies do not consume; other babies
consume one and reduce remaining growth time by ten percent; adults able to fall in love consume
and enter `600`-tick love; other adults do not consume. Wolf side/tame/owner/health/age/love/
ability ordering remains with `MOB-BREED-001`.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. They store no use/hunger, death/fire/attacker/Looting,
live recipe, machine, recipe knowledge, structure/table cursor, merchant, Villager brain, Wolf or
advancement state.

Recipe reload changes future cooking and death-table smelting. Loot reload changes future Sheep,
village and gift evaluation. Tag reload changes future Wolf admission. Trade reload changes
future offers without rewriting existing ones; advancement reload changes listeners. Completed
actions are not replayed. Resource reload independently controls language, models and textures.

**Client and wire projection:**

Generic stack encoding projects raw ID `1294` or `1295` plus component patches. Locked English
names are `Raw Mutton` and `Cooked Mutton`; both use common rarity, no subtype tooltip and no
forced glint. Direct definitions select same-named generated models and textures.

Both appear exactly once and only in Food & Drinks, ordered Raw Porkchop, Cooked Porkchop, Raw
Mutton, Cooked Mutton, Raw Chicken, Cooked Chicken.

**Branches and aborts:**

Identity/count/food/tag; hunger/ability/use; adult/death/fire/direct attacker/smelts-loot/live
recipe/Looting and meat-before-wool sequence; three recipes/machines/unlocks; village rolls;
Butcher trade and hero state; Wolf state; persistence/reload and client/wire.

**Constants and randomness:**

Raw IDs `1294/1295`; max `64`; food `2/1.2` and `6/9.6`; eat `32`; death base `1..2` plus
`round(L*U)`; cooking `200/100/600`, XP `0.35`; village `3/14` per `1..5` roll, count `1..3`;
trade `7→1`, uses/XP `16/20`; hero gift `1/5`; Wolf heal `4/12`.

**Side effects:**

Food/stat/criterion/event/shrink; raw/cooked meat then wool table cursor; cooking result/unlock/XP;
village loot; merchant offer/economy; hero gift; Wolf health/growth/love/consume; persistence,
wire and client projection.

**Gates:**

Food/hunger/use; adult/drop-enabled Sheep; fire/smelts-loot/live recipe/Looting; machine recipe;
village table; level-three Butcher; adult Butcher/Hero behavior; live Wolf-food and Wolf state;
registry/component decode and client resources.

**State read/written:**

Reads stack/components/tags, player food/use/progression, Sheep death/color/sheared/attacker/
enchantment/recipe state, machine, village/table, merchant/Villager, Wolf, persistence and
resources. Writes only the states listed above.

**Failure behavior:**

Unadmitted use commits nothing. Baby/drop-disabled Sheep emits no ordinary table; missing
smelting recipe leaves meat raw. Missing machine recipe yields no normal result. Unselected
village/gift entries, invalid/exhausted trade and ineligible Villager/Wolf state emit or consume
nothing. Missing client resources grant no authority.

**Boundary cases and quirks:**

Sheared and color state affect the second wool pool, not Mutton. Death smelting converts the full
base count before Looting. Village loot and the trade use Raw Mutton; the hero gift is Cooked.
Both items are separate Balanced-Diet requirements.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#run`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{mutton,cooked_mutton}.json`;
`data/minecraft/tags/item/{meat,wolf_food}.json`;
`data/minecraft/loot_table/{entities/sheep,chests/village/village_butcher,gameplay/hero_of_the_village/butcher_gift}.json`;
`data/minecraft/recipe/cooked_mutton*.json`;
`data/minecraft/advancement/{recipes/food/cooked_mutton*,husbandry/balanced_diet}.json`;
`data/minecraft/{villager_trade/butcher/3/mutton_emerald,tags/villager_trade/butcher/level_3,trade_set/butcher/level_3}.json`;
`assets/minecraft/{items,models/item,textures/item}/{mutton,cooked_mutton}.*`;
`ITM-FURNACE-001`; `ITM-CAMPFIRE-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-BREED-001`; `MOB-RAID-001`;
`WGEN-JIGSAW-VILLAGES-001`; `CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-065`.

**Test vectors:**

Exercise default, component-removed and patched stacks through use/hunger/abilities, containers
and anvil. Kill adult/baby Sheep across drop rule, fire, attacker, `smelts_loot`, live recipe,
Looting, color and sheared state while tracing meat-first/wool-second cursor order.

Cook in all three domains through recipe/time/output/reload/unlock/XP boundaries. Generate every
village roll, level-three Butcher order/offer lifecycle and hero-gift branch. Feed both identities
to every Wolf state; reload all data/resources, persist/reload/synchronize and verify raw IDs,
names, models, textures and exact Food-tab neighbors.
