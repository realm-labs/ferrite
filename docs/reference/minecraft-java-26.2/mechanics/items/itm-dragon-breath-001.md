# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-DRAGON-BREATH-001` — Dragon's Breath converts Dragon-owned clouds into a Lingering-Potion reagent

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-BREW-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ITM-POTION-001`, `ENT-001`, `ENT-LIFECYCLE-001`,
`ENT-PROJECTILE-001`, `MOB-AI-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked uncommon plain-item registration/components, exhaustive typed
code/data references, Glass Bottle cloud branch and filled-result helper, ordered container mix,
single advancement and direct client assets determine every Dragon's-Breath-specific branch.
Generic interaction prediction, cloud creation/lifecycle, Brewing Stand transaction, potion
contents, progression, stacks and inventories remain with the cited owners.

**Applies when:**

A `dragon_breath` stack is created, moved, renamed, persisted, synchronized, offered to a Brewing
Stand, selected in a tab or rendered; or a Glass Bottle is used while an eligible Dragon-owned
area-effect cloud is inside the acquisition query before and after stack, entity, mix,
advancement or resource reload.

**Authoritative state:**

`minecraft:dragon_breath` is raw item ID `1320`. It registers through the plain-item path with
default properties, is uncommon, nondamageable and has max stack `64`. It belongs to no direct
item tag.

Its registered components are only the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no food, consumable, cooldown, remainder, tool, equipment, repairable or
identity-specific glint state.

**Transition and ordering:**

The Breath identity does not override hand use or block use. A prototype stack's air use returns
generic `PASS`; a block click participates only in ordinary block-first interaction and fallback
handling. A component-patched stack can activate a generic component owner, but the identity
itself never consumes a stack, starts active use, emits a sound/game event/particle, increments
item use or changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, mob-interaction, equipment, repair, furnace fuel, brewing fuel,
composting, trade, loot-table or crafting-recipe branch. Glass Bottle acquisition, one
advancement and one Brewing Stand container mix own its operational joins.

**Glass Bottle acquisition admission:**

`BottleItem.use` begins by querying area-effect clouds whose bounding boxes intersect the player's
current bounding box inflated by `2` on every axis. Its predicate accepts only a cloud that is
alive and whose resolved living owner is an `EnderDragon`.

The predicate does not inspect cloud potion contents, particle, age, wait state, duration,
radius, Dragon fight state or how the cloud was created. The branch performs no raycast,
line-of-sight, block permission or source-fluid test. Consequently any admitted cloud takes
precedence over the Bottle's later Water raycast.

When multiple clouds pass, the branch selects list index `0`, the first cloud returned by the
level entity query. It does not sort by distance, radius, age or entity ID and changes only that
one cloud.

No cloud makes this acquisition branch abort without Breath effects, after which the ordinary
source-only Water raycast can run under `ITM-POTION-001`. An alive Dragon-owned cloud is the only
locked survival acquisition source; no bundled loot, gift, trade, block/entity death or crafting
record directly emits Breath. Administration and custom data can still construct ordinary stacks.

**Admitted acquisition transaction:**

For the selected cloud, the authoritative server transaction runs in this order:

1. Read its current radius and call `setRadius(old - 0.5)`. The setter clamps to `0..32`, so a
   radius below `0.5` becomes `0`; it does not immediately discard the cloud.
2. Play dedicated event `item.bottle.fill_dragonbreath` at the player's position in the Neutral
   source, volume `1`, pitch `1`, excluding no entity from the server audience.
3. Emit `minecraft:fluid_pickup` at the player's position with the player as source.
4. For a server player, trigger `player_interacted_with_entity` using the pre-conversion held
   Glass Bottle stack and selected cloud.
5. Award the Glass Bottle `item_used` statistic.
6. Pass one newly constructed default count-one Breath stack to the generic filled-result helper,
   then return `SUCCESS` with the helper result marked as the hand transformation.

`AreaEffectCloud.setRadius` ignores the client-side call; its synchronized radius remains server
authoritative. The client can still predict the successful hand result and local observables, with
the interaction/correction owners converging to the server stack and entity metadata.

There is no positive-radius gate. A still-alive Dragon-owned cloud already at radius `0` remains
eligible, and another admitted use again clamps it to `0`. Removal occurs only when the cloud's
generic duration/radius-per-tick/lifecycle path later discards it; the Bottle branch itself never
does.

**Bottle count, ability and insertion branches:**

The output is always a fresh default Breath; Glass Bottle component patches are not copied.

For a survival player, the filled-result helper consumes one Bottle. If that empties the held
stack, the new Breath becomes the returned hand stack. Otherwise it tries to add the Breath to
inventory and drops it at the player without random throw when insertion fails; the reduced Bottle
stack remains held.

For an infinite-material player, the Bottle is retained. The helper first searches the complete
inventory for an item-and-components-equal default Breath. If one exists, it creates no additional
stack. Otherwise it attempts to insert one default Breath and ignores insertion failure. Cloud
radius, sound, game event, interaction criterion, Bottle-use stat and `SUCCESS` all occur before or
independently of whether this creative insertion adds an item.

Generic stack consumption, inventory merge, failed-insertion drop and hand synchronization remain
with `ITM-CONTAINER-001` and `ITM-POTION-001`.

**Brewing container join:**

The feature-enabled vanilla mix builder registers exactly one Breath container recipe:

`Splash Potion + Dragon's Breath -> Lingering Potion`.

It matches Breath by item identity, so ordinary component patches on the ingredient do not block
the edge. Potion and Lingering Potion containers do not match; Gunpowder owns the prerequisite
Potion-to-Splash edge. Breath is recognized only as a container-mix ingredient, not as a
potion-holder mix or `brewing_fuel`.

For each bottle slot `0..2` in order, a matching Splash Potion with a present base-potion holder is
replaced by one default Lingering Potion of that same holder. The conversion constructs a fresh
stack from only the holder: custom effects/color/name and unrelated component patches on the
Splash Potion are not copied.

The container edge makes any nonempty Splash Potion item brewable before inspecting its potion
contents. If such a stack has absent/empty contents with no base holder, the 400-tick transaction
can start and complete; `mix` then returns that bottle unchanged, but the stand still consumes one
Breath after processing all three slots and emits brew event `1035`. Other admitted slots in the
same transaction can still convert.

A successful stand commit transforms every matching slot, consumes only one ingredient Breath
total and leaves no remainder. Ingredient component changes do not cancel an active brew while its
item identity remains Breath, though final brewability is rechecked.

Fuel uses, timer/start/cancellation, bottle order, automation and persistence remain with
`ITM-BREW-001`. Base/custom potion effects, Lingering Potion throw/cloud behavior and the generic
unfiltered brewed-potion player-menu criterion remain with `ITM-POTION-001`,
`ITM-ADVANCEMENT-001` and the effect/entity owners.

**Progression:**

`end/dragon_breath` has one `inventory_changed` criterion requiring Breath. It uses one single-name
requirement, has no rewards or experience and sends its telemetry event on completion. The
display is a goal, uses Breath as its icon and has locked English title `You Need a Mint` and
description `Collect Dragon's Breath in a Glass Bottle`.

Its `end/kill_dragon` parent controls advancement-tree placement rather than adding a second
criterion; Breath possession is the complete local requirement. The earlier
`player_interacted_with_entity` trigger is a separate generic trigger for matching custom or
bundled listeners and is not this advancement's criterion.

If survival insertion fails, the dropped output does not satisfy the inventory criterion until it
is picked up. A count-one Bottle replacement or successful inventory insertion can satisfy it.
The creative no-duplicate path already requires an equal Breath somewhere in inventory; a failed
first insertion produces no new possession.

**Persistence and reload boundary:**

Breath stacks persist and synchronize identity, count and arbitrary ordinary component patches.
The selected cloud separately persists its owner/lifecycle fields and radius and synchronizes that
radius through entity data. Neither stack stores which cloud produced it, prior radius, Bottle
count/ability, query ordering, sound/event/stat/criterion state, brewing slot/fuel/timer/container
mix or advancement transaction.

A rebuilt baseline mix table retains the Splash-to-Lingering edge while all three item features
are enabled; existing stacks and in-flight stand state are not retroactively rewritten.
Advancement reload can independently replace the possession record. Resource reload independently
controls Breath and advancement language/model presentation. Cloud unload/reload and Dragon-owner
resolution remain with the entity lifecycle owners.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1320` plus the stack's component patch. Its
uncommon-rarity name uses locked English text `Dragon's Breath`; the plain class adds no subtype
tooltip or forced glint.

The direct item definition selects generated model `minecraft:item/dragon_breath` and its
same-named texture. It appears exactly once and only in Ingredients, ordered Gunpowder, Dragon's
Breath, Fermented Spider Eye.

The acquisition projects predicted/synchronized held stacks, the radius entity-data update,
neutral fill sound and `FLUID_PICKUP` game event through their generic client owners. The
advancement separately projects its goal display and telemetry-backed completion state.

**Branches and aborts:**

Breath identity/count/components; generic hand/block/container/anvil path; Glass Bottle hand/count/
patch/ability/inventory capacity; level side, cloud query/list order/alive/owner/radius/lifecycle;
sound/game-event/criterion/stat/result order; stand feature/fuel/slot/timer/ingredient identity,
Splash identity/base holder/custom content; advancement possession; save/entity unload/mix/
advancement/resource reload, wire, language, model and tab context.

**Constants and randomness:**

Raw item ID `1320`; uncommon rarity; max stack `64`; player-box inflation `2`; selected list index
`0`; radius decrement `0.5`; setter clamp `0..32`; sound volume/pitch `1/1`; output count `1`;
owner brew duration `400`; one Breath per completed transaction across up to three bottles. No
Breath-specific branch consumes RNG.

**Side effects:**

Possible cloud radius/entity-data change; sound and game event; generic entity-interaction
criterion; Bottle-use stat; Bottle consumption/retention; Breath hand/inventory/drop result;
Brewing Stand ingredient, bottle, timer/fuel/event state; advancement/telemetry state; ordinary
stack persistence/wire state; name, direct model and one Ingredients-tab entry.

**Gates:**

Generic interaction admission; at least one alive area-effect cloud with resolved Ender Dragon
owner inside the inflated player AABB; server authority for radius; inventory/ability result
branch; valid Brewing Stand fuel, feature-enabled container edge and at least one Splash Potion
identity; exact inventory possession criterion; valid registry/stack/entity decode; client
language/model and tab bootstrap.

**State read/written:**

Reads Breath and Bottle identity/count/components, player pose/bounds/ability/inventory, level
entity-query order, cloud alive/owner/radius/lifecycle, stand slots/fuel/timer/mix table,
advancement/player state, persisted stack/entity data and client resources. Writes only the
cloud/result/effect/stat, brewing, progression, persistence and client projection listed above.

**Failure behavior:**

Breath use itself has no subtype success or mutation. No eligible cloud falls through to the
Bottle's Water raycast or `PASS`. An unresolved/non-Dragon owner or dead/outside cloud is skipped.
Creative insertion failure loses the attempted output after all earlier acquisition side effects.
Missing fuel, disabled/missing mix or no Splash Potion prevents a brew; a holderless Splash Potion
can instead consume the transaction while remaining unchanged. Missing/replaced advancement or mix
data removes those future paths without rewriting stacks. Client resource absence follows generic
missing translation/model fallback and cannot grant authority.

**Boundary cases and quirks:**

The query is inflated-AABB and first-returned, not raycast/nearest, and ignores cloud effect,
visibility and positive radius. Bottling clamps a small cloud to zero without immediate discard,
so generic lifecycle timing controls whether it remains eligible. Creative mode retains the
Bottle and ensures at most one components-equal default Breath, but a full inventory can receive
none after the cloud was reduced. Brewing accepts holderless Splash identity, consumes Breath and
emits the event even when that bottle cannot be reconstructed. A valid conversion preserves only
the base potion holder, not custom potion details. The dedicated advancement observes later
inventory possession, not the earlier cloud-interaction trigger.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.BottleItem#use`;
`net.minecraft.world.item.BottleItem#turnBottleIntoItem`;
`net.minecraft.world.item.ItemUtils#createFilledResult`;
`net.minecraft.world.entity.player.Inventory#contains`;
`net.minecraft.world.entity.AreaEffectCloud#setRadius`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing#hasMix`;
`net.minecraft.world.item.alchemy.PotionBrewing#mix`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addContainerRecipe`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.advancements.packs.VanillaTheEndAdvancements`;
`reports/registries.json#minecraft:item`;
`reports/minecraft/components/item/dragon_breath.json`;
`data/minecraft/advancement/end/dragon_breath.json`;
`assets/minecraft/{items,models/item,textures/item}/dragon_breath.*`;
`ITM-BREW-001`; `ITM-POTION-001`; `ITM-ADVANCEMENT-001`;
`ENT-LIFECYCLE-001`; `ENT-PROJECTILE-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-048`.

**Test vectors:**

Exercise default/patched Breath stacks through hands, blocks, containers and anvil. Use
default/patched Bottle stacks at every count/ability/inventory-capacity boundary with zero/one/
multiple clouds across AABB edges, query orders, alive state, every owner, radius below/equal/
above `0.5` and lifecycle timing; trace both sides and every side effect in order. Brew all
default/custom/holderless Potion, Splash and Lingering containers across three slots, fuel/timer/
feature/ingredient-component changes and reload. Trigger interaction versus possession criteria
with held/inserted/dropped/creative outputs. Persist/reload/synchronize stacks/clouds and capture
raw ID, rarity, name, tooltip, model, advancement display and exact Ingredients position before/
after resource reload.

**Limits:**

This leaf does not duplicate Glass Bottle Water acquisition, generic item/container interaction,
area-effect-cloud or Ender Dragon creation/lifecycle/AI, Brewing Stand transaction, potion effects
or Lingering Potion runtime, advancement engine or client correction. Those remain with their
cited owners; this rule fixes the Breath identity and its exact cloud-output, container-mix,
progression and presentation joins.
