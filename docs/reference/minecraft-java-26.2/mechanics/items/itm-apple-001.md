# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-APPLE-001` — Apples join ordinary food, leaf and chest acquisition, composting, equine feeding, Golden-Apple crafting and a Farmer offer

**Parent:** `BLK-001`, `BLK-BREAK-001`, `PLY-005`, `PLY-006`,
`PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`, `ITM-001`,
`ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`, `ITM-ANVIL-001`,
`ENT-001`, `ENT-005`, `MOB-BREED-001`, `WGEN-PIPELINE-001`,
`WGEN-STRUCTURE-IGLOO-001`, `WGEN-STRUCTURE-STRONGHOLD-001`,
`WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, the sole direct item tag, every exact
`Items.APPLE` class reference, two leaf tables, six chest tables, one recipe-input edge, the
Balanced-Diet criterion/icon, composter registration, equine dispatch, one Farmer trade set and
client resources determine every Apple-specific branch. Generic consumption, crafting, loot,
merchant, composter, animal, structure and stack behavior remains with the cited owners.

**Applies when:**

An `apple` stack is dropped by Oak or Dark Oak Leaves, emitted by one of six chest tables, bought
from a level-two Farmer, eaten, offered to an equine, inserted into a Composter, consumed by the
Golden Apple recipe, moved, renamed, persisted, synchronized, selected in Food & Drinks, rendered,
or observed before and after component, tag, loot, recipe, advancement, trade or resource reload.

**Authoritative state:**

`minecraft:apple` is raw item ID `921`. It is common, nondamageable and has maximum stack `64`.
The plain-item registration supplies food nutrition `4`, saturation `2.4` and omitted/default
`can_always_eat=false`, plus the otherwise-default `1.6`-second (`32`-tick) eat consumable with no
consume-effect entries.

The remaining registered components are the common empty modifiers/enchantments/lore, item-break
sound, translated name, direct item-model key, repair cost, swing animation, tooltip display and
use effects. There is no cooldown, use remainder, tool, equipment, repairable, rarity override,
identity-specific glint or identity-specific item subclass behavior.

Its only direct item tag is `#minecraft:horse_food`. Apple is absent from
`#minecraft:horse_tempt_items` and every Piglin-loved, Rabbit-food and brewing-input tag.
Composter admission is instead code-built by exact item identity at chance `0.65f`.

**Transition and ordering:**

Player consumption and progression:

In-air player use enters the consumable path only when the food listener admits it. An ordinary
survival player at food level `20` therefore gets `FAIL`; lower hunger admits use, and a player
ability that permits full-hunger eating independently admits it. An admitted use starts the
default eat animation with `CONSUME`. Block and entity clicks retain their earlier interaction
dispatch before an unconsumed result can reach the edible-item fallback.

Removing only the food component leaves the empty consumable intact: use is admitted even at full
hunger, but completion applies no nutrition or saturation. Removing only the consumable makes
plain in-air use pass and prevents food application. A patched food component supplies its live
nutrition, saturation and always-edible flag while the unchanged consumable retains no effects.

Interruption, release or live-hand/component replacement before completion commits no statistic,
criterion, nutrition, game event or shrink. At successful server completion, generic active-use
ordering emits final eat effects, awards the item-used statistic, triggers `consume_item` against
the live pre-shrink stack, applies food, emits `EAT`, and consumes one unless the user has infinite
materials. Food level is clamped after `+4`, and saturation is clamped after `+2.4`. There is no
consume-effect probability draw, status-effect offer or remainder.

Apple is one of the `40` independent AND requirements in telemetry-enabled
`husbandry/balanced_diet`, and is also that challenge's display icon. The pre-shrink criterion
therefore advances before nutrition; icon selection adds no second criterion or creative entry.

Golden-Apple crafting sink:

No bundled recipe creates an Apple. Its sole direct recipe role is the center input of
`golden_apple`, whose shaped `3x3` grid surrounds one Apple with eight Gold Ingots and creates one
default Golden Apple. Taking the result consumes the Apple and all eight Ingots; no input has a
remainder and input component patches do not propagate.

The Golden Apple recipe advancement tests Gold-Ingot possession or exact recipe unlock. Apple
possession is not an unlock criterion, so obtaining or eating an Apple alone grants no recipe.
The full result registration, matching, knowledge and take transaction remains
`ITM-GOLDEN-APPLE-001`.

Oak and Dark Oak leaf acquisition:

Only `blocks/oak_leaves` and `blocks/dark_oak_leaves` can emit Apple. Each table uses its matching
`minecraft:blocks/<leaves>` named sequence and evaluates three one-roll pools in order: leaf versus
sapling, Stick, then Apple. The Apple pool is skipped when the tool is exact Shears or has Silk
Touch level at least one.

For every other tool, the Apple entry first tests `survives_explosion`, then the Fortune
`table_bonus` vector:

| Fortune level | Apple chance after explosion survival |
| ---: | ---: |
| `0` | `0.005` |
| `1` | `0.0055555557` |
| `2` | `0.00625` |
| `3` | `0.008333334` |
| `4+` | `0.025` |

Success emits one default Apple with no count function. Earlier sapling and Stick conditions,
counts and explosion work advance the same named cursor before this pool; failure never retries or
falls back to another Apple entry. Breaking naturally generated and player-placed leaves uses the
same live table.

Chest acquisition:

Six direct entries are unconditional, set the listed uniform count, and participate in ordinary
weighted selection with replacement:

| loot table and pool | pool rolls | Apple weight / total per roll | selected count | other pool work in table order |
| --- | ---: | ---: | ---: | --- |
| `chests/igloo_chest`, pool 0 | `2..8` | `15/63 = 5/21` | `1..3` | later one guaranteed Golden Apple roll |
| `chests/spawn_bonus_chest`, pool 2 | `3` | `5/11` | `1..2` | follows one Axe and one Pickaxe roll; later `4` rolls |
| `chests/stronghold_crossing`, pool 0 | `1..4` | `15/62` | `1..3` | none |
| `chests/stronghold_corridor`, pool 0 | `2..3` | `15/101` | `1..3` | later one roll |
| `chests/village/village_plains_house`, pool 0 | `3..8` | `10/43` | `1..5` | later one Bundle/Empty roll |
| `chests/village/village_weaponsmith`, pool 0 | `3..8` | `15/107` | `1..3` | later one Bundle/Empty roll |

Each table uses its exact like-named `minecraft:chests/<path>` random sequence. Repeated
multi-roll selection may create multiple separate Apple stacks. Earlier and later pools cannot
alter an emitted stack but advance their table cursor in order. The built-in trade-rebalance pack
does not replace any of these six records. Bonus-chest generation, structure/template placement,
lazy table evaluation and bounded container insertion remain with `WGEN-PIPELINE-001`, the named
structure owners and `ITM-LOOT-001`.

Level-two Farmer acquisition:

The baseline Farmer level-two tag contains exactly ordered Pumpkin-for-Emerald,
Emerald-for-Pumpkin-Pie and Emerald-for-Apple records. Its set requests two offers, disables
duplicates by default and uses random sequence `minecraft:trade_set/farmer/level_2`. Selection
removes `nextInt(3)`'s record, then removes `nextInt(2)`'s record from the survivors. All three
records are predicate-free and produce offers, so Apple inclusion is exactly `2/3`; selected
offer order follows the two draws.

The Apple offer accepts one matching Emerald and returns four default Apples. It has maximum uses
`16`, villager XP `5`, reputation discount multiplier `0.05`, and no second cost, input predicate
or output modifier. Generic Farmer level-up/restock, price/demand/reputation adjustment, atomic
trade commit, exhaustion and publication remain with the merchant owners.

Composter insertion:

Player-held insertion at level `0` succeeds without RNG. Levels `1..6` consume one
`nextDouble()` and increment exactly when it is strictly below the Java-float chance `0.65f`
widened for comparison. Success writes level plus one with flags `3`, emits `BLOCK_CHANGE`, and
`6 -> 7` schedules maturation after `20` ticks; failure preserves state.

Either level-`0..6` result emits level event `1500` with success encoded by whether the state
changed, awards the Apple-used statistic and calls `consume(1, player)`, preserving
infinite-material holders. Level `7` returns success for a held Apple without insertion, event,
statistic or consumption. Level `8` falls through to ordinary item-on-block handling.

Automation exposes one top input slot only below level `7`. It accepts a compostable stack once,
runs the same deterministic-first-level/strict-double transition, emits event `1500`, and removes
the one-slot stack whether the chance succeeded or failed. The direct insertion helper likewise
shrinks one after every admitted level-`0..6` attempt. Maturation `7 -> 8`, ready sound,
Bone-Meal extraction and event `1500`'s client sound/particle rendering remain with the Composter
and protocol owners.

Ordinary Horse, Donkey and Mule feeding:

Unridden Horse and Abstract-Chested-Horse interaction consult live `horse_food` before the shared
hard-coded feed table, so Apple reaches ordinary Horse, Donkey and Mule unless an earlier
adult-tamed secondary-use inventory branch wins. Llama, Camel and Zombie Horse use other food
selectors; an ordinary unmounted Skeleton Horse bypasses the specialized feed-table check.

Apple assigns heal `3`, baby growth `60` seconds and temper increment `3`. On the server, the
table first puts a tamed age-zero equine not already in love into `600`-tick love with the player
as cause. It then, in order:

1. heals `3` when health is below maximum;
2. if age-unlocked and baby, emits the happy-particle offer and ages up `60` seconds;
3. if temper is below maximum and either an earlier effect succeeded or the equine is untamed,
   adds `3` through the clamped temper modifier.

Any success opens the synchronized mouth, plays the subtype eating sound unless silent at volume
`1` and pitch `1+(U1-U2)*0.2`, emits `EAT`, consumes one through the player-ability-aware rule and
returns server success. The registered player-food transaction is not invoked.

A tamed full-health adult not already in love therefore consumes solely for love. An untamed
adult never receives love but can consume for healing or temper. A full-health maximum-temper
untamed adult and a full-health already-loving tamed adult return `PASS`. Babies do not enter love
but can heal, grow and raise temper. Horse and Donkey can later use eligible love to parent; Mule
can retain love but cannot mate.

Vehicle dispatch and non-temptation:

Horse and Abstract-Chested-Horse delegate vehicle interaction to generic `Animal.mobInteract`
before the specialized feed table. Live `horse_food` membership then consumes one for
`600`-tick adult love or advances an age-unlocked baby by
`floor((remainingBabyTicks/20)*0.1)` seconds. It does not heal, alter temper, open the mouth, play
the equine eating sound, emit specialized `EAT`, or apply player food.

A tamed Skeleton Horse vehicle also delegates to this generic path, so an adult can enter unusable
love while unmounted adults and age-locked babies gain no Apple consequence. Zombie Horse, Llama
and Camel selectors reject direct horse-tag membership. Unnatural untamed vehicle fixtures can
receive generic love because `Animal` tests age/love rather than tame state.

Apple's absence from `horse_tempt_items` means normal Horse, Donkey and Mule do not target a
player merely for holding one. Its direct `horse_food` membership does not leak into
`TemptGoal`; Golden Apple and Enchanted Golden Apple retain that separate behavior.

Persistence and reload boundary:

Apple stacks persist and synchronize identity, count and arbitrary ordinary component patches.
They store no active-use progress, hunger, recipe knowledge, leaf/chest cursor, structure context,
offer lifecycle, Composter state or animal state; those belong to their player, world, merchant,
machine and entity owners.

Loot reload independently changes future leaf and chest evaluations. Recipe/advancement reload
changes future matching and listeners; tag reload changes future equine admission; trade reload
changes future Farmer sets without rewriting existing offers. The code-built composter chance and
hard-coded equine row do not reload. Completed uses, drops, crafts, offers, insertions and feeds
are not replayed. Resource reload independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `921` plus the stack's component patch. Its
common-rarity name uses locked English text `Apple`; the plain class adds no subtype tooltip or
forced glint.

The direct item definition selects generated model `minecraft:item/apple` and its same-named
texture. It appears exactly once in Food & Drinks, ordered before Golden Apple, Enchanted Golden
Apple and Melon Slice. Its Balanced-Diet icon role adds no creative entry. Apple adds no packet
layout or numeric mapping.

**Branches and aborts:**

Identity/count/components and horse tag; hand/block/active-use/hunger; Balanced-Diet
criterion/icon; Golden-Apple recipe grid/take/unlock exclusion; Oak/Dark-Oak tool/enchantment/
explosion/Fortune branches and both cursors; six chest records with every pool/roll/weight/count/
cursor/insertion branch; Farmer set/order/offer/economy; player/automated Composter level/chance/
ability/event path; equine subtype/vehicle/tame/secondary-use/health/age-lock/temper/love/
temptation exclusion; save, component/tag/loot/recipe/advancement/trade/resource reload, wire,
language, model and tab.

**Constants and randomness:**

Raw ID `921`; common rarity; max stack `64`; food `4/2.4`; eat `32`; no consume-effect draw;
leaf Fortune chances `0.005/0.0055555557/0.00625/0.008333334/0.025`; chest odds/counts Igloo
`5/21,1..3`, Bonus `5/11,1..2`, Crossing `15/62,1..3`, Corridor `15/101,1..3`, Plains
`10/43,1..5`, Weaponsmith `15/107,1..3`; Farmer inclusion `2/3`, one Emerald to four Apples,
uses/XP/discount `16/5/0.05`; compost chance `0.65`, maturation `20`, event `1500`; equine
heal/growth/temper `3/60/+3`, love `600`; generic growth ten percent; eating pitch consumes two
subtype RNG floats.

**Side effects:**

Nutrition/saturation, statistic, criterion, event and shrink; Golden-Apple recipe input removal;
leaf and chest stacks plus named cursors; Farmer offers, result, uses, XP and economy; Composter
item/stat/state/game-event/level-event/schedule; equine health/age/temper/love/mouth/sound/
particles/event; ordinary stack persistence/wire state; name, icon, model and tab.

**Gates:**

Food/hunger and live consumable admission; uninterrupted same-stack completion; exact shaped
recipe and active snapshot; leaf tool/enchantment/explosion/Fortune and chest structure/table/
roll/container admission; level-two Farmer set and live offer; code-built compostable identity,
level and player/automation path; live horse-food plus equine subtype/state precedence;
registry/stack decode; client language/model/tab bootstrap.

**State read/written:**

Reads stack identity/count/components/tag, player use/hunger/progression, recipe state, leaf tool/
explosion context, six chest/structure contexts, merchant state, Composter level/RNG, equine state,
persistence and client resources. Writes only the consumption, progression, crafting, loot,
trade, compost, animal, stack and client state listed above.

**Failure behavior:**

Full-hunger ordinary survival use returns `FAIL`; interrupted use commits no finish. Invalid
Golden-Apple grids produce no result and Apple possession does not unlock that recipe. Shears,
Silk Touch, failed explosion survival or failed Fortune chance emits no Apple from Leaves.
Unselected chest/trade records emit alternatives. Failed admitted composting still consumes the
finite player's or automation's Apple; levels `7/8` do not. Removed horse-food or ineligible
equine state consumes nothing unless another specified path succeeds. Missing/replaced reloadable
data changes only future attempts. Client-resource absence follows generic fallback and cannot
grant authority.

**Boundary cases and quirks:**

Apple is a normal hunger-gated food and has no effect RNG. It is a recipe input but Apple
possession does not unlock that recipe. Only two leaf identities emit it; Shears and Silk Touch
skip the whole Apple pool, and Fortune level four or above clamps to `0.025`. The first Composter
level always succeeds while later failed attempts still consume the Apple. A level-seven
Composter reports success without consumption. The Farmer offer is present only two-thirds of the
time. Ordinary equines use love-first hard-coded `3/60/+3`, while vehicle dispatch switches to
percentage growth/love; holding Apple does not tempt them.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.food.Foods`;
`net.minecraft.world.item.component.Consumable#startConsuming`;
`net.minecraft.world.item.component.Consumable#onConsume`;
`net.minecraft.world.food.FoodProperties#onConsume`;
`net.minecraft.world.food.FoodData#eat`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.level.block.ComposterBlock$InputContainer#setChanged`;
`net.minecraft.world.item.ItemStack#consume`;
`net.minecraft.world.entity.animal.equine.Horse#mobInteract`;
`net.minecraft.world.entity.animal.equine.AbstractChestedHorse#mobInteract`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#fedFood`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#handleEating`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#addBehaviourGoals`;
`net.minecraft.world.entity.animal.equine.SkeletonHorse#mobInteract`;
`net.minecraft.world.entity.animal.equine.ZombieHorse#isFood`;
`net.minecraft.world.entity.animal.Animal#mobInteract`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.tags.VanillaItemTagsProvider`;
`net.minecraft.data.loot.BlockLootSubProvider#createOakLeavesDrops`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,recipe,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/apple.json`;
`data/minecraft/tags/item/horse_food.json`;
`data/minecraft/recipe/golden_apple.json`;
`data/minecraft/advancement/husbandry/balanced_diet.json`;
`data/minecraft/loot_table/blocks/{oak_leaves,dark_oak_leaves}.json`;
`data/minecraft/loot_table/chests/{igloo_chest,spawn_bonus_chest,stronghold_crossing,stronghold_corridor,village/{village_plains_house,village_weaponsmith}}.json`;
`data/minecraft/{villager_trade/farmer/2/emerald_apple,tags/villager_trade/farmer/level_2,trade_set/farmer/level_2}.json`;
`assets/minecraft/{items,models/item,textures/item}/apple.*`;
`ITM-GOLDEN-APPLE-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`;
`ITM-HUNGER-001`; `MOB-BREED-001`; `WGEN-PIPELINE-001`;
`WGEN-STRUCTURE-IGLOO-001`; `WGEN-STRUCTURE-STRONGHOLD-001`;
`WGEN-JIGSAW-VILLAGES-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-057`.

**Test vectors:**

Exercise default, food/consumable-removed and arbitrary component-patched Apple stacks through
both hands, full/low hunger, abilities, interruption, containers and anvil. Assert no effect draw,
exact food/stat/criterion/event order and the Balanced-Diet criterion/icon distinction.

Break Oak and Dark Oak Leaves with bare, Shears, Silk Touch and every Fortune level under
controlled explosion draws while tracing all three pools and named cursors. Generate all six
chest tables through every roll, count, repeated selection, earlier/later pool and insertion
branch. Match/take the Golden Apple recipe and prove Apple possession does not unlock it.

Create every Farmer level-two selection and offer lifecycle while tracing `nextInt(3)` then
`nextInt(2)`. Insert finite/infinite-player and automated Apples into every Composter level across
below/equal/above chance draws. Feed every Horse/Donkey/Mule/Skeleton/Zombie/Llama/Camel fixture
across vehicle, tame, secondary use, health, age lock, temper, love, horse-food and temptation
state.

Persist/reload/synchronize stacks and capture raw ID, common name, absent forced glint, model/
texture, Balanced-Diet icon and exact Food & Drinks position before and after every reload domain.

**Limits:**

This leaf does not duplicate generic food completion, shaped crafting, loot execution/container
insertion, Composter maturation/extraction, merchant economics, animal mating/persistence,
structure placement or stack/resource codecs. Those remain with their cited owners; this rule
fixes Apple identity and its exact food, acquisition, recipe-input, trade, compost, equine and
presentation joins.
