# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ARMOR-STAND-001` — Armor-stand placement, equipment, damage and projection form one durable entity transaction

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-DISPENSER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-ADVANCEMENT-001`, `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-DAMAGE-REDUCE-001`, `ENT-KNOCKBACK-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `WGEN-JIGSAW-VILLAGES-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item/entity registration, item and armor-stand bytecode, stack-to-entity
configuration, dispenser dispatch, damage tags, recipe/progression/entity-loot data, village
structure audit and client renderer/assets determine every identity-specific placement, equipment,
damage, persistence and projection boundary. Generic player admission, entity lifecycle, equipment
packets and base living damage remain with their cited owners.

**Applies when:**

An `armor_stand` stack is created, crafted, selected in a tab, used on a block or dispensed; an
armor-stand entity is created from that stack, a structure or administration; a player exchanges
equipment with it; it moves, collides, takes damage, breaks, saves, reloads or is rendered.

**Authoritative state:**

`minecraft:armor_stand` is raw item ID `1284`, common, nondamageable, max stack `16`, and in no
direct item tag. Its registered components are only the common empty modifiers/enchantments/lore,
item-break sound, name, direct item-model key, rarity, repair cost, swing animation, tooltip
display and use effects. It has no consumable, equippable or entity-data component by default.

The corresponding entity is static entity-type raw ID `5`, category `MISC`, serializable and
summonable, with normal dimensions `0.5 × 1.975`, eye height `1.7775`, tracking range `10` and
ordinary living max health `20`. It adds zero step height. Small mode uses dimensions
`0.25 × 0.9875` and eye height `0.9875`; marker mode uses fixed `0 × 0` dimensions.

Subtype state is an `invisible` Boolean, `disabledSlots` integer and seven synchronized values:
flags byte followed by head, body, left-arm, right-arm, left-leg and right-leg rotations. The flags
are small `0x01`, show arms `0x04`, no base plate `0x08` and marker `0x10`; other bits have no
subtype meaning. Default rotations `(X,Y,Z)` are head/body `(0,0,0)`, left arm `(-10,0,-10)`,
right arm `(-15,0,10)`, left leg `(-1,0,-1)` and right leg `(1,0,1)`.

`disabledSlots` uses equipment IDs mainhand `0`, feet `1`, legs `2`, chest `3`, head `4`, offhand
`5`, body `6`, saddle `7`. Bit `1 << id` disables general use, `1 << (id+8)` disables taking from
an occupied slot and `1 << (id+16)` disables putting into an empty slot. Armor stands reject body
and saddle unconditionally. A hidden-arms stand also rejects both hand slots through its general
slot predicate.

**Transition and ordering:**

Hand placement runs the following subtype transaction.

Clicking Down fails before target or collision work. Every other face wraps the use in a
`BlockPlaceContext`, takes its resulting clicked position and builds the normal entity-type box at
that block's bottom center. Both projections require `noCollision(null, box)` and an empty result
from the unfiltered entity query over the same box; either failure returns `FAIL` without a count
change.

The server creates type armor stand with reason `SPAWN_ITEM_USE`, vertical collision adjustment
enabled against the target and block below, and stack post-processing. Creation first consumes one
level `nextFloat`, installs a wrapped random yaw, then applies stack configuration in this order:

1. `custom_name` and `custom_data` implicit components are copied to entity state when present.
2. A present `entity_data` payload is loaded only when its recorded type is armor stand. This type
   is not operator-only, so an ordinary placing player is not filtered by the operator gate.
3. The item then retains the entity's resulting position but replaces yaw with
   `floor((wrapDegrees(contextRotation-180)+22.5)/45)*45` and pitch with zero.

A null creation result returns `FAIL` before consumption. Otherwise the server calls
`addFreshEntityWithPassengers` without observing its result, plays `armor_stand_place` in the
Blocks source at volume `0.75`, pitch `0.8`, and emits `ENTITY_PLACE` from the stand with the
placing player as source. Both the passing client projection and server then shrink the source by
one and return `SUCCESS`. A failed add therefore does not restore the item or suppress sound/event.

**Dispenser join:**

The explicit dispenser behavior targets the block adjacent to captured facing and chains a facing
post-processor before the same stack-component and type-matched entity-data processors. It spawns
with reason `DISPENSER`, no vertical adjustment and no living source; creation still consumes a
random yaw before the facing processor replaces it with `Direction.toYRot()`. A nonnull result is
added and shrinks one item; null leaves the stack. This behavior inherits the unconditional
default wrapper, so either outcome still publishes level events `1000` then `2000` with captured
facing as specified by `ITM-DISPENSER-001`. It performs no hand-placement collision/entity
preflight.

**Equipment interaction:**

Marker stands and name-tag use delegate immediately to the generic living interaction. A spectator
gets `SUCCESS` without mutation. Every other client interaction predicts `SUCCESS_SERVER`; the
server alone selects and swaps equipment.

A nonempty held stack selects its `equippable` slot when that slot is usable by the stand, otherwise
mainhand. A generally disabled slot fails. Hand targets additionally fail while arms are hidden.
An empty hand starts from mainhand and selects the first occupied region in this exact order after
dividing hit Y by entity scale and age scale:

- feet for `0.1 <= y < 0.9` when small or `< 0.55` otherwise;
- chest for `0.9 <= y < 1.9` when small or `< 1.6` otherwise;
- legs for `0.4 <= y < 1.4` when small or `< 1.2` otherwise;
- head for `y >= 1.6`;
- otherwise occupied mainhand, then occupied offhand.

Earlier tests win overlapping ranges. If the geometrically selected slot is generally disabled,
selection falls back to mainhand. The swap proceeds only when the selected slot currently contains
an item for empty-hand removal. Any unhandled case delegates to generic living interaction.

An occupied slot with its take bit set or an empty slot with its put bit set refuses mutation.
With infinite materials, a nonempty held stack going into an empty slot copies exactly one and
leaves the hand unchanged. Otherwise a held count greater than one can fill only an empty slot by
splitting one. The remaining path atomically installs the entire held stack in the stand and puts
the former slot stack in that same hand. Successful subtype swaps return `SUCCESS_SERVER`; no
inventory insertion or ground-drop fallback participates.

**Physics and damage state machine:**

The stand is never pushable and ignores direct pushes. Its living push scan instead pushes only
rideable minecarts within its own box whose squared distance is at most `0.2`. Marker or no-gravity
state disables its effective AI/travel physics; marker additionally has no pick box, ignores block
triggers, uses piston reaction `IGNORE` and blocks no building. Normal stands keep the ordinary
living reaction and dimensions. Armor stands are unaffected by potions and lightning, report
`attackable=false`, and reject a protected player attack through the level's `mayInteract`
boundary.

Server damage is ordered:

1. Removed stands return false. With `mob_griefing=false`, a damage source whose responsible entity
   is a `Mob` returns false.
2. `#bypasses_invulnerability` removes the stand with `KILLED` and emits `ENTITY_DIE`, but returns
   false and produces no subtype sound, particles or drops.
3. Ordinary invulnerability, the subtype invisible flag or marker mode returns false.
4. `#is_explosion` plays break sound, evaluates the empty intrinsic entity loot table, drops
   eligible equipment, kills and returns false. The stand item itself is not dropped.
5. `#ignites_armor_stands` ignites a nonburning stand for five seconds; an already burning stand
   loses `0.15` health through the subtype damage helper. Either returns false.
6. `#burns_armor_stands` removes `4` health only while health is above `0.5`, then returns false.
7. Only `#can_break_armor_stand` or `#always_kills_armor_stands` continues. A responsible player
   lacking `mayBuild` is rejected.

A creative-player source then plays break sound, emits ten oak-plank block particles centered at
two-thirds height with per-axis spread width/4, height/4, width/4 and speed `0.05`, kills, returns
true and drops neither stand nor equipment.

For other permitted hits, `gameTime-lastHit <= 5` or membership in
`#always_kills_armor_stands` breaks immediately. Otherwise event byte `32` is broadcast,
`ENTITY_DAMAGE` is emitted with the responsible entity, `lastHit=gameTime`, and the method returns
true without health loss. Event `32` makes the client play `armor_stand_hit` locally at volume
`0.3`, pitch `1`, records client `lastHit`, and drives a renderer wiggle for the first five ticks.

Immediate noncreative break first creates one default armor-stand item, copies only the entity's
custom name to it, and pops it at the stand block. It then plays break sound, evaluates the empty
entity table and clears every equipment slot; each nonempty stack without
`prevent_equipment_drop` is popped one block above. Prevented stacks are cleared without a drop.
Finally it emits the same ten particles, removes with `KILLED` and emits `ENTITY_DIE`.

Subtype fire damage uses health rather than the two-hit clock: if the fixed subtraction leaves
health at or below `0.5`, it runs the no-stand-item break path and kills; otherwise it writes health
and emits `ENTITY_DAMAGE`. The locked damage tags are exact:

- ignite: `in_fire`, `campfire`;
- burn: `on_fire`;
- can break: `player_explosion` and `#is_player_attack`;
- always kill: `arrow`, `trident`, `fireball`, `wither_skull`, `wind_charge`.

The explosion branch precedes these sets. The empty entity loot table names random sequence
`minecraft:entities/armor_stand` but has no pools, so intrinsic loot consumes no selection draws
and emits nothing.

**Crafting, progression and structure creation:**

The shaped recipe is `///`, ` / `, `/_/`, where `/` is stick and `_` smooth-stone slab, producing
one stand with default components. Its recipe advancement is granted by either obtaining a
smooth-stone slab or already unlocking the recipe, and rewards that same recipe. Generic recipe
matching, remaining items and recipe-book synchronization remain with their cited owners.

No standard data loot table directly awards the stand item. The two audited taiga-village
structure records instead create armor-stand entities from saved payload—one with an iron helmet
and one with an iron chestplate—using `STRUCTURE` reason and no mob finalization, as fixed by
`WGEN-JIGSAW-VILLAGES-001`.

**Persistence boundary:**

The entity saves ordinary living identity, position/motion/rotation, health, equipment, custom
name/data, gravity/invulnerability/fire and lifecycle state plus subtype keys `Invisible`, `Small`,
`ShowArms`, `DisabledSlots`, `NoBasePlate`, optional true `Marker`, and `Pose`. Pose is a six-part
record whose missing members take the defaults above. Reload applies subtype flags, recomputes
`noPhysics` from marker/no-gravity, then applies a decoded pose when present.

The transient two-hit `lastHit` clock is not subtype save data. It resets across durable reload,
whereas all equipment, mask, flags and rotations survive. Item-form persistence retains ordinary
stack identity/count/component patches; only custom name is synthesized onto the manual break
drop, so arbitrary source-stack components do not round-trip through an ordinary placed and
broken stand unless they first changed durable entity state through supported entity data.

**Client and wire projection:**

Static entity type ID `5` is sent by the generic add-entity family. After eight base-entity and
seven living-entity accessors, armor-stand metadata indices are exactly `15` flags byte and
`16..21` six `ROTATIONS` values; serializer IDs are `0` and `9`. Equipment uses the generic
equipment packet and item/component mapping. Spawn, metadata, equipment, event `32`, motion,
removal and correction ordering remain with the cited protocol/entity owners.

The renderer uses normal/small stand models, the fixed
`textures/entity/armorstand/armorstand.png`, humanoid armor, held-item, wings and custom-head
layers. It projects flags, six poses and interpolated body yaw; marker mode selects its special
cutout/translucent render path, and a name is shown only when custom-name visibility is true. The
item is a direct generated model with texture `item/armor_stand`. It appears in both Functional
Blocks and Redstone Blocks in locked bootstrap order.

**Branches and aborts:**

Face/target/collision/entity occupancy/side/create/add; every component and typed entity-data
branch; hand/dispenser source, vertical adjustment and yaw; spectator/marker/name tag/arms,
held/slot/click region/mask/ability/count; marker/small/gravity/physics; removed/game rule/damage
tag/invulnerability/visibility/player ability/creative/two-hit/fire-health/drop prevention; recipe,
structure, save/load, metadata, equipment, renderer and tab context.

**Constants and randomness:**

Item/entity raw IDs `1284/5`; max stack `16`; dimensions and flags as above; one spawn
`nextFloat`, hand yaw quantum `45°` with `22.5°` bias; place sound `0.75/0.8`; equipment IDs and
mask offsets `0/8/16`; minecart squared distance `0.2`; hit window/wiggle `5` ticks; event `32`;
ignite `5` seconds, in-fire damage `0.15`, on-fire damage `4`, destruction threshold `0.5`;
particles `10` at speed `0.05`. No subtype damage or loot randomness exists beyond the stated
spawn draw.

**Side effects:**

Stack count/components; entity creation/addition/position/rotation/flags/pose/health/fire/equipment/
removal; hand swaps and drops; recipe unlock/output; sounds, particles, game/entity events;
persistence; spawn/metadata/equipment/event/removal packets; item/entity models, name and tabs.

**Gates:**

Generic interaction and feature admission; non-Down face; target collision and entity vacancy;
server creation; component/entity-data type; spectator/marker/name-tag delegation; slot usability,
arms and three mask planes; player ability/count; game rule, removal/invulnerability/visibility/
marker and exact damage tags; `mayBuild`/`mayInteract`; health/time; drop-prevention enchantment;
recipe criterion; saved decode; client resource and tracking admission.

**State read/written:**

Reads held stack/components/count, face/rotation/target/block collisions/entities, level RNG/time/
game rule, player mode/abilities/permissions, equipment and masks, health/fire/flags/pose, damage
source/tags, recipe/structure data, saved state and client resources. Writes exactly the stack,
entity, world effect, persistence and client-projection state listed above.

**Failure behavior:**

Down, blocked boxes and occupied boxes fail without consumption. Client passing preflight predicts
success; server null creation fails before shrink, while add failure is ignored after creation.
Dispenser null creation keeps the stack but retains wrapper events. Refused equipment swaps fall
through or fail as stated without partial movement. Most rejected damage returns false; fire and
explosion branches intentionally return false after effects. Drop spawning/addition results are
not transactionally observed, and no later failure restores cleared equipment or the source item.

**Boundary cases and quirks:**

The spawn yaw RNG draw is observable even though hand and dispenser paths overwrite its value.
Stack entity data is applied before final hand yaw and may alter other durable entity fields. A
fresh world's first permitted hit can break immediately when `gameTime <= 5`, because saved
`lastHit` begins at zero. Small-mode click ranges overlap and retain feet/chest/legs test order.
Invisible and marker stands refuse subtype damage; marker is also zero-sized and unpickable.
Creative destruction drops nothing, normal two-hit/always-kill destruction drops a name-only stand
plus eligible equipment, and explosion/fire-health destruction drops equipment but no stand.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.ArmorStandItem`;
`net.minecraft.world.entity.EntityType`; `net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.Entity`; `net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.EquipmentSlot`;
`net.minecraft.world.entity.decoration.ArmorStand`;
`net.minecraft.world.entity.decoration.ArmorStand$ArmorStandPose`;
`net.minecraft.core.dispenser.DispenseItemBehavior$1`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.client.renderer.entity.ArmorStandRenderer`;
`net.minecraft.client.model.object.armorstand.ArmorStandModel`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/armor_stand.json`;
`data/minecraft/recipe/armor_stand.json`;
`data/minecraft/advancement/recipes/decorations/armor_stand.json`;
`data/minecraft/loot_table/entities/armor_stand.json`;
`data/minecraft/tags/damage_type/{ignites_armor_stands,burns_armor_stands,can_break_armor_stand,always_kills_armor_stands}.json`;
`assets/minecraft/items/armor_stand.json`;
`assets/minecraft/models/item/armor_stand.json`;
`assets/minecraft/textures/item/armor_stand.png`;
`assets/minecraft/textures/entity/armorstand/armorstand.png`;
`ENT-LIFECYCLE-001`; `ENT-DAMAGE-001`; `ENT-ENTITY-DROPS-001`;
`ITM-DISPENSER-001`; `ITM-RECIPE-001`; `WGEN-JIGSAW-VILLAGES-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-034`.

**Test vectors:**

Cross every face/target/collision/entity/side/add result with default and patched name/custom/
typed-entity-data stacks, all rotations, hands, counts and abilities; compare hand and all six
dispenser facings with deterministic RNG. Exhaust marker/name-tag/spectator and every occupied/
empty equipment slot, click threshold, arms state and all three mask planes. Apply every damage
tag/source/game-rule/permission/visibility/marker/creative/time/health/equipment-enchantment branch.
Persist/reload every flag, pose, mask, equipment, health and stack state; create both taiga records;
capture exact spawn/metadata/equipment/event/removal traffic and normal/small/marker/item/tab
projection.

**Limits:**

This leaf does not duplicate generic player packet admission, entity ownership/tracking, living
equipment attributes, damage reduction, item-entity insertion, recipe engine, structure placement,
stack/entity codec or packet layouts. Those remain with the cited owners; this rule fixes the
armor-stand identity and every subtype-specific join across them.
