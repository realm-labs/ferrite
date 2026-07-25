# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-PHANTOM-MEMBRANE-001` — Phantom Membranes join two loot paths to Elytra repair and Slow Falling brewing

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`PLY-MOVE-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`,
`ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-BREW-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ITM-POTION-001`,
`ENT-001`, `ENT-005`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`ENT-EFFECT-001`, `MOB-SPAWN-001`, `MOB-PHANTOM-SPAWN-001`, `MOB-AI-001`,
`MOB-BREED-001`, `WGEN-DIMENSION-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, exhaustive code/data references, both direct
loot tables, cat goal and environment-attribute inputs, Elytra repair component, brewing graph,
potion payloads and client assets determine every Membrane-specific branch. Generic Phantom and
cat behavior, loot, anvil, brewing, effects, stacks and inventories remain with the cited owners.

**Applies when:**

A `phantom_membrane` stack is created, looted, moved, renamed, persisted, synchronized, offered to
an anvil or brewing stand, selected in a tab, rendered or observed before and after loot, mix,
timeline or resource reload.

**Authoritative state:**

`minecraft:phantom_membrane` is raw item ID `889`. It registers through the plain-item path with
default properties, is common, nondamageable and has max stack `64`. It belongs to no direct item
tag.

Its registered components are only the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no food, consumable, cooldown, remainder, tool, equipment, repairable or
identity-specific glint state.

**Transition and ordering:**

The identity does not override hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but the identity itself never
consumes a stack, starts active use, emits a sound/game event/particle, increments item use or
changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, mob-interaction, equipment, composting or villager branch. Phantom
loot, cat gift loot, target-side repairability and brewing own the operational joins below.

**Phantom acquisition:**

The `entities/phantom` table has one pool and one roll. Its `killed_by_player` condition is tested
before the entry, so failure consumes neither the base-count nor Looting-bonus draw and emits no
Membrane.

An admitted pool creates one Membrane, replaces its count with a uniformly drawn integer `0..1`,
then applies enchanted count increase. With a living attacking entity whose Looting level is
`L > 0`, the second function draws a fresh uniform float `U` in `[0,1)` and adds
`round(L * U)`. With no living attacker or `L = 0`, it returns without that draw or bonus. No count
limit is configured, and a zero base count can be revived by a positive Looting bonus before
generic empty-stack filtering.

The table uses random sequence `minecraft:entities/phantom`. Player-kill attribution and the
attacking entity used by Looting are separate context facts: the pool requires the former, while
the optional bonus additionally requires the latter. Phantom special spawning and insomnia gates
remain with `MOB-PHANTOM-SPAWN-001`; death admission, table invocation, empty-stack filtering and
world-drop placement remain with `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

**Cat morning-gift acquisition:**

The second baseline source is `gameplay/cat_morning_gift`. A tame cat's relax-on-owner goal can
start only while it is not ordered to sit, its player owner is sleeping within squared distance
`100`, the owner occupies a bed, and no other nearby cat already occupies the selected relaxation
space. Cat tame/owner state and goal scheduling remain with `MOB-BREED-001` and `MOB-AI-001`.

When that goal stops, it first clears lying state. Only an owner sleep timer of at least `100`
reaches the chance draw. It consumes one level RNG float and invokes the gift path exactly when
that value is strictly below the live
`minecraft:gameplay/cat_waking_up_gift_chance` environment attribute at the cat. The attribute
defaults to `0`; the locked circular `day` timeline supplies constant keyframes tick `362` value
`0` and tick `23667` value `0.7`, so the normal wake marker at tick `0` resolves `0.7`. Dimension,
timeline and modifier resolution remain with `WGEN-DIMENSION-001`.

After chance success, cat RNG chooses attempted teleport offsets `nextInt(11)-5`,
`nextInt(5)-2`, `nextInt(11)-5` around the leash holder when leashed or the cat otherwise. The
teleport result is ignored, and gift context is built from the cat's resulting position and
identity.

The one-roll gift table has six alternatives of weight `10`—Rabbit Hide, Rabbit Foot, Chicken,
Feather, Rotten Flesh and String—and Membrane at weight `2`. Total weight is `62`, so conditioned
on table evaluation Membrane is selected with probability `1/31` and emits one default stack.
Combined with live gift chance `g`, a qualified goal stop emits Membrane with probability `g/31`;
at the normal locked `g=0.7` wake value this is `7/310`.

The gift table uses random sequence `minecraft:gameplay/cat_morning_gift`, distinct from the
preceding level and cat RNG draws. Its output callback inserts an item entity at the cat's block
position offset one horizontal unit along body rotation; insertion failure is ignored.

No bundled chest, fishing, block-drop, trade or other entity/gift table directly emits a Membrane.
Administration and custom data can still create ordinary stacks through generic item/loot
boundaries.

**Elytra repair join:**

Membrane is not intrinsically a repair tool. The default Elytra prototype's `repairable` component
contains exactly `minecraft:phantom_membrane`; anvil admission reads the base stack's current
component. Removing or replacing that component rejects Membrane, while a patched damageable item
whose repair set includes Membrane accepts it.

For the default max-damage `432` Elytra, each accepted Membrane removes up to
`floor(432/4)=108` damage and adds one material-repair operation level. The loop stops when damage
reaches zero or the addition stack is exhausted, records exactly the number of Membranes used and
consumes that many on result take. Thus damage `432` needs four, while damage `1..108` needs one.

Material repair preempts enchantments carried by the addition stack and copies no Membrane
component patch into the output. Prior-work cost, rename/enchantment compatibility, pickup levels,
addition consumption, anvil damage and failure behavior remain with `ITM-ANVIL-001`.

**Brewing join:**

The vanilla mix builder registers the direct edge Awkward plus Membrane to Slow Falling. It is an
`addMix` edge, not a start mix: Water plus Membrane does not become Mundane, and every other
non-Awkward baseline potion remains unmatched. The target potion contains one amplifier-zero Slow
Falling effect for `1800` ticks (`90` seconds).

Redstone Dust, not another Membrane, owns the later Slow Falling to Long Slow Falling edge, whose
effect lasts `4800` ticks (`240` seconds). No Glowstone strong form is registered.

A completed brew transforms every matching bottle slot in owner order, then consumes one
ingredient Membrane with no remainder and emits the generic brew event. Membrane is not a direct
member of `brewing_fuel`, is not furnace fuel and cannot prepay the stand's fuel uses; a separate
valid fuel source is required.

Slot admission, fuel uses, 400-tick timer/cancellation, bottle transforms, automation and the
player-menu take criterion remain with `ITM-BREW-001` and `ITM-ADVANCEMENT-001`. Potion
consumption/projection and effect merge/ticks remain with `ITM-POTION-001` and `ENT-EFFECT-001`;
Slow Falling's movement consequence remains with `PLY-MOVE-001`.

No locked crafting recipe consumes or emits Membrane. Taking a Membrane-produced potion from a
Brewing Stand as a server player can independently satisfy the unfiltered `nether/brew_potion`
criterion; automation extraction does not run that player slot hook.

**Persistence and reload boundary:**

Membrane stacks persist and synchronize identity, count and arbitrary ordinary component patches.
They store no Phantom death/attacker context, Looting level, count draws, cat owner/goal/timeline
state, gift selection, repair transaction, brewing slot/fuel/timer or potion mix. Those values
belong to their entity, world, loot, anvil and machine owners.

Loot reload can independently replace either source table for future evaluations. Timeline or
dimension data reload changes future live gift-chance resolution. A rebuilt baseline mix table
retains the direct Awkward edge while its holders are enabled; existing stacks, repaired Elytra
and in-flight machine state are not retroactively rewritten. Prototype/stack components control
future repair admission. Resource reload independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `889` plus the stack's component patch. Its
common-rarity name uses locked English text `Phantom Membrane`; the plain class adds no subtype
tooltip or forced glint.

The direct item definition selects generated model `minecraft:item/phantom_membrane` and its
same-named texture. It appears exactly once and only in Ingredients, ordered Turtle Helmet,
Phantom Membrane, Field Masoned Banner Pattern.

**Branches and aborts:**

Identity/count/components; generic hand/block/container/anvil path; Phantom player-kill gate,
attacker/Looting/base/bonus draws; cat tame/sit/owner/sleep/bed/range/space/goal-stop state, live
gift chance, teleport and weighted selection; target repairable/damage/addition state; brewing
fuel/bottle/potion state; save, loot/timeline/mix/resource reload, wire, language, model and tab
context.

**Constants and randomness:**

Raw item ID `889`; common rarity; max stack `64`; Phantom base count uniform integer `0..1`;
Looting bonus `round(L * U)` for `U` uniform `[0,1)`; cat owner squared range `100`, minimum sleep
timer `100`, gift attribute default `0`, day keyframes `362→0` and `23667→0.7`, teleport offsets
`[-5,5]/[-2,2]/[-5,5]`; gift weight `2/62=1/31`, combined normal-wake rate `7/310`; Elytra max
damage `432` and repair quantum `108`; Slow Falling durations `1800/4800`; owner brew duration
`400`.

**Side effects:**

Possible Phantom or gift loot stack and named-sequence cursor; chance/teleport RNG and cat
position; generic world drop/pickup; Elytra damage, level/addition/anvil state; brewing ingredient,
bottles/timer/event; brewed-potion progress; ordinary stack persistence/wire state; name, direct
model and one Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; player-attributed Phantom death and living attacker for
Looting; qualified cat relax goal, owner sleep timer, live attribute chance and current gift
table; target-side repairable component and positive damage; valid brewing fuel, Awkward bottle
and enabled mix; valid registry/stack decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, Phantom death/attacker
context, both loot sequences, cat/owner/bed/goal/sleep/position/RNG state, environment attribute,
target damage/repair component, anvil state, brewing slots/fuel/timer/mix table, progression state,
persisted stacks and client resources. Writes only the loot, cat movement, repair, brewing,
progression, stack and client projection listed above.

**Failure behavior:**

Use has no subtype success or mutation. A non-player-attributed Phantom kill, suppressed/missing
table or zero final count emits no Membrane; absent living attacker removes only the Looting
bonus. An unqualified cat goal, sleep timer below `100`, failed chance or alternate weighted entry
emits none; failed teleport does not cancel a successful gift, and failed gift insertion is
ignored. Missing/changed target repairability or zero damage rejects material repair. Missing fuel
or a non-Awkward bottle prevents a brew; Membrane is not itself fuel. Missing/replaced loot,
timeline or mix data removes those future paths without rewriting stacks. Client resource absence
follows generic missing translation/model fallback and cannot grant authority.

**Boundary cases and quirks:**

Phantom loot requires player-kill attribution even though its Looting bonus separately tests a
living attacker, and a zero base can be revived by that bonus. Cat acquisition spends a chance
draw before teleport RNG and a distinct named-sequence gift draw; teleport failure is
non-transactional. Membrane repairability belongs to the target stack rather than the ingredient.
Membrane directly brews only Awkward, with no Water start edge or strong form, and cannot fuel the
stand.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.ItemStack#isValidRepairItem`;
`net.minecraft.world.inventory.AnvilMenu#createResult`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#run`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#stop`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#giveMorningGift`;
`net.minecraft.world.entity.LivingEntity#dropFromGiftLootTable`;
`net.minecraft.world.attribute.EnvironmentAttributes`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`reports/registries.json#minecraft:{item,potion,mob_effect,environment_attribute}`;
`reports/minecraft/components/item/{phantom_membrane,elytra}.json`;
`data/minecraft/loot_table/{entities/phantom,gameplay/cat_morning_gift}.json`;
`data/minecraft/timeline/day.json`;
`assets/minecraft/{items,models/item,textures/item}/phantom_membrane.*`;
`ITM-BREW-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ITM-ANVIL-001`;
`ITM-POTION-001`; `ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `MOB-PHANTOM-SPAWN-001`;
`MOB-AI-001`; `MOB-BREED-001`; `WGEN-DIMENSION-001`; `PLY-MOVE-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-045`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate Phantom
deaths across player attribution, attacker types, Looting levels and every count/float boundary
while tracing the named sequence. Sweep every cat goal admission/stop, sleep-timer and live
attribute boundary, teleport result and gift weight with all three RNG sources separated. Repair
default and component-patched Elytra/other items across damage/addition/cost boundaries. Brew
Water, Awkward, Slow Falling and every other potion with valid/invalid fuel before/after mix
replacement. Persist/synchronize stacks and capture raw ID, name, tooltip, model and exact
Ingredients position before/after reload.

**Limits:**

This leaf does not duplicate Phantom spawning/AI/death, cat taming/relaxation, environment
attribute resolution, generic loot emission, anvil calculation/take, brewing transaction/
automation, potion/effect/movement behavior or stack/resource codecs. Those remain with their
cited owners; this rule fixes the Membrane identity and its exact acquisition, repair, brewing,
progression and presentation joins.
