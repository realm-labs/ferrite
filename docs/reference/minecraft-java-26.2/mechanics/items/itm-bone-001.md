# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BONE-001` — Bones join six skeletal deaths, eight chest records and fishing junk to Wolf taming, begging and Bone Meal crafting

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`,
`ENT-005`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-PROJECTILE-001`,
`MOB-AI-001`, `MOB-BREED-001`,
`WGEN-PIPELINE-001`, `WGEN-STRUCTURE-DESERT-PYRAMID-001`,
`WGEN-STRUCTURE-JUNGLE-TEMPLE-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`,
`WGEN-JIGSAW-ANCIENT-CITY-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, the complete exact-item class and data
reference set, six entity tables, five base chest tables and three optional Trade-Rebalance
replacements, the fishing junk entry, one recipe/unlock, exact Wolf interaction and BegGoal
branches, advancement records and direct client resources determine every Bone-specific branch.
Generic death, loot, fishing, structure/container, crafting, progression, mob AI, stack and client
behavior remains with the cited owners.

**Applies when:**

A `bone` stack is emitted by a skeletal death, chest or fishing table, held near or used on a
Wolf, consumed by the Bone-Meal recipe, moved, renamed, persisted, synchronized, rendered or
observed before and after loot, recipe, advancement, data-pack or resource reload.

**Authoritative state:**

`minecraft:bone` is raw item ID `1112`. It is a common, nondamageable plain `Item` with maximum
stack `64`. Its registered components are the common empty attribute modifiers, enchantments and
lore, item-break sound, translated name, direct item-model key, repair cost, swing animation,
tooltip display and use effects. It has no food, consumable, cooldown, use remainder, tool,
equipment, repairable, fuel or identity-specific glint state.

Bone has no direct item-tag membership. In particular, it is not in `wolf_food`: Wolf taming and
BegGoal test its exact item identity in code. Arbitrary ordinary component patches do not change
that identity test.

**Transition and ordering:**

Skeletal death acquisition:

Six entity tables contain a one-roll Bone pool:

- `entities/bogged`, `entities/parched`, `entities/skeleton` and `entities/stray` evaluate an
  Arrow pool first, then the Bone pool;
- `entities/wither_skeleton` evaluates Coal first, Bone second and its player-gated skull pool
  last;
- `entities/skeleton_horse` contains only the Bone pool.

The Bone entry always creates a default stack, replaces its count with a uniform integer `0..2`,
then applies enchanted count increase. With a living attacking entity and Looting level `L>0`,
that function spends a fresh float `U` in `[0,1)` and adds `round(L*U)`; absent/nonliving attacker
or level zero returns without that draw. Effective count is therefore
`B + round(L*U)` for `B in 0..2`, without an explicit final limit. Base zero emits nothing unless
the Looting bonus revives it.

No killed-by-player condition gates these Bone pools. Baby/death-rule/table admission, attack
context, equipment drops, output insertion and removal paths that bypass ordinary death remain
with the entity and loot owners. The complete named sequences are
`minecraft:entities/{bogged,parched,skeleton_horse,skeleton,stray,wither_skeleton}`; earlier and
later pools retain their stated order on the same table cursor.

Chest acquisition:

Every admitted Bone entry creates a default stack, replaces its count and permits repeated
selections. The standard pack contains:

| Table and pool | Rolls | Bone probability per roll | Count |
|---|---:|---:|---:|
| `chests/ancient_city`, pool 0 | uniform `5..10` | `5/84` | uniform `1..15` |
| `chests/desert_pyramid`, pool 0 | uniform `2..4` | `25/247` | uniform `4..6` |
| `chests/desert_pyramid`, pool 1 | `4` | `10/50 = 1/5` | uniform `1..8` |
| `chests/jungle_temple`, pool 0 | uniform `2..6` | `20/89` | uniform `4..6` |
| `chests/simple_dungeon`, pool 2 | `3` | `10/40 = 1/4` | uniform `1..8` |
| `chests/woodland_mansion`, pool 2 | `3` | `10/40 = 1/4` | uniform `1..8` |

Desert pool 1 selects uniformly by weight among Bone, Gunpowder, Rotten Flesh, String and Sand.
The Simple-Dungeon and Woodland-Mansion Bone pools omit Sand and select among the other four.
Ancient City has a later trim pool; Desert Pyramid has its four-roll equal-junk pool after the
first pool and a later trim pool; Jungle Temple has a later trim pool; Simple Dungeon has two
earlier pools; Woodland Mansion has two earlier pools and a later trim pool. Those evaluations do
not change a Bone already emitted but advance the same named table sequence in pool order.

Enabling the bundled Trade Rebalance pack replaces the first three chest tables:

- Ancient City's Bone row remains `5/84`, count `1..15`, across `5..10` rolls;
- Desert Pyramid's first denominator becomes `237`, so its Bone row becomes `25/237`, count
  `4..6`, across `2..4` rolls; its four-roll Bone pool remains `1/5`, count `1..8`;
- Jungle Temple's Bone row remains `20/89`, count `4..6`, across `2..6` rolls.

The replacements preserve the respective named sequences
`minecraft:chests/{ancient_city,desert_pyramid,jungle_temple}`. The two unreplaced tables use
`minecraft:chests/{simple_dungeon,woodland_mansion}`. Structure admission, placement, lazy chest
materialization, one `nextLong` loot seed per placed container, repeated output insertion and
table-opening criteria remain with the worldgen, structure and loot owners.

Fishing acquisition and retrieval:

The root `gameplay/fishing` table makes one weighted selection among junk, treasure and fish. For
loot-context luck `l`, the effective integer weights are

`J = max(floor(10 - 2l), 0)`, `T = max(floor(5 + 2l), 0)` and
`F = max(floor(85 - l), 0)`.

Treasure is absent unless the hook's `in_open_water` predicate passes. When junk is selected,
`gameplay/fishing/junk` makes one roll. Bone has weight `10`; eligible total weight is `100`
outside Jungle, Sparse Jungle and Bamboo Jungle, or `110` in those biomes because the conditional
weight-`10` Bamboo entry joins. Bone therefore has conditional junk probability `1/10` outside
those biomes and `1/11` inside them, emits exactly one default stack and runs no entry function.

When the root denominator is positive, the full conditional Bone probability is

`J / (J + F + (open_water ? T : 0)) * (jungle ? 1/11 : 1/10)`.

At `l=0`, this is `1/100` in open non-jungle water, `1/110` in open jungle water, `1/95`
outside open non-jungle water and `2/209` outside open jungle water. Root and nested work use
distinct random sequences `minecraft:gameplay/fishing` and `minecraft:gameplay/fishing/junk`.

Retrieval still triggers the generic `fishing_rod_hooked` criterion, creates and attempts to insert
the item entity, emits one XP orb of uniform value `1..6`, damages the rod and removes the hook.
Bone is not in `fishes`, so this catch does not increment `fish_caught` and does not satisfy the
four-fish `husbandry/fishy_business` item predicate. Bite/open-water state, motion, insertion,
criterion context, XP and rod transaction remain with the fishing and loot owners.

Bone-Meal crafting and recipe progression:

The sole bundled recipe consuming Bone is shapeless `bone_meal`, group `bonemeal`. Exactly one
nonempty crafting slot must contain an exact Bone; taking the result consumes one and emits three
default Bone Meal. It copies no Bone component patch and has no remainder. Multiple Bones in that
slot permit repeated crafts but do not multiply one result transaction.

Its no-display `recipes/root` advancement has one OR requirement containing exact Bone possession
and exact `bone_meal` recipe unlock. Either criterion grants only that recipe. Bone-Meal
possession does not satisfy the inventory criterion. Craft-grid discovery, transaction commit,
remainder handling, recipe-book state and Bone Meal's later block/dispenser behavior remain with
the crafting, progression and item-use owners.

Untamed-Wolf interaction:

Wolf checks taming only on the logical server, after determining it is untamed. Admission requires
the held stack to match exact Bone identity and the Wolf not to be angry. It does not require a
tag, food component, owner, missing health or player-kill history.

An admitted attempt first consumes one through the player-aware stack helper, then spends
`nextInt(3)`. Infinite-material players retain their count but still perform the attempt. Result
ordering is:

- on zero, `tame(player)` sets tame state and owner with Wolf subtype side effects and triggers
  `tame_animal` for a server player; navigation stops, target clears, ordered-to-sit becomes true
  and entity event `7` is broadcast;
- on one or two, no tame/owner/navigation/target/sit state changes and entity event `6` is
  broadcast.

Events `7/6` render seven heart/smoke particles client-side; they do not themselves own the tame
state. Either admitted branch returns `SUCCESS_SERVER`. An angry or already-tame Wolf reaches
later superclass/subtype interaction handling without consuming Bone or spending this roll.
Generic entity-interaction networking, ability-aware count restoration and tame persistence remain
with the player, stack and tame owners.

Successful Wolf taming can complete telemetry-enabled `husbandry/tame_an_animal`, whose sole
criterion accepts any tame-animal trigger. It also advances the one matching Wolf-variant
criterion of telemetry-enabled `husbandry/whole_pack`; that challenge requires all nine variants
independently and awards `50` experience. Whole Pack displays Bone as its icon, but does not test
the consumed stack: its predicates inspect the tamed entity's `wolf/variant` component.

Wolf begging:

Every Wolf installs `BegGoal` at priority `9` with LOOK control and distance `8`. It selects the
nearest player admitted by noncombat targeting within that range, then tests both hands in enum
order. A hand is interesting when its stack is exact Bone or passes the Wolf's live `wolf_food`
tag test. Bone therefore works for tame, untamed and angry Wolves whenever scheduling and the
generic targeting gates admit this goal; it is not converted into food and is not consumed.

On start, the Wolf's synchronized interested flag becomes true and the goal requests
`adjustedTickDelay(40 + nextInt(40))`. Each running goal tick looks at the player's
`(x, eyeY, z)` with yaw speed `10`, the Wolf's maximum head-X rotation, then decrements the timer.
It continues only while the player is alive, squared distance is at most `64`, time remains and
either hand remains interesting. Stop clears interested state and the player reference. Goal
arbitration, noncombat targeting, tick-rate adjustment and client head/tail interpolation remain
with `MOB-AI-001`.

**Persistence and reload boundary:**

Bone stacks persist and synchronize identity, count and arbitrary ordinary component patches.
They store no death context, loot-table cursor, structure/container seed, fishing hook/luck,
craft transaction, recipe knowledge, Wolf owner/tame/anger/sit/target/navigation/beg timer or
advancement state. Those belong to their entity, table, structure, hook, player, Wolf and
progression owners.

Loot reload changes future entity, chest and fishing evaluations. Enabling or disabling Trade
Rebalance selects future replacement chest records without rewriting already materialized
contents. Recipe/advancement reload changes future matching and listeners. Bone's taming and exact
BegGoal identity checks are code-built rather than tag-built; a `wolf_food` reload can change the
other interesting stacks but cannot remove Bone. Completed drops, fishing retrievals, crafts and
tame attempts are not replayed. Resource reload independently controls name, item definition,
model and texture.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1112` plus the component patch. Its
common-rarity name uses locked English text `Bone`; the plain class adds no subtype tooltip or
forced glint.

The direct item definition selects `minecraft:item/bone`. That model uses parent `item/handheld`,
same-named layer-zero texture and a dedicated head-display transform. Bone appears exactly once
and only in Ingredients, ordered Wheat, Bone, Bone Meal. Whole Pack separately renders it as an
advancement icon; that presentation adds no item or tab entry.

**Branches and aborts:**

Identity/count/components; six named skeletal tables and attacker/Looting/base-count state; five
base chest tables, three replacement records and every pool/roll/weight/count/sequence; fishing
luck/open-water/biome/root/nested/retrieval state; recipe grid/knowledge/reload; Wolf side,
tame/anger/player ability/roll/variant/progression; BegGoal targeting/hands/range/timer/scheduling;
save/reload, wire, language, item/model/texture and tab.

**Constants and randomness:**

Raw ID `1112`; common rarity; max stack `64`; entity base count `0..2` plus
`round(L*U)`; chest values exactly as tabled; fishing root `J/T/F`, junk weight `10/100` or
`10/110`; recipe input/result `1/3`; tame chance `1/3`; tame events `7/6`; BegGoal priority/range
`9/8`, squared continuation range `64`, timer request `40+nextInt(40)`, yaw speed `10`; Whole Pack
variants/XP `9/50`.

**Side effects:**

Default Bone stack emission and named loot cursor; fishing item/XP/hook transaction without
fish-caught increment; Bone consumption and three Bone Meal plus recipe knowledge; tame attempt,
owner/tame/navigation/target/sit/criterion/event or failure event; synchronized interested flag
and Wolf look control; ordinary persistence, wire and direct client presentation.

**Gates:**

Ordinary entity-death/table admission and attacker context; structure/container/table and optional
pack selection; fishing hook/root/open-water/biome/luck admission; exact one-Bone shapeless grid;
server-side untamed exact-Bone non-angry Wolf; BegGoal scheduling/noncombat target/live held
identity; registry/stack decode; client language/model/tab bootstrap.

**State read/written:**

Reads stack identity/count/components, entity death/attacker/enchantment state, loot and structure
contexts, fishing hook/luck/biome state, recipe/grid/player knowledge, Wolf tame/anger/AI/variant
and player abilities/hands, persistence and client resources. Writes only the loot, fishing,
craft, progression, tame, interested/look, stack and client state listed above.

**Failure behavior:**

A base/bonus total of zero emits no Bone. Unselected or condition-excluded loot entries emit their
alternatives; disabled/replaced tables affect only future evaluations. A nonmatching craft grid
has no result. Client-side, angry, tame or non-Bone Wolf interaction does not enter this tame
branch. A failed tame roll still spends the admitted Bone and emits smoke. A missing/dead/out-of-
range/no-longer-interesting player stops begging without consuming anything. Missing client
resources follow generic fallback and cannot grant authority.

**Boundary cases and quirks:**

Looting can revive a zero-count skeletal Bone roll. Desert Pyramid has two independent
Bone-bearing pools, while Simple Dungeon and Woodland Mansion evaluate their Bone pool only after
two earlier pools. Trade Rebalance changes Desert's first Bone denominator but not its second,
Ancient or Jungle Bone odds. Fishing Bone gives normal retrieval XP but no fish statistic.
Component-patched Bone still tames and attracts Wolves. The Whole Pack's Bone is only its icon;
advancement progress comes from the tamed Wolf variant.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.entity.animal.wolf.Wolf#registerGoals`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.world.entity.animal.wolf.Wolf#tryToTame`;
`net.minecraft.world.entity.animal.wolf.Wolf#isFood`;
`net.minecraft.world.entity.ai.goal.BegGoal`;
`net.minecraft.world.entity.TamableAnimal#tame`;
`net.minecraft.world.item.ItemStack#consume`;
`net.minecraft.world.entity.projectile.FishingHook#retrieve`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.TradeRebalanceChestLoot`;
`net.minecraft.data.loot.packs.VanillaFishingLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,recipe,loot_table,advancement}`;
`reports/minecraft/components/item/bone.json`;
`data/minecraft/loot_table/{entities/{bogged,parched,skeleton_horse,skeleton,stray,wither_skeleton},chests/{ancient_city,desert_pyramid,jungle_temple,simple_dungeon,woodland_mansion},gameplay/fishing/junk}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/{ancient_city,desert_pyramid,jungle_temple}.json`;
`data/minecraft/recipe/bone_meal.json`;
`data/minecraft/advancement/{recipes/misc/bone_meal,husbandry/{tame_an_animal,whole_pack}}.json`;
`assets/minecraft/{items,models/item,textures/item}/bone.*`;
`ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `MOB-AI-001`; `MOB-BREED-001`;
`WGEN-PIPELINE-001`; `WGEN-STRUCTURE-DESERT-PYRAMID-001`;
`WGEN-STRUCTURE-JUNGLE-TEMPLE-001`; `WGEN-STRUCTURE-WOODLAND-MANSION-001`;
`WGEN-JIGSAW-ANCIENT-CITY-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-062`.

**Test vectors:**

Generate all six skeletal contexts across death admission, attacker kind, Looting levels,
base/bonus endpoints and complete ordered cursors. Generate all base and replacement chest pools
across rolls, selections, counts, repeated hits, named cursors and container insertion. Fish across
luck, open-water and three-jungle-biome boundaries while tracing root/junk sequences, retrieval,
XP, criterion and statistics.

Match/take the Bone-Meal recipe with plain and component-patched Bones through every grid and
knowledge state. Attempt Wolf interaction on both logical sides across tame/anger/ability/roll/
variant states and verify progression ordering. Hold Bone in either hand across every Wolf state,
target/range/timer/goal-conflict boundary. Reload each data/resource domain, persist/reload/
synchronize stacks and owners, and verify raw ID, name, handheld/head model, texture, icon and
exact Ingredients neighbors.
