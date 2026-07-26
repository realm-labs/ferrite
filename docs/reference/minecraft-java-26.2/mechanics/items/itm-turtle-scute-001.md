# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-TURTLE-SCUTE-001` — Turtle Scute joins one-shot Turtle adulthood loot to Helmet crafting, repair, Water Breathing, brewing and two villager purchases

**Parent:** `SIM-004`, `SIM-005`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`,
`ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`,
`ITM-RECIPE-001`, `ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ITM-EQUIP-001`, `ITM-BREW-001`, `ENT-001`,
`ENT-005`, `ENT-EFFECT-001`, `MOB-001`, `MOB-004`, `MOB-BREED-001`,
`WGEN-PIPELINE-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, the sole direct tag, Turtle/AgeableMob growth
code, both Turtle loot tables, the Turtle Helmet recipe/components/material/player effect and
brewing join, both exact trade records/sets, all `1,212` decoded templates and direct client
resources determine every Turtle-Scute-specific branch. Generic aging, loot, crafting, anvil,
equipment, effects, brewing, merchant, persistence, packet and rendering algorithms retain their
cited owners.

**Applies when:**

A Turtle crosses from baby to adult, Turtle Scutes occupy a crafting grid, anvil addition,
villager input, stack/container/save/wire field or client tab, or a resulting Turtle Helmet is
equipped, repaired or used as a brewing ingredient before and after loot, recipe, advancement,
tag, trade, potion or resource reload.

**Authoritative state:**

`minecraft:turtle_scute` is raw item ID `916`, a common nondamageable plain `Item` with maximum
stack `64`. It has no food, consumable, use, remainder, equipment, tool, cooldown, durability,
fuel, compost or identity-specific glint behavior. Its sole direct item tag is
`#minecraft:repairs_turtle_helmet`, whose sole member is Turtle Scute.

Default Turtle Helmet is raw item ID `915`, a one-stack head item with maximum/current damage
`275/0`, armor/toughness `2/0`, enchantability `9`, Turtle equip sound and equipment asset
`minecraft:turtle_scute`. Its repairable component names the Scute tag. These are Helmet state,
but they determine the Scute's exact crafting and repair consequences.

**Transition and ordering:**

### One-shot Turtle adulthood acquisition

An ordinary baby begins at age `-24000`. On each alive server AI tick, an unlocked baby increments
age by one. `AgeableMob#setAge` invokes its boundary callback only when the sign changes. Turtle's
override first runs the generic boundary behavior, then continues only when the new state is
adult, the level is a server level and `mob_drops` is true.

An admitted boundary evaluates `gameplay/turtle_grow` under named sequence
`minecraft:gameplay/turtle_grow` with the Turtle as the gift context. Its one roll has one entry:
one default Turtle Scute. Emitted stacks spawn at the Turtle. The table's boolean result is
discarded; growth emits no Scute-specific sound or game event.

This is one-shot. If `mob_drops` is false, or a missing/reloaded table emits nothing, the Turtle
still becomes adult and later rule enablement or loot reload does not retry. Changing an adult
back to a baby does not emit; a later negative-to-nonnegative crossing attempts again. The Turtle
death table emits only Seagrass and a lightning-conditioned Bowl, never a Scute.

Seagrass is the sole `turtle_food` member. Feeding an unlocked baby with `R=-age` remaining ticks
consumes one and advances
`20 * floor(floor(R / 20) * 0.1)` ticks, clamped at adulthood. Equivalently it removes ten percent
of whole remaining seconds, truncated to whole seconds. At `R<200`, the admitted feed can consume
and succeed while advancing zero; natural ticking later crosses the boundary. An age-locked baby
does not admit this acceleration. Golden Dandelion locking/unlocking and generic forced-age,
particle and interaction correction remain `MOB-BREED-001`.

### Helmet recipe and direct unlock

No bundled recipe produces Turtle Scute. The sole direct recipe consumes five in:

```text
XXX
X.X
```

The width-three, height-two shape may occupy the upper or lower two rows of a `3x3` grid.
Horizontal mirroring is identical. Wrong identities, missing Scutes, a filled center or any extra
item fail. Taking the result consumes five and emits one default Turtle Helmet without copying
Scute patches.

The no-display recipe advancement has one OR requirement: prior Turtle-Helmet recipe knowledge or
possession of exact Turtle Scute. Either independently grants the recipe. Pattern normalization,
result capacity, atomic consumption, knowledge publication and persistence remain generic.

### Anvil material repair

The default Helmet's live repairable holder set admits Turtle Scute. Generic anvil material
repair removes up to `floor(currentHelmetMaxDamage / 4)` damage per addition, charges one
operation level per consumed material and repeats until repaired or exhausted. Default maximum
`275` therefore removes up to `68` per Scute: damage `275` requires five, because four remove
`272` and the fifth removes the final `3`.

Damage zero, a quarter of zero, removed tag membership, a patched-away repairable component,
insufficient input/cost or a non-Scute addition blocks or bounds the generic transaction.
Patching the Helmet maximum changes the quarter through integer truncation. Preview/take,
prior-work cost, XP, rename, input consumption and anvil-damage RNG remain `ITM-ANVIL-001`.

### Resulting Helmet behavior and brewing boundary

The default Helmet equips only in the head slot and supplies armor `2`. Each player tick checks
whether the eyes are outside the Water fluid tag and exact Turtle Helmet is validly equipped. If
so, it requests amplifier-zero Water Breathing for `200` ticks, with ambient false, particles
hidden and icon visible. Repeated dry ticks refresh through generic effect merging. Entering water
or removing the Helmet stops refresh; the existing effect then follows generic countdown/merge
rules. Durability damage, armor reduction, enchantments and rendering remain their dedicated
owners.

Hardcoded brewing accepts exact Turtle Helmet, not loose Turtle Scute, for Awkward to Turtle
Master. The brewing stand consumes one admitted Helmet to transform up to three admitted bottle
stacks after the generic transaction. Turtle Master effect contents and Redstone/Glowstone
extensions remain `ENT-EFFECT-001`, `ITM-BREW-001` and their ingredient leaves. Thus five Scutes
can indirectly reach brewing only after the fixed Helmet craft.

### Two level-four villager purchases

Leatherworker level four selects amount two without replacement from exactly two candidates, so
its `4` Scutes to one Emerald offer is guaranteed. Cleric level four selects amount two from three,
so the identical offer has inclusion probability `2/3`. Both have maximum uses `12`, XP `30` and
reputation discount `0.05`. Trade Rebalance replaces neither record, tag nor set. Offer
construction consumes nothing; generic trading owns adjusted cost and atomic exchange.

An exhaustive decoded scan finds zero exact Turtle-Scute identities across all `1,212` structure
templates. The adulthood table is the only bundled loot table with a direct Scute entry. No chest,
death, fishing, archaeology, gift, barter or template provides another direct source.

**Persistence and reload boundary:**

Scute stacks, Turtle age/age-lock state, recipe knowledge, Helmet state, effects and merchant
offers persist with their owners. Unload pauses Turtle aging without wall-clock catch-up. Loot
reload changes only a future adulthood boundary and never replays a completed one. Recipe,
advancement, tag, trade and potion reload changes future matching, grants, repair, offer
construction or brewing only. Existing offers and completed growth/craft/repair/equip/brew/trade
transactions retain state. Resource reload independently changes projection.

**Wire and client projection:**

Generic stack publication uses item ID `916`; no Turtle-Scute-specific packet exists. English
name is `Turtle Scute`. Its item definition selects one untinted `item/generated` model and
`minecraft:item/turtle_scute` texture with no condition, animation, tint or special renderer.

Ingredients orders Glow Ink Sac, Turtle Scute, Armadillo Scute and Slime Ball. Turtle Scute appears
once and in no other ordinary creative tab. The same-named equipment asset separately maps
Turtle Helmet to adult and baby humanoid equipment textures. This asset does not make a loose
Scute equippable.

**Branches and aborts:**

Default/patched Scute and repair tag; age negative/zero/positive, natural/feeding/direct set,
locked/unlocked, alive/dead, server/client, mob-drops on/off, table emitted/empty and repeated
crossing; recipe upper/lower/invalid and direct/prior unlock; Helmet default/patched
damage/maximum/repairable and anvil cost; dry/wet/equipped/removed/effect merge; Helmet/Scute
brewing controls; both profession sets/current costs; zero templates; persistence/reload/wire/
client branches are distinct.

**Constants and randomness:**

Scute/Helmet IDs `916/915`; Scute stack `64`; baby start age `-24000`; growth output `1`;
feeding acceleration `20*floor(floor(R/20)*0.1)`; recipe input/result `5/1`; Helmet
damage/armor/toughness/enchantability `275/2/0/9`; anvil repair `floor(275/4)=68`; effect
duration/amplifier `200/0`; Leatherworker inclusion `1`, Cleric `2/3`; exchange
`4:1`, uses/XP/discount `12/30/0.05`; templates/matches `1212/0`.

**Side effects:**

Turtle age and spawned Scute; food/age-lock state under generic owners; recipe grant and
five-input Helmet result; anvil preview/result/XP/inputs; equipped attributes and Water Breathing;
brewing ingredient/result; villager input/output, uses, XP and economy effects; durable owner
state, synchronization and exact client projection.

**Gates:**

Alive server baby/unlocked age transition, `mob_drops` and growth table; Turtle-food interaction;
shaped grid/result capacity and inventory/knowledge criterion; live repairable holder, damage and
anvil inputs/cost; valid head equipment and eye-fluid state; exact Helmet/brew state/fuel/bottles;
profession/level/set/current cost; registry/stack decode and client resources.

**Boundary cases and quirks:**

Turtle death never emits a Scute; adulthood is the sole direct loot source. A disabled rule or
empty table loses that crossing permanently rather than retrying. Feeding below `200` remaining
ticks can consume Seagrass while accelerating zero. Five Scutes craft a `275`-damage Helmet and
five are also required to repair it from maximum damage because quarter repair truncates to `68`.
The Helmet refreshes Water Breathing only while eyes are dry, then leaves a `200`-tick reserve for
submersion. Turtle Helmet, not Turtle Scute, is the brewing ingredient. The loose item and Helmet
equipment asset share a name without sharing equipment behavior.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.entity.AgeableMob#aiStep`;
`net.minecraft.world.entity.AgeableMob#setAge`;
`net.minecraft.world.entity.AgeableMob#ageUp`;
`net.minecraft.world.entity.animal.Animal#mobInteract`;
`net.minecraft.world.entity.animal.turtle.Turtle#ageBoundaryReached`;
`net.minecraft.world.entity.LivingEntity#dropFromGiftLootTable`;
`net.minecraft.world.item.ItemStack#isValidRepairItem`;
`net.minecraft.world.inventory.AnvilMenu#createResult`;
`net.minecraft.world.item.equipment.ArmorMaterials`;
`net.minecraft.world.entity.player.Player#turtleHelmetTick`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,entity_type,potion,mob_effect,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{turtle_scute,turtle_helmet}.json`;
`data/minecraft/tags/item/{repairs_turtle_helmet,turtle_food}.json`;
`data/minecraft/loot_table/{gameplay/turtle_grow,entities/turtle}.json`;
`data/minecraft/recipe/turtle_helmet.json`;
`data/minecraft/advancement/recipes/combat/turtle_helmet.json`;
`data/minecraft/{villager_trade/{leatherworker/4/turtle_scute_emerald,cleric/4/turtle_scute_emerald},tags/villager_trade/{leatherworker/level_4,cleric/level_4},trade_set/{leatherworker/level_4,cleric/level_4}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/turtle_scute.*`;
`assets/minecraft/equipment/turtle_scute.json`;
`assets/minecraft/textures/entity/equipment/{humanoid,humanoid_baby}/turtle_scute.png`;
`assets/minecraft/lang/en_us.json`;
`ITM-ANVIL-001`; `ITM-BREW-001`; `ENT-EFFECT-001`; `MOB-BREED-001`;
`EXP-ITM-096`.

**Test vectors:**

Run `EXP-ITM-096` across natural, fed, locked and direct-set Turtle ages at every sign and
truncation boundary, both mob-drops states and emitted/empty growth tables; assert one-shot
emission and death absence. Match upper/lower/invalid Helmet grids and both unlock alternatives.
Repair default/patched Helmets at every damage, material, tag, component and cost boundary.

Tick equipped/removed Helmets through dry/wet transitions and effect merges; brew Helmet versus
Scute/control ingredients; construct and transact both profession sets. Scan every template,
persist/reload/synchronize owners and assert raw IDs, names, model/texture, Ingredients order and
separate adult/baby Helmet equipment textures.

**Limits:**

Generic aging, food interaction, loot execution, shaped crafting, anvil commits, equipment,
effect merging, brewing, merchant, persistence, packet and renderer control flow remains with
cited owners. Turtle AI/death, Helmet combat, potion contents and client humanoid rendering retain
their dedicated owners. This leaf fixes Turtle Scute identity and its exact acquisition, sink,
absence and projection joins.
