# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-HARNESS-001` — Harnesses equip one adult Happy Ghast and gate its four-passenger flight controls

**Parent:** `PLY-004`, `PLY-005`, `PLY-006`, `ITM-001`, `ITM-003`, `ITM-004`, `ITM-005`,
`ITM-006`, `ITM-007`, `ENT-001`, `ENT-002`, `ENT-005`, `MOB-004`, `MOB-005`, `CLI-001`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item registration, equippable dispatch, Happy Ghast interaction and
vehicle code, recipes, advancements, tags, sounds and client assets close all sixteen harness
identities and their exact entity join.

**Applies when:**

A scoped harness is crafted, recolored, used on or dispensed toward an entity, equipped, sheared,
dropped, persisted, reloaded or projected, or an adult Happy Ghast's harness-dependent temptation,
mounting, passengers or flight controls are evaluated.

**Authoritative state:**

| Color | item ID | body asset |
|---|---:|---|
| white | `866` | `minecraft:white_harness` |
| orange | `867` | `minecraft:orange_harness` |
| magenta | `868` | `minecraft:magenta_harness` |
| light blue | `869` | `minecraft:light_blue_harness` |
| yellow | `870` | `minecraft:yellow_harness` |
| lime | `871` | `minecraft:lime_harness` |
| pink | `872` | `minecraft:pink_harness` |
| gray | `873` | `minecraft:gray_harness` |
| light gray | `874` | `minecraft:light_gray_harness` |
| cyan | `875` | `minecraft:cyan_harness` |
| purple | `876` | `minecraft:purple_harness` |
| blue | `877` | `minecraft:blue_harness` |
| brown | `878` | `minecraft:brown_harness` |
| green | `879` | `minecraft:green_harness` |
| red | `880` | `minecraft:red_harness` |
| black | `881` | `minecraft:black_harness` |

Every identity is a common plain `Item` with maximum stack size one. Its `equippable` component
selects body slot, the same-colored asset, `#minecraft:can_equip_harness`, equip-on-interact,
shearing, `entity.happy_ghast.equip` and `entity.happy_ghast.unequip`. Codec defaults leave
dispensable, swappable and damage-on-hurt true, but the stack has no maximum-damage, damage or
attribute component and therefore supplies no durability or defense. The allowed-entity tag
contains only Happy Ghast.

**Transition and ordering:**

#### Direct equip and dispenser dispatch

The stack's entity-interaction hook attempts component equip before the target's own mob
interaction. Admission requires a live target, an empty body slot, the allowed-entity tag and the
target's slot gate. Happy Ghast admits body equipment only while alive and adult. The server
splits exactly one item from the hand, including in creative mode, writes it to body and marks that
slot for guaranteed equipment drop. The equip callback plays the component sound with a seeded
entity-RNG long and emits `EQUIP`; direct target equip returns success without awarding the generic
item-used statistic. Held use in air passes because the player is not in the allowed-entity tag.
Wrong entities, babies, dead targets and occupied body slots pass without mutation.

`ITM-DISPENSER-001` owns scheduled selection and the generic fallback. Its dynamic equippable
branch chooses the first encounter-order living entity in the front cell that is alive,
nonspectating, allowed, slot-admitting and empty in body. An admitted adult Happy Ghast receives
one harness, a guaranteed body-slot drop and persistence-required state. If no candidate qualifies,
the nested default behavior ejects the item. Babies, occupied adults and all other entity types
therefore take no equip mutation.

#### Shearing, drops and reload validity

Entity interaction processes leash cutting before equipment shearing. A successful leash removal
damages the shears once and leaves the harness for that click. Otherwise a non-secondary-use
shears interaction can inspect equipment only while the Happy Ghast has no passengers. Body is
the first scoped shearing candidate unless `prevent_armor_change` blocks a noncreative player.
Success damages the shears once, empties body, emits `UNEQUIP` and `SHEAR`, spawns the exact
equipped stack at the passenger-attachment height, triggers `player_sheared_equipment`, and then
plays the component's explicit unequip sound. The bundled wolf-armor advancement predicates do not
accept a harness.

Direct and dispenser equip both mark body for guaranteed death drop; generic entity-death
equipment release owns the lethal transaction. Direct equip does not force persistence, while
dispenser equip does. The Happy Ghast loot table contains no harness pool.

An equipped stack remains stored if live data removes Happy Ghast from `can_equip_harness`, but it
ceases to satisfy the valid-body-equipment predicate. That disables harness-dependent temptation,
mounting and control without removing the stack. The client equipment layer still renders its
asset because projection reads the body stack's equippable asset rather than rechecking the
allowed-entity or adult gates.

#### Temptation, mounting and flight

An adult without valid body equipment is tempted by `happy_ghast_tempt_items`: snowball followed
by all sixteen harnesses. A baby or validly harnessed adult instead uses `happy_ghast_food`, which
contains only snowball. Equip therefore removes harness temptation immediately; live item/entity
tag reload can change later evaluations.

For an adult, the held stack's equip attempt runs before Happy Ghast interaction. If that does not
consume, valid body equipment plus ordinary use starts the player riding on the server; secondary
use delegates instead. Up to four direct passengers are admitted. The first passenger is the
controller only while the body equipment remains valid, the passenger is a player and the
still-timeout is zero. Adding the first passenger plays
`entity.happy_ghast.harness_goggles_down`; removing the last clears home and plays
`entity.happy_ghast.harness_goggles_up`.

Ridden horizontal input uses player strafe directly. Forward maps player pitch to vertical
`-sin(pitch)` and horizontal `cos(pitch)`; backward reverses both at half strength. Jump adds
`0.5` vertical input. The vector is scaled by `3.9000000953674316 * flying_speed`, whose base
attribute is `0.05`; travel uses `flying_speed * 5/3`. Target rotation is half player pitch and
full player yaw; yaw closes eight percent of its wrapped difference each tick, and body, head and
previous yaw are synchronized.

The persisted `still_timeout` suppresses the controller. Load grants a 60-tick grace before
decrement; non-spectator players standing in the thin region above can reset it to at most ten,
and passenger add/remove also clips or sets it to ten. An adult is a flying vehicle; a baby is not.
Dismount resolves at entity X/Z and bounding-box maximum Y.

#### Crafting, progression and projection

Each color has a shaped equipment recipe in group `harness`, pattern `LLL/G#G`: three leather, two
glass and the exact matching wool yield one default harness. Each color also has a shapeless
`harness_dye` recipe taking its exact dye plus one explicitly listed harness of any of the other
fifteen colors. Same-color recoloring has no recipe, and accepted recoloring discards the input
component patch.

The sixteen base-recipe advancements unlock from direct recipe unlock or possession of
`dried_ghast`; obtaining one can unlock all sixteen. The sixteen dye-recipe advancements unlock
from direct recipe unlock or possession of the exact target dye. No other bundled advancement,
trade, loot, fuel or composting record emits or consumes a harness.

Each item has a direct generated model and like-named item texture. Each equipment asset has one
`happy_ghast_body` layer using its like-named texture and selects adult or baby harness geometry
from the rendered entity state. Ridden goggles use X rotation zero and Y `14`; otherwise they use
X rotation `-0.7854` and Y `9`. Tools & Utilities orders all sixteen in the table's dye order
after saddle and before carrot on a stick; ordinary visibility supplies search output.

**Branches and aborts:**

Direct/dispenser equip; client/server; adult/baby/dead/wrong entity; empty/occupied body;
allowed/disallowed live tag; leash/no leash; passenger/no passenger; secondary/ordinary use;
creative/noncreative prevention; valid/invalid stored equipment; first/later/no passenger;
forward/backward/strafe/jump; still timeout; base/dye/same-color/invalid recipe; data/resource
reload.

**Constants and randomness:**

Item IDs `866..881`; entity ID `58`; stack maximum `1`; direct passenger maximum `4`; base recipe
`3` leather, `2` glass and `1` wool; movement scale `3.9000000953674316`; base flying speed
`0.05`; travel multiplier `5/3`; backward factor `-0.5`; jump `0.5`; yaw interpolation `0.08`;
load grace `60`; still timeout maximum `10`; goggles positions `14/9` and unmounted rotation
`-0.7854`. Equip sound seeding consumes one entity RNG long; generic loot/death owners retain
their randomness.

**Side effects:**

Held/dispenser stack count, body equipment and guaranteed drop, dispenser persistence flag, shears
damage, recovered/death item entity, passenger graph, home and still timeout, movement/rotation,
recipe unlocks, criteria, sounds, game events and client equipment/item projection.

**Gates:**

Live allowed-entity and temptation tags; life, age, body-slot and current-equipment state;
dispenser front-cell candidate order; leash, passenger and secondary-use state;
`prevent_armor_change`; recipe ingredients and unlock snapshot; valid harness for mount/control;
controller identity and still timeout; client model context.

**State read/written:**

Reads stack components/count, live item/entity tags, target type/age/life/equipment/passengers,
player input/use state, dispenser candidates, recipes/advancements and client render state. Writes
stack count, body equipment/drop chance, persistence flag, shears damage, item entities,
passengers/home/timeout, entity velocity/rotation, progression, events and projection state.

**Failure behavior:**

Direct equip failures pass and allow later target interaction. Dispenser admission failure ejects
through the default behavior. A leash removal wins over shearing; passengers or secondary use
prevent equipment shear. Wrong or same-color recipes do not match. Invalidated stored equipment
is retained and rendered but cannot enable mount/control or select the harnessed temptation path.

**Persistence boundary:**

Harness stacks and Happy Ghast body equipment/drop chance persist through generic stack/entity
storage; passengers and `still_timeout` persist through entity ownership, while input, active
interaction, equip/shear attempt and sound/game-event emission do not. Recipe unlocks and completed
criteria persist separately. Data reload replaces four relevant tags, thirty-two recipes and
thirty-two recipe advancements without rewriting existing stacks or equipment. Resource reload
replaces item/equipment models, textures and language only.

**Boundary cases and quirks:**

Creative direct equip still consumes one held item. Equip-on-target awards no item-used statistic.
Damage-on-hurt is true but a harness has no durability component. A leashed entity must lose its
leash on an earlier click before the same shears can recover equipment. A ridden Happy Ghast
cannot be sheared. Reload can leave a visibly equipped but functionally invalid harness. Dye
recipes exclude their own target color and discard all source patches.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.equipment.Equippable`;
`net.minecraft.world.item.ItemStack`; `net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.Mob`; `net.minecraft.world.entity.animal.HappyGhast`;
`net.minecraft.client.renderer.entity.HappyGhastRenderer`;
`net.minecraft.client.model.HappyGhastHarnessModel`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,entity_type,sound_event}`;
`reports/minecraft/components/item/*_harness.json`;
`data/minecraft/tags/item/{harnesses,happy_ghast_food,happy_ghast_tempt_items}.json`;
`data/minecraft/tags/entity_type/can_equip_harness.json`;
`data/minecraft/recipe/*_harness*.json`;
`data/minecraft/advancement/recipes/combat/*_harness*.json`;
`assets/minecraft/equipment/*_harness.json`;
`assets/minecraft/{items,models/item,textures/item}/*_harness.*`;
`assets/minecraft/textures/entity/equipment/happy_ghast_body/*_harness.png`;
`PLY-INTERACT-001`; `PLY-INPUT-001`; `PLY-MOVE-SPECIAL-001`; `ITM-USE-001`;
`ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-ADVANCEMENT-001`; `ITM-DISPENSER-001`;
`ENT-VEHICLE-001`; `ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `CLI-UI-001`;
`PROTO-PLAY-SERVERBOUND-ENTITY-SESSION-001`; `PROTO-PLAY-SERVERBOUND-MOVEMENT-001`;
`PROTO-PLAY-CLIENTBOUND-ENTITY-STATE-001`; `EXP-ITM-021`.

**Test vectors:**

Exercise all sixteen identities across direct/dispenser equip, every age/type/body/tag and rejected
candidate boundary. Cut leashes and shear across passenger, secondary-use, creative and prevention
branches; kill under generic equipment-drop controls. Mount one through five passengers, invalidate
and restore tags, drive every input/timeout branch and verify sounds, rotation, motion and goggles.
Craft every base/dye identity with patched, same-color and invalid controls; trigger all unlocks,
persist/reload and render every item/equipment/tab context.

**Limits:**

This leaf does not duplicate generic dispenser scheduling/ejection, entity damage/death equipment
release, passenger graph codecs, movement validation, recipe allocation, advancement completion,
protocol codecs or resource-pack loading. Those remain with the cited owners; this rule fixes the
sixteen harness identities, exact Happy Ghast joins and their observable ordering.
