# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-ENTITY-DROPS-001` — Entity drops gate seven differently placed itemization branches

**Parent:** `ENT-001`, `ENT-003`, `BLK-006`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the live Boolean rule and all seven direct readers fix vehicle, painting,
container-vehicle, item-frame, invalid-leash, falling-block and copper-golem-statue consequences,
including mutations before the read and non-drop side effects inside the guarded regions.

**Applies when:**

A vehicle is destroyed, a painting or item frame removes its carried/displayed item, a leash holder
becomes unable to interact with the level, a falling block cannot place or times out, or an oxidized
copper golem completes statue conversion while leashed.

**Authoritative state:**

The current server level's `entity_drops` rule; vehicle subtype/name/container slots and damage
source; decoration fixed/item/map/drop-chance state and remover; leash holder/data/interactability;
falling carried state, landing predicates, `dropItem`, `cancelDrop` and time; converted golem leash
plus already-committed statue/equipment/removal state.

**Transition and ordering:**

The `DROPS` Boolean rule defaults to `true` and has no change callback. Every reader samples the
live level value only when its branch is reached; a change neither removes existing item entities
nor retroactively materializes an already-suppressed drop.

**Base vehicles:** `VehicleEntity#destroy(ServerLevel,Item)` kills the vehicle first, then reads the
rule. False returns with the vehicle already removed. True constructs one stack of the supplied
vehicle item, assigns the vehicle's nullable custom name component, and calls `spawnAtLocation`;
the item-entity result is ignored. Damage-source destruction delegates to this method with
`getDropItem`. Thus the rule never cancels damage/destruction, passenger teardown or earlier damage
effects; it only guards the carrier stack after kill.

Chest boats destroy the carrier first, then call `ContainerEntity#chestVehicleDestroyed`; container
minecarts call their ordinary minecart destruction first, then the same helper. These are two
independent live reads. The helper's false branch suppresses both inventory itemization and its
Piglin notification. True first visits every container slot in ascending index through
`Containers.dropContents` at the vehicle coordinates. Each stack is destructively split into
`10+nextInt(21)` pieces after three position doubles; every piece consumes six velocity doubles and
is offered as an item entity, so even an empty slot consumes the three position draws. It then
tests the damage source's direct entity—not its causing entity—and, only for a player, calls
`angerNearbyPiglins(level,player,true)`. All contents are attempted before that Piglin query.

**Paintings:** `Painting#dropItem` reads the rule before every local effect. False suppresses both
`PAINTING_BREAK` and the painting item. True plays that sound at volume/pitch 1, then a removing
player with infinite materials suppresses only the item; a null or other remover spawns one
painting item. Hanging-entity removal and support/damage admission occur outside this method.

**Item frames:** Full frame removal plays the subtype break sound before the private helper and
emits `BLOCK_CHANGE` afterward, regardless of this rule. A damage event that removes only the
displayed item invokes the same helper with `dropFrame=false`, then emits `BLOCK_CHANGE` and the
subtype remove-item sound. The helper returns immediately for a fixed frame; those outer sound/
event calls still retain their own caller behavior.

For a nonfixed frame the helper snapshots the displayed stack and calls `setItem(EMPTY)` before it
reads the rule. That state publication, bounding-box recalculation and comparator-neighbor update
therefore survive every false branch. With the rule false, a null remover additionally calls
`removeFramedMap` on the old stack, while a nonnull remover returns without that explicit map-data
cleanup; neither frame nor displayed stack itemizes and no drop-chance RNG is consumed. With the
rule true, an infinite-material player performs map cleanup and returns with no items. Otherwise
`dropFrame=true` spawns the frame stack. A nonempty displayed stack is copied, removed from framed
map data, then consumes exactly one entity RNG float and spawns only when `draw < dropChance`.

**Invalid leash endpoints:** At the start of `Leashable#tickLeash`, delayed saved data may first be
restored. When either the leashee or its current holder cannot interact with the level, the live
rule chooses `dropLeash()` when true and `removeLeash()` when false. Both clear leash data, invoke
`onLeashRemoved`, send a tracking `ClientboundSetEntityLinkPacket(entity,null)` on a server level,
and notify the previous holder in that order; only the true path also spawns a lead. The following
holder reread is null, so ordinary distance/elastic work ends. Manual detach and the later
distance-snap behavior do not read this rule and are not generalized by this branch.

**Falling blocks:** The rule is read only after the per-entity `dropItem` field passes. Successful
placement never reads it, and `cancelDrop` discards plus calls the broken-after-fall hook without an
item independently of it. When placement eligibility passes but `setBlock(...,3)` returns false,
`dropItem && entity_drops` discards, calls the subtype broken hook, then spawns the carried block
item; if either gate is false, the landed entity remains to retry. When replacement/survival/
still-falling eligibility itself fails, the entity discards regardless, but calls that hook and
spawns the item only behind both gates. At strict timeout (`time>100` outside vertical bounds or
`time>600` anywhere), both gates control only the carried item; the entity always discards and no
broken hook is called. All subtype hook effects inside a guarded failure branch are therefore
suppressed together with its item, not merely the item entity.

**Copper-golem statue conversion:** Pose RNG, flags-3 statue offer/reread, identity copy, preserved-
equipment drops, golem discard and `COPPER_GOLEM_BECOME_STATUE` sound all precede the rule and remain
unchanged. Only when the discarded golem was still leashed does `turnToStatue` read it: true calls
`dropLeash`, false calls `removeLeash`. Both unlink/send/notify as above, but only true creates the
lead. In particular, false does not suppress preserved equipment drops.

**Branches and aborts:**

Rule false/true at each independent read; named/base/container vehicle and direct/causing player;
painting null/infinite/ordinary remover; frame full/content-only/fixed, null/infinite remover,
empty/map stack and drop-chance equality; missing/restored/invalid leash endpoint; falling cancel,
placement eligibility/write result/drop field/time bounds; successful/failed statue conversion and
remaining leash.

**Constants and randomness:**

Rule default `true`; painting sound volume/pitch `1`; container split `10..30`; frame drop comparison
is strict `<`; falling timeout thresholds `100/600`. Base carrier, painting, leash and statue gates
consume no RNG. Rule false skips container slot/piece draws and the frame's one float. Falling item
spawn internals are entity-owned; no branch adds a selection draw merely for the rule.

**Side effects:**

Entity kill/discard; vehicle, inventory, painting, frame, displayed stack, lead and falling-block
item entities; destructive container splitting; Piglin anger ingress; painting/frame sounds and
game event; synced frame item/comparator/map state; leash callback/link packet/holder notification;
falling broken hooks; statue equipment drops. Admission results for spawned items are ignored and
already committed state is never rolled back.

**Gates:**

Server-side caller and live rule; vehicle/container subtype; damage source/direct entity; infinite
materials; fixed frame and remover presence; item/map/drop chance; leash data and endpoint level
interaction; falling per-entity field, cancel/landing/write/timeout; completed statue subtype and
remaining leash.

**Boundary cases and quirks:**

The rule's location differs per consumer: it precedes a painting sound, follows vehicle kill,
follows frame emptying, and follows the entire statue conversion. A container vehicle may perform
two reads in one destruction. Disabling it can keep a failed-write falling entity alive to retry,
while the same setting discards an ineligible or timed-out one. It never globally suppresses every
lead or equipment drop.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`; `net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.world.entity.vehicle.VehicleEntity#destroy`;
`net.minecraft.world.entity.vehicle.boat.AbstractChestBoat#destroy`;
`net.minecraft.world.entity.vehicle.minecart.AbstractMinecartContainer#destroy`;
`net.minecraft.world.entity.vehicle.ContainerEntity#chestVehicleDestroyed`;
`net.minecraft.world.Containers#dropContents`;
`net.minecraft.world.entity.decoration.painting.Painting#dropItem`;
`net.minecraft.world.entity.decoration.ItemFrame#hurtServer`;
`net.minecraft.world.entity.decoration.ItemFrame#dropItem`;
`net.minecraft.world.entity.decoration.ItemFrame#removeFramedMap`;
`net.minecraft.world.entity.Leashable#dropLeash`;
`net.minecraft.world.entity.Leashable#removeLeash`;
`net.minecraft.world.entity.Leashable#tickLeash`;
`net.minecraft.world.entity.item.FallingBlockEntity#tick`;
`net.minecraft.world.entity.animal.golem.CopperGolem#turnToStatue`; `ENT-VEHICLE-001`;
`BLK-FALL-001`; `BLK-COPPER-GOLEM-STATUE-001`; `MOB-UNIVERSAL-ANGER-001`; `EXP-ENT-006`.

**Test vectors:**

Toggle the rule before and between carrier/container reads; cross every vehicle subtype/name/slot
size, direct-versus-causing player and rejected item admission while tracing RNG. Remove paintings
and frame contents/frames through null, survival, infinite and ordinary causes with fixed/map/
drop-chance boundaries. Invalidate each leash endpoint and compare link/lead order. Force every
falling cancel, eligibility, write-failure and exact timeout boundary under both gates. Convert a
leashed golem with zero/multiple preserved equipment slots; assert only the lead differs.
