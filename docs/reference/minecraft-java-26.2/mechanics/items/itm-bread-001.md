# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BREAD-001` — Bread joins broad chest and Trial loot, two Farmer outputs, composting and code-built Villager food accounting

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`,
`ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-005`, `MOB-AI-001`,
`MOB-BREED-001`, `MOB-RAID-001`, `BLK-TRIAL-SPAWNER-001`,
`WGEN-PIPELINE-001`, `WGEN-STRUCTURE-STRONGHOLD-001`,
`WGEN-STRUCTURE-MINESHAFT-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`,
`WGEN-JIGSAW-VILLAGES-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, shaped recipe, eighteen chest records, normal
Trial consumables, Farmer gift/trade data, the direct pickup tag, code-built Composter chance,
Villager food accounting/sharing/breeding and Farmer workstation bytecode, advancements and
direct client resources determine every Bread-specific branch. Generic use, crafting, loot,
structures, Trial encounters, merchants, Villager scheduling/inventory, breeding, blocks, stacks
and client algorithms remain with the cited owners.

**Applies when:**

A `bread` stack is crafted by a player or Farmer, emitted by chest/Trial/gift loot or a Farmer
offer, eaten, inserted into a Composter, picked up/held/shared/digested by a Villager, used toward
Villager breeding, moved, renamed, persisted, synchronized or rendered before and after component,
tag, recipe, loot, trade, advancement or resource reload.

**Authoritative state:**

`minecraft:bread` is raw item ID `981`. It is a common nondamageable plain `Item` with maximum
stack `64`, food nutrition `5` and saturation `6.0`, and the ordinary empty `32`-tick eat
consumable with no consume-effect entries or remainder.

Its other default components are the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. Its sole direct item tag is `villager_picks_up`, reached alongside the recursively
included plantable seeds, Wheat and Beetroot.

Composter bootstrap separately maps Bread to Java float chance `0.85f`. Villager code separately
maps Bread identity to `4` food points. Neither code-built map consults the registered food/
consumable components or the pickup tag.

**Transition and ordering:**

Player consumption and progression:

In-air use enters generic consumption only below full hunger or when ability permits full-hunger
eating. Interruption or live-hand/component replacement before completion commits nothing.
Successful server completion emits eat effects, awards the statistic, triggers `consume_item`
against the live pre-shrink stack, applies food, runs the empty effect list, emits `EAT` and
shrinks one unless materials are infinite.

Default Bread adds `5` food and `6.0` saturation subject to generic clamps, spends no
consume-effect RNG and leaves no remainder. Bread is one independent requirement of
telemetry-enabled `husbandry/balanced_diet`; all `40` foods award `100` experience.

Player crafting and recipe progression:

The sole recipe is a shaped one-row pattern `###`, where `#` is exact Wheat. It fits any one row
of a `3×3` Crafting Table, is horizontally symmetric, does not fit a `2×2` grid and rejects any
extra occupied cell. A match consumes one Wheat from each of the three cells and emits one
default Bread; input patches do not propagate and Wheat has no remainder.

The no-display recipe advancement has one OR requirement: possessing Wheat or already knowing
`minecraft:bread` grants the recipe. Recipe reload changes future player/grid matching and
output, but not the Farmer's hard-coded workstation conversion below.

Chest acquisition:

Every listed Bread entry is unconditional once its pool runs. Rolls select with replacement,
each selection emits a default stack, and the row's count function replaces its count:

| Table and pool | Rolls | Bread weight / total | Count |
|---|---:|---:|---:|
| `chests/abandoned_mineshaft`, 1 | `2..4` | `15/98` | `1..3` |
| `chests/simple_dungeon`, 1 | `1..4` | `20/125 = 4/25` | `1` |
| `chests/spawn_bonus_chest`, 2 | `3` | `3/11` | `1..2` |
| `chests/stronghold_corridor`, 0 | `2..3` | `15/101` | `1..3` |
| `chests/stronghold_crossing`, 0 | `1..4` | `15/62` | `1..3` |
| `chests/woodland_mansion`, 1 | `1..4` | `20/175 = 4/35` | `1` |
| `chests/village/village_armorer`, 0 | `1..5` | `4/8 = 1/2` | `1..4` |
| `chests/village/village_cartographer`, 0 | `1..5` | `15/50 = 3/10` | `1..4` |
| `chests/village/village_desert_house`, 0 | `3..8` | `10/36 = 5/18` | `1..4` |
| `chests/village/village_mason`, 0 | `1..5` | `4/13` | `1..4` |
| `chests/village/village_plains_house`, 0 | `3..8` | `10/43` | `1..4` |
| `chests/village/village_savanna_house`, 0 | `3..8` | `10/46 = 5/23` | `1..4` |
| `chests/village/village_snowy_house`, 0 | `3..8` | `10/53` | `1..4` |
| `chests/village/village_taiga_house`, 0 | `3..8` | `10/54 = 5/27` | `1..4` |
| `chests/village/village_tannery`, 0 | `1..5` | `5/16` | `1..4` |
| `chests/village/village_temple`, 0 | `3..8` | `7/19` | `1..4` |
| `chests/village/village_toolsmith`, 0 | `3..8` | `15/53` | `1..3` |
| `chests/village/village_weaponsmith`, 0 | `3..8` | `15/107` | `1..3` |

Each table uses its same-named `minecraft:` random sequence. The optional Trade Rebalance pack
replaces `abandoned_mineshaft`; its Bread pool retains the exact `2..4`, `15/98`, `1..3` row,
although the replacement adds separate later work. Structure starts/templates, chest placement,
lazy seed assignment, table invocation, output shuffling and insertion remain with the loot and
worldgen owners.

Normal Trial-Spawner acquisition:

All `14` normal Trial-Chamber configurations omit `loot_tables_to_eject` and inherit the default
equal-weight normal consumables/key list. The table choice is fixed when the encounter's first
registered-player reward becomes due and reused for remaining registered UUIDs. Normal
consumables therefore has probability `1/2`.

`spawners/trial_chamber/consumables` then makes one roll over total weight `10`: Cooked Chicken
`3`, Bread `3`, Baked Potato `2`, Regeneration Potion `1` and Swiftness Potion `1`. Conditional
Bread probability is `3/10`, count is uniform `1..3`, and the sequence is
`minecraft:spawners/trial_chamber/consumables`. Marginal probability per registered-player
evaluation is `3/20`; multiple players are correlated through the encounter's shared table
choice. Ominous configurations use a distinct consumables table without Bread.

Farmer offer and Hero gift:

Farmer level one has five predicate-free records and selects two without replacement, so
`farmer/1/emerald_bread` has inclusion probability `2/5` under
`minecraft:trade_set/farmer/level_1`. It consumes one Emerald and gives six default Bread, with
maximum uses `16`, default villager XP `1` and reputation discount `0.05`. There is no second
cost, predicate, output modifier or double-price enchantment. Trade Rebalance replaces neither
the tag nor record.

An admitted adult Farmer Hero gift chooses uniformly among one default Bread, Pumpkin Pie and
Cookie. Bread probability is `1/3` under
`minecraft:gameplay/hero_of_the_village/farmer_gift`. Initial eligible cooldown is `600`; later
cooldown is `600 + nextInt(6001)`, target range is five blocks, behavior lasts at most `100`
ticks and throws only after elapsed time exceeds `20`. Gift admission/navigation/throw/cleanup
remain `MOB-RAID-001`; offer economy and menu work remain merchant-owned.

Farmer workstation crafting:

When Farmer work reaches a still-present job-site Composter, `WorkAtComposter` calls `makeBread`
before its seed-composting pass. Bread conversion ignores recipe data, advancement state and
Composter level:

1. count every Bread identity in the eight-slot Villager inventory; if the total is strictly
   greater than `36`, return;
2. count Wheat and set `batches = min(3, floor(wheat/3))`; zero returns;
3. remove `3*batches` Wheat by identity across the inventory;
4. construct one default Bread stack of count `batches` and add it to the inventory;
5. spawn any uninserted remainder at the Farmer with offset `0.5`.

Bread count exactly `36` therefore still permits three batches and can become `39`; later work
stops. Count tests include arbitrarily patched Bread/Wheat identities, while the constructed
Bread is default and does not inherit patches. The inventory add can merge only compatible
stacks; full or patch-incompatible storage drops the remainder after Wheat has already been
removed.

The following `compostItems` pass recognizes only Wheat Seeds and Beetroot Seeds, not Bread.
Thus a Farmer crafts Bread at this workstation but never inserts its Bread into the Composter,
despite direct/automated Composter admission below.

Villager pickup and food accounting:

A Villager wants a ground Bread stack when the live stack is in `villager_picks_up` and its
inventory can add at least part of it. Generic wanted-item navigation, `mobGriefing`, reach,
pickup delay/ownership, partial insertion and item-entity cleanup gates remain `MOB-AI-001`.
Tag reload can remove future pickup admission; already held Bread remains food.

Every held Bread identity contributes `4 * count` code-built food points regardless components.
`wantsMoreFood` is true below `12` inventory points; `hasExcessFood` is true at or above `24`.
`canBreed` requires persisted hidden `foodLevel + inventory points >= 12`, an awake Villager and
age exactly zero. Starting from hidden level zero, three Bread satisfy the threshold.

When an admitted breeding behavior reaches its birth timestamp, each parent first consumes
inventory food in slot order until hidden `foodLevel >= 12`, adding four and removing one Bread
per step, then subtracts exactly `12`. Overshoot remains hidden; for example a starting hidden
level `1` consumes three Bread, reaches `13`, and retains `1`. This work runs for both parents
before the behavior searches/commits the child bed and spawn, so bed or offspring failure does
not refund food. Villager Bread digestion never executes the item consumable, applies player
nutrition or reads patched food.

Villager food sharing:

During close Villager interaction, a source with inventory food points at least `24` shares from
the four code-built food identities when either the source is a Farmer or the recipient has fewer
than `12` inventory points. The helper scans inventory slots and stops at the first matching stack
that passes one of these count gates:

- count above half that stack's live maximum throws `floor(count/2)`;
- otherwise count above `24` throws `count-24`;
- count `24` or lower throws nothing.

For default max `64`, a Bread stack `25..32` throws `1..8`; `33..64` throws `16..32`. A source
can therefore satisfy `hasExcessFood` with only six Bread—or many stacks totaling enough
points—yet throw none when no individual matching stack exceeds `24`.

The source stack shrinks, but the thrown stack is reconstructed from item identity and count, so
all source Bread patches are discarded. Generic throw motion, item pickup and recipient inventory
then apply. Farmer-only Wheat sharing and mutually requested profession-item sharing are separate
later branches.

Composter insertion:

Player-held insertion at level `0` succeeds without RNG. Levels `1..6` consume one
`nextDouble()` and increment exactly when it is strictly below Java float `0.85f`, widened to
`0.8500000238418579` for comparison. Success writes level plus one with flags `3`, emits
`BLOCK_CHANGE`, and `6 -> 7` schedules maturation after `20` ticks; failure preserves state.

Either level-`0..6` result emits level event `1500` with success encoded by whether state changed,
awards the Bread-used statistic and calls `consume(1, player)`, preserving infinite-material
holders. Level `7` returns success for held Bread without insertion, event, statistic or
consumption. Level `8` delegates to ordinary item-on-block handling.

Automation exposes one top input slot only below level `7`. It accepts a compostable stack once,
runs the same deterministic-first-level/strict-double transition, emits event `1500`, and removes
the one-slot stack whether chance succeeded or failed. Direct insertion likewise shrinks one after
every admitted level-`0..6` attempt. Maturation, Bone-Meal extraction and event rendering remain
with the Composter/block/client owners.

**Persistence and reload boundary:**

Bread stacks persist identity, count and patches. A Villager separately persists its eight-slot
inventory and hidden `FoodLevel` as a signed byte; ordinary reachable Bread accounting keeps it
bounded around the threshold. Player active-use/hunger, grid, loot cursor, structure/Trial,
merchant, gift, AI memory, breeding/bed and Composter state persist with their owners.

Recipe reload changes player crafting but not Farmer workstation conversion. Loot reload changes
future chest/Trial/gift evaluation. Tag reload changes future ground pickup only; code-built
food points, workstation crafting and compost chance do not reload. Trade and advancement reload
change future offers/listeners. Completed work is not replayed. Resource reload independently
controls name/model/texture and advancement presentation.

**Client and wire projection:**

Generic stack encoding projects raw ID `981` plus patches. The locked English name is `Bread`;
it is common with no forced glint or subtype tooltip. Its direct item definition selects the
ordinary generated `item/bread` model and same-named texture.

Bread appears exactly once in Food & Drinks, ordered Pufferfish, Bread, Cookie, Cake.

**Branches and aborts:**

Identity/count/components/direct pickup tag; player use; shaped crafting/unlock; eighteen chest
tables and optional replacement; normal Trial correlation; Farmer offer/gift/workstation paths;
Villager pickup/food/share/breed state; Composter direct/automation; Balanced Diet, persistence,
reload, wire, model and tab.

**Constants and randomness:**

Raw ID `981`; max `64`; player food `5/6.0`; eat `32`; recipe `3→1`; eighteen chest rows above;
normal Trial table `1/2`, Bread `3/10`, count `1..3`, marginal `3/20`; Farmer trade inclusion
`2/5`, `1→6`, uses/XP `16/1`; gift `1/3`; workstation guard `>36`, batches
`min(3,floor(wheat/3))`; Villager Bread points `4`, wants/excess/breed thresholds `12/24/12`;
sharing stack thresholds `max/2` then `24`; Composter `0.8500000238418579`, maturation `20`.

**Side effects:**

Player food/use/progression; crafting output/unlock; chest/Trial/gift loot and cursors; merchant
offer/result/economy; Farmer inventory conversion/drop; Villager item pickup/inventory/hidden food/
sharing/breeding; Composter level/event/stat/consumption/schedule; persistence, wire and client
projection.

**Gates:**

Player hunger/use; exact grid/recipe; loot/structure/Trial/gift admission; Farmer level/set/offer
and job-site work; Villager tag/inventory/food/profession/recipient/breed/bed state; Composter
level/chance/side; progression listeners; registry/decode/client bootstrap.

**State read/written:**

Reads Bread stack/components, player state, grid/recipe, loot/structure/Trial, merchant/gift,
Farmer job/inventory, Villager pickup/inventory/food/breed state, Composter state/RNG and client
resources. Writes only the consumption, crafting, loot, trade, AI/inventory, breeding, compost and
projection state listed above.

**Failure behavior:**

Unadmitted use commits nothing. Invalid grids do not craft. Unselected loot rows emit alternatives.
Invalid/exhausted offers commit nothing. Missing job-site Composter or Bread count above `36`
prevents Farmer conversion; zero batches prevent removal; failed insertion after conversion drops
the result rather than refunding Wheat. Full Villager inventory rejects pickup. Sharing can pass
the excess-food gate yet throw nothing. Breeding failure after the birth timestamp does not refund
digested food. Composter chance failure still consumes an admitted level-`1..6` input.

**Boundary cases and quirks:**

Farmer workstation crafting is a second recipe-independent `3 Wheat -> 1 Bread` path, runs before
seed composting and overshoots `36` to at most `39`. Villagers value Bread at four points rather
than its player nutrition five, count patched stacks by identity and never invoke their
consumables. Six Bread satisfy excess-food accounting but no default stack of six is shareable;
sharing rebuilds a default stack and loses patches. Parents digest before bed/child success.
Composter level zero is deterministic while a failed later attempt still consumes Bread.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.entity.npc.villager.Villager#canBreed`;
`net.minecraft.world.entity.npc.villager.Villager#wantsToPickUp`;
`net.minecraft.world.entity.npc.villager.Villager#hasExcessFood`;
`net.minecraft.world.entity.npc.villager.Villager#wantsMoreFood`;
`net.minecraft.world.entity.npc.villager.Villager#countFoodPointsInInventory`;
`net.minecraft.world.entity.npc.villager.Villager#eatAndDigestFood`;
`net.minecraft.world.entity.ai.behavior.WorkAtComposter#makeBread`;
`net.minecraft.world.entity.ai.behavior.WorkAtComposter#compostItems`;
`net.minecraft.world.entity.ai.behavior.TradeWithVillager#throwHalfStack`;
`net.minecraft.world.entity.ai.behavior.VillagerMakeLove#tick`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.TradeRebalanceChestLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement,villager_trade,trade_set,trial_spawner_config}`;
`reports/minecraft/components/item/bread.json`;
`data/minecraft/{recipe/bread,advancement/recipes/food/bread}.json`;
`data/minecraft/tags/item/villager_picks_up.json`;
`data/minecraft/loot_table/chests/{abandoned_mineshaft,simple_dungeon,spawn_bonus_chest,stronghold_corridor,stronghold_crossing,woodland_mansion}.json`;
`data/minecraft/loot_table/chests/village/{village_armorer,village_cartographer,village_desert_house,village_mason,village_plains_house,village_savanna_house,village_snowy_house,village_taiga_house,village_tannery,village_temple,village_toolsmith,village_weaponsmith}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/abandoned_mineshaft.json`;
`data/minecraft/loot_table/{spawners/trial_chamber/consumables,gameplay/hero_of_the_village/farmer_gift}.json`;
`data/minecraft/trial_spawner/trial_chamber/**/normal.json`;
`data/minecraft/{villager_trade/farmer/1/emerald_bread,tags/villager_trade/farmer/level_1,trade_set/farmer/level_1}.json`;
`data/minecraft/advancement/husbandry/balanced_diet.json`;
`assets/minecraft/{items,models/item,textures/item}/bread.*`;
`ITM-CRAFT-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`;
`MOB-AI-001`; `MOB-BREED-001`; `MOB-RAID-001`; `BLK-TRIAL-SPAWNER-001`;
`WGEN-PIPELINE-001`; `WGEN-STRUCTURE-STRONGHOLD-001`;
`WGEN-STRUCTURE-MINESHAFT-001`; `WGEN-STRUCTURE-WOODLAND-MANSION-001`;
`WGEN-JIGSAW-VILLAGES-001`; `WGEN-JIGSAW-TRIAL-CHAMBERS-001`; `CLI-UI-001`;
`CLI-EFFECT-001`; `EXP-ITM-071`.

**Test vectors:**

Exercise default/removed/patched stacks through use; every offset/extra/wrong Wheat grid and
unlock route; every Bread chest row under baseline/Trade-Rebalance tables; all normal Trial
configurations/registered-player correlations; Farmer offers and gifts.

Drive Farmer Composter work across Bread counts `35..40`, Wheat `0..12`, full/compatible/
patch-incompatible inventories and every Composter level. Trace exact removal, default result,
remainder drop and subsequent seed-only compost pass before/after recipe reload.

Drop default/patched Bread around every Villager profession with tag/rule/inventory boundaries.
Cross hidden/inventory food points around `12/24`, every sharing stack count around `24/32`,
Farmer/recipient state and full breeding/bed/offspring success/failure. Verify patch loss on
sharing and no consumable execution.

Exercise every Composter level and below/equal/above chance draw on held and automated paths.
Reload all domains, persist/reload Villager food/inventory and stacks, synchronize, then verify
raw ID, name, ordinary generated model/texture and exact Food-tab neighborhood.
