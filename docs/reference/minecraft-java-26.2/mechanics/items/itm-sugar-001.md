# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-SUGAR-001` — Sugar joins two crafting and Witch sources to equine feeding, Swiftness brewing and three foods

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ITM-DRINK-CONTAINER-001`, `ITM-EGG-001`,
`ITM-FERMENTED-SPIDER-EYE-001`, `ITM-POTION-001`, `ENT-001`, `ENT-005`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-EFFECT-001`, `MOB-SPAWN-001`,
`MOB-BREED-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components and tag, exhaustive code/data references, both
output recipes and unlocks, Witch loot, three consumer recipes, vanilla brewing graph, potion
payloads, equine interaction dispatch and client assets determine every Sugar-specific branch.
Generic entity death, loot, crafting, brewing, animal, effect, stack and inventory behavior remains
with the cited owners.

**Applies when:**

A `sugar` stack is created, looted, crafted, moved, renamed, persisted, synchronized, used on an
equine, offered to a Brewing Stand, consumed in a recipe, selected in a tab, rendered or observed
before and after tag, loot, recipe, advancement, mix or resource reload.

**Authoritative state:**

`minecraft:sugar` is raw item ID `1113`. It registers through the plain-item path with default
properties, is common, nondamageable and has max stack `64`.

Its registered components are only the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no food, consumable, cooldown, remainder, tool, equipment, repairable or
identity-specific glint state.

Its sole direct item tag is `#minecraft:horse_food`. The locked tag contains Wheat, Sugar, Hay
Block, Apple, Carrot, Golden Carrot, Golden Apple and Enchanted Golden Apple. Membership affects
the equine interaction joins below; it does not make Sugar player food.

**Transition and ordering:**

The identity does not override player hand use or block use. A prototype stack's air use returns
generic `PASS`; a block click participates only in ordinary block-first interaction and fallback
handling. A component-patched stack can activate a generic component owner, but the identity
itself never starts active use.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, equipment, repair, composting or furnace branch. Its locked
creation paths are two crafting recipes and Witch loot; its operational sinks are three recipes,
equine feeding and brewing.

**Crafting acquisition and progression:**

Both output recipes are shapeless and share recipe-book group `sugar`:

- one Sugar Cane returns one default Sugar and no remainder;
- one Honey Bottle returns three default Sugar and one default Glass Bottle remainder.

Neither output copies input component patches. The Honey Bottle's registered use remainder owns
the Glass Bottle; manual result take installs, merges or drops it through the generic per-slot
remainder algorithm. Additional occupied inputs prevent either one-ingredient recipe from matching.

Each recipe has its own no-display `recipes/root` advancement with one OR requirement. Sugar Cane
possession or exact `sugar_from_sugar_cane` recipe unlock awards only that recipe; Honey Bottle
possession or exact `sugar_from_honey_bottle` unlock awards only the other. Sugar possession itself
satisfies neither advancement and does not retroactively identify its source.

No other locked crafting, chest, fishing, gift, barter or trade output directly emits Sugar.
Sugar Cane and Honey Bottle acquisition remain with their own block/item/loot/trade owners;
administration and custom data can still create ordinary Sugar stacks through generic boundaries.

**Witch acquisition:**

The first pool of `entities/witch` draws a uniform inclusive roll count `1..3`. Each roll selects
independently from Glowstone Dust, Sugar, Spider Eye, Glass Bottle and Gunpowder at weight `1` plus
Stick at weight `2`; total weight is `7`, so Sugar selection is `1/7` per roll.

An admitted Sugar entry creates one stack, replaces its count with a uniform integer `0..2`, then
applies enchanted count increase. With a living attacking entity and Looting `L>0`, a fresh float
`V` in `[0,1)` adds `round(L*V)`; absent/nonliving attacker or level zero returns without that
draw. Thus the effective count is `B+round(L*V)` for base `B in 0..2`, and Looting can revive base
zero. There is no player-kill condition and no entry count limit.

Without positive Looting, base zero emits nothing and bases one/two emit that count. Repeated pool
rolls can select Sugar more than once. The later one-roll Redstone pool does not alter Sugar but
advances the same random sequence after every first-pool roll and applicable count branch.

The full table uses random sequence `minecraft:entities/witch`. Witch spawning/AI and death/table
admission remain with the mob/entity owners; weighted selection, count functions, splitting and
world-drop placement remain with `ITM-LOOT-001`, `ENT-DEATH-001` and
`ENT-ENTITY-DROPS-001`.

**Crafting sinks:**

Exactly three bundled recipes consume Sugar:

- Cake is shaped `AAA/BEB/CCC`: three Milk Buckets, two Sugar, one
  `#minecraft:eggs` member and three Wheat return one default Cake plus three Buckets;
- Pumpkin Pie is shapeless Pumpkin, Sugar and one `#minecraft:eggs` member, returning one default
  pie;
- Fermented Spider Eye is shapeless Spider Eye, Brown Mushroom and Sugar, returning one default
  Eye.

Each consumes the stated Sugar count with no Sugar remainder and copies no Sugar component patch.
Sugar possession is not an unlock criterion for any of the three: Cake uses egg possession,
Pumpkin Pie uses Pumpkin or Carved Pumpkin possession, and Fermented Spider Eye uses Spider Eye
possession; each also has its exact recipe-unlocked alternative. Result behavior remains with
`ITM-DRINK-CONTAINER-001`, `ITM-EGG-001` and `ITM-FERMENTED-SPIDER-EYE-001`.

**Brewing join:**

The vanilla mix builder registers Sugar through the start-mix helper. It adds Water plus Sugar to
Mundane and Awkward plus Sugar to Swiftness. Swiftness contains amplifier-zero Speed for `3600`
ticks (`180` seconds).

Redstone Dust, not another Sugar, extends Swiftness to `9600` ticks (`480` seconds); Glowstone Dust
creates amplifier-one Strong Swiftness for `1800` ticks (`90` seconds). Fermented Spider Eye
separately corrupts Swiftness to Slowness and Long Swiftness to Long Slowness.

Potion, Splash Potion and Lingering Potion all retain their container item while a matching source
holder receives fresh target contents. Holderless/custom-effect-only contents do not match.
Custom contents fields are not preserved. The ingredient test uses Sugar item identity, so
arbitrary Sugar component patches are accepted and discarded when the ingredient is consumed.

A completed brew transforms matching bottle slots `0..2` in order, consumes one Sugar for up to
three outputs, leaves unmatched bottles unchanged and emits event `1035`. Sugar has no remainder,
is not in `brewing_fuel`, is not furnace fuel and cannot prepay stand fuel. Fuel, the `400`-tick
transaction, cancellation, automation and player take criterion remain with `ITM-BREW-001` and
`ITM-ADVANCEMENT-001`; potion/effect behavior remains with `ITM-POTION-001` and
`ENT-EFFECT-001`.

**Ordinary unmounted equine feeding:**

Horse and Abstract-Chested-Horse interaction check the live `horse_food` tag before calling the
shared feed table, so this path applies to unmounted Horses, Donkeys and Mules unless an earlier
adult-tamed secondary-use inventory branch wins. Llamas, Camels and Zombie Horses override their
food tags, while an ordinary unmounted Skeleton Horse bypasses this food check.

Sugar assigns heal `1`, baby growth `30` seconds and temper increment `3`. The server applies these
in order:

1. if health is below maximum, heal `1` and mark success;
2. if baby, growth-positive and not age-locked, make the happy-particle offer, age up `30` seconds
   and mark success;
3. if temper is below maximum and either an earlier effect succeeded or the equine is untamed,
   increase temper by `3` through its clamped modifier and mark success.

If any branch succeeds, `eating()` opens the synchronized mouth, plays the subtype eating sound
unless silent at volume `1` and pitch `1+(U1-U2)*0.2`, then emits `EAT`. `fedFood` consumes one
Sugar through the player's ability-aware rule and returns server success. Consequently an untamed,
full-health adult can consume Sugar solely for temper, whereas a tamed full-health adult with no
growth/temper consequence returns `PASS` and consumes nothing. Sugar never sets love on this
ordinary unmounted table; only the golden foods have explicit love branches there.

**Vehicle dispatch quirk:**

Horse and Abstract-Chested-Horse interaction delegate to `AbstractHorse` before their food table
when the equine is a vehicle. A tamed Skeleton Horse also delegates directly. `AbstractHorse` in
turn delegates vehicle interaction to generic `Animal.mobInteract`, which sees inherited
`horse_food` membership but does not use Sugar's heal/growth/temper table.

For an adult server-player target not already in love, this generic route consumes one Sugar,
sets love time `600` ticks with the player as cause and broadcasts hearts. It does not require
missing health, inspect temper, call the equine `eating()` helper or emit its `EAT` game event;
generic `Animal.playEatingSound` is empty here. After dismount, only a tamed, adult, full-health
Horse or Donkey with a compatible mate can actually parent. Mule and Skeleton Horse can retain the
love state but their mating predicate remains false.

For an age-unlocked baby vehicle, the generic route instead consumes Sugar and advances it by
`floor((remainingBabyTicks/20)*0.1)` seconds with forced-age effects, rather than the fixed
`30`-second Sugar table. An adult already in love or age-locked baby has no server mutation and
consumes nothing, although the client-side generic food branch can temporarily predict consume.
Reloading `horse_food` removes or redirects all future admissions without changing existing
health, age, temper or love state.

**Persistence and reload boundary:**

Sugar stacks persist and synchronize identity, count and arbitrary ordinary component patches.
They store no Witch context/roll/count cursor, crafting source, recipe knowledge, brewing
slot/fuel/timer/holder or equine health/age/temper/love. Those values belong to the entity, loot,
recipe manager, player progression, Brewing Stand and animal owners.

Loot reload can replace future Witch drops; recipe/advancement reload can replace future crafting
and unlocks; a rebuilt baseline mix retains both Sugar start edges while holders/items are
feature-enabled. Tag reload controls future equine admission. Completed drops, crafts, brews and
feeds are not replayed. Resource reload independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1113` plus the stack's component patch. Its
common-rarity name uses locked English text `Sugar`; the plain class adds no subtype tooltip or
forced glint.

The direct item definition selects generated model `minecraft:item/sugar` and its same-named
texture. It appears exactly once and only in Ingredients, ordered Blaze Powder, Sugar, Rabbit's
Foot.

**Branches and aborts:**

Identity/count/components/tag; generic hand/block/container/anvil path; two output and three input
recipes plus unlock listeners; Witch rolls/weight/attacker/Looting/base/bonus and adjacent pool;
stand fuel/timer/container/holder/custom-content; equine subtype/vehicle/secondary-use/tame/health/
age-lock/temper/love/silence; save, tag/loot/recipe/advancement/mix/resource reload, wire, language,
model and tab context.

**Constants and randomness:**

Raw ID `1113`; common rarity; max stack `64`; crafting one Cane to one Sugar and one Honey Bottle
to three Sugar plus Bottle; Witch rolls `1..3`, Sugar weight `1/7`, base `0..2`, bonus
`round(L*U[0,1))`; recipe sinks Sugar counts Cake `2`, Pie `1`, Eye `1`; Swiftness payloads
`3600@0`, `9600@0`, `1800@1`; owner brew `400`; ordinary equine heal/growth/temper `1/30/+3`,
sound pitch `1+(U1-U2)*0.2`; vehicle love `600`, baby growth ten percent of remaining whole
seconds.

**Side effects:**

Possible Witch item stacks and named-sequence cursor; crafting inputs/results/remainders and recipe
knowledge; Brewing Stand ingredient/bottles/timer/event and potion/effect state; equine/player
stack, health, age, temper, love/cause, mouth, sound, particle/event state; ordinary stack
persistence/wire state; name, direct model and one Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; exact recipe inputs and active recipe snapshot; Witch
death/table and weighted/count/Looting branches; valid stand fuel plus Water/Awkward source holder;
live horse-food tag, equine dispatch and applicable ordinary/generic animal consequence;
registry/stack decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components/tags, crafting inputs/knowledge, Witch death/attacker/loot
state, brewing slots/fuel/timer/mix/contents, equine/player state, persistence and client resources.
Writes only the loot, crafting, progression, brewing, feeding, stack and client state listed above.

**Failure behavior:**

Player use has no subtype success. Invalid recipe inputs produce no result. Unselected Witch entry
or final count zero emits no Sugar. Missing fuel or unmatched holder prevents brewing. Removed tag,
earlier inventory dispatch or no applicable heal/growth/temper/love consequence consumes no Sugar
on the server. Missing/replaced loot, recipe, advancement, mix or tag data removes future paths
without rewriting completed state. Client resource absence follows generic fallback and cannot
grant authority.

**Boundary cases and quirks:**

Honey yields three Sugar but leaves a Glass Bottle; Sugar Cane yields one without a remainder.
Witch Sugar is ungated by player kill, and Looting can revive base zero. Sugar start-mixes Water to
Mundane as well as Awkward to Swiftness. Ordinary unmounted Sugar never breeds an equine, but the
vehicle-first dispatch can put an adult Horse/Donkey/Mule/Skeleton Horse into love; only Horse and
Donkey can later use that state to parent.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#run`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#run`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.world.level.storage.loot.entries.LootPoolSingletonContainer$EntryBase#getWeight`;
`net.minecraft.world.entity.animal.equine.Horse#mobInteract`;
`net.minecraft.world.entity.animal.equine.AbstractChestedHorse#mobInteract`;
`net.minecraft.world.entity.animal.equine.SkeletonHorse#mobInteract`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#mobInteract`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#fedFood`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#handleEating`;
`net.minecraft.world.entity.animal.Animal#mobInteract`;
`net.minecraft.world.entity.AgeableMob#getSpeedUpSecondsWhenFeeding`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addStartMix`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,entity_type,potion,mob_effect,recipe,loot_table,advancement}`;
`reports/minecraft/components/item/sugar.json`;
`data/minecraft/tags/item/horse_food.json`;
`data/minecraft/loot_table/entities/witch.json`;
`data/minecraft/recipe/{sugar_from_sugar_cane,sugar_from_honey_bottle,cake,pumpkin_pie,fermented_spider_eye}.json`;
`data/minecraft/advancement/recipes/{misc/{sugar_from_sugar_cane,sugar_from_honey_bottle},food/{cake,pumpkin_pie},brewing/fermented_spider_eye}.json`;
`assets/minecraft/{items,models/item,textures/item}/sugar.*`;
`ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-BREW-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-DRINK-CONTAINER-001`; `ITM-EGG-001`;
`ITM-FERMENTED-SPIDER-EYE-001`; `ITM-POTION-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `ENT-EFFECT-001`; `MOB-BREED-001`; `CLI-UI-001`;
`CLI-EFFECT-001`; `EXP-ITM-050`.

**Test vectors:**

Exercise default/patched Sugar through hands, blocks, containers and anvil. Match both output and
all three input recipes across permutations, remainders, extra inputs and every unlock criterion.
Kill Witches across roll/weight/base/attacker/Looting boundaries while tracing the full named
sequence. Brew Water/Awkward/Swiftness and all controls in every potion container. Feed every
Horse/Donkey/Mule/Skeleton/Zombie/Llama/Camel state across vehicle, secondary-use, tag, health,
age-lock, tame, temper, love and silence boundaries. Persist/synchronize and capture raw ID, name,
tooltip, model and exact Ingredients position before/after every reload domain.

**Limits:**

This leaf does not duplicate Sugar Cane/Honey acquisition, Witch spawning/AI/death, generic loot,
recipe/result-take, Brewing Stand, potion/effect, animal breeding/persistence or stack/resource
codecs. Those remain with their cited owners; this rule fixes Sugar identity and its exact
acquisition, crafting, brewing, equine and presentation joins.
