# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BOAT-001` — Boat items place exact vehicles while chest boats separate passenger and storage ownership

**Parent:** `PLY-004`, `PLY-005`, `ITM-001`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ENT-001`, `ENT-002`, `ENT-005`, `MOB-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item/entity registration, vehicle subclasses, interaction and removal
code, container and loot hooks, recipes, advancements, tags, trades and client assets close all
twenty boat, chest-boat and raft item identities.

**Applies when:**

Any scoped item is held-used, dispensed, crafted, burned, traded, destroyed, persisted, reloaded or
projected, or its exact placed vehicle is mounted, opened, removed or picked.

**Authoritative state:**

| Wood/form | ordinary item / entity ID | chest item / entity ID | vehicle class |
|---|---:|---:|---|
| oak boat | `891` / `89` | `892` / `90` | `Boat` / `ChestBoat` |
| spruce boat | `893` / `125` | `894` / `126` | `Boat` / `ChestBoat` |
| birch boat | `895` / `12` | `896` / `13` | `Boat` / `ChestBoat` |
| jungle boat | `897` / `74` | `898` / `75` | `Boat` / `ChestBoat` |
| acacia boat | `899` / `0` | `900` / `1` | `Boat` / `ChestBoat` |
| cherry boat | `901` / `23` | `902` / `24` | `Boat` / `ChestBoat` |
| dark-oak boat | `903` / `33` | `904` / `34` | `Boat` / `ChestBoat` |
| pale-oak boat | `905` / `94` | `906` / `95` | `Boat` / `ChestBoat` |
| mangrove boat | `907` / `81` | `908` / `82` | `Boat` / `ChestBoat` |
| bamboo raft | `909` / `9` | `910` / `8` | `Raft` / `ChestRaft` |

Every row is a one-to-one `BoatItem` mapping to the tabulated entity type. All items are
common-rarity, maximum-stack-size-one identities with no family-specific default component. Every
entity is a `MISC` type with no entity loot table, width `1.375`, height and eye height `0.5625`,
and tracking range `10`. The ordinary forms carry at most two passengers; chest forms carry at
most one and own 27 inventory slots.

**Transition and ordering:**

#### Held placement

Held use takes a player point-of-view block hit with `Fluid.ANY`. A miss returns pass. Before using
a block hit, it sweeps pickable entities in the player box expanded five blocks along the view
vector and inflated by one; if the player's eye lies inside any candidate box inflated by that
entity's pick radius, use returns pass. The gate is containment, not nearest-ray distance.

A surviving block hit creates the item's exact entity type with reason `SPAWN_ITEM_USE`. A null
factory result returns fail. Creation fixes the entity position to the exact hit coordinates,
applies the default stack configuration on a server level, and then copies player yaw. The default
configuration applies implicit stack components before explicit `entity_data`; the generic entity
consumer observes `custom_name` and `custom_data`, while the entity-data component retains its
ordinary type and permission validation.

Collision is tested against the complete new entity box. Collision failure returns fail without
spawning, consuming, emitting or awarding. On the server, collision success calls
`addFreshEntity`, emits `ENTITY_PLACE` at the hit with the player as source, and consumes one item;
the boolean result of `addFreshEntity` is not inspected, so a rejected admission still consumes
and emits. Both sides return consuming success and invoke the matching item-used statistic; the
client neither creates the authoritative entity nor consumes the stack.

#### Passenger and container interaction

Ordinary boat/raft interaction first runs the common vehicle base hook. If that remains pass,
secondary use or at least 60 out-of-control ticks returns pass. Otherwise the client predicts
success; the server returns success only when `startRiding` succeeds and pass when it fails.

A chest form first delegates the same mount branch. When that branch consumes, it does not open
storage. If it returns pass, secondary use or inability to add another passenger opens the
container and returns success; therefore an empty chest boat normally mounts, while secondary use
opens it and a full/otherwise non-passenger-admitting chest boat opens even on ordinary use. A
failed `startRiding` while the vehicle still reports passenger capacity remains pass rather than
falling through to open.

Successful container interaction uses the generic three-row chest menu. On the server it emits
`CONTAINER_OPEN` with the player and angers nearby piglins. Spectators cannot create a menu while a
loot table is pending. A nonspectator open materializes pending loot before exposing slots.
Validity requires the entity not be removed and the player remain within entity-interaction range
of its bounding box with extra distance `4`. Closing emits `CONTAINER_CLOSE` at the entity with the
living container user as context.

All direct slot reads and writes materialize pending loot first. Writes clamp to both stack and
container maximum; `setChanged` itself is a no-op. A chest boat has exactly 27 slots and exposes no
inventory/container component on its matching item.

#### Storage, loot and removal

When a loot table is pending, save writes `LootTable` plus `LootTableSeed` only when the seed is
nonzero and omits ordinary items. Otherwise it writes `Items`. Load clears all 27 slots, reads the
nullable loot key and seed defaulting to zero, and reads `Items` only in the no-loot-table branch.
Materialization constructs chest loot context at the entity position; a player contributes luck
and `this_entity`, receives the generate-loot trigger, and the pending table is cleared before
filling. Loot evaluation and fill allocation remain with `ITM-LOOT-001`.

Chest contents are released by the chest-vehicle `remove` override when the server-side removal
reason `shouldDestroy`, including `KILLED` and `DISCARDED`. It repeatedly splits randomized
count-`10..30` item entities until every source stack is empty. This happens independently of
`entity_drops`; unload and changed-dimension removal do not release contents.

For a killed regular boat/raft, the common vehicle destruction path emits the exact matching
default item only when `entity_drops` is true. A chest form enters virtual removal during that same
kill, so its contents empty first; the matching chest item is considered afterward under
`entity_drops`, and the final chest-destroy helper then sees an empty inventory. If direct damage
came from a player, that helper angers nearby piglins only under the same enabled `entity_drops`
gate. Creative discard therefore releases chest contents but normally emits no matching vehicle
item and does not take this helper's anger branch. Only the vehicle custom name is copied to a
destruction-produced stack; entity/custom data and chest contents do not round-trip. Pick block
always returns a default matching item and preserves none of those fields.

Vehicle damage admission, wobble, health, creative removal, collision, movement, input, dismount
and passenger positioning remain `ENT-VEHICLE-001`. The ordinary boat/chest-boat passenger ride
height is passenger height divided by three; raft/chest-raft uses passenger height times
`0.8888889`, and the chest form's sole passenger has longitudinal offset `0.15` instead of zero.
The locked fall callback tracks and resets fall distance but has no historical plank/stick
destruction branch.

#### Crafting, fuel, trades and progression

Each ordinary form has one shaped recipe in group `boat`, pattern `# #` above `###`, using exactly
its matching planks and producing one matching boat or raft. Each chest form has one shapeless
recipe in group `chest_boat`, combining one chest and its exact ordinary form; assembly creates a
default chest result and does not copy components from the source boat.

An ordinary recipe advancement unlocks from direct recipe unlock or entering water. A chest recipe
advancement unlocks from direct recipe unlock or inventory possession of any member of `boats`;
because that tag recursively contains `chest_boats`, an already owned chest boat also satisfies
this alternative.

The `boats` item tag orders the nine ordinary boats, bamboo raft and the complete `chest_boats`
tag. The chest tag orders nine chest boats and bamboo chest raft. All twenty therefore receive the
vanilla tag-derived `1200`-tick fuel time (`200 * 6`); none is excluded by
`non_flammable_wood`.

Exactly five fisherman level-five records buy one boat for one emerald with maximum uses `12`,
villager XP `30` and reputation discount `0.05`: oak for plains, spruce for taiga or snow, jungle
for desert or jungle, acacia for savanna, and dark oak for swamp. The level-five trade set chooses
two distinct candidates using random sequence `minecraft:trade_set/fisherman/level_5`; generic
candidate/type selection, allocation and commit remain progression owners. No other scoped item is
directly emitted or consumed by bundled trades, and no scoped item has noncrafting loot acquisition.

The entity tag `boat` contains only the ten nonchest entity types. `ride_a_boat_with_a_goat`
triggers on starting to ride a vehicle in that tag while it has a goat passenger; chest forms
cannot satisfy the vehicle-type predicate. The advancement is telemetry-enabled and uses oak boat
as its icon. Generic listener lifetime and completion remain `ITM-ADVANCEMENT-001`.

#### Dispenser and client projection

All twenty identities join `ITM-DISPENSER-001`: a water block directly ahead places one block
above the target baseline, while air ahead over water uses zero vertical offset; other terrain
delegates to the nested default eject behavior. The spawn center is shifted along facing by
`0.5625 + entity width / 2` and vertically by `1.125 * facingY`, applies the stack configuration
and facing yaw, and consumes once after a nonnull creation even if entity admission later fails.

Each item uses a direct generated model and one like-named texture. Wood vehicles use
`BoatRenderer`/`BoatModel` or `ChestBoat` equivalents; bamboo uses
`RaftRenderer`/`RaftModel`. Model-layer paths are `boat/<wood>` or `chest_boat/<wood>`, backed by
matching `textures/entity/boat` or `textures/entity/chest_boat` textures. Boat rendering adds a
water-mask patch while not underwater; raft rendering has no water patch. Chest interaction uses
the generic three-row container screen.

Tools & Utilities orders the paired ordinary/chest identities after warped fungus on a stick:
oak, spruce, birch, jungle, acacia, dark oak, mangrove, cherry, pale oak, then bamboo raft, before
rail. Ordinary visibility also supplies parent/search output. Redstone Blocks additionally shows
oak chest boat then bamboo chest raft after TNT minecart and before oak door.

**Branches and aborts:**

Miss/block/other hit; eye inside/outside pickable box; null entity; colliding/free placement;
client/server and accepted/rejected entity admission; ordinary/chest and boat/raft; secondary use,
capacity and riding success; pending/realized loot and spectator; killed/discarded/unloaded removal;
`entity_drops` and player/nonplayer damage; exact recipe/tag/trade/advancement variant; dispenser
water/air-over-water/fallback; submerged/non-submerged render.

**Constants and randomness:**

Item IDs `891..910`; tabulated entity IDs; width `1.375`, height/eye height `0.5625`, tracking
range `10`; POV range `5`, sweep inflation `1`; capacities `2/1`; chest slots `27`, validity extra
distance `4`; ordinary/chest ride-height factors `1/3` and `0.8888889`; chest passenger offset
`0.15`; split counts `10..30`; fuel `1200`; trade `1:1`, max uses `12`, XP `30`, discount `0.05`;
dispenser offsets `0.5625 + width/2` and `1.125 * facingY`. Content splitting and trade-set
selection consume RNG; common vehicle physics and loot RNG retain their owners.

**Side effects:**

Held stack, new vehicle, passenger graph, chest storage/loot state, menu and piglin anger,
destruction item entities, statistics, advancements, recipes/trades/fuel state, game events and
client vehicle/item/container projection.

**Gates:**

Hit kind and eye obstruction; exact item-to-entity mapping; factory/collision/admission result;
vehicle control and passenger admission; secondary use; removal reason and `entity_drops`; pending
loot and spectator; menu validity; recipe ingredients; live tags, villager type and trade-set
snapshot; dispenser terrain; render subtype and underwater state.

**State read/written:**

Reads held stack/components, view/hit/entity boxes, world collision, player yaw and abilities,
vehicle/passenger/damage/removal state, chest items/loot/name, game rules, recipes/tags/trades and
client render context. Writes item count, vehicle and passenger graph, chest/loot/menu state,
removal drops, piglin anger, statistics/progression, game events and client-visible entities/UI.

**Failure behavior:**

Misses and eye-obstructed use pass; null creation or collision fails without consumption; server
entity-admission false is deliberately ignored after collision; failed riding passes; spectator
pending-loot menu creation returns null; unload/dimension transfer preserves rather than scatters
contents; disabled entity drops suppresses the matching vehicle item but not destructive-removal
chest contents; recipe mismatches and dispenser fallback retain their generic owners.

**Persistence boundary:**

Vehicle identity, transform, passengers, custom name/data and chest items or pending loot persist
through ordinary entity save/load. Chest load is mutually exclusive between pending loot and
materialized items. Placement hit/sweep, admission result, active menu, trade/loot/split RNG,
dispenser attempt and render water mask do not persist. Data reload replaces recipes,
advancements, item/entity tags and trade records without rewriting existing stacks, vehicles,
passengers or materialized chest contents; resource reload replaces models and textures only.

**Boundary cases and quirks:**

The eye gate tests containment in every nearby inflated pickable box rather than the chosen ray
target. A post-collision entity-admission failure still consumes and emits. Chest contents can
scatter with `entity_drops` disabled because removal owns them before itemization. An occupied
chest boat can turn ordinary interaction into container open. Chest crafting discards source-boat
components. Destruction preserves at most custom name and pick block preserves no entity fields;
neither returns storage. Chest boats do not satisfy the goat advancement's nonchest entity tag. No
scoped vehicle has fall-to-planks
behavior in 26.2.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.BoatItem#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`;
`net.minecraft.world.item.Item#getPlayerPOVHitResult(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.level.ClipContext$Fluid)`;
`net.minecraft.world.entity.EntityType#createDefaultStackConfig(net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.entity.LivingEntity)`;
`net.minecraft.world.entity.vehicle.boat.AbstractBoat`;
`net.minecraft.world.entity.vehicle.boat.Boat`;
`net.minecraft.world.entity.vehicle.boat.Raft`;
`net.minecraft.world.entity.vehicle.boat.AbstractChestBoat`;
`net.minecraft.world.entity.vehicle.boat.ChestBoat`;
`net.minecraft.world.entity.vehicle.boat.ChestRaft`;
`net.minecraft.world.entity.vehicle.VehicleEntity#destroy(net.minecraft.server.level.ServerLevel,net.minecraft.world.item.Item)`;
`net.minecraft.world.entity.vehicle.VehicleEntity#destroy(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`;
`net.minecraft.world.entity.vehicle.ContainerEntity`;
`net.minecraft.world.Containers#dropContents(net.minecraft.world.level.Level,net.minecraft.world.entity.Entity,net.minecraft.world.Container)`;
`net.minecraft.world.inventory.ChestMenu#threeRows(int,net.minecraft.world.entity.player.Inventory,net.minecraft.world.Container)`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes(net.minecraft.core.HolderLookup$Provider,net.minecraft.world.flag.FeatureFlagSet,int)`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`net.minecraft.client.renderer.entity.BoatRenderer`;
`net.minecraft.client.renderer.entity.RaftRenderer`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/{*_boat,*_raft}.json`;
`data/minecraft/tags/{item/{boats,chest_boats},entity_type/boat}.json`;
`data/minecraft/recipe/{*_boat,*_raft}.json`;
`data/minecraft/advancement/{recipes/transportation/{*_boat,*_raft},husbandry/ride_a_boat_with_a_goat}.json`;
`data/minecraft/{villager_trade/fisherman/5/*_boat,trade_set/fisherman/level_5,tags/villager_trade/fisherman/level_5}.json`;
`assets/minecraft/{items,models/item}/{*_boat,*_raft}.json`;
`assets/minecraft/textures/entity/{boat,chest_boat}/*.png`;
`ITM-USE-001`; `ITM-CONTAINER-001`; `ITM-CONTAINER-CLOSE-001`; `ITM-RECIPE-001`;
`ITM-CRAFT-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ITM-DISPENSER-001`;
`ENT-VEHICLE-001`; `ENT-ENTITY-DROPS-001`; `MOB-UNIVERSAL-ANGER-001`; `CLI-UI-001`;
`PROTO-PLAY-SERVERBOUND-ENTITY-SESSION-001`; `EXP-ITM-018`.

**Test vectors:**

Use every item across miss, obstruction, collision and admission outcomes; verify exact entity,
position, yaw, components, consumption, stat and event order. Mount/open every passenger,
secondary-use, underwater and control state. Exercise all 27 slots, pending loot, spectator,
validity, save/load and close; remove under every reason, game rule and attacker while tracing
content/item/anger order. Craft all twenty recipes, reload both tags and advancements, burn every
item, select every fisherman type and goat criterion. Dispense across terrain/facing branches and
render every item/entity/type/water/tab context.

**Limits:**

This leaf does not duplicate generic vehicle physics/damage/dismount, loot evaluation, menu packet
convergence, recipe allocation, advancement completion, trade-set selection, piglin anger, entity
save framing, dispenser scheduling or protocol codecs. Those behaviors remain with the cited
owners; this rule fixes the scoped identities, call-site ordering, data joins and observable
boundaries.
