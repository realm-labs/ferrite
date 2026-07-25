# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-MINECART-001` — Minecart items place exact rail vehicles while subtype state never round-trips through destruction

**Parent:** `PLY-005`, `ITM-001`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ENT-001`, `ENT-002`, `ENT-005`, `WGEN-003`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item/entity registration, common placement and dispenser code, all six
subtype hooks, recipes, advancements, tags, mineshaft generation and client assets close the
complete `MinecartItem` family.

**Applies when:**

A scoped item is used on a rail, dispensed, crafted, picked, destroyed, persisted, reloaded or
projected, or its exact placed vehicle is mounted, opened, fueled, activated or edited.

**Authoritative state:**

| Item | item ID | entity / ID | default displayed block / offset | subtype state |
|---|---:|---|---|---|
| `minecart` | `882` | `minecart` / `85` | air / `6` | one rideable passenger |
| `chest_minecart` | `883` | `chest_minecart` / `25` | north chest / `8` | 27 slots or pending loot |
| `furnace_minecart` | `884` | `furnace_minecart` / `56` | north furnace, lit by fuel / `6` | fuel and horizontal push |
| `tnt_minecart` | `885` | `tnt_minecart` / `134` | TNT / `6` | fuse and two explosion factors |
| `hopper_minecart` | `886` | `hopper_minecart` / `65` | hopper / `1` | five slots or pending loot, enabled flag |
| `command_block_minecart` | `1293` | `command_block_minecart` / `29` | command block / `6` | command, output and activation throttle |

All six are maximum-stack-one `MinecartItem` identities without another family-specific default
component. The first five are common rarity. Command-block minecart is epic and has no survival
recipe or ordinary-tab admission.

**Transition and ordering:**

#### Player placement

The generic stack use-on gate first enforces adventure-mode build/`can_place_on` admission. The
minecart hook then requires the clicked state to belong to the live `rails` block tag; failure
returns fail. A `BaseRailBlock` supplies its actual shape. A tag-injected non-rail block is treated
as north-south. The spawn center is clicked X/Z plus `0.5`, and clicked Y plus `0.0625`; a sloped
shape adds another `0.5` Y.

The item's exact entity type is created with spawn reason `DISPENSER`, despite this being player
use. Creation sets the initial position, applies implicit stack configuration before explicit
`entity_data`, and supplies the player to that component transaction. A null factory result fails.
With `MINECART_IMPROVEMENTS` enabled, creation first adjusts the cart to the rail and placement then
fails if any `AbstractMinecart` intersects its resulting bounding box. Legacy placement performs no
minecart-overlap or general collision rejection.

On a server level, a surviving cart is offered to entity admission and the Boolean result is
ignored. The caller then emits `ENTITY_PLACE` at the clicked position with the player and the
block state below as context. Both sides shrink the held stack by one and return item-interaction
success; generic stack use-on consequently awards the matching item-used statistic. Shrink is not
creative-exempt. A rejected server admission therefore still consumes, emits and reports success.

#### Dispenser placement

All six identities are explicit `ITM-DISPENSER-001` entries. A front rail uses vertical offset
`0.6` when sloped and `0.1` otherwise. Front air above a rail uses `-0.4` only for a non-downward
facing and a sloped lower rail, otherwise `-0.9`. Other terrain delegates to nested default
ejection. Spawn X/Z is dispenser center plus `1.125 * facing`; Y is
`floor(centerY) + facingY + offset`. A nonnull exact cart receives stack configuration, is offered
to admission, then consumes one even when admission returns false. Null creation keeps the stack.

#### Subtype interaction and activation

The ordinary cart admits mounting only when empty and the player is not using secondary action.
The locked server path calls `startRiding` in its outer admission and again when choosing its
return, so the first call can install the passenger while the second returns false and the literal
server result is pass. Client prediction returns success. Passenger motion, both legacy and
improved rail engines, collision, activator-rail ejection/damage and dismount remain
`ENT-VEHICLE-001`.

Chest interaction always attempts the three-row menu. A pending loot table prevents spectator menu
creation, but the interaction result still consumes; the chest override then emits
`CONTAINER_OPEN` and angers nearby piglins on the server. Close emits `CONTAINER_CLOSE`. Hopper
interaction similarly attempts its five-slot menu and returns success, but has no chest override
for those open/close events or piglin anger. Both container types materialize pending loot before
nonspectator access and require a nonremoved cart within entity-interaction range plus four.

Furnace interaction always returns success. Coal or charcoal admitted by the live
`furnace_minecart_fuel` tag adds `3600` ticks only when the result is at most `32000`, sets
horizontal push to cart position minus player position, and consumes one through living-entity
item semantics. Wrong fuel or an over-cap offer still consumes the action without changing stack,
fuel or push. Fuel and push then select the furnace propulsion, speed, smoke and lit-block
projection owned by `ENT-VEHICLE-001`.

A powered activator rail primes an unprimed TNT cart to fuse `80`; ignition, collision/fall
explosion, destruction shortening, `tnt_explodes` and explosion power remain the vehicle owner.
A powered activator rail disables hopper pickup and an unpowered one enables it. Command-block
minecart executes no more often than every four entity ticks on powered activator rails; a
game-master interaction opens its client edit screen, while command admission, persistence and
feedback remain `BLK-COMMAND-001`.

#### Destruction, pick and container order

With `entity_drops` true, ordinary, chest, furnace, TNT and hopper destruction selects their
matching default item and preserves only custom name. Command-block minecart deliberately selects
ordinary `minecart` instead. Its pick result is nevertheless a default epic
`command_block_minecart`; every other pick result is its matching default item. No pick or
destruction result preserves entity data, custom display state, passengers, contents, loot,
fuel/push, enabled state, fuse/explosion factors or command state.

Destructive removal of chest and hopper carts scatters their materialized contents before common
matching-item evaluation and independently of `entity_drops`; unload and changed-dimension removal
retain storage. The later container-destruction helper rereads `entity_drops` for its now-empty
content pass and direct-player piglin notification. Primed or igniting TNT paths may explode
instead of returning the cart item. Generic damage thresholds, removal and random splitting remain
the entity owners.

#### Crafting, acquisition and progression

One shaped recipe uses five iron ingots in `# #` above `###` for a default ordinary minecart. Four
shapeless recipes combine one default-or-patched minecart with chest, furnace, TNT or hopper for
the matching default cart item; input components are discarded. No command-block-minecart recipe
exists.

The ordinary recipe unlocks from direct unlock or iron-ingot possession. Each four subtype recipes
unlocks from direct unlock or exact ordinary-minecart possession. That same minecart-possession
criterion also unlocks the rail recipe, so the family owns six recipe-book joins across five
scoped outputs. No bundled loot table, trade, fuel or compost record emits or consumes a scoped
item.

Mineshaft corridor generation can create a chest minecart with pending
`chests/abandoned_mineshaft` loot after either one-percent bay gate. This is an indirect
noncrafting chest-minecart-item source only after later destructive itemization; generation itself
creates the entity, not the item.

#### Client projection and creative inventory

Every item uses a direct generated model and like-named item texture. All six entities use the
common minecart model and `textures/entity/minecart/minecart.png`, then render the subtype's
default or synchronized custom display block at its offset. Furnace fuel changes its displayed
furnace's lit property; chest uses offset eight and hopper one while the others use six.

Tools & Utilities and Redstone Blocks both order ordinary, hopper, chest, furnace and TNT minecart
after activator rail. Tools & Utilities continues into instruments/music discs; Redstone Blocks
continues with oak chest boat and bamboo chest raft. Ordinary parent visibility supplies search.
Operator Utilities exposes command-block minecart only with permissions, after the three command
blocks and before jigsaw.

**Branches and aborts:**

Adventure admission; live rail tag; base/non-base rail and flat/slope shape; legacy/improved
movement; no/overlapping cart; null/non-null factory; client/server and accepted/rejected entity
admission; held/dispenser; ordinary/container/furnace/TNT/hopper/command subtype; pending/materialized
loot and spectator; fuel membership/cap; powered/unpowered activator; `entity_drops`; destructive/
nondestructive removal; base/subtype/missing recipe; permitted/operator-hidden projection.

**Constants and randomness:**

Item/entity IDs are tabulated; stack maximum `1`; held center offsets `0.5`, `0.0625` and slope
`0.5`; dispenser horizontal factor `1.125` and vertical offsets `0.6/0.1/-0.4/-0.9`; chest/hopper
slots `27/5`; menu range extra `4`; furnace fuel increment/cap `3600/32000`; TNT fuse `80`;
command throttle `4`; mineshaft gates `1/100` each. Placement consumes no RNG. Container splitting,
loot, TNT and minecart motion keep their owning RNG streams.

**Side effects:**

Held/dispenser stack, exact entity and configured fields, passenger graph, menus/storage/loot,
furnace fuel/push, TNT fuse/explosion, hopper enabled/pickup state, command state/output, destruction
items and container contents, piglin anger, statistics, recipes/progression, game events and client
item/entity/UI projection.

**Gates:**

Build/`can_place_on`; live rails and fuel tags; feature flag and intersecting cart; factory and
entity admission; dispenser terrain/facing; subtype/passenger/secondary-use state; spectator and
menu validity; fuel cap; rail power; command permission/throttle; live game rules; recipe and
advancement snapshot; tab permission and render subtype.

**State read/written:**

Reads stack components/count, player/build state, clicked rail/shape, feature flags and entity
boxes, dispenser terrain, entity subtype fields, tags/recipes/advancements/worldgen and client
state. Writes stack count, entity/configuration/passengers, containers/loot, fuel/push, fuse,
enabled/command state, drops, menus, progression, events and projection.

**Failure behavior:**

Adventure rejection passes before the item hook; non-rail, null creation or improved-mode overlap
fails without consumption/event. Legacy overlap is admitted. Server entity-admission failure is
ignored after those gates. Invalid furnace fuel still returns success without mutation. Spectator
pending loot still produces interaction success but no menu. Disabled entity drops suppresses the
carrier item, not destructive-removal container contents. Recipe mismatches retain generic owners.

**Persistence boundary:**

Minecart identity, transform, passengers, custom display state/offset/name/data and first-tick/
rotation state persist generically. Chest/hopper store pending loot or materialized slots; hopper
stores enabled. Furnace stores `PushX`, `PushZ` and signed-short `Fuel`; TNT stores fuse plus
clamped explosion factors; command-block minecart stores its command carrier. Placement,
interaction/menu, dispenser and crafting attempts do not persist. Recipe unlocks and advancement
criteria persist separately.

Data reload changes later rail/fuel tag, recipe, advancement, mineshaft and loot selection without
rewriting stacks or existing entities. The minecart-improvements feature set belongs to world
configuration rather than an ordinary live tag reload. Resource reload replaces item/block/entity
models, textures and UI assets.

**Boundary cases and quirks:**

Player placement uses spawn reason `DISPENSER`. A tag-injected non-rail state becomes flat
north-south placement. Legacy placement permits overlapping carts, while improved placement rejects
only intersecting minecarts and not arbitrary collisions. Rejected server admission still consumes.
Creative player placement still shrinks one. Ordinary server mounting can install the passenger
while returning pass because it calls `startRiding` twice. Invalid furnace fuel still consumes the
interaction. Container contents can scatter with entity drops disabled. Command-block minecart
destruction returns an ordinary cart, unlike its pick result.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.MinecartItem`;
`net.minecraft.world.entity.vehicle.minecart.AbstractMinecart`;
`net.minecraft.world.entity.vehicle.minecart.AbstractMinecartContainer`;
`net.minecraft.world.entity.vehicle.minecart.Minecart`;
`net.minecraft.world.entity.vehicle.minecart.MinecartChest`;
`net.minecraft.world.entity.vehicle.minecart.MinecartFurnace`;
`net.minecraft.world.entity.vehicle.minecart.MinecartTNT`;
`net.minecraft.world.entity.vehicle.minecart.MinecartHopper`;
`net.minecraft.world.entity.vehicle.minecart.MinecartCommandBlock`;
`net.minecraft.client.renderer.entity.MinecartRenderer`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/{minecart,*_minecart}.json`;
`data/minecraft/tags/{block/rails,item/furnace_minecart_fuel}.json`;
`data/minecraft/recipe/{minecart,chest_minecart,furnace_minecart,tnt_minecart,hopper_minecart}.json`;
`data/minecraft/advancement/recipes/transportation/{minecart,chest_minecart,furnace_minecart,tnt_minecart,hopper_minecart,rail}.json`;
`data/minecraft/loot_table/chests/abandoned_mineshaft.json`;
`assets/minecraft/{items,models/item,textures/item}/{minecart,*_minecart}.*`;
`assets/minecraft/textures/entity/minecart/minecart.png`;
`PLY-INTERACT-001`; `ITM-USE-001`; `ITM-CONTAINER-001`; `ITM-CONTAINER-CLOSE-001`;
`ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`;
`ITM-DISPENSER-001`; `ENT-VEHICLE-001`; `ENT-ENTITY-DROPS-001`;
`MOB-UNIVERSAL-ANGER-001`; `BLK-COMMAND-001`; `WGEN-STRUCTURE-MINESHAFT-001`;
`CLI-UI-001`; `PROTO-PLAY-SERVERBOUND-BLOCK-001`;
`PROTO-PLAY-SERVERBOUND-ENTITY-SESSION-001`; `EXP-ITM-022`.

**Test vectors:**

Place and dispense all six items across adventure, rail-tag, every shape/facing, null creation,
legacy/improved overlap and rejected-admission branches; assert exact type, transform, components,
count, stat and event order. Exercise all subtype interaction, rail activation, menu/storage,
fuel/fuse/enabled/command and destruction/pick branches. Craft every recipe with patched inputs,
trigger all six unlock joins, generate/break mineshaft chest carts, persist/reload and render every
item/entity/display/menu/tab state.

**Limits:**

This leaf does not duplicate legacy/improved rail physics, damage/explosion arithmetic, container
allocation, hopper transfer, loot evaluation, command execution, recipe allocation, progression,
entity persistence framing, dispenser scheduling or protocol codecs. Those remain with the cited
owners; this rule fixes the six item-to-entity mappings, their call-site ordering and exact data,
persistence and client joins.
