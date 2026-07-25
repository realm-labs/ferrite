# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-FEATHER-001` — Feather joins bird death, Cat gifts and structure loot to arrows, brushes, writing, fireworks and Fletcher trade

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `PLY-005`, `PLY-006`,
`PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`, `ITM-002`, `ITM-003`,
`ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-RECIPE-SERIALIZER-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-CHICKEN-001`, `ENT-001`, `ENT-KNOCKBACK-001`, `MOB-001`, `MOB-004`,
`MOB-AI-001`, `WGEN-DIMENSION-001`, `WGEN-STRUCTURE-SHIPWRECK-001`,
`WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, complete exact-item loot, recipe,
advancement and trade records, all `1,212` templates and exact client resources determine every
Feather-specific branch. Generic death, Cat AI, loot, crafting, Firework-Star, merchant,
structure, persistence, packet and rendering algorithms retain the cited owners.

**Applies when:**

`minecraft:feather` is created by bird death, Cat gift or structure loot, used in Arrow, Brush,
Writable Book or Firework Star crafting, sold to a Fletcher, moved, renamed, persisted,
synchronized or rendered before and after recipe, advancement, loot, trade or resource reload.

**Authoritative state:**

Feather is raw item ID `977`, a common nondamageable plain `Item` with maximum stack `64` and no
direct item tag. Its default component map has no food, consumable, remainder, fuel, compost,
equipment, durability, projectile, cooldown, trim, repair, inventory-tick or identity-specific use
branch.

**Transition and ordering:**

### Bird-death and Cat-gift acquisition

An admitted adult Chicken death evaluates its Feather pool before its meat pool under sequence
`minecraft:entities/chicken`. The first pool creates Feather, replaces count with uniform integer
`B in 0..2`, then applies Looting count increase. With a living attacking entity and Looting level
`L>0`, it spends a fresh uniform float `U in [0,1)` and adds `round(L*U)`. Absent/nonliving
attacker or level zero skips that draw; only a positive final count emits. Looting can therefore
revive base zero. The later independent meat base/bonus draws remain `ITM-CHICKEN-001`.

An admitted Parrot death makes one roll under sequence `minecraft:entities/parrot`. Its sole entry
uses base `B in 1..2` and the same optional `round(L*U)` increase with a fresh float. Tame state,
variant, fire and killer/player-kill status do not select another Feather branch; entity
death/table admission remains generic.

A tame Cat's relax-on-owner goal can attempt a morning gift after its qualified sleep sequence.
On stop it first requires owner sleep timer at least `100`, then spends one level RNG float against
the live `minecraft:gameplay/cat_waking_up_gift_chance` environment attribute. The normal wake
marker resolves `g=0.7`. A passed chance attempts Cat teleport offsets, ignores teleport failure,
then evaluates `gameplay/cat_morning_gift` at the resulting Cat position.

Feather is one of six weight-`10` entries; Phantom Membrane has weight `2`, for total `62`.
Conditional Feather probability is therefore `10/62 = 5/31`, count one. A qualified stop emits
Feather with probability `5g/31`, or `7/62` at normal `g=0.7`. Chance, teleport and table selection
use distinct RNG sources; the table sequence is `minecraft:gameplay/cat_morning_gift`. Its output
callback spawns the item entity at the Cat block position offset one horizontal unit along body
rotation. Goal scheduling, sleep, timeline/attribute resolution and item-entity insertion remain
with their owners.

### Structure/container acquisition

Three chest tables directly select Feather:

| Table / pool | rolls | Feather weight / pool total | count |
|---|---:|---:|---:|
| chests/shipwreck_map `1` | `3` | `10/38 = 5/19` | `1..5` |
| chests/village/village_fletcher `0` | `1..5` | `6/23` | `1..3` |
| chests/village/village_plains_house `0` | `3..8` | `1/43` | `1` |

Each roll is independent and can select the entry repeatedly. Shipwreck map-room and Village
container placement, seeds, named sequences, capacity and commit remain with their structure/
loot owners. Together with Chicken, Parrot and Cat gift, these are exactly six bundled
Feather-emitting tables. No block, fishing, archaeology, Trial reward, barter, raid gift or other
entity/chest table directly emits Feather.

### Four recipe joins and progression

Feather participates in four recipes:

- Arrow is a shaped vertical Flint/Stick/Feather column, movable to any grid column; it consumes
  one of each and emits four default Arrows.
- Brush is a shaped vertical Feather/Copper-Ingot/Stick column, likewise movable among columns;
  it consumes one of each and emits one default Brush.
- Writable Book is shapeless and consumes one exact Book, Ink Sac and Feather for one default
  Writable Book.
- The always-available special Firework-Star recipe admits at most one exact Feather as its
  `burst` shape ingredient.

The Firework matcher still requires exactly one exact Gunpowder and at least one component-bearing
live `dyes` member, permits at most one shape, one exact Diamond trail and one exact
Glowstone-Dust twinkle input, and rejects all other identities. One Feather sets
`FIREWORK_EXPLOSION.shape=burst`; a second Feather or any second shape rejects the grid. Assembly
consumes it once and preserves the generic row-major colors, empty fade colors and optional
trail/twinkle flags on a default Firework Star.

Arrow, Brush and Writable Book have distinct recipe advancements; Firework Star has none. Feather
possession is one inventory alternative for Arrow only. Brush listens for Copper Ingot and
Writable Book for Book; prior knowledge is the other alternative in each case. Default results do
not copy Feather patches. Pattern normalization, shapeless matching, result capacity, atomic
consumption, component construction and knowledge publication remain generic.

### Fletcher purchase

Baseline `trade_set/fletcher/level_4` selects amount `2` from exactly two predicate-free tagged
candidates, so both become offers and `fletcher/4/feather_emerald` is guaranteed for a newly
populated level-four Fletcher. It wants `24` identity-matching Feather, gives one default Emerald,
has maximum uses `16`, Villager XP `30` and reputation discount coefficient `0.05`, with no second
cost or item/result modifier.

Offer generation consumes no Feather. Successful generic trade validates and consumes the
current adjusted first cost, transfers the Emerald, increments uses and applies merchant/player
effects atomically. Trade Rebalance does not replace this Fletcher record or set. Selection
sequence, price/demand/reputation adjustment, exhaustion, restock and menu synchronization remain
merchant-owned.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. Entities, Cat goal state, containers, recipe
knowledge, Firework Stars and merchant offers persist with their owners. Recipe, advancement,
loot and trade reload changes only future evaluation; completed deaths, gifts, loot, crafts and
offers are not replayed or rewritten. Existing offers retain their constructed costs/results.
Resource reload independently changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `977`; no Feather-specific packet exists. The English name
is `Feather`. It selects one untinted `item/generated` flat with texture `item/feather`. Its item
model additionally fixes the head display transform to rotation `[0,0,45]`, translation
`[-1,13,7]` and scale `[1,1,1]`.

Ingredients orders String, Feather, Snowball, then the three Egg variants. Feather appears once
and in no other ordinary tab. It has no conditional model, tint, animation or special renderer.

**Branches and aborts:**

Default/patched stack; Chicken/Parrot attacker and Looting paths; qualified/unqualified Cat gift;
three chest pools; three ordinary recipes/listeners plus special Firework shape; guaranteed
Fletcher offer; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Feather ID `977`; stack `64`; Chicken `0..2 + round(LU)`, Parrot
`1..2 + round(LU)`; Cat selection `5/31`, combined normal `7/62`; chest rows `3`; emitting tables
`6`; recipes/listeners/direct Feather unlocks `4/3/1`; Arrow output `4`; trade
`24 -> 1`, uses/XP/discount `16/30/0.05`, inclusion `1`; templates/matches `1212/0`; head
rotation/translation/scale `[0,0,45]/[-1,13,7]/[1,1,1]`.

**Side effects:**

Death/gift/container item output; crafted Arrow, Brush, Writable Book or burst Firework Star;
recipe knowledge; merchant Feather consumption/Emerald output; durable stack, entity, container,
offer and Firework state; synchronization and exact client projection.

**Gates:**

Entity/death/adult/loot and living-attacker Looting state; Cat owner/sleep/chance/teleport/table;
structure/table roll; exact grid/dye tag/shape uniqueness/result capacity; profession/level/
trade-set/current cost; registry/stack decode and client resources.

**State read/written:**

Reads all gates above and writes only the death, gift, loot, crafting, advancement, offer,
durable, wire and projection state listed above.

**Failure behavior:**

Rejected death/gift/table selection emits no Feather. Wrong ordinary grid or duplicate/invalid
Firework ingredient emits no result. Rejected or exhausted merchant offers consume nothing.
Reload affects future evaluation only; decode failure follows generic stack policy.

**Boundary cases and quirks:**

Chicken base zero can be revived by Looting; Parrot base cannot be zero. Cat Feather probability
matches the five other weight-`10` rows and differs from Membrane. Feather is an exact
Firework-Star shape selector but has no recipe advancement there. Only Arrow knowledge listens
directly for Feather. References to the distinct `minecraft:feather_falling` enchantment in Trial
Rare and Trade Rebalance data are neither Feather sources nor sinks. The client model is a normal
flat in inventories but carries an explicit head transform.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.crafting.FireworkStarRecipe`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/feather.json`;
`data/minecraft/loot_table/{entities/{chicken,parrot},gameplay/cat_morning_gift,chests/{shipwreck_map,village/{village_fletcher,village_plains_house}}}.json`;
`data/minecraft/recipe/{arrow,brush,firework_star,writable_book}.json`;
`data/minecraft/advancement/recipes/{combat/arrow,tools/brush,misc/writable_book}.json`;
`data/minecraft/{villager_trade/fletcher/4/feather_emerald,tags/villager_trade/fletcher/level_4,trade_set/fletcher/level_4}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/feather.*`;
`assets/minecraft/lang/en_us.json`;
`ITM-CHICKEN-001`; `ITM-RECIPE-SERIALIZER-001`;
`WGEN-STRUCTURE-SHIPWRECK-001`; `WGEN-JIGSAW-VILLAGES-001`;
`EXP-ITM-085`.

**Test vectors:**

Run `EXP-ITM-085` across default/patched Feather, admitted/rejected Chicken and Parrot deaths with
every attacker/Looting boundary, qualified Cat sleep/chance/teleport/gift paths, all three chest
rows, all four recipes and three listeners, the Fletcher set/offer and every template. Reload each
domain, persist/synchronize all owners and assert item ID, name, flat/head model and Ingredients
order.

**Limits:**

Generic entity death, Cat AI/timeline, loot, structure, crafting, special Firework-Star, merchant,
packet and renderer control flow remains with cited owners. Chicken meat behavior remains
`ITM-CHICKEN-001`; Arrow, Brush, Writable Book, Firework Star and Emerald behavior retain their
own owners. This leaf fixes the exact Feather identity, source/sink joins, absences and projection.
