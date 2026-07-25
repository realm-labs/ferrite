# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ARMADILLO-SCUTE-001` — Armadillo Scutes join timed shedding and brushing to Wolf Armor crafting and repair

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ITM-DISPENSER-001`, `ENT-001`, `ENT-005`,
`ENT-DAMAGE-REDUCE-001`, `MOB-001`, `MOB-004`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, the sole direct tag, every exact
`Items.ARMADILLO_SCUTE` class reference, both acquisition tables, Armadillo and Wolf interaction
code, the Wolf Armor material/components, its shaped recipe and three relevant advancements, the
anvil material path and direct client resources determine every scute-specific branch. Generic
loot, crafting, anvil, equipment, entity and stack behavior remains with the cited owners.

**Applies when:**

An Armadillo's shedding timer expires, a player or Dispenser brushes an Armadillo, Scutes occupy a
Wolf Armor recipe or anvil addition slot, a player uses one on a sitting armored Wolf, or a Scute
stack is moved, renamed, persisted, synchronized or projected in Ingredients before and after
component, tag, loot, recipe, advancement or resource reload.

**Authoritative state:**

`minecraft:armadillo_scute` is raw item ID `917`. It is a common, nondamageable plain `Item` with
maximum stack `64`, no food, consumable, use, remainder, equipment, tool, cooldown, repairable,
fuel, compost or identity-specific glint behavior.

Alongside the common empty modifiers/enchantments/lore, break sound, name, model, repair cost,
swing animation, tooltip display and use effects, it has no noncommon default component. Its sole
direct tag is `#minecraft:repairs_wolf_armor`, and that tag contains only Armadillo Scute.

Default Wolf Armor is a one-stack body item with maximum damage `64`, current damage `0`, armor
value `11`, allowed entity Wolf, shearing enabled and repairable set
`#minecraft:repairs_wolf_armor`. Its armor material uses durability multiplier `4`, body defense
`11`, enchantability `10`, equip sound `armor.equip_wolf` and equipment asset
`minecraft:armadillo_scute`. Those are Wolf Armor state, but they determine the exact crafting and
repair consequences of Scutes.

**Transition and ordering:**

Timed natural shedding:

An Armadillo constructor initializes `scute_time` to
`nextInt(20 * SECONDS_PER_MINUTE * 5) + 20 * SECONDS_PER_MINUTE * 5`, hence uniformly
`6000..11999` server AI ticks. Each alive Armadillo decrements it once in
`customServerAiStep`, after its brain/activity update and before the generic Animal AI tail.

When the decremented value remains positive, shedding does nothing. At zero or below,
`shouldDropLoot` admits only an adult while `mob_drops` is true. A baby or disabled rule leaves the
timer at zero/negative and retries on every later alive AI tick; growing up or enabling the rule
therefore makes the next tick eligible without drawing a replacement delay.

An admitted attempt evaluates `gameplay/armadillo_shed`, a one-roll `gift` table whose sole entry
is one default Armadillo Scute, under named sequence `minecraft:gameplay/armadillo_shed`. Each
emitted stack is spawned at the Armadillo. If evaluation reports at least one emitted stack, the
Armadillo plays `armadillo_scute_drop` at volume `1` and pitch
`1 + (nextFloat() - nextFloat()) * 0.2`, then emits game event `ENTITY_PLACE`. An empty, missing or
reloaded table can report no emission and suppress both effects.

The admitted attempt draws and stores a new `6000..11999` timer whether table evaluation emitted
an item or not. The death table `entities/armadillo` is empty, so killing an Armadillo never uses
this path and emits no Scute from its ordinary entity table.

Brushing:

Armadillo interaction tests an exact Brush before its scared-state rejection and ordinary Animal
interaction. `brushOffScute` rejects babies only. Every adult is admitted regardless of scared
state, natural timer, `mob_drops`, prior brushes or player ownership; there is no brushing
cooldown and brushing does not change `scute_time`.

On the server, an admitted brush evaluates the one-roll `entity_interact` table
`brush/armadillo`, whose sole entry is one default Scute and whose named sequence is
`minecraft:brush/armadillo`. The context includes the Armadillo, the interacting entity and the
Brush stack; player brushing supplies the player and Dispenser brushing supplies null. Emitted
stacks spawn at the Armadillo.

The table's boolean result is deliberately ignored. After the attempt, an adult always plays
`armadillo_brush`, emits `ENTITY_INTERACT` and returns true even when the table emitted nothing.
The player interaction then requests `16` durability damage from the held Brush in its hand slot
and returns `SUCCESS`. A default ordinary-player Brush takes all `16`; the common durability
helper gives infinite-materials server players zero damage and lets applicable enchantments
modify positive damage. The Dispenser scans eligible front Armadillos in encounter order, applies
the same method to the first adult, requests `16` damage with no player ability bypass and stops;
all-baby/no-target scans retain the tool and report failure as specified by
`ITM-DISPENSER-001`.

Because the adult branch returns success, a server player interaction triggers
`adventure/brush_armadillo` afterward from the pre-interaction Brush copy and post-interaction
Armadillo. Its exact Brush and Armadillo predicates then grant “Isn't It Scute?” even if a
reloaded brush table emitted no Scute. Dispenser calls have no player criterion.

Wolf Armor recipe and unlock:

No bundled recipe creates an Armadillo Scute. The shaped `wolf_armor` recipe consumes six:

```text
X..
XXX
X.X
```

where every `X` is an exact Armadillo Scute and dots are empty. Because the pattern occupies all
three rows and columns, it has no grid translation; horizontal mirroring permits the single top
Scute at either upper corner. Extra items, a filled upper-middle or lower-middle cell, wrong
identities or missing Scutes fail.

Taking the result consumes all six, emits one default Wolf Armor and leaves no remainder.
Arbitrary Scute component patches do not propagate. The no-display `recipes/root` child has one
OR requirement containing exact recipe unlock and possession of an Armadillo Scute; either grants
the recipe independently.

Direct repair on a Wolf:

Wolf interaction considers this repair only after tame-wolf food, collar-dye and empty-body
equipment branches. The default Scute reaches it because it is neither Wolf food nor a collar dye
and is not itself equippable. Admission requires all of:

1. the Wolf is tame and the player is its owner;
2. the Wolf is currently in its sitting pose;
3. its body slot contains damaged Wolf Armor;
4. that armor's live `repairable` holder set admits the offered stack.

The default holder set resolves the sole-member `repairs_wolf_armor` tag. Admission directly
shrinks one Scute, plays `wolf_armor_repair`, computes
`floor(currentArmorMaxDamage * 0.125)`, subtracts that from current damage with a zero clamp and
returns `SUCCESS`. Default Wolf Armor therefore repairs exactly `8` damage per Scute. The outer
player interaction restores a decreased held count for infinite-materials players; ordinary
players spend one.

The success then evaluates `husbandry/repair_wolf_armor` using the pre-interaction Scute copy and
the post-interaction Wolf. It requires exact Armadillo Scute and body-slot Wolf Armor with damage
exactly zero, so only the interaction that fully repairs the armor grants “Good as New.” A
successful partial repair still consumes, sounds and returns success without granting it.

Untamed, non-owner, standing, unarmored, undamaged or nonrepairable cases fall through to later
Wolf behavior and consume no Scute in this branch. Live tag removal blocks default repair.
Replacing the equipped armor's `repairable` component can independently admit or reject the
candidate, and its live maximum damage changes the one-eighth amount through integer truncation.

Anvil material repair:

The same default repairable holder lets Armadillo Scutes repair Wolf Armor in an anvil. This is a
separate generic material transaction: each addition removes up to
`floor(currentArmorMaxDamage / 4)` damage, costs one operation level and repeats until repaired or
the addition is exhausted. Default Armor therefore repairs up to `16` per Scute and uses at most
four from maximum damage, unlike the direct Wolf interaction's `8`.

Anvil preview/take, prior-work cost, experience, rename, output repair-cost update, input
consumption and anvil-damage RNG remain `ITM-ANVIL-001`. Removing tag membership or patching away
the armor's repairable component blocks both default direct and anvil material admission; it does
not invalidate already repaired armor.

Persistence and reload boundary:

Scute stacks persist and synchronize identity, count and arbitrary ordinary component patches.
They store no Armadillo timer, loot cursor, recipe knowledge, craft/anvil transaction, Wolf owner,
pose or equipped armor; those belong to entity, world, player and menu owners.

Armadillos persist `scute_time` exactly alongside their state. Unload pauses the countdown with
no wall-clock catch-up. A missing saved field retains the constructor's freshly sampled timer;
loading a present zero or negative value preserves it for the next eligibility check.

Loot reload changes future shedding/brushing evaluation; recipe/advancement reload changes future
matching and grants; item-tag or Wolf Armor component replacement changes future repair admission.
Completed drops, crafts, repairs and grants are not replayed. Resource reload independently
replaces language, item model/texture, advancement text/icons and Wolf Armor equipment textures.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `917` plus the component patch. The common English
name is `Armadillo Scute`; the plain item adds no subtype tooltip or forced glint.

Its direct item definition selects generated model `minecraft:item/armadillo_scute` and the
same-named texture. It appears once in Ingredients, ordered Turtle Scute, Armadillo Scute, Slime
Ball. The Wolf Armor equipment asset named `armadillo_scute` separately selects base and dyeable
overlay Wolf-body textures; naming that asset after the material does not equip or render a loose
Scute. The item adds no packet layout or numeric mapping.

**Branches and aborts:**

Identity/count/components and repair tag; Armadillo alive/age/rule/timer/table/emission and saved
field; player/dispenser brushing, age/context/table/emission/tool durability and criterion; recipe
orientation/grid/take/unlock; Wolf tame/owner/pose/body/damage/repairable/ability and advancement;
anvil preview/take; save, component/tag/loot/recipe/advancement/resource reload; wire, language,
model, texture, advancement and tab.

**Constants and randomness:**

Raw ID `917`; common rarity; max stack `64`; shed timer `6000 + nextInt(6000)` ticks; shed pitch
`1 + (nextFloat - nextFloat) * 0.2`; shed/brush table output `1`; requested Brush damage `16`;
recipe input `6`, output `1`; default Wolf Armor max damage `64`, direct repair
`floor(64/8)=8`, anvil repair `floor(64/4)=16`; tab neighbors Turtle Scute/Slime Ball.

**Side effects:**

Spawned Scute entities and named loot cursors; Armadillo timer, sounds and game events; Brush
durability and player criterion; recipe grant, six-input crafting result; Wolf Armor damage,
repair sound, Scute consumption and advancement; anvil preview/result/experience/inputs/block
event; ordinary stack persistence/wire and client presentation.

**Gates:**

Alive adult Armadillo, `mob_drops`, timer and shed table; adult brush target and interaction loot
context; recipe grid and snapshot; tame owner, sitting pose, damaged equipped body armor and live
repairable membership; anvil inputs/cost/player; registry/stack decode; client language/model/tab
context.

**State read/written:**

Reads Scute identity/count/components/tags, Armadillo age/timer/world rule/loot context, Brush and
interactor, recipe/advancement records, Wolf tame/owner/pose/equipment and armor components, anvil
inputs/player/block context, persistence and resources. Writes only the loot, Armadillo, tool,
progression, crafting, Wolf Armor, anvil, stack and client state listed above.

**Failure behavior:**

Positive timer, baby status or disabled mob drops suppresses shedding; the last two retain an
expired timer. Empty shed loot resets the timer without item, sound or event. Babies reject
brushing without a durability request; an adult empty brush table still reaches durability,
sound and event work. Invalid recipe grids produce no result. Failed Wolf repair gates consume
nothing; partial repair does not grant the advancement. Missing/replaced reloadable data changes
only future attempts. Client-resource absence follows generic fallback and cannot grant
authority.

**Boundary cases and quirks:**

Natural shedding is mob-drops-gated but brushing is not. An expired ineligible shed timer retries
each tick instead of resampling. Brushing has no cooldown and does not reset the natural timer.
Adult brush side effects ignore whether loot emitted. Wolf direct repair restores one eighth while
anvil material repair restores one quarter. The repair advancement observes post-repair damage
zero but tests the pre-interaction exact Scute. The same `armadillo_scute` name denotes both the
loose item and a Wolf Armor equipment asset, without making the loose item equippable.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#customServerAiStep`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#pickNextScuteDropTime`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#mobInteract`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#brushOffScute`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#addAdditionalSaveData`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#readAdditionalSaveData`;
`net.minecraft.world.entity.LivingEntity#shouldDropLoot`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.world.item.ItemStack#isValidRepairItem`;
`net.minecraft.world.inventory.AnvilMenu#createResult`;
`net.minecraft.world.item.Item$Properties#wolfArmor`;
`net.minecraft.world.item.equipment.ArmorMaterials`;
`net.minecraft.world.item.equipment.EquipmentAssets`;
`net.minecraft.server.network.ServerGamePacketListenerImpl`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.loot.packs.VanillaEntityInteractLoot`;
`net.minecraft.data.tags.VanillaItemTagsProvider`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement}`;
`reports/minecraft/components/item/{armadillo_scute,wolf_armor}.json`;
`data/minecraft/tags/item/repairs_wolf_armor.json`;
`data/minecraft/loot_table/{gameplay/armadillo_shed,brush/armadillo,entities/armadillo}.json`;
`data/minecraft/recipe/wolf_armor.json`;
`data/minecraft/advancement/{adventure/brush_armadillo,husbandry/repair_wolf_armor,recipes/combat/wolf_armor}.json`;
`assets/minecraft/{items,models/item,textures/item}/armadillo_scute.*`;
`assets/minecraft/equipment/armadillo_scute.json`;
`assets/minecraft/textures/entity/equipment/wolf_body/armadillo_scute*.png`;
`ITM-DISPENSER-001`; `ITM-ANVIL-001`; `ENT-DAMAGE-REDUCE-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-059`.

**Test vectors:**

Run natural timers at `1/0/-1/6000/11999` across adult/baby, alive/dead and mob-drops on/off
transitions; force emitted/empty/missing shed tables, exact delay and pitch draws, save/unload/load
and named cursors. Brush adult/baby and scared/idle Armadillos repeatedly with player and
Dispenser contexts under emitted/empty tables; assert no cooldown/timer mutation and exact tool,
sound, event and criterion boundaries.

Match mirrored/unmirrored and every invalid Wolf Armor grid, possession/recipe grants and patched
Scutes. Cross default/enchantment-patched Brushes with ordinary/infinite player and Dispenser
durability contexts. Direct-repair tame/untamed, owner/non-owner, sitting/standing, armor
absent/full/damage `1/8/9/63/64`, finite/infinite and default/removed/patched repair membership;
assert count, sound, damage and post-state advancement. Repeat through anvil at all
damage/addition/cost boundaries. Capture raw ID, common name/glint, item model/texture, Wolf
equipment-asset separation and Ingredients neighbors.

**Limits:**

This leaf does not duplicate generic loot execution, shaped crafting, anvil commits, Wolf Armor
damage absorption/rendering, Armadillo AI, Dispenser scanning, advancement persistence or
stack/resource codecs. Those remain with their cited owners; this rule fixes Armadillo Scute
identity and its exact acquisition, crafting, repair and presentation joins.
