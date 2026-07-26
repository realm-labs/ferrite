# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-STRING-001` — String couples a placeable Tripwire network to block, mob, fishing, archaeology, barter, chest, crafting and villager paths

**Parent:** `SIM-003`, `SIM-004`, `SIM-005`, `SIM-SCHEDULE-001`,
`SIM-RANDOM-001`, `BLK-001`, `BLK-STATE-001`, `BLK-002`, `BLK-003`,
`BLK-004`, `BLK-005`, `BLK-007`, `BLK-UPDATE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`, `ITM-001`,
`ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-KNOCKBACK-001`, `MOB-001`, `MOB-004`,
`MOB-AI-001`, `RED-001`, `RED-UPDATE-001`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, Tripwire/Hook classes, block states, all exact-item loot,
recipe, advancement and trade records, all `1,212` decoded templates and exact client resources
determine every String-specific branch. Generic breaking, loot, fishing, archaeology, crafting,
merchant, structure, persistence, packet, neighbor-update and rendering algorithms retain their
cited owners.

**Applies when:**

`minecraft:string` is placed as Tripwire, acquired from block/entity/container/gift/barter/
fishing/archaeology loot, consumed by crafting or villager trade, moved, persisted, synchronized
or rendered before and after block, loot, recipe, advancement, trade, tag or resource reload.

**Authoritative state:**

String is raw item ID `976`, a common nondamageable maximum-`64` block item registered from
`minecraft:tripwire`, raw block ID `402`, with a custom item name. It has no direct item tag and
no food, consumable, fuel, compost, repair, equipment, durability, projectile, cooldown,
inventory-tick or intrinsic use component. Item placement delegates to the generic block-item
owner.

Tripwire states occupy protocol IDs `9599..9726` and are the Cartesian product of seven booleans:
`attached`, `disarmed`, `east`, `north`, `powered`, `south` and `west`. All default false, so the
default state is `9726`; there are `128` states. The block is no-collision, is destroyed by
pistons and belongs directly to `wall_post_override`.

**Transition and ordering:**

### Placement, contact and scheduled rescan

Placement independently sets each horizontal connection. A side connects to another Tripwire or
to a Tripwire Hook whose facing is opposite that side. Horizontal neighbor updates recompute only
the corresponding connection bit. Rotation and mirror permute the four direction bits. Attached
wire has a full-width shape from Y `1` to `2.5`; unattached wire spans Y `0` to `8`. The same
shape bounds contact scanning, but neither shape collides.

On placement or state change, the wire scans only South and West for its source Hook. Each scan
checks distances `1..41`, stops at a non-Tripwire or wrong-facing Hook and asks the first matching
Hook to recalculate. On removal, the old state is first supplied as powered. Shears additionally
write `disarmed=true` with flags `260` and emit the shear game event before generic removal.

Entity contact acts only on the server. An already-powered wire or one with a pending scheduled
tick does nothing. Otherwise at least one entity intersecting the current shape must not ignore
block triggers. A power transition writes with flags `3` and recalculates the Hook source. A
pressed wire schedules a `10`-tick rescan; release schedules a zero-delay tick. The scheduled
callback rescans only if the live wire is still powered.

### Two-ended Hook transaction

A Hook scans forward through positions `1..41`. It accepts an opposite-facing Hook only at
distance `2..41`, so a valid line contains `1..40` intervening wires. Attachment requires that
the changed line was not removed and every relevant wire is not disarmed. Power is the OR of
undisarmed powered wires, then is suppressed unless the line attaches. The supplied changed state
substitutes at its position and schedules a Hook tick after `10` ticks.

Recalculation writes the opposite Hook with reverse facing and the computed attached/powered bits,
notifies its neighbors, writes the originating Hook unless that Hook is being removed, and, when
attachment changes, rewrites the attached bit of every still-present intermediate Tripwire or
Hook. State writes use flags `3`. Each end emits the corresponding sound/game event: activation
volume/pitch `0.4/0.6`, deactivation `0.4/0.5`, attachment `0.4/0.7`, or detachment volume `0.4`
and pitch `1.2 / (random*0.2 + 0.9)`.

A powered Hook emits weak signal `15` on every side and direct signal `15` only toward its facing;
otherwise both are zero. Hook and generic redstone owners retain neighbor convergence and signal
consumer ordering.

### Block and entity acquisition

Cobweb has one alternatives pool. Exact Shears or Silk Touch level at least one emits one Cobweb;
otherwise it emits one String subject to per-unit explosion decay. Tripwire emits one String
subject to explosion survival regardless of tool or state. Shears disarming affects the Hook
network, not that loot result. Neither table emits XP.

Four death tables directly emit String without a killed-by-player condition:

| Entity | base count | living-attacker Looting |
|---|---:|---:|
| Cat | `0..2` | none |
| Cave Spider | `0..2` | `round(LU)` |
| Spider | `0..2` | `round(LU)` |
| Strider | `2..5` | `round(LU)` |

For Looting, `U` is independently uniform in `[0,1]`. Spider Eye pools are separate and do not
gate String.

### Gift, barter, fishing, archaeology and chests

Cat morning gift selects String at weight `10` from total `62`, hence `5/31`, and emits one.
Piglin bartering selects it at weight `20` from total `469` and sets count uniformly to `3..9`.
The parent fishing router chooses among junk `10/-2`, open-water treasure `5/+2` and fish
`85/-1`; generic fishing owns quality. In junk, String has weight `5`: unconditional total `100`
gives `1/20`, while Jungle, Sparse Jungle or Bamboo Jungle admits weight-`10` Bamboo and gives
`1/22`. Trail-Ruins common archaeology has total weight `45`; String has weight `1` and emits one.

Seven baseline chest pools directly select String:

| Table / pool | rolls | String weight / pool total | count |
|---|---:|---:|---:|
| Bastion Bridge `2` | `2..4` | `1/5` | `1..6` |
| Bastion Hoglin Stable `1` | `3..4` | `1/14` | `3..8` |
| Bastion Other `2` | `3..4` | `1/13` | `4..6` |
| Desert Pyramid `1` | `4` | `10/50 = 1/5` | `1..8` |
| Pillager Outpost `3` | `2..3` | `4/22 = 2/11` | `1..6` |
| Simple Dungeon `2` | `3` | `10/40 = 1/4` | `1..8` |
| Woodland Mansion `2` | `3` | `10/40 = 1/4` | `1..8` |

Trade Rebalance replaces the Desert-Pyramid and Pillager-Outpost tables but preserves these
String rows exactly. Counting each live baseline identity once, the two blocks, four entities,
gift, barter, junk, archaeology and seven chests are exactly `17` direct tables. The parent
fishing table is an indirect router.

### Nine crafting sinks and asymmetric unlocks

Exactly nine shaped recipes consume String: Bow (`3`), Bundle (`1`), Candle (`1`), Crossbow (`2`),
Fishing Rod (`2`), Lead (`5` to emit `2`), Loom (`2`), Scaffolding (`1` to emit `6`) and
four-String White Wool (`4`). All other results emit one. No recipe produces String.

Exactly nine recipe advancements listen directly for String as an OR alternative to prior recipe
knowledge: Bow, Bundle, Candle, Crossbow, Fishing Rod, Lead, Loom, Tripwire Hook and White Wool.
Scaffolding consumes String but does not listen for it; Tripwire Hook listens for String but does
not consume it. Pattern normalization, mirroring, live tag lookup, output capacity, atomic
consumption, remainder handling and knowledge publication remain generic.

### Two villager purchases and one stored stack

Baseline Fisherman level one selects amount two without replacement from four candidates, so its
`20` String to one Emerald offer has inclusion probability `1/2`; maximum uses/XP/discount are
`16/2/0.05`. Fletcher level three selects amount two from exactly two candidates, so its `14`
String to one Emerald offer is guaranteed; maximum uses/XP/discount are `16/20/0.05`. Trade
Rebalance replaces neither record, tag nor set. Generic trading owns adjusted costs and the
atomic exchange.

An exhaustive decoded scan finds exactly one String identity in all `1,212` structure templates:
`trial_chambers/intersection/intersection_2.nbt` stores count `3` as the item inside one Decorated
Pot whose four sherd entries are three Bricks and one Flow Pottery Sherd. Pot placement, break and
stored-item behavior retain their existing owners. No other template stores String.

**Persistence and reload boundary:**

Block states, scheduled ticks, stacks, containers, recipe knowledge and merchant offers persist
with their owners. Loot, recipe, advancement, trade and tag reload changes future evaluation or
offer construction only; it does not replay completed contact, propagation, breaks, deaths,
fishing, archaeology, crafts or trades. Existing offers retain constructed state. Resource
reload independently changes projection only.

**Wire and client projection:**

Generic publication uses item ID `976`, block ID `402` and the exact block state. No
String-specific packet exists. English names are `String` for the item and `Tripwire` for the
block. The item definition selects one untinted `item/generated` model and
`minecraft:item/string` texture with no condition, animation, tint or special renderer.

String appears in Ingredients between Bone Meal and Feather and in Redstone Blocks between
Tripwire Hook and Lectern. It appears in no other ordinary creative tab. Tripwire blockstate
projection ignores `powered` and `disarmed`; `attached` plus the four directional bits select
`32` variants from the attached/unattached `n`, `ne`, `ns`, `nse` and `nsew` model families with
rotations.

**Branches and aborts:**

Default/patched String; each placement side; attached/disarmed/powered/directional state;
qualifying/ignored entities; pending/no tick; lines of zero, `1..40` or overlength wires;
opposite/misfacing/no Hook; Shears/ordinary removal; Cobweb Silk/Shears/explosion; four death
tables and Looting; gift/barter/fishing biome/archaeology/chests; nine recipes and nine direct
unlocks; both professions selected/unselected/current-cost; one template; persistence/reload/
wire/client branches are distinct.

**Constants and randomness:**

Item/block IDs `976/402`; stack `64`; state IDs `9599..9726`, booleans/states `7/128`; contact and
Hook ticks `10`; Hook scan distance `1..41`, valid wires `1..40`; signals `15`; Cat/Spider/
Cave-Spider/Strider base `0..2/0..2/0..2/2..5`; gift `5/31`; barter `20/469`, count `3..9`;
junk `1/20` or `1/22`; archaeology `1/45`; chest rows/direct tables `7/17`; recipes/direct
unlocks `9/9`; Fisherman inclusion/exchange `1/2`, `20:1`; Fletcher `1`, `14:1`;
templates/matches/stored count `1212/1/3`; Tripwire client variants `32`.

**Side effects:**

Tripwire/Hook state, ticks, neighbor notifications, sounds, game events and redstone output;
block/entity/container/gift/barter/fishing/archaeology loot; crafted results and knowledge;
villager input/output, uses, XP and economy effects; stored pot item; durable owner state,
synchronization and exact client projection.

**Gates:**

Server side/entity trigger/pending tick; connection direction/Hook facing/line length/disarmed;
tool/Shears/Silk/explosion; attacker/Looting; fishing quality/open-water/biome; loot roll/weight/
count; brush completion; shaped grid/live tags/result capacity; advancement inventory/knowledge;
profession/level/set/current cost; registry/stack/block-state decode and client resources.

**Boundary cases and quirks:**

The String item is a custom-named Tripwire block item despite using a flat generated model.
Tripwire scans only South and West because the two directions canonically find each line once.
Lines need at least one wire and accept at most `40`; disarmed wire prevents attachment. A release
schedules zero-delay work while continued pressure schedules a `10`-tick rescan. Shears disarm
before removal but do not suppress String loot. Fishing's Jungle-only Bamboo changes the junk
denominator. Scaffolding and Tripwire Hook form reciprocal recipe/unlock exceptions. One template
stores String directly even though all chest sources are loot-table joins.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.TripWireBlock`;
`net.minecraft.world.level.block.TripWireHookBlock`;
`net.minecraft.world.level.block.state.properties.BlockStateProperties`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/blocks.json#minecraft:tripwire`;
`reports/minecraft/components/item/string.json`;
`data/minecraft/tags/block/wall_post_override.json`;
`data/minecraft/loot_table/{blocks/{cobweb,tripwire},entities/{cat,cave_spider,spider,strider},gameplay/{cat_morning_gift,piglin_bartering,fishing,fishing/junk},archaeology/trail_ruins_common,chests/{bastion_bridge,bastion_hoglin_stable,bastion_other,desert_pyramid,pillager_outpost,simple_dungeon,woodland_mansion}}.json`;
`data/minecraft/overlay/trade_rebalance/loot_table/chests/{desert_pyramid,pillager_outpost}.json`;
`data/minecraft/recipe/{bow,bundle,candle,crossbow,fishing_rod,lead,loom,scaffolding,white_wool_from_string}.json`;
`data/minecraft/advancement/recipes/**/*.json`;
`data/minecraft/{villager_trade/{fisherman/1/string_emerald,fletcher/3/string_emerald},tags/villager_trade/{fisherman/level_1,fletcher/level_3},trade_set/{fisherman/level_1,fletcher/level_3}}.json`;
`data/minecraft/structure/trial_chambers/intersection/intersection_2.nbt`;
`assets/minecraft/{items/string.json,models/item/string.json,textures/item/string.png,blockstates/tripwire.json,models/block/tripwire*.json,lang/en_us.json}`;
`ITM-RECIPE-SERIALIZER-001`; `EXP-ITM-095`.

**Test vectors:**

Run `EXP-ITM-095` across every String/Tripwire state, placement, entity, tick, Hook-line,
disarming, signal and break branch; every direct loot row and fishing router; all nine recipes,
nine direct unlocks and both trades. Scan every template, persist/reload/synchronize owners and
assert IDs, names, item model, two creative positions and all `32` Tripwire projection variants.

**Limits:**

Generic block placement/breaking, collision, loot, fishing, archaeology, crafting, merchant,
structure, persistence, packet, redstone-convergence and renderer control flow remains with cited
owners. Hooks, Cobweb, mobs, structures, Decorated Pot and every crafted result retain dedicated
owners. This leaf fixes the exact String/Tripwire joins, coupled network, absences and projection.
