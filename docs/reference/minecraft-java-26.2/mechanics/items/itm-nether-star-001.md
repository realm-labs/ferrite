# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-NETHER-STAR-001` — Nether Stars turn one extended-lived Wither drop into an explosion-resistant Beacon ingredient

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `RED-EXPLOSION-001`,
`BLK-BEACON-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked item registration/components, exhaustive code/data references,
Wither custom-death branch, item-entity damage/lifetime methods, one Beacon recipe and unlock, and
direct client assets determine every Nether-Star-specific branch. Generic Wither death, world-item
movement/damage/pickup, crafting, progression, stacks and inventories remain with the cited
owners.

**Applies when:**

A `nether_star` stack is created, dropped by a Wither, exposed as a world item, picked up, moved,
renamed, persisted, synchronized, offered to crafting, selected in a tab, rendered or observed
before and after damage-tag, recipe or resource reload.

**Authoritative state:**

`minecraft:nether_star` is raw item ID `1270`. It uses the plain-item registration path, is rare,
nondamageable and has max stack `64`. It belongs to no direct item tag.

Its prototype adds `minecraft:damage_resistant={types:"#minecraft:is_explosion"}` and
`minecraft:enchantment_glint_override=true` to the common empty modifiers/enchantments/lore,
item-break sound, translated name, direct item-model key, repair cost, swing animation, tooltip
display and use effects. It has no food, consumable, cooldown, remainder, tool, equipment or
repairable state.

The locked `is_explosion` damage-type tag contains exactly `fireworks`, `explosion`,
`player_explosion` and `bad_respawn_point`. This tag is a predicate used by the component; it is
not item membership.

**Transition and ordering:**

The identity does not override hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but the identity itself never
consumes a stack, starts active use, emits a sound/game event/particle, increments item use or
changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, mob-interaction, equipment, repair, fuel, brewing, composting or
villager branch. Wither death, world-item damage and the recipe consumer own the operational joins
below.

**Wither acquisition and extended lifetime:**

The only locked baseline source is `WitherBoss#dropCustomDeathLoot`; no bundled loot table directly
emits a Nether Star. Inside the generic mob-loot gate, the Wither first invokes its superclass
custom-drop hook and then spawns exactly one default Nether Star at its death location. The method
does not inspect recent player damage, kill credit or Looting, so those values do not change the
count. Closing the generic loot gate, including the owner-defined `mob_drops` suppression, prevents
the custom branch.

If world-item creation returns an entity, the Wither immediately calls `setExtendedLifetime`,
setting its age to `-6000`. Ordinary item ticking increments every finite age and discards at age
`6000`, giving this created entity 12,000 admitted age-increment ticks rather than the ordinary
6,000. This is entity state, not a stack component: pickup followed by an ordinary later drop
creates the usual age-zero item entity. Unload pauses ticking, while item-entity save/reload
preserves the signed age.

Generic spawn positioning, pickup delay, initial motion, merge behavior, pickup and death ordering
remain with `ENT-DEATH-001` and `ENT-ENTITY-DROPS-001`. Merging compatible world stacks retains the
owner-defined minimum age, so an extended-lived entity can transfer the earlier age to the merged
entity without modifying either stack's components.

**Explosion resistance:**

`ItemEntity#hurtServer` applies base invulnerability and its mob-griefing source gate first. It
then asks the current stack `canBeHurtBy` before marking hurt, changing health or emitting its
damage game event. A default Nether Star returns false for any source in `is_explosion`, so those
four damage types leave world-item health, lifetime and stack unchanged and the method reports no
admitted hurt.

Other damage types remain generic: the resistance does not imply fire, lava, cactus, void or
all-damage immunity. A component patch that removes/replaces `damage_resistant`, or a reload that
changes `is_explosion`, changes future admission without changing identity. Explosion exposure,
knockback and block phases remain with `RED-EXPLOSION-001`; this leaf owns only the stack-selected
damage rejection.

The component applies to every world-item entity holding a qualifying current stack, not only the
one created by a Wither. Conversely, the 12,000-tick lifetime applies only when the Wither invoked
the entity setter and is not granted to administratively created or normally dropped stars.

**Recipe and progression:**

The sole bundled recipe consuming a Star is shaped `beacon`:

```text
GGG
GSG
OOO
```

`G` is Glass, `S` is one Nether Star and `O` is Obsidian. Five Glass, one Star and three Obsidian
return one default Beacon. The recipe copies no input component patch and no ingredient has a
remainder. Generic shaped matching, consumption and result transfer remain with `ITM-RECIPE-001`
and `ITM-CRAFT-001`; later Beacon placement, beam and pyramid effects remain with
`BLK-BEACON-001`.

The recipe advancement places Star possession and exact `beacon` recipe-unlocked criteria in one
two-entry OR requirement; either awards only the Beacon recipe. The separate
`nether/summon_wither` advancement uses a Nether Star only as its display icon. Its sole criterion
requires the player to summon a Wither, sends its configured telemetry event, and neither checks
for nor rewards a Star. Killing the Wither is not that summoning criterion.

No reverse recipe, trade, chest, fishing, block-drop or other entity-drop record produces a Star.
Administration and custom data can still create an ordinary stack through generic item boundaries.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches, including
damage resistance and glint override. They store no killer credit, loot gate, Wither source,
world-item age/health/pickup delay, recipe identity or advancement progress.

A live world item separately saves its stack plus signed `Age`, `Health`, `PickupDelay`, owner and
thrower state; reloading a Wither-created Star therefore retains the remaining extended lifetime.
Picking it up discards that entity state. Damage-tag reload changes future component predicate
resolution. Recipe/advancement reload can replace crafting and unlock records without rewriting
stacks. Resource reload independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1270` plus the stack's component patch. The
default rare name uses locked English text `Nether Star` and has no subtype tooltip. Its true
glint-override component forces the ordinary unenchanted stack to glint; an explicit patched false
value suppresses that forced result under the generic client component owner.

The direct item definition selects generated model `minecraft:item/nether_star` and its same-named
texture. It appears exactly once and only in Ingredients, ordered Heavy Core, Nether Star, Ender
Pearl.

**Branches and aborts:**

Identity/count/components; generic hand/block/container/anvil path; admitted Wither death and
generic loot gate; world-item creation success, age/tick/unload/save/merge/pickup; damage source,
base invulnerability, mob-griefing source gate and live damage-resistance tag; shaped grid/counts;
Star possession versus recipe-unlocked and summon-Wither criterion; save, data/resource reload,
wire, glint, language, model and tab context.

**Constants and randomness:**

Raw item ID `1270`; rare rarity; max stack `64`; one Star per admitted Wither custom drop; created
age `-6000`; discard threshold `6000`; effective lifetime `12000` admitted increments; Beacon
inputs five Glass, one Star and three Obsidian to one Beacon. No identity-specific random draw
changes the custom-drop count, lifetime, resistance, recipe or presentation.

**Side effects:**

One possible world stack and extended item age; generic item motion/merge/pickup state; rejected or
admitted world-item damage; crafting inputs/result; advancement and recipe known/highlight state;
ordinary stack/item-entity persistence and wire state; forced glint, name, direct model and one
Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; admitted Wither death and open mob-loot gate; successful
world-item creation; finite ticking age and loaded entity; base/item damage admission and live
damage tag; exact crafting grid; exact inventory or recipe-unlocked criterion; summoned-Wither
event for the icon-only advancement; valid registry/stack decode; client language/model and tab
bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, Wither death/loot state,
world-item age/health/pickup/merge state, damage source/tags/gamerule, recipe/advancement registries
and player progression state, persisted stack/entity and client resources. Writes only the
drop/entity, damage response, crafting, progression, stack and client projection listed above.

**Failure behavior:**

Use has no subtype success or mutation. A suppressed loot gate or failed world-item creation leaves
no extended-lived Star entity. Picking the item up loses the extended entity lifetime even though
the stack survives. Explosion-tagged damage is rejected before health/event mutation; untagged
damage follows generic item-entity behavior. Invalid or insufficient crafting leaves inputs
unchanged. Missing/replaced recipe or advancement data removes those future paths without
rewriting stacks. Client resource absence follows generic missing translation/model fallback and
cannot grant authority.

**Boundary cases and quirks:**

The drop is custom code inside the loot gate but has no player-kill or Looting condition. Extended
lifetime belongs to that created item entity, whereas explosion resistance belongs to the stack
component and follows the stack through pickup and redrop. Explosion resistance does not make the
item fire-resistant. The summoning advancement's Star icon is presentation only and does not
describe the later death drop.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.entity.boss.wither.WitherBoss#dropCustomDeathLoot`;
`net.minecraft.world.entity.item.ItemEntity#hurtServer`;
`net.minecraft.world.entity.item.ItemEntity#setExtendedLifetime`;
`net.minecraft.world.item.ItemStack#canBeHurtBy`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaNetherAdvancements`;
`reports/registries.json#minecraft:item`;
`reports/minecraft/components/item/nether_star.json`;
`data/minecraft/tags/damage_type/is_explosion.json`;
`data/minecraft/recipe/beacon.json`;
`data/minecraft/advancement/{recipes/misc/beacon,nether/summon_wither}.json`;
`assets/minecraft/{items,models/item,textures/item}/nether_star.*`;
`ITM-RECIPE-001`; `ITM-ADVANCEMENT-001`; `ENT-DAMAGE-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `RED-EXPLOSION-001`; `BLK-BEACON-001`; `CLI-UI-001`;
`CLI-EFFECT-001`; `EXP-ITM-041`.

**Test vectors:**

Exercise default and component-patched stacks through hands, blocks, containers and anvil. Kill
Withers across loot-gate, killer and Looting states; trace created entity, age, pickup delay,
motion, merge, unload/reload, pickup/redrop and despawn thresholds. Apply every damage type before
and after tag/component reload while recording admission, health, event and removal. Match/craft
the Beacon grid and trigger both recipe-unlock alternatives plus the independent summoning
advancement. Persist/synchronize stacks and item entities and capture raw ID, rarity, forced glint,
name, tooltip, model and exact Ingredients position before/after resource reload.

**Limits:**

This leaf does not duplicate Wither summoning/AI/death, generic mob-loot gating, item-entity
movement/merge/pickup/damage, explosion calculation, crafting consumption, recipe-book/advancement
state or Beacon block behavior. Those remain with their cited owners; this rule fixes the Star
identity, custom drop/lifetime, stack resistance, recipe, progression and presentation joins.
