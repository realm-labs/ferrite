# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-GLISTERING-MELON-SLICE-001` — Glistering Melon Slices join recipe, portal and Farmer sources to Piglin admiration and Healing brewing

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ITM-FERMENTED-SPIDER-EYE-001`, `ITM-POTION-001`,
`ENT-EFFECT-001`, `MOB-AI-001`, `WGEN-STRUCTURE-RUINED-PORTAL-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components and tag, the sole recipe and unlock, exhaustive
chest/trade data, vanilla brewing graph, potion payloads, Piglin AI bytecode and client assets
determine every Glistering-Melon-Slice-specific branch. Generic crafting, chest generation,
merchant lifecycle, Piglin activity, brewing, effect, stack and inventory behavior remains with
the cited owners.

**Applies when:**

A `glistering_melon_slice` stack is crafted, generated in Ruined Portal loot, offered by a
level-five Farmer, moved, renamed, persisted, synchronized, noticed or picked up by a Piglin,
offered to a Brewing Stand, selected in a tab, rendered or observed before and after tag, loot,
recipe, advancement, trade, mix or resource reload.

**Authoritative state:**

`minecraft:glistering_melon_slice` is raw item ID `1158`. It registers through the plain-item path
with default properties, is common, nondamageable and has max stack `64`.

Its registered components are only the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no food, consumable, cooldown, remainder, tool, equipment, repairable or
identity-specific glint state.

Its sole direct item tag is `#minecraft:piglin_loved`. That membership admits the Piglin joins
below but does not make the slice barter currency: the exact barter item remains Gold Ingot.

**Transition and ordering:**

The identity does not override player hand or block use. A prototype stack's air use returns
generic `PASS`; a block click participates only in ordinary block-first interaction and fallback
handling. Direct interaction with a Piglin does not hand this noncurrency item over or generate
barter loot. A component-patched stack can activate a generic component owner, but the identity
itself never starts active use.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, equipment, repair, composting or furnace branch. Its locked
creation paths are one shaped recipe, one chest table and one Farmer offer; its operational sinks
are brewing and Piglin pickup.

**Crafting acquisition and progression:**

The sole recipe is shaped:

```text
###
#X#
###
```

`#` is Gold Nugget and `X` is Melon Slice. A match therefore consumes eight nuggets and one
centered slice, returns one default Glistering Melon Slice and has no remainder. The full 3-by-3
pattern is mirror- and rotation-invariant; an extra occupied input, missing nugget or displaced
center fails ordinary shaped matching. Input component patches are not copied to the result.

Its no-display `recipes/root` advancement has one OR requirement containing Melon Slice possession
and exact `glistering_melon_slice` recipe unlock. Either criterion awards only this recipe.
Possessing Gold Nuggets or the crafted result does not satisfy the inventory criterion.

**Ruined Portal chest acquisition:**

The first `minecraft:chests/ruined_portal` pool draws an inclusive uniform `4..8` rolls with
replacement. Its direct entries total weight `398`: five weight-40 entries, eleven weight-15
entries, six weight-5 entries and three default-weight entries. Glistering Melon Slice is one
weight-5 entry, so each roll selects it with probability `5/398`; every selected entry creates one
default stack and replaces its count with an inclusive uniform integer `4..12`.

One chest can select the entry repeatedly, producing separate stacks subject to generic container
insertion. The same named sequence then evaluates the table's second one-roll pool: Empty has
weight `1`, Lodestone has weight `2` and the latter counts `1..2`. That later pool does not create
or alter slices but remains part of the shared deterministic cursor. Structure placement and
chest/table invocation remain `WGEN-STRUCTURE-RUINED-PORTAL-001` and `ITM-LOOT-001`.

**Guaranteed level-five Farmer acquisition:**

The base Farmer level-five tag contains exactly two records: Golden Carrot and
`farmer/5/emerald_glistening_melon_slice`. The record key intentionally spells `glistening`; its
output identity is the correctly spelled `minecraft:glistering_melon_slice`.

The level-five trade set requests two records, disables duplicates by default and uses random
sequence `minecraft:trade_set/farmer/level_5`. With exactly two eligible records, every baseline
set contains both offers and randomizes only their order. The slice offer accepts four matching
Emeralds at its base price and returns three default Glistering Melon Slices, has maximum uses
`12`, grants `30` villager XP and uses reputation discount multiplier `0.05`. It has no merchant
predicate, additional cost or output modifier.

This is an acquisition source, not a buying sink. Generic Farmer level-up/restock, offer
construction, price/demand/reputation adjustment, input predicate, atomic trade commit,
exhaustion and publication remain with the merchant owners.

**Healing brewing graph:**

The vanilla mix builder registers Glistering Melon Slice through the start-mix helper. It adds
Water plus Slice to Mundane and Awkward plus Slice to Healing. Healing carries one-tick Instant
Health at amplifier `0`.

Glowstone Dust separately maps Healing to Strong Healing, whose one-tick Instant Health has
amplifier `1`. There is no Redstone or Long-Healing edge. Fermented Spider Eye separately maps
Healing to Harming and Strong Healing to Strong Harming; those corruption edges remain
`ITM-FERMENTED-SPIDER-EYE-001`.

Every admitted edge works for Potion, Splash Potion and Lingering Potion container items. The
container identity is retained while fresh target potion contents replace the source contents;
custom color, custom effects, custom name and duration scale from the source contents are not
preserved. A holder must be present and match Water or Awkward, so holderless custom-only contents
do not match. Ingredient admission tests slice identity, accepting arbitrary component patches
and discarding them on consumption.

A completed brew transforms matching bottle slots `0..2` in order, consumes one slice for up to
three outputs, leaves unmatched bottles unchanged and emits event `1035`. The slice has no
remainder, is not Brewing Stand fuel and is not furnace fuel. Fuel admission, the `400`-tick
transaction, cancellation and player-menu take criterion remain `ITM-BREW-001` and
`ITM-ADVANCEMENT-001`. Potion consumption/projection and Instant Health application remain
`ITM-POTION-001` and `ENT-EFFECT-001`.

**Piglin-loved pickup and admiration:**

Subject to generic baby-ignore, repellent, attack/admirer, reachability and inventory gates, a
Piglin can want a Glistering-Melon-Slice item entity. Pickup takes exactly one non-nugget slice,
leaving the rest in the entity, erases `TIME_TRYING_TO_REACH_ADMIRE_ITEM`, moves the slice to the
off hand and sets `ADMIRING_ITEM` for `119` ticks. Moving it to the off hand first drops any
previous offhand stack through the generic entity path.

When holding ends, an adult does not generate barter loot for the slice. It first attempts
equipment replacement and otherwise stores the stack; because a plain slice is not equippable,
that path reaches Piglin inventory, with any overflow thrown toward a generic random position.
Baby offhand finalization retains its separate equipment/main-hand policy under `MOB-AI-001`.

A player holding a loved slice also satisfies `isPlayerHoldingLovedItem`. That feeds the Piglin
sensor/look/nearest-wanted-player boundary and the associated jealous-sound decision without
transferring the held item. All remaining sensing, memories, activity arbitration, combat,
inventory/equipment policy, sounds and navigation remain `MOB-AI-001`.

**Persistence and reload boundary:**

Slice stacks persist and synchronize identity, count and arbitrary ordinary component patches.
They store no recipe knowledge, chest RNG/cursor, structure context, offer lifecycle, Piglin brain
memory/timer or brewing slot/fuel/timer/mix. Those values belong to their player, world, loot,
trade, entity and machine owners; Piglin-held stacks use the entity's generic persistence paths.

Loot reload changes future Ruined Portal evaluations; recipe/advancement reload changes future
matching and listeners; tag reload changes future Piglin-loved tests; trade reload changes future
Farmer offer sets; a rebuilt baseline mix retains both slice start edges while their holders are
enabled. Completed crafting, loot, trades, pickups and brews are not replayed. Resource reload
independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1158` plus the stack's component patch. Its
common-rarity name uses locked English text `Glistering Melon Slice`; the plain class adds no
subtype tooltip or forced glint.

The direct item definition selects generated model `minecraft:item/glistering_melon_slice` and its
same-named texture. It appears exactly once and only in Ingredients, ordered Rabbit's Foot,
Glistering Melon Slice, Spider Eye.

**Branches and aborts:**

Identity/count/components/tag; generic hand/block/container/anvil path; recipe shape and unlock
listeners; Ruined Portal roll/entry/count/later pool and container insertion; Farmer set/order/
offer/economy; stand fuel/timer/container/holder/custom-content; Piglin subtype/activity/
repellent/attack/inventory/offhand/equipment/player-held state; save, tag/loot/recipe/advancement/
trade/mix/resource reload, wire, language, model and tab context.

**Constants and randomness:**

Raw ID `1158`; common rarity; max stack `64`; recipe `8` Gold Nuggets plus `1` Melon Slice to `1`;
portal first-pool rolls `4..8`, total weight `398`, slice weight `5`, count `4..12`; later Empty/
Lodestone weights `1/2`, Lodestone count `1..2`; trade `4` Emeralds to `3` slices, uses `12`, XP
`30`, discount `0.05`, amount `2` from two records; Healing payloads one tick at amplifiers `0/1`;
owner brew duration `400`; Piglin admiration `119` ticks.

**Side effects:**

Possible crafted, chest and trade item stacks; recipe knowledge and two named-sequence cursors;
Farmer offer/use/XP/economy state; Brewing Stand ingredient/bottles/timer/event and potion/effect
state; Piglin item-entity count, offhand/inventory/equipment, memory, look and sound decisions;
ordinary stack persistence/wire state; name, direct model and one Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; exact shaped recipe and active recipe snapshot; Ruined
Portal table/roll/weight/count and container admission; level-five Farmer offer and valid trade
inputs; valid stand fuel plus Water/Awkward source holder; live Piglin-loved tag and generic Piglin
pickup/activity gates; registry/stack decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components/tags, crafting inputs/knowledge, chest loot and insertion
state, Farmer trade registries/economy, brewing slots/fuel/timer/mix/contents, Piglin/player/item
entity state, persistence and client resources. Writes only the crafting, progression, loot,
trade, brewing, Piglin-held, stack and client state listed above.

**Failure behavior:**

Player use has no subtype success. Missing, displaced, extra or wrong recipe inputs produce no
result. An unselected portal entry emits no slice. A missing or exhausted Farmer offer commits
nothing. Missing fuel, holderless contents or an unlisted holder prevents brewing; unmatched
slots stay unchanged. A Piglin rejected by generic pickup/activity gates takes nothing, and direct
player interaction cannot barter the slice. Missing/replaced tag, loot, recipe, advancement, mix
or trade data removes future paths without rewriting completed state. Client resource absence
follows generic fallback and cannot grant authority.

**Boundary cases and quirks:**

The recipe surrounds one Melon Slice with eight nuggets, but only Melon Slice possession unlocks
it. Portal selection is independently weight `5/398` on every one of `4..8` rolls and can repeat.
The Farmer resource key misspells `glistening`, yet both level-five offers are guaranteed because
the set draws two distinct records from two. Start mixing makes Mundane from Water as well as
Healing from Awkward, but Healing has no long form. Loved-item pickup causes admiration, not
bartering, and removes only one slice from a larger item entity.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#run`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.world.level.storage.loot.entries.LootPoolSingletonContainer$EntryBase#getWeight`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.ItemCost#test`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isLovedItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#wantsToPickup`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#pickUpItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#stopHoldingOffHandItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isPlayerHoldingLovedItem`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing#hasPotionMix`;
`net.minecraft.world.item.alchemy.PotionBrewing#mix`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addStartMix`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,potion,mob_effect,recipe,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/glistering_melon_slice.json`;
`data/minecraft/tags/item/piglin_loved.json`;
`data/minecraft/loot_table/chests/ruined_portal.json`;
`data/minecraft/recipe/glistering_melon_slice.json`;
`data/minecraft/advancement/recipes/brewing/glistering_melon_slice.json`;
`data/minecraft/{villager_trade/farmer/5/emerald_glistening_melon_slice,tags/villager_trade/farmer/level_5,trade_set/farmer/level_5}.json`;
`assets/minecraft/{items,models/item,textures/item}/glistering_melon_slice.*`;
`ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-BREW-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-FERMENTED-SPIDER-EYE-001`; `ITM-POTION-001`;
`ENT-EFFECT-001`; `MOB-AI-001`; `WGEN-STRUCTURE-RUINED-PORTAL-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-051`.

**Test vectors:**

Exercise default/patched slices through hands, blocks, containers and anvil. Match the shaped
recipe across orientation, extra/absent/wrong inputs, patches and each unlock criterion. Generate
Ruined Portal chests across every first-pool roll/entry/count and the later Empty/Lodestone branch
while tracing the named cursor. Build every level-five Farmer set/order and use/exhaust/reload the
slice offer. Brew Water/Awkward/Healing and all controls in Potion/Splash/Lingering containers.
Run adult/baby Piglins through every pickup, admiration, offhand, equipment/inventory and
player-held-loved gate before/after tag reload. Persist/synchronize and capture raw ID, name,
tooltip, model and exact Ingredients position before/after every reload domain.

**Limits:**

This leaf does not duplicate generic recipe matching/result take, Ruined Portal generation/chest
insertion, loot execution, Farmer/merchant lifecycle, Brewing Stand transaction, potion/effect
behavior, Piglin sensing/activity/inventory, or stack/resource codecs. Those remain with their
cited owners; this rule fixes the slice identity and its exact crafting, acquisition, brewing,
Piglin-loved and presentation joins.
