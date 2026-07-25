# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-PUFFERFISH-001` — Pufferfish join toxic food, aquatic and Wolf feeding, death and fishing loot, a Fisherman sink and Water Breathing

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-HUNGER-001`, `ITM-ANVIL-001`, `ITM-POTION-001`, `ENT-001`, `ENT-005`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-PROJECTILE-001`,
`ENT-EFFECT-001`, `MOB-SPAWN-001`, `MOB-AI-001`, `MOB-BREED-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, the complete direct and nested item-tag
consumer set, entity and fishing loot, hook retrieval, Fisherman trade resources and selection
bytecode, brewing graph, advancements and client assets determine every Pufferfish-item-specific
branch. Pufferfish entity spawning/AI, generic death, fishing, effects, animal state machines,
merchant execution, brewing, stacks and inventories remain with the cited owners.

**Applies when:**

A `pufferfish` stack is emitted by Pufferfish death or fishing, caught by a player, eaten, offered
to a Dolphin, Wolf or Nautilus, supplied to a Brewing Stand, sold to a level-five Fisherman,
moved, renamed, persisted, synchronized, selected in a tab, rendered or observed before and after
loot, tag, trade, advancement, mix or resource reload.

**Authoritative state:**

`minecraft:pufferfish` is raw item ID `1089`. It is common, nondamageable and has max stack `64`.
It registers through the plain-item path with these operational components:

- food nutrition `1`, saturation `0.2` and default `can_always_eat=false`;
- the otherwise-default `1.6`-second (`32`-tick) eat consumable, with one apply-effects consumer
  holding this ordered list: Poison amplifier `1` for `1200` ticks, Hunger amplifier `2` for
  `300` ticks, then Nausea amplifier `0` for `300` ticks. All are nonambient and show particles
  and icons through their defaults.

The remaining registered components are the common empty modifiers/enchantments/lore, item-break
sound, translated name, direct item-model key, repair cost, swing animation, tooltip display and
use effects. There is no cooldown, use remainder, tool, equipment, repairable or
identity-specific glint state.

Its direct tags are `#minecraft:fishes`, `#minecraft:nautilus_taming_items` and
`#minecraft:wolf_food`. `fishes` contains Cod, Cooked Cod, Salmon, Cooked Salmon, Pufferfish and
Tropical Fish; `nautilus_food` recursively contains the complete `fishes` tag, so this identity is
also ordinary Nautilus food. The separate taming tag contains Pufferfish Bucket and Pufferfish.
Tag membership, not the registered food component, admits all mob interactions below.

**Transition and ordering:**

Player consumption and ordered effects:

In-air player use enters the consumable path only when the food listener admits it. A normal
survival player at food level `20` therefore gets `FAIL`; lower hunger admits use, and a player
ability that permits eating at full hunger independently admits it. An admitted use begins the
default eat animation with `CONSUME`. Block clicks retain block-first handling before an
unconsumed result can reach the edible-item fallback.

Removing only the food component leaves the consumable and its toxic effect list intact:
`canConsume` then has no food gate, so use is admitted even at full hunger and completion applies
no nutrition or saturation. Removing only the consumable makes plain in-air item use pass and
prevents both food and effects. A patched food component supplies its live nutrition, saturation
and always-edible flag while the unchanged consumable retains the three default toxic offers.

Interruption, release or live-hand/component replacement before completion commits no statistic,
criterion, nutrition, effect, game event or shrink. At successful server completion, generic
active-use ordering:

1. emits its configured final eat effects;
2. awards the item-used statistic and triggers `consume_item` against the live pre-shrink stack;
3. runs the food listener, clamping hunger after `+1` and saturation after `+0.2`;
4. runs the single apply-effects consumer, which draws one entity RNG float and, at omitted
   probability default `1.0`, always enters its effect loop;
5. offers fresh Poison `1200@1`, Hunger `300@2` and Nausea `300@0` instances in that order;
6. emits the `EAT` game event and shrinks one unless the user has infinite materials.

The probability-one branch still spends its one float before the strict comparison; the three
effects do not draw separately. Immunity, current-effect strength/duration, hidden chains and
callbacks can independently reject or merge each ordered offer under `ENT-EFFECT-001` without
preventing later offers or rolling back nutrition, event or consumption. There is no remainder.
Infinite-material use retains the fish but still commits the admitted food/effect/progression
transaction.

Pufferfish is one of the 40 independent AND requirements in `husbandry/balanced_diet`. Because
`consume_item` precedes nutrition and effects, completion advances this requirement from the
pre-shrink stack even if one or all later effect offers are rejected.

**Pufferfish-death acquisition:**

The `entities/pufferfish` table has two ordered one-roll pools under random sequence
`minecraft:entities/pufferfish`. The first has one Pufferfish entry and sets its count to the
constant `1`; it has no killed-by-player, attacker, Looting or chance condition. Every admitted
ordinary death-table evaluation therefore emits exactly one default Pufferfish even when the
death lacks player credit.

The later pool draws a `0.05` random-chance float and emits one default Bone Meal only on success.
That draw and output cannot alter the already emitted fish but remain later work on the same named
sequence. Entity spawning, death admission, loot invocation, item insertion and removal paths
that bypass normal death remain with `MOB-SPAWN-001`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

**Fishing acquisition and retrieval:**

The root `gameplay/fishing` table makes one weighted selection among junk, treasure and fish. For
loot-context luck `l`, the effective integer weights are

`J = max(floor(10 - 2l), 0)`, `T = max(floor(5 + 2l), 0)` and
`F = max(floor(85 - l), 0)`.

The treasure candidate is absent unless the hook's `in_open_water` predicate is true.
`FishingHook#retrieve` supplies `l` as its nonnegative stored Luck-of-the-Sea contribution plus
the player's float Luck attribute. Once fish is selected, `gameplay/fishing/fish` makes one
weighted choice among Cod `60`, Salmon `25`, Tropical Fish `2` and Pufferfish `13`, total `100`.
A Pufferfish selection emits exactly one default stack without functions.

When the root denominator is positive, the conditional Pufferfish probability is

`F / (J + F + (open_water ? T : 0)) * 13/100`.

At `l=0` this is `221/2000` in open water and `221/1900` outside open water. The root and nested
tables use distinct random sequences `minecraft:gameplay/fishing` and
`minecraft:gameplay/fishing/fish`; selecting another root or fish entry preserves the
corresponding branch-specific cursor.

After generating the complete loot collection, retrieval first triggers
`fishing_rod_hooked`; the Pufferfish criterion is one of four names in the single OR requirement
of telemetry-enabled `husbandry/fishy_business`, so catching any one of Cod, Tropical Fish,
Pufferfish or Salmon completes it. Retrieval then creates and inserts the item entity, inserts one
XP orb of uniform value `1..6`, and, because the result is in `fishes`, increments the player's
`fish_caught` statistic by one. The criterion therefore observes the generated stack before the
item entity, XP and statistic are committed. Item-entity insertion failure is not checked and
does not suppress the later XP or statistic.

Hook bite/retrieval admission, open-water history, item motion, insertion failures, XP, rod
damage and hook removal remain `ENT-PROJECTILE-001` and `ITM-LOOT-001`. Picking up a Pufferfish
from another source does not trigger Fishy Business or the fishing statistic.

**Guaranteed level-five Fisherman sink:**

The Fisherman record wants four identity-matching Pufferfish and gives one default Emerald. It has
maximum uses `12`, villager XP `30` and reputation discount `0.05`, with no additional cost,
merchant predicate, output modifier or double-price enchantment.

The level-five tag contains that record plus five boat-purchase records. Its trade set requests
two offers, disables duplicates by default and uses random sequence
`minecraft:trade_set/fisherman/level_5`. Candidate selection repeatedly removes a random record;
a boat whose villager-variant predicate fails creates no offer and does not count toward the two
successes. For each of the seven locked villager variants, exactly one boat record matches:
Oak for Plains, Spruce for Taiga or Snow, Jungle for Desert or Jungle, Acacia for Savanna, or Dark
Oak for Swamp. Pufferfish and that one boat are therefore the only two successful baseline
candidates, so both offers are guaranteed; rejected-boat draws only vary their order and cursor.

Offer creation does not consume fish. Each successful purchase later consumes four matching
Pufferfish and produces one Emerald until the offer is exhausted. Profession leveling, price
adjustment, demand, restock, trade commit and menu synchronization remain generic Villager and
merchant owners. A custom variant matching no boat can exhaust the list with only the
still-guaranteed Pufferfish offer.

**Water-Breathing brewing graph:**

The vanilla mix builder registers exactly Awkward plus Pufferfish to ordinary Water Breathing.
It uses direct `addMix`, not the start-mix helper: Water plus Pufferfish does not make Mundane or
any other potion. Ordinary Water Breathing carries amplifier-zero Water Breathing for `3600`
ticks (`180` seconds).

Redstone Dust separately maps it to Long Water Breathing for `9600` ticks (`480` seconds).
There is no Strong form, Glowstone edge or Fermented-Spider-Eye corruption edge. Every admitted
edge works for Potion, Splash Potion and Lingering Potion container items. Container identity is
retained while fresh target contents replace source contents; custom color, effects, name and
duration scale are not preserved.

A holder must be present and match Awkward. Ingredient admission tests Pufferfish identity,
accepting arbitrary component patches and discarding them when one ingredient is consumed. A
completed brew transforms matching bottle slots `0..2` in order, consumes one fish for up to three
outputs, leaves unmatched bottles unchanged and emits event `1035`. Pufferfish has no remainder,
is not Brewing Stand fuel and is not furnace fuel. Fuel admission, the `400`-tick transaction,
cancellation and player-menu take criterion remain `ITM-BREW-001` and
`ITM-ADVANCEMENT-001`; potion use/projection and Water Breathing ticking remain
`ITM-POTION-001` and `ENT-EFFECT-001`.

**Dolphin, Wolf and Nautilus interactions:**

A Dolphin accepts any nonempty `fishes` stack, plays Dolphin Eat at volume/pitch `1/1` on the
server and consumes one. An age-unlocked baby advances by ten percent of its remaining whole
seconds, truncated to an integer, with forced-age accounting. Otherwise—including an adult or
age-locked baby—it sets `gotFish=true`, enabling its later treasure-search behavior. The
interaction returns success in either branch. Feeding does not invoke the Pufferfish consumable
and applies no Poison, Hunger or Nausea; later Dolphin AI owns use and clearing of `gotFish`.

Wolf dispatch first gives a tamed, injured Wolf a healing branch. It consumes one, heals
`2 * nutrition = 2` HP and plays its eating sound, regardless of feeder ownership; this takes
precedence over growth or love. A patched food component heals twice its live nutrition; if food
is absent, Wolf's fallback heal is still `2` HP. Every other Wolf state falls through to generic
Animal feeding. An age-unlocked baby consumes one for the same ten-percent remaining growth. An
age-zero adult not already in love consumes one and enters `600`-tick love even when untamed;
untamed Wolves still cannot mate because their subtype `canMate` requires both partners tame.
Already-loving adults and age-locked babies consume nothing; for a tamed Wolf, the ordinary
owner-only sit toggle can then handle that unconsumed fallback. None of these paths applies the
registered toxic effects.

Every interaction with an Abstract Nautilus first marks it persistent. A baby delegates directly
to generic Animal feeding; because Pufferfish reaches `nautilus_food` through `fishes`, an
age-unlocked baby consumes one for ten-percent remaining growth. An untamed adult instead tests
the separate taming tag before healing or generic feeding: it consumes one, draws
`nextInt(3)`, tames and assigns the player exactly on zero, stops navigation and emits event `7`
on success or event `6` on failure, then plays its eating sound. Plain Pufferfish is not bucket
food and therefore creates no Water Bucket remainder.

For a tamed adult, an injured state consumes one first, heals `2 * nutrition = 2` HP and plays its
eating sound, again without an owner gate. A patched food component heals twice its live
nutrition, while an absent food component uses Nautilus's distinct `1`-HP fallback. A
full-health, age-zero adult instead reaches generic feeding and consumes one for `600`-tick love
when not already in love. Secondary use opens the tamed adult inventory before all nonempty-item
branches, so it consumes no fish. Pufferfish in the recursive `nautilus_food` tag also satisfies
the Nautilus temptation predicate independently of tame state. Navigation, temptation, mate
search, offspring, owner, persistence and later AI remain `MOB-AI-001` and
`MOB-BREED-001`.

**Recipe, persistence and reload boundary:**

No bundled crafting, cooking, stonecutting or smithing recipe consumes or emits Pufferfish. Its
configured sinks are brewing, feeding/taming and the Fisherman purchase above.

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no active-use progress, hunger/effect state, loot sequence, hook/open-water/luck state,
advancement/statistic state, merchant cursor/offer uses, stand timer/mix, or mob age/health/love/
tame/temptation/`gotFish` state. Those values belong to their player, world, entity, loot,
progression, merchant and machine owners.

Loot reload can independently replace the entity, root-fishing or nested-fish table for future
evaluations. Tag reload changes future Dolphin/Wolf/Nautilus admission, fish-stat tests and
Nautilus temptation. Trade reload changes future offer generation without rewriting existing
offers. Advancement reload changes future listeners, while a rebuilt baseline mix retains the
single Awkward edge when holders/items are enabled. Completed uses, drops, catches, feeds,
offers and brews are not replayed. Resource reload independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1089` plus the stack's component patch. Its
common-rarity name uses locked English text `Pufferfish`; the plain class adds no subtype tooltip
or forced glint.

The direct definition selects generated model `minecraft:item/pufferfish` and its same-named
texture. It appears exactly once in Ingredients, ordered Spider Eye, Pufferfish, Magma Cream, and
exactly once in Food & Drinks, ordered Tropical Fish, Pufferfish, Bread.

**Branches and aborts:**

Identity/count/components, three direct tags and the recursive `nautilus_food` path;
hand/block/active-use/hunger and
three current-effect states; entity-death admission and later Bone-Meal chance; hook nibble,
open-water/luck/root/fish selection, criterion/item/XP/stat order; villager variant, record
removal/predicate/order and offer state; stand fuel/timer/container/holder/custom contents;
Dolphin age/lock, Wolf tame/health/age/love, Nautilus age/tame/health/secondary-use/tame roll and
temptation; save, loot/tag/trade/advancement/mix/resource reload, wire, language, model and both
tab contexts.

**Constants and randomness:**

Raw ID `1089`; common rarity; max stack `64`; food `1/0.2`; eat `32` ticks; one
probability-`1` float shared by Poison `1200@1`, Hunger `300@2`, Nausea `300@0`; death fish count
`1` then Bone Meal chance `0.05`; root fishing `J/T/F` formula and fish weight `13/100`, XP
`1..6`; Fisherman amount `2`, cost/result `4/1`, uses/XP/discount `12/30/0.05`; Water Breathing
`3600@0` and `9600@0`; Dolphin/Wolf/Nautilus baby growth ten percent of remaining whole seconds;
Wolf/Nautilus love `600`; Nautilus tame one-in-three; default mob heal `2`.

**Side effects:**

Possible nutrition/saturation, three effects and one RNG draw; death/fishing outputs and named
cursors; fishing criterion, item entity, XP and statistic; merchant offers, inputs, Emerald,
uses/XP/economy; stand ingredient/bottles/timer/event and potion state; Dolphin age/forced age/
`gotFish`/sound, Wolf health/age/love/sound, Nautilus persistence/tame/owner/navigation/event/
health/age/love/sound/temptation; ordinary stack persistence/wire state and two client entries.

**Gates:**

Food/hunger and live consumable admission; uninterrupted same-stack completion; ordinary
Pufferfish death-table invocation; hook nibble/open-water/luck and both tables; exact advancement
listeners; level-five Fisherman set and merchant variant/predicate/offer validity; valid stand
fuel and Awkward holder; live tags and every mob subtype/state precedence; registry/stack decode;
client language/model/tab bootstrap.

**State read/written:**

Reads stack identity/count/components/tags, player active-use/hunger/effects/progression/statistics,
entity death and two fishing contexts, hook/player/loot state, Villager variant/trade state,
brewing slots/fuel/timer/mix/contents, three mob states, persistence and client resources. Writes
only the consumption, effect, loot, fishing, progression, trade, brewing, feeding/taming, stack
and client state listed above.

**Failure behavior:**

Full-hunger ordinary survival use returns `FAIL`; interruption commits no finish. Rejected effect
offers do not undo eating or later offers. A missing death table emits nothing; an unselected
fishing entry emits its alternative and no fish statistic for a non-`fishes` result. Invalid
trade resources or an exhausted offer prevent sale. Water and all non-Awkward holders do not
brew. Ineligible mob love/growth states consume nothing except the explicit Dolphin `gotFish`,
tamed-injured heal and untamed-adult Nautilus tame branches. Missing/replaced components, loot,
tags, trade, advancement or mix data remove future paths without rewriting completed state.
Client resource absence follows generic fallback and cannot grant authority.

**Boundary cases and quirks:**

One probability float admits all three ordered toxic effects. Pufferfish death always emits the
fish without player credit, then evaluates unrelated Bone Meal. Open water adds treasure weight
and therefore lowers the baseline fish probability from `221/1900` to `221/2000`. Fishy Business
triggers before item/XP/stat insertion, while `fish_caught` comes from the `fishes` tag. Failed
boat predicates are removed without counting a Fisherman success, making the Pufferfish offer
guaranteed rather than a naive `2/6` inclusion. Pufferfish has only the Awkward brewing edge:
unlike start-mix ingredients it makes no Mundane potion from Water. Mob feeding never executes
the item's toxic consumer; untamed adult Wolves can enter love but cannot mate, and untamed adult
Nautilus use is always a taming attempt.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.component.Consumable#startConsuming`;
`net.minecraft.world.item.component.Consumable#onConsume`;
`net.minecraft.world.food.FoodProperties#onConsume`;
`net.minecraft.world.food.FoodData#eat`;
`net.minecraft.world.item.consume_effects.ApplyStatusEffectsConsumeEffect#apply`;
`net.minecraft.world.entity.projectile.FishingHook#retrieve`;
`net.minecraft.world.entity.animal.dolphin.Dolphin#mobInteract`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.world.entity.animal.wolf.Wolf#isFood`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#interact`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#mobInteract`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#isFood`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#tryToTame`;
`net.minecraft.world.entity.animal.nautilus.NautilusAi#getTemptations`;
`net.minecraft.world.entity.animal.Animal#mobInteract`;
`net.minecraft.world.entity.TamableAnimal#feed`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addMix`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,entity_type,potion,mob_effect,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/pufferfish.json`;
`data/minecraft/tags/item/{fishes,wolf_food,nautilus_food,nautilus_taming_items}.json`;
`data/minecraft/loot_table/{entities/pufferfish,gameplay/fishing,gameplay/fishing/fish}.json`;
`data/minecraft/{villager_trade/fisherman/5/pufferfish_emerald,tags/villager_trade/fisherman/level_5,trade_set/fisherman/level_5}.json`;
`data/minecraft/advancement/husbandry/{balanced_diet,fishy_business}.json`;
`assets/minecraft/{items,models/item,textures/item}/pufferfish.*`;
`ITM-USE-001`; `ITM-HUNGER-001`; `ITM-BREW-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-POTION-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `ENT-PROJECTILE-001`; `ENT-EFFECT-001`;
`MOB-AI-001`; `MOB-BREED-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-053`.

**Test vectors:**

Exercise default, food/consumable-removed and arbitrary component-patched stacks through both
hands, full/nonfull hunger, infinite materials, interruption and every current Poison/Hunger/
Nausea merge state while tracing the one shared effect draw and ordering. Kill Pufferfish through
all attribution/attacker/Looting controls and trace guaranteed fish plus the later `0.05` Bone
Meal branch on `minecraft:entities/pufferfish`.

Retrieve fishing loot across nibble admission, open-water true/false, luck weight boundaries,
every root and `100` nested weight unit, criterion/item-insertion/XP/stat failures and both named
sequences. Generate level-five Fisherman offers for all seven variants and a no-match custom
variant while tracing every removed candidate, failed predicate, offer order and cursor; then
use, exhaust and restock the purchase.

Brew plain/patched fish through every Potion/Splash/Lingering container with Water, Awkward,
Water Breathing and control holders, testing long, Glowstone and Fermented controls. Feed adult,
baby and age-locked Dolphins; tame/untamed, injured/full-health, adult/baby, love/owner-state
Wolves; and tame/untamed, injured/full-health, adult/baby, secondary-use Nautilus fixtures across
every tame roll and tag reload. Assert no mob receives the registered toxic effects, then
persist/reload/synchronize and verify exact name, model, texture and both tab positions.
