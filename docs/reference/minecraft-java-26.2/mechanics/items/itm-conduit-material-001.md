# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CONDUIT-MATERIAL-001` — Conduit materials join treasure, mob, fishing and trade acquisition to one fixed recipe

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-005`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-PROJECTILE-001`, `MOB-SPAWN-001`,
`MOB-WANDERING-TRADER-001`, `WGEN-STRUCTURE-BURIED-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked plain-item registrations and components, exhaustive item references,
the buried-treasure, Nautilus and fishing tables, Drowned equipment branch, wandering-trader
record/set, Conduit recipe and unlock, and direct client assets determine every identity-specific
branch. Generic structure chest population, entity death/equipment drops, fishing retrieval,
merchant transactions, crafting, progression, stacks and inventories remain with the cited
owners.

**Applies when:**

A `heart_of_the_sea` or `nautilus_shell` stack is created, looted, fished, traded, moved, renamed,
persisted, synchronized, offered to crafting, selected in a tab, rendered or observed before and
after loot, trade, recipe or resource reload.

**Authoritative state:**

`minecraft:heart_of_the_sea` is raw item ID `1369`; `minecraft:nautilus_shell` is raw item ID
`1363`. Both register through the plain-item path with default properties. Each is uncommon,
nondamageable and max stack `64`.

Neither identity belongs to a direct item tag. Their registered components are only the common
empty modifiers/enchantments/lore, item-break sound, translated name, direct item-model key,
repair cost, swing animation, tooltip display and use effects. Neither identity has food,
consumable, cooldown, remainder, tool, equipment or repairable state. The Shell's villager-trade
tag is a registry tag over trade records, not an item tag.

**Transition and ordering:**

Neither identity overrides hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but no identity-specific hand or
block branch consumes a stack, starts active use, emits a sound/game event/particle, increments
item use or changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identities add no dispenser, mob-interaction, equipment, repair, enchantment, fuel, brewing or
composting branch. Loot, entity equipment, fishing, trade and recipe consumers own the joins
below.

**Heart acquisition:**

The only locked baseline Heart source is the first pool of `chests/buried_treasure`. That pool has
one roll and one unconditional, unweighted Heart entry with no function, so every admitted
evaluation emits exactly one default Heart before the table continues to its later pools. Those
later selections cannot remove or mutate it. The table uses random sequence
`minecraft:chests/buried_treasure`; the Heart pool itself needs no selection or count draw.

Buried-treasure placement, chest orientation, structure seed and deferred first-open evaluation
remain with `WGEN-STRUCTURE-BURIED-001` and `ITM-LOOT-001`. Removing or replacing the table before
evaluation can remove this guarantee. No other bundled loot, trade, equipment or recipe record
directly produces a Heart.

**Nautilus and Drowned acquisition:**

The `entities/nautilus` table has one one-roll Shell pool. It first requires player kill credit,
then consumes one float for an enchanted-chance condition. With no positive Looting level the
chance is `0.05`. If the attacking entity is living and has Looting level `L > 0`, the chance is
`0.060000002 + 0.01 * (L - 1)`. The condition passes exactly when `nextFloat() < chance`, and then
emits one default Shell. The table uses random sequence `minecraft:entities/nautilus`.

Player credit and the attacking entity are distinct context gates: player credit can pass while an
absent or nonliving attacker supplies `L = 0` and therefore the five-percent chance. Without player
credit the pool aborts before its random-chance condition.

Separately, after superclass spawn finalization, a Drowned with an empty offhand consumes one level
random float. If it is below `0.03`, the Drowned installs one default Shell in that slot and marks
the offhand as a guaranteed equipment drop. A nonempty offhand skips both this draw and placement.
This check precedes and is independent of the later Natural/Structure-only nautilus-jockey branch,
so the Shell check itself is not restricted to those two spawn reasons.

While equipped, Drowned item pickup refuses to replace an existing Shell. An admitted later death
can therefore emit it through generic guaranteed-equipment-drop processing without the Nautilus
table's player-credit gate. Removal without the ordinary death/equipment-drop transaction still
does not create a world item. Drowned spawning, equipment persistence, death admission and drop
placement remain with `MOB-SPAWN-001`, `ENT-DEATH-001` and `ENT-ENTITY-DROPS-001`.

**Fishing acquisition:**

The root `gameplay/fishing` table makes one weighted selection among junk, treasure and fish.
For loot-context luck `l`, their effective integer weights are

`J = max(floor(10 - 2l), 0)`, `T = max(floor(5 + 2l), 0)` and
`F = max(floor(85 - l), 0)`.

The treasure candidate, and therefore `T`, is absent unless the hook's `in_open_water` predicate
is true. `FishingHook#retrieve` supplies `l` as its nonnegative stored Luck-of-the-Sea contribution
plus the player's float Luck attribute. Once treasure is selected, its nested table makes one
uniform selection among six default-weight entries; Shell is one entry and emits exactly one
without functions.

Thus, when open water is true and the root denominator is positive, the conditional Shell
probability is `T / (J + T + F) * 1/6`. At `l = 0` it is `5/100 * 1/6 = 1/120`. Without open water
it is zero. The root and nested tables use random sequences `minecraft:gameplay/fishing` and
`minecraft:gameplay/fishing/treasure`; selecting another root or treasure entry preserves its own
branch-specific function draws.

Hook bite/retrieval admission, open-water history, loot invocation, criterion trigger, item-entity
motion, XP, rod damage and hook removal remain with `ENT-PROJECTILE-001` and `ITM-LOOT-001`. The
Shell entry adds no enchantment or damage function of its own.

**Wandering-trader acquisition:**

The Shell trade record wants five Emeralds and gives one default Shell. It permits five uses, sets
reputation discount `0.05`, and inherits XP `1`; it has no second cost, merchant predicate, output
modifier or double-price enchantment.

That record is one of 76 uniform candidates in the baseline `wandering_trader/common` trade tag.
The common set chooses five distinct candidates with random sequence
`minecraft:trade_set/wandering_trader/common`, so a valid baseline generation includes this offer
with probability `5/76`. Selection creates the offer but does not create a Shell; each successful
purchase commits the generic cost/output transaction until the offer is exhausted. Trader
spawning/despawn remains with `MOB-WANDERING-TRADER-001`, while offer creation, price adjustment,
purchase, restock and merchant-menu synchronization remain generic trade behavior.

**Recipe and progression:**

The sole bundled recipe consuming either identity is shaped `conduit`: eight Shells surround one
Heart in a full three-by-three grid and return one default Conduit. The recipe copies no input
component patch and neither ingredient has a remainder. Generic shaped matching, input consumption
and result transfer remain with `ITM-RECIPE-001` and `ITM-CRAFT-001`; placement and activation of
the resulting block are outside this material leaf.

The recipe advancement has Heart possession, Shell possession and exact `conduit` recipe-unlocked
criteria in one three-entry OR requirement. Satisfying any one awards only the Conduit recipe.
The Heart criterion's locked internal name is `has_nautilus_core`, but it still matches
`heart_of_the_sea`; the name adds no Nautilus entity condition. Repeated possession after the
advancement is complete does not create ingredients or craft the result.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no structure seed/table cursor, killer credit, Looting level, Drowned spawn draw/equipment
guarantee, hook open-water/luck state, trade-set draw/uses, recipe identity or advancement
progress. Those values belong to the structure/loot transaction, entity, hook, merchant offer,
recipe manager and player progression state.

Loot reload can independently replace buried-treasure, Nautilus, root fishing and nested treasure
tables for future evaluations. Trade reload can replace the record, candidate tag or common set
before future offer generation without changing existing stacks. Recipe/advancement reload can
replace crafting and unlock records without rewriting stacks. Resource reload independently
controls names and models.

**Client and wire projection:**

Generic item-stack encoding projects raw item IDs `1369` and `1363` plus each stack's component
patch. Their uncommon-rarity names use locked English text `Heart of the Sea` and `Nautilus
Shell`; neither plain class adds a subtype tooltip.

Each direct item definition selects its generated same-named model and texture. Both appear exactly
once and only in Ingredients, ordered Prismarine Crystals, Nautilus Shell, Heart of the Sea, Fire
Charge. Their registry-ID order differs from their tab order: Shell's raw ID is lower even though
the Heart is described first by this rule.

**Branches and aborts:**

Identity/count/components; generic hand/block/container/anvil path; buried table present and
evaluation admitted; Nautilus player credit, attacker type, Looting level and chance draw; Drowned
offhand empty and spawn draw; fishing nibble/retrieval/open-water/luck/root/nested selection;
trade-record/tag/set validity and selected/exhausted offer; shaped grid/counts; each of three OR
criteria; save, data/resource reload, wire, language, model and tab context.

**Constants and randomness:**

Raw IDs Heart/Shell `1369/1363`; uncommon rarity; max stack `64`; buried Heart `1`; Nautilus Shell
chance `0.05` at `L = 0` and `0.060000002 + 0.01 * (L - 1)` at `L > 0`; Drowned offhand chance
`0.03`; fishing root weights `10/-2`, `5/+2`, `85/-1`, nested Shell share `1/6`, default total
chance `1/120`; trade five Emeralds to one Shell, five uses, five distinct of 76 candidates;
recipe eight Shells plus one Heart to one Conduit.

**Side effects:**

Loot stacks and named-sequence cursors; Drowned offhand/equipment guarantee and possible world
drop; fishing loot item/XP/criterion consequences; merchant offers, inputs, uses and output;
crafting inputs/result; advancement and recipe known/highlight state; ordinary stack
persistence/wire state; names, direct models and two Ingredients-tab entries.

**Gates:**

Generic stack/container/anvil admission; valid buried chest/table; admitted entity death and player
credit for the Nautilus table; Drowned offhand vacancy/spawn finalization and later equipment-drop
admission; valid hook retrieval/open-water/luck/table state; valid trade record/tag/set and
merchant transaction; exact crafting grid; exact inventory or recipe-unlocked criterion; valid
registry/stack decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, buried chest/table state,
entity spawn/equipment/death/attacker context, hook/open-water/player-luck state, trade registries
and offer state, recipe/advancement registries and player progression state, persisted stack and
client resources. Writes only the loot, equipment/drop, fishing, trade, crafting, progression,
stack and client projection listed above.

**Failure behavior:**

Use has no subtype success or mutation. A missing/replaced buried table removes its future Heart
guarantee. Missing Nautilus player credit or a failed chance emits no table Shell. Occupied Drowned
offhand or a draw at least `0.03` creates no equipped Shell; later nondeath removal does not drop
one. Closed-water fishing cannot select treasure. An absent/invalid/unselected/exhausted trade
cannot yield a Shell. Invalid or insufficient crafting leaves inputs unchanged. Client resource
absence follows generic missing translation/model fallback and cannot grant authority.

**Boundary cases and quirks:**

The Heart is unconditional within an admitted table evaluation, not necessarily present in every
world chest before loot is evaluated. Nautilus player credit and the Looting-bearing attacker are
separate. Drowned equipment is a spawn-time Shell source outside the Nautilus loot table, and its
guaranteed-drop marker is not itself a world drop. Open water gates the fishing treasure candidate
before luck-weighted selection. Either one Heart or one Shell in inventory unlocks a recipe whose
actual craft needs all nine ingredients.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.entity.monster.zombie.Drowned#finalizeSpawn`;
`net.minecraft.world.entity.monster.zombie.Drowned#canReplaceCurrentItem`;
`net.minecraft.world.entity.projectile.FishingHook#retrieve`;
`net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition#test`;
`net.minecraft.world.level.storage.loot.entries.LootPoolSingletonContainer$EntryBase#getWeight`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaFishingLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.tags.VillagerTradesTagsProvider`;
`reports/registries.json#minecraft:item`;
`reports/minecraft/components/item/{heart_of_the_sea,nautilus_shell}.json`;
`data/minecraft/loot_table/{chests/buried_treasure,entities/nautilus,gameplay/fishing,gameplay/fishing/treasure}.json`;
`data/minecraft/{villager_trade/wandering_trader/emerald_nautilus_shell,tags/villager_trade/wandering_trader/common,trade_set/wandering_trader/common}.json`;
`data/minecraft/{recipe/conduit,advancement/recipes/misc/conduit}.json`;
`assets/minecraft/{items,models/item,textures/item}/{heart_of_the_sea,nautilus_shell}.*`;
`ITM-LOOT-001`; `ITM-RECIPE-001`; `ITM-ADVANCEMENT-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `ENT-PROJECTILE-001`; `MOB-WANDERING-TRADER-001`;
`WGEN-STRUCTURE-BURIED-001`; `CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-040`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate admitted and
replaced buried tables, Nautilus deaths across player-credit/attacker/Looting/draw branches,
Drowned finalization across offhand/spawn/draw/death boundaries, fishing across open-water and
luck-weight transitions, and common trade selection/exhaustion while recording every RNG cursor.
Match/craft the Conduit grid and trigger all three unlock alternatives before/after data reload.
Persist/synchronize stacks and capture raw IDs, names, tooltips, models and exact Ingredients order
before/after resource reload.

**Limits:**

This leaf does not duplicate buried-structure placement, Nautilus or Drowned spawning/AI/death,
generic loot/equipment emission, fishing-hook lifecycle/retrieval consequences, merchant-menu
transactions, crafting consumption, recipe-book/advancement state or Conduit block behavior. Those
remain with their cited owners; this rule fixes the two material identities and their exact
acquisition, recipe, progression and presentation joins.
