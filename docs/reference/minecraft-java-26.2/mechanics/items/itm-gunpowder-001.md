# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-GUNPOWDER-001` — Gunpowder joins hostile-mob, structure and archaeology loot to TNT, charges, fireworks and Splash Potions

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `PLY-005`, `PLY-006`,
`PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`, `ITM-002`, `ITM-003`,
`ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-RECIPE-SERIALIZER-001`,
`ITM-CRAFT-001`, `ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ITM-FIREWORK-STAR-001`, `ENT-001`,
`ENT-KNOCKBACK-001`, `MOB-001`, `MOB-004`, `MOB-AI-001`,
`WGEN-STRUCTURE-DESERT-PYRAMID-001`, `WGEN-STRUCTURE-SHIPWRECK-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, complete exact-item loot, recipe, advancement,
brewing and trade records, all `1,212` templates and exact client resources determine every
Gunpowder-specific branch. Generic entity death, loot, archaeology, crafting, brewing, merchant,
structure, persistence, packet and rendering algorithms retain the cited owners.

**Applies when:**

`minecraft:gunpowder` is selected from Creeper, Ghast, Witch, chest or archaeology loot, bought
from a Wandering Trader, consumed by Fire Charge, Firework Star, Firework Rocket or TNT crafting or
by a brewing stand, moved, renamed, persisted, synchronized or rendered before and after loot,
recipe, advancement, brewing, trade or resource reload.

**Authoritative state:**

Gunpowder is raw item ID `978`, a common nondamageable plain `Item` with maximum stack `64` and no
direct item tag. Its default component map has no food, consumable, remainder, fuel, compost,
equipment, durability, projectile, cooldown, trim, repair, inventory-tick or identity-specific use
branch.

**Transition and ordering:**

### Hostile-mob acquisition

Creeper and Ghast each have one unconditional Gunpowder pool with one roll. Witch selects
Gunpowder from its first multi-entry pool:

| entity | pool / rolls | selection | base count |
|---|---:|---:|---:|
| Creeper | `0` / `1` | guaranteed row | `0..2` |
| Ghast | `1` / `1` | guaranteed row | `0..2` |
| Witch | `0` / `1..3` | weight `1/7` per roll | `0..2` |

Every selected row then applies the same optional Looting increase. With a living attacking entity
carrying Looting level `L>0`, it spends a fresh uniform float `U in [0,1)` and adds
`round(L*U)`. Absent/nonliving attacker or level zero skips that draw. Only a positive final count
emits, so base zero can be revived by Looting; a selected Witch row can remain invisible. Witch
rolls are independent and may select Gunpowder repeatedly.

None of the Gunpowder rows requires a player kill. Creeper's later skeleton-attacker music-disc
pool, Ghast's earlier Tear pool and later reflected-fireball/player Music Disc Tears pool, and
Witch's later Redstone pool are independent and do not replace or modify Gunpowder. Charged state
does not change the Creeper row. A Creeper that completes its own explosion follows Creeper
discard/explosion handling rather than an admitted ordinary death-loot transaction.

### Chest and archaeology acquisition

Five structure-facing tables directly select Gunpowder:

| Table / pool | rolls | Gunpowder weight / total | count |
|---|---:|---:|---:|
| chests/woodland_mansion `2` | `3` | `10/40 = 1/4` | `1..8` |
| chests/simple_dungeon `2` | `3` | `10/40 = 1/4` | `1..8` |
| chests/shipwreck_supply `0` | `3..10` | `3/84 = 1/28` | `1..5` |
| chests/desert_pyramid `1` | `4` | `10/50 = 1/5` | `1..8` |
| archaeology/desert_pyramid `0` | `1` | `1/8` | `1` |

Each chest roll is independent and can select the row repeatedly. The archaeology table selects
one of eight equal entries for each admitted suspicious block. Named loot sequences, chest
placement/materialization, archaeology brushing and commit remain with their owners.

Trade Rebalance replaces the Desert-Pyramid table but preserves this pool's four rolls,
five-entry/weight-`50` denominator, Gunpowder weight `10` and count `1..8`; the other four tables
are not overridden. The three entity and five structure-facing tables are exactly the eight
bundled Gunpowder-emitting tables. No block, fishing, gift, barter, raid or other chest/
archaeology table directly emits Gunpowder.

An exact UTF scan finds zero Gunpowder identity strings across all `1,212` structure templates.
All structure acquisition is therefore loot-table driven rather than a stored stack.

### Wandering-Trader acquisition

`trade_set/wandering_trader/uncommon` selects amount two without replacement from `15`
predicate-free records. The Gunpowder offer therefore has inclusion probability `2/15`. It wants
one Emerald and gives four default Gunpowder, with maximum uses `2`, codec-default XP `1` and
reputation discount coefficient `0.05`; it has no second cost or item/result modifier.

Trade Rebalance does not replace this set or record. Offer generation consumes nothing.
Successful generic trade validates and consumes the current adjusted Emerald cost, transfers four
Gunpowder, increments uses and applies merchant/player effects atomically. Selection sequence,
price adjustment, exhaustion, menu synchronization and Wandering-Trader lifetime remain
merchant-owned.

### Five crafting joins

Gunpowder participates in five recipe records:

- Fire Charge is shapeless: one exact Gunpowder, Blaze Powder and either exact Coal or Charcoal
  produce three default Fire Charges.
- Firework Star is always-available special crafting. It requires exactly one Gunpowder fuel and
  at least one component-bearing live `dyes` member; a second Gunpowder rejects. Assembly emits one
  Star with a new explosion record.
- Firework Rocket Simple is shapeless: one exact Gunpowder and Paper produce three default
  Firework Rockets.
- Firework Rocket is always-available special crafting. It requires one exact Paper, `1..3`
  Gunpowder stacks and any number of exact Firework Stars, rejecting a fourth fuel or foreign
  identity. Assembly emits three Rockets with `FIREWORKS.flight_duration` equal to fuel count and
  copies only present Star explosion components in row-major order.
- TNT is the full `3x3` alternating pattern with five exact Gunpowder and four independently
  selected Sand or Red Sand, producing one default TNT.

Only Firework Rocket Simple and TNT have recipe advancements. Each accepts prior recipe knowledge
or direct Gunpowder possession as its single requirement, so recipes/listeners/direct Gunpowder
unlocks count `5/2/2`. Fire Charge and both special recipes have no advancement.

One Paper plus one Gunpowder matches both Rocket records. With no retained crafting holder,
key-sorted lookup tests `minecraft:firework_rocket` before
`minecraft:firework_rocket_simple`, so the special recipe wins. A retained matching Simple holder
can win first instead. The results are component-equal: default Firework Rocket already carries
flight duration `1` and an empty explosion list, matching the special assembly for one fuel and no
Star. Holder/recipe-award identity can still differ. Two/three fuel or any Star matches only the
special recipe.

Default fixed results do not copy Gunpowder patches. Pattern allocation, special component
construction, result capacity, atomic consumption and knowledge publication remain generic.
Downstream Fire Charge ignition/projectile use, TNT placement/fuse/explosion and Rocket flight/
explosion behavior read the crafted outputs under their own owners.

### Potion-to-Splash brewing

`PotionBrewing.addVanillaMixes` registers one container recipe:
ordinary Potion + exact Gunpowder -> Splash Potion. It accepts every potion holder for which the
ordinary Potion container is admitted, including empty or custom holders; Splash and Lingering
Potion inputs are not source containers for this edge.

One Gunpowder starts the generic `400`-tick transaction and at commit can convert every admitted
ordinary Potion among bottle slots `0..2`, in slot order, before the ingredient shrinks by one.
Non-Potion bottles remain unchanged. Container conversion changes item identity to Splash Potion
and preserves only the potion holder; unrelated/custom potion-content details are not copied.
There is no recipe knowledge, advancement, XP or RNG.

An invalid-only bottle set does not start; removing the final valid Potion cancels an active
transaction under `ITM-BREW-001`. Gunpowder is an ingredient, not brewing fuel.

**Persistence and reload boundary:**

Stacks persist identity, count and arbitrary patches. Entities, containers, suspicious blocks,
recipe knowledge, Firework payloads, brewing stands and merchant offers persist with their owners.
Loot, recipe, advancement, tag and trade reload changes future evaluation or offer construction
only; completed deaths, loot, brushes, crafts, brews and offers are not replayed or rewritten.
Existing offers and active brewing retain their constructed/remembered state. Resource reload
independently changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `978`; no Gunpowder-specific packet exists. The English
name is `Gunpowder`. It selects one untinted `item/generated` flat with texture
`item/gunpowder`, without a conditional model, tint, animation, explicit display transform or
special renderer.

Ingredients orders Glowstone Dust, Gunpowder, Dragon's Breath, Fermented Spider Eye and Blaze
Powder. Gunpowder appears once and in no other ordinary tab.

**Branches and aborts:**

Default/patched stack; Creeper/Ghast guaranteed-row and Witch weighted/repeated selections with
zero/positive and attacker/Looting paths; five structure tables and baseline/overlay Desert
variants; selected/unselected merchant; five crafting records including singleton/excess fuel and
overlapping Rocket holders; Potion/mixed/invalid brewing; persistence/reload/wire/client paths are
distinct.

**Constants and randomness:**

Gunpowder ID `978`, stack `64`; entity rows `3`, count `0..2+round(LU)`, Witch rolls/selection
`1..3,1/7`; structure rows `5` with `3x1/4`, `3x1/4`, `3..10x1/28`,
`4x1/5`, `1x1/8`; emitting tables `8`; trade `1 Emerald -> 4`, uses/XP/discount
`2/1/0.05`, inclusion `2/15`; recipes/listeners/direct unlocks `5/2/2`; outputs Fire Charge/
Rocket/TNT `3/3/1`; special Rocket fuel `1..3`; brew ticks/bottles `400/3`;
templates/matches `1212/0`.

**Side effects:**

Entity/chest/archaeology Gunpowder output; merchant Gunpowder; crafted Fire Charges, Firework Star,
Rockets or TNT; recipe knowledge; converted Splash Potions; crafting/brewing/merchant consumption;
durable stack, container, Firework, brewing and offer state; synchronization and exact client
projection.

**Gates:**

Entity death/table roll/attacker/Looting; structure/table/archaeology admission; merchant set/
current cost; exact grid/live dye component/fuel singleton or `1..3` limit/result capacity;
retained recipe holder; Potion container/ingredient/fuel/timer; registry/stack decode and client
resources.

**State read/written:**

Reads all gates above and writes only the loot, crafting, advancement, brewing, offer, durable,
wire and projection state listed above.

**Failure behavior:**

Rejected death/table/trade selection or zero final entity count emits no Gunpowder. Wrong crafting
grid, second Star fuel or fourth Rocket fuel emits no result. Invalid brewing bottles do not
start/convert. Rejected or exhausted merchant offers consume nothing. Reload affects future
evaluation only; decode failure follows generic stack policy.

**Boundary cases and quirks:**

Looting can revive zero Creeper, Ghast or Witch base count. Creeper skeleton-disc and Ghast
reflected-fireball-disc conditions do not gate Gunpowder. Trade Rebalance changes other
Desert-Pyramid rows but not Gunpowder's denominator or count. The overlapping one-fuel Rocket
recipes produce equal stacks while retaining distinct recipe identities. Gunpowder both creates a
Firework Star and determines Rocket flight, but Stars are optional Rocket inputs. A single brewing
ingredient can convert three Potions and is not a fuel item.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction`;
`net.minecraft.world.item.crafting.FireworkStarRecipe`;
`net.minecraft.world.item.crafting.FireworkRocketRecipe`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{gunpowder,firework_rocket}.json`;
`data/minecraft/loot_table/{entities/{creeper,ghast,witch},chests/{woodland_mansion,simple_dungeon,shipwreck_supply,desert_pyramid},archaeology/desert_pyramid}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/desert_pyramid.json`;
`data/minecraft/recipe/{fire_charge,firework_star,firework_rocket_simple,firework_rocket,tnt}.json`;
`data/minecraft/advancement/recipes/{misc/firework_rocket_simple,redstone/tnt}.json`;
`data/minecraft/{villager_trade/wandering_trader/emerald_gunpowder,tags/villager_trade/wandering_trader/uncommon,trade_set/wandering_trader/uncommon}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/gunpowder.*`;
`assets/minecraft/lang/en_us.json`;
`ITM-RECIPE-001`; `ITM-BREW-001`; `ITM-FIREWORK-STAR-001`;
`WGEN-STRUCTURE-DESERT-PYRAMID-001`; `WGEN-STRUCTURE-SHIPWRECK-001`;
`WGEN-STRUCTURE-WOODLAND-MANSION-001`; `EXP-ITM-089`.

**Test vectors:**

Run `EXP-ITM-089` across default/patched Gunpowder, every Creeper/Ghast/Witch count and Looting
path, all five structure tables in baseline/Trade-Rebalance, the Wandering offer, all five recipes
including retained-holder overlap and every special-fuel boundary, Potion/mixed/invalid brewing,
every template, reload domains, persisted/synchronized owners and exact ID/name/model/tab
projection.

**Limits:**

Generic entity death, loot, archaeology, structure, crafting, Firework, TNT, Fire Charge, brewing,
merchant, stack codec, packet and renderer control flow remains with cited owners. Creeper, Ghast,
Witch, output items, potion holders and Emerald retain their own owners. This leaf fixes exact
Gunpowder identity, source/sink joins, absences and projection.
