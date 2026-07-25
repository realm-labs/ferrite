# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-NAUTILUS-ARMOR-001` — Nautilus armor equips one tamed adult mount and makes zombie protection nondamageable

**Parent:** `PLY-005`, `PLY-006`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`,
`ITM-006`, `ITM-007`, `ENT-001`, `ENT-002`, `ENT-005`, `MOB-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item registration/components, generic equippable and equipment code,
nautilus interaction/menu code, zombie daylight handling, recipes, advancements, loot tables, tags
and client assets close all five armor identities and their normal/zombie-nautilus joins.

**Applies when:**

A scoped armor stack is generated, transformed, recycled, used on or dispensed toward an entity,
inserted into or removed from a nautilus menu, equipped, sheared, damaged as an item entity,
persisted, reloaded or projected; or its attributes and zombie-nautilus daylight protection are
evaluated.

**Authoritative state:**

| tier | item raw ID | equipment asset | armor | toughness | knockback resistance | extra identity |
|---|---:|---|---:|---:|---:|---|
| copper | `1368` | `minecraft:copper` | `4` | `0` | — | — |
| iron | `1364` | `minecraft:iron` | `5` | `0` | — | — |
| golden | `1365` | `minecraft:gold` | `7` | `0` | — | direct `piglin_loved` member |
| diamond | `1366` | `minecraft:diamond` | `11` | `2` | — | — |
| netherite | `1367` | `minecraft:netherite` | `19` | `3` | `0.10000000149011612` | resists `#minecraft:is_fire` |

Every identity is a common plain `Item` with maximum stack size one, ordinary break sound,
empty enchantments, repair cost zero and the listed body-slot add-value modifiers. None has
`max_damage`, `damage`, `repairable` or `enchantable`; all five are nondamageable, cannot be
repaired, and have no ordinary enchanting-table or supported-book route. A command- or
component-patched stack can still carry enchantments, including an effect that prevents armor
change.

Every `equippable` component selects body, the tier asset,
`#minecraft:can_wear_nautilus_armor`, `item.armor.equip_nautilus`, equip-on-interact, shearing and
`item.armor.unequip_nautilus`. It explicitly disables damage-on-hurt; defaults leave dispenser
and swap support enabled. The locked allowed-entity tag contains exactly entity IDs `88`
(`nautilus`) and `153` (`zombie_nautilus`). Both entities admit body/saddle slots only while
alive, adult and tamed.

**Transition and ordering:**

### Direct equip, dispenser and mount menu

`AbstractNautilus.interact` first marks the entity persistence-required, even when the later
interaction rejects. The inherited entity interaction handles leash/shears before
`AbstractNautilus.mobInteract`. A baby delegates without trying held-item equip. A tamed adult
under secondary use attempts to open its menu before food or held-item hooks; if it is a vehicle,
only its own passenger may open it. Ordinary use checks taming/healing food before invoking the
stack's entity-interaction hook.

The armor hook accepts only a live allowed target whose body slot admits use and is empty. Under
the locked tag this is a tamed adult normal or zombie nautilus. The server splits exactly one
held item, including in creative mode, writes body, marks that slot guaranteed-drop, emits the
generic equip callback with the nautilus equip sound, and returns success without awarding the
generic item-used statistic. Untamed, baby, dead, disallowed and occupied targets pass to later
mob interaction without an armor mutation. Secondary use on a tamed adult opens the menu rather
than directly equipping the held armor.

`ITM-DISPENSER-001` owns the retained four-tick dispatch and encounter-order target search. Its
dynamic equippable branch additionally requires `canUseSlot`, empty body and the nautilus
body/saddle dispenser override. Success moves one stack to body, marks guaranteed drop and
persistence; no qualifying target takes the nested default ejection path.

The nautilus inventory has zero cargo columns. Its menu contains exactly saddle slot `0`, body
slot `1`, then the 36 player slots. Both equipment slots are active only under the alive,
adult, tamed gate. Body has maximum size one and admits a stack only when its equippable slot and
live allowed-entity membership match the mount. Insertion calls the equipment callback, marks
body guaranteed-drop and persistence-required. Pickup is denied by `prevent_armor_change` for a
noncreative player. The menu remains valid while the mount is alive, its zero-size backing
container is unchanged, and the player is riding it or remains within entity-interaction range
`4.0`. A rider can therefore secondary-use the menu to remove armor even though passenger state
blocks shearing.

### Shearing, death and tag invalidation

Shears first remove any leash connections and return after one shears damage. Otherwise equipment
shearing requires a live mob with no passengers and ordinary rather than secondary use. The slot
scan reaches BODY before SADDLE, so an armored and saddled nautilus loses armor on the first
qualifying equipment-shear click. `prevent_armor_change` blocks a noncreative shear.

Success damages shears once, empties body, emits `UNEQUIP` and `SHEAR`, spawns the exact stored
stack at the passenger attachment, triggers `player_sheared_equipment`, and plays
`item.armor.unequip_nautilus`. Direct, dispenser and menu insertion all make the equipped stack a
guaranteed equipment death drop; generic entity-death ownership performs the lethal release.

The allowed-entity named holder set is live across data reload. Removing a nautilus type prevents
later direct, dispenser and menu insertion but does not evict a stored body stack. Existing
attribute modifiers, daylight protection and rendering read the equipped stack and remain active;
menu/shears removal still can recover it. Adding a body-slot-capable living type can widen later
generic equip admission beyond the locked two-entity membership.

### Defense and zombie daylight protection

While equipped, the exact body modifiers join generic armor/toughness damage reduction and, for
netherite, knockback resistance. `damage_on_hurt=false` prevents the generic equipment-damage
pass, and the stacks are nondamageable independently. Netherite's item entity ignores damage
types in live `#is_fire`; this component does not make the wearer fireproof.

`zombie_nautilus` belongs to live `#minecraft:burn_in_daylight` and overrides its sun-protection
slot from HEAD to BODY. On each admitted server sunlight burn tick, any nonempty body stack
returns before the eight-second ignition. A damageable protector would add `nextInt(2)` damage
and could break, but every scoped armor is nondamageable, consumes no draw in that branch and
protects indefinitely until removed. Ordinary nautilus does not enter the daylight-burn path.

### Acquisition, transformation and progression

Each of `underwater_ruin_small`, `underwater_ruin_big`, `buried_treasure`,
`shipwreck_map`, `shipwreck_supply` and `shipwreck_treasure` has one independent armor-pool roll:
empty/copper/iron/golden/diamond weights are exactly `148/20/10/5/2`, total `185`, and each armor
entry sets count one. Thus one evaluated pool yields any armor with probability `1/5`, split
`4/37`, `2/37`, `1/37` and `2/185`; no scoped chest table emits netherite. Structure owners fix
chest installation while `ITM-LOOT-001` owns deferred evaluation and random-sequence state.

Copper, iron and golden armor are explicit alternatives in their matching nugget smelting and
blasting recipes. One input emits one default copper/iron/gold nugget, discards the source patch,
and grants `0.1` experience after the generic `200`/`100`-tick process. Possessing the matching
armor is one OR criterion that can unlock each of its two recycling records.

The only recipe that emits scoped armor is
`netherite_nautilus_armor_smithing`: netherite-upgrade template, exact diamond armor base and
`#minecraft:netherite_tool_materials` addition yield one netherite armor while the generic
smithing-transform owner copies the base patch. Direct recipe unlock or possession of the
addition tag unlocks it. Copper, iron, golden and diamond armor have no crafting recipe; diamond
has no recycling recipe, and no bundled trade, fuel or composting record handles any tier.

### Projection

Each identity has a direct generated flat item model and like-named item texture. The shared
`copper`, `iron`, `gold`, `diamond` and `netherite` equipment assets each provide one
`nautilus_body` layer with a same-named texture. Normal and zombie renderers copy body equipment
into render state and use the common `NautilusArmorModel`; normal base geometry independently
selects adult/baby, while zombie variant geometry independently selects normal/coral form.

The nautilus menu uses `textures/gui/container/nautilus.png`, saddle and
`container/slot/nautilus_armor_inventory` placeholders and no chest-slot sprite. Combat orders
copper, iron, golden, diamond, netherite after wolf armor and before totem of undying, despite
the raw registration order placing copper last. Ordinary search visibility follows the five
items' common registration.

**Branches and aborts:**

Normal/zombie and live-tag-expanded entity; server/client; alive/dead; adult/baby; tame/untamed;
ordinary/secondary use; food/nonfood; empty/occupied body; direct/dispenser/menu insertion;
rider/nonrider vehicle; leash/no leash; body/saddle shear order; creative/noncreative prevention;
allowed-tag invalidation; daylight tag/environment/light/weather/sky admission; damageable
near-miss; every loot weight, recycle/smithing input, reload and render context.

**Constants and randomness:**

Item raw IDs `1364..1368` with copper at `1368`; entity IDs `88/153`; stack maximum `1`;
attributes `4/5/7/11/19`, toughness `0/0/0/2/3`, netherite knockback resistance
`0.10000000149011612`; menu range `4.0`; loot weights `148/20/10/5/2`; one armor count; recycle
experience `0.1`, times `200/100`; sunlight ignition `8` seconds. Equip sound seeding consumes
the generic entity RNG long. Scoped nondamageable sun protection consumes no `nextInt(2)`;
loot and generic death owners retain their own randomness.

**Side effects:**

Held/dispenser/menu stack count, body equipment and attributes, guaranteed-drop and persistence
flags, shears damage, recovered/death item entity, sunlight ignition suppression, recipe output
and experience, recipe unlocks, sounds, criteria, game events, menu state and client
item/equipment projection.

**Gates:**

Live allowed-entity, burn-in-daylight, fire-damage and piglin-loved tags; life, age, tame,
equipment, passenger, leash and secondary-use state; dispenser candidate order; enchantment
prevention; environment/light/weather/sky state; loot weights; recipe ingredients/unlock
snapshot; current resources.

**State read/written:**

Reads stack identity/components/count, live tags, target type/life/age/tame/equipment/passengers,
player use/creative state, dispenser candidates, environment attributes/light/weather/sky,
recipes, loot, advancements and render state. Writes stack count, body equipment/drop chance,
persistence flag, shears damage, item entities, recipe outputs/experience, progression, events,
menus and projection state.

**Failure behavior:**

Rejected direct equip passes into later food/ride interaction; secondary use opens a qualifying
menu first. Rejected dispenser admission ejects through the default behavior. A leash removal
wins over equipment shear; passengers or secondary use prevent shear. Invalidated allowed
membership retains existing equipment. Failed daylight admission does not inspect body. Wrong
recipe inputs and nonselected loot weights emit no scoped armor.

**Persistence boundary:**

Armor identity, component patch, body equipment and drop chance persist through generic
stack/entity `equipment` codecs. Tame/age and the nautilus passenger graph persist through their
entity owners. Interaction, menu operation in progress, shear, sunlight evaluation, sounds and
events do not resume. Data reload replaces allowed/burn/fire/piglin tags, seven recipes, seven
recipe advancements and six loot tables without rewriting stored stacks/equipment/progress.
Resource reload replaces item/equipment models, textures, menu sprites and language.

**Boundary cases and quirks:**

Creative direct equip still consumes the sole held item. Every nautilus interaction marks
persistence before admission. Secondary use opens the menu before held armor can directly equip.
BODY precedes SADDLE under shearing, but a rider can remove body armor through the menu. All five
tiers have large armor modifiers yet no durability or ordinary repair/enchant route.
Zombie-nautilus sunlight protection checks only nonempty BODY, so nondamageable armor never
degrades and remains protective even after allowed-entity tag invalidation. Netherite protects
the dropped item, not its wearer, from fire damage types. Creative-tab order differs from raw IDs.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.Item$Properties#nautilusArmor`;
`net.minecraft.world.item.equipment.ArmorMaterial#createAttributes`;
`net.minecraft.world.item.equipment.Equippable`; `net.minecraft.world.item.ItemStack`;
`net.minecraft.world.entity.Entity#interact`; `net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.Mob`; `net.minecraft.world.entity.animal.nautilus.AbstractNautilus`;
`net.minecraft.world.entity.animal.nautilus.ZombieNautilus#sunProtectionSlot`;
`net.minecraft.world.inventory.NautilusInventoryMenu`;
`net.minecraft.client.gui.screens.inventory.NautilusInventoryScreen`;
`net.minecraft.client.renderer.entity.NautilusRenderer`;
`net.minecraft.client.renderer.entity.ZombieNautilusRenderer`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/{copper,iron,golden,diamond,netherite}_nautilus_armor.json`;
`data/minecraft/tags/entity_type/{can_wear_nautilus_armor,burn_in_daylight}.json`;
`data/minecraft/tags/item/piglin_loved.json`;
`data/minecraft/recipe/{copper,iron,gold}_nugget_from_{smelting,blasting}.json`;
`data/minecraft/recipe/netherite_nautilus_armor_smithing.json`;
`data/minecraft/advancement/recipes/{misc/*_nugget_from_*,combat/netherite_nautilus_armor_smithing}.json`;
`data/minecraft/loot_table/chests/{underwater_ruin_small,underwater_ruin_big,buried_treasure,shipwreck_map,shipwreck_supply,shipwreck_treasure}.json`;
`assets/minecraft/equipment/{copper,iron,gold,diamond,netherite}.json`;
`assets/minecraft/{items,models/item,textures/item}/*_nautilus_armor.*`;
`assets/minecraft/textures/entity/equipment/nautilus_body/{copper,iron,gold,diamond,netherite}.png`;
`PLY-INTERACT-001`; `ITM-USE-001`; `ITM-CONTAINER-001`; `ITM-DISPENSER-001`;
`ITM-FURNACE-001`; `ITM-SMITHING-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`;
`ITM-ENCHANT-001`; `ITM-ANVIL-001`; `ENT-DAMAGE-REDUCE-001`; `ENT-KNOCKBACK-001`;
`ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `CLI-UI-001`;
`WGEN-STRUCTURE-BURIED-001`; `WGEN-STRUCTURE-SHIPWRECK-001`;
`WGEN-STRUCTURE-OCEAN-RUIN-001`; `EXP-ITM-025`.

**Test vectors:**

Exercise all five tiers across direct/dispenser/menu equip and every life/age/tame/body/tag
boundary. Open/close and shift-click the zero-column menu as rider/nonrider; test leash,
passenger, body-before-saddle, prevention and creative shearing/death recovery. Apply every
attribute and sunlight gate, including invalidated tags and damageable near-misses. Replay all
six loot pools, six recycling recipes/unlocks and netherite smithing/unlock with patched inputs.
Persist/reload and render every item, normal/zombie equipment, variant, age, menu and Combat-tab
context.

**Limits:**

This leaf does not duplicate generic dispenser scheduling/ejection, armor/toughness formulas,
knockback resolution, equipment save/death codecs, sunlight-environment calculation, furnace,
smithing, loot, advancement or resource-pack algorithms. Those remain with the cited owners;
this rule fixes the five identities, exact normal/zombie-nautilus joins and their observable
ordering.
