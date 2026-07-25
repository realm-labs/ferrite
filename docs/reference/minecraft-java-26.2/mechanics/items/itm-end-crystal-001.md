# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-END-CRYSTAL-001` — End crystals join constrained item placement to explosive End-fight state

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-ADVANCEMENT-001`,
`ENT-001`, `ENT-LIFECYCLE-001`, `ENT-005`, `ENT-DAMAGE-001`,
`ENT-DAMAGE-REDUCE-001`, `ENT-KNOCKBACK-001`, `ENT-007`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-PROJECTILE-001`, `ENV-FIRE-001`,
`WGEN-PIPELINE-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item/entity registration, item-use and End-crystal bytecode, dragon-fight
joins, recipe/progression data, End-spike owner and client assets/renderers determine the complete
identity-specific transaction. Generic interaction admission, entity lifecycle, explosions,
damage-source filtering, recipes, advancement evaluation and End-spike/dragon-respawn construction
remain with their cited owners.

**Applies when:**

An `end_crystal` stack is created, crafted, selected in a tab or used on a block; an End-crystal
entity is placed, generated, summoned, saved, loaded, ticked, picked, attacked, killed, linked to a
dragon fight or rendered; or its recipe, fight, worldgen and client resources reload.

**Authoritative state:**

`minecraft:end_crystal` is raw item ID `1312`, common, nondamageable, max stack `64`, and belongs to
no direct item tag. Its prototype has `enchantment_glint_override=true`, so the ordinary unenchanted
stack still glints. Otherwise it has only the common empty modifiers/enchantments/lore, item-break
sound, translated name, direct item-model key, repair cost, swing animation, tooltip display and
use effects. It has no consumable, food, cooldown, entity-data or use-remainder component.

The matching entity is static entity-type raw ID `45`, category `MISC`, serializable and summonable,
with no loot table, fire immunity, dimensions `2 × 2`, tracking range `16` and update interval
`2,147,483,647`. It sets `blocksBuilding=true`, emits no movement event and is pickable. Its
subtype state is nullable beam target plus show-bottom Boolean. Defaults are no beam and
`showBottom=true`.

Construction consumes one `nextInt(100000)` for public animation counter `time`. Each entity tick
increments that counter, applies ordinary block effects and portal handling, then, on a server
level with a dragon fight, replaces air at its block position with the position-derived fire
state. The fire write result is ignored. The counter is transient client-local/server-local
animation phase, not synchronized subtype authority.

**Transition and ordering:**

After generic block interaction and adventure placement admission, `useOn` reads the clicked block
state. Anything other than exact obsidian or bedrock returns `FAIL`. The clicked face is otherwise
irrelevant: the candidate lower cell is always `clickedPos.above()`.

The candidate lower cell must satisfy `isEmptyBlock`. No separate emptiness or collision test is
made for the cell above it. The method then queries every entity in the exact axis-aligned box
`[x,x+1] × [y,y+2] × [z,z+1]`; any returned entity fails placement. These gates run on both logical
sides and do not mutate the stack.

On the server only, a passing placement constructs an End crystal directly at
`(x+0.5,y,z+0.5)`, sets `showBottom=false`, and calls `addFreshEntity` without observing its
Boolean result. It then emits `ENTITY_PLACE` at the lower cell with the placing player as source
and, when the level exposes an End dragon fight, calls `tryRespawn`. Thus add failure does not
suppress the event or fight check.

After that server-only block, both projections call `shrink(1)` and return `SUCCESS`. The subtype
method does not inspect player abilities. The server game-mode wrapper instead snapshots and
restores count around item use for infinite-material players; ordinary players retain the shrink.
Success awards the generic item-used statistic, and the server interaction wrapper triggers
`item_used_on_block` using the pre-use stack snapshot. Failure awards neither and changes no count.
Using the item in air follows the ordinary item `PASS` path.

**Dragon-fight and dragon joins:**

The post-placement `tryRespawn` call is inert unless the dragon is killed and no respawn stage is
already active. Its portal discovery, four horizontal portal-side crystal queries and complete
respawn-stage transaction are owned by `WGEN-PIPELINE-001`. This leaf fixes only that every passing
server placement invokes the check after the ignored entity add and game event, including
placements outside the ritual positions.

When any crystal is destroyed, the current End fight is notified with that crystal and the
original damage source. Destroying a referenced ritual crystal while a respawn stage is active
aborts the sequence, resets stage time, restores ordinary spike-crystal state and respawns the
active exit portal. Otherwise the fight recounts spike crystals and, when its dragon UUID resolves,
forwards the destruction to that dragon.

The dragon maintains a nearest crystal. Every tick whose dragon `tickCount` is divisible by `10`,
a present nonremoved nearest crystal heals it by `1` up to max health. Independently, one
`nextInt(10)` is consumed each dragon tick; on zero it scans End crystals in its bounding box
inflated by `32` and retains the first strict nearest by squared distance. A removed retained
crystal is cleared before healing and search.

On forwarded destruction, a player damage-source entity is used directly; otherwise the dragon
selects the nearest combat-admissible player within `64` of the crystal's integer block position.
If the destroyed crystal was the dragon's retained nearest, the dragon head receives `10` damage
from a new explosion source attributed to the crystal and selected player. The current dragon
phase is then notified; the holding-pattern phase starts a strafe only when that player exists and
is attackable. These dragon responses do not alter the crystal's already committed removal.

**Damage, explosion and removal:**

Client damage returns false for ordinary base invulnerability or a source whose responsible entity
is an Ender dragon; every other source is accepted client-side without removing the local entity.
Server damage applies the same gates. Once admitted, the float amount is ignored.

An admitted hit on a not-yet-removed crystal first removes it with reason `KILLED`. If the incoming
damage type is not in `is_explosion`, it then creates a server explosion centered exactly at the
crystal with radius `6`, no fire and block interaction. When the incoming source has an entity, the
new explosion source uses the crystal as direct entity and that entity as owner; otherwise the
explicit explosion damage source is absent. Incoming explosion damage therefore removes the
crystal without recursively creating another explosion.

Only after the optional explosion completes does the crystal notify the End fight using the
original source. Nearby crystals removed by the generated explosion can consequently notify the
fight before the initiating crystal. An already removed crystal skips removal, explosion and
notification but still returns true from server damage.

Direct `kill(ServerLevel)` instead notifies the fight with generic damage before delegating to
generic kill; it never creates the radius-6 explosion. Unload, discard and other generic removal
paths do not call the subtype destruction notifier. No path evaluates an intrinsic loot table or
drops the item. Wind-charge projectiles explicitly ignore End crystals in their owned hit filter.

**Crafting, progression and creation boundaries:**

The shaped recipe is `GGG/GEG/GTG`, where `G` is glass, `E` an ender eye and `T` a ghast tear. It
consumes seven glass, one eye and one tear and produces one default crystal stack. Its recipe
advancement is granted by either obtaining an ender eye or already unlocking the recipe, and
rewards that recipe.

The End `respawn_dragon` advancement uses this item only as its display icon; its
`summoned_entity` criterion observes the eventual dragon, not placement of a crystal. No locked
loot table, trade, mob path or structure payload creates the item, and an exhaustive scan of all
`1,212` locked structure templates contains no `end_crystal` identity.

`WGEN-PIPELINE-001` owns the other standard entity source. Ordinary End-spike generation creates
show-bottom crystals with no beam and no invulnerability. Respawn-stage spike rebuilds instead
create invulnerable crystals beaming at `(0,128,0)`. Item placement always forces only
`showBottom=false`; it does not copy stack components into entity state.

**Persistence and reload boundary:**

The entity saves nullable beam target under `beam_target` and always writes show-bottom under
`ShowBottom`; a missing saved show-bottom defaults true. Generic entity state retains identity,
UUID, position/rotation/motion, invulnerability, fire, portal and lifecycle fields. The random
`time` counter is neither saved nor synchronized, so construction after reload consumes a new
animation offset.

Item persistence retains ordinary identity, count and component patches. Placing and later picking
the entity is not a component round trip: `getPickResult` creates a fresh default end-crystal
stack, and the entity has no ordinary break drop. Recipe reload can replace crafting/unlock data;
worldgen/fight state owns its durable reload behavior; resource reload independently replaces item
and entity rendering.

**Client and wire projection:**

The stack projects raw item ID `1312` and its ordinary component patch. Entity pairing projects
static type ID `45`, position/rotation and two subtype metadata values after the eight base-entity
slots: index `8` is optional block position with serializer ID `11`, and index `9` is Boolean with
serializer ID `8`. Defaults are absent/true. Spawn, metadata, explosion, removal and correction
packet layouts remain with their protocol owners.

The item definition selects generated model `minecraft:item/end_crystal` and texture
`minecraft:item/end_crystal`; its fixed glint overlays that model. It appears in Combat after TNT
and before Snowball, and in Functional Blocks after Enchanting Table and before Brewing Stand.

The entity renderer has shadow radius `0.5`, scales the baked End-crystal model by `2`, toggles its
base from show-bottom, and animates nested glass/cube parts from client-local `time`. It uses
`textures/entity/end_crystal/end_crystal.png`. A beam target produces an offset to that block's
center and the dragon beam renderer uses the crystal-beam texture. A nonnull beam also forces
distance/frustum admission beyond the generic entity result.

**Branches and aborts:**

Clicked identity/face, lower-cell air, two-cell entity occupancy, side, add result, player ability,
count and adventure admission; item versus spike/summon creation; fight presence/dragon-killed/
respawn-stage/ritual position; tick fire state; removed/invulnerable/dragon/explosion/other damage
source; source entity/player search/nearest crystal/current dragon phase; kill versus hurt versus
other removal; recipe/unlock/advancement, persistence/reload, metadata, item/entity model and tab.

**Constants and randomness:**

Item/entity raw IDs `1312/45`; max stack `64`; dimensions `2 × 2`; tracking range `16`; update
interval `2,147,483,647`; placement box `1 × 2 × 1`; center offsets `(0.5,0,0.5)`; one shrink;
animation draw bound `100000`; heal interval/amount `10/1`; nearest-search chance `1/10` and inflate
`32`; player range `64`; nearest-crystal dragon damage `10`; explosion radius `6`, fire false;
metadata indices/serializers `8/11` and `9/8`; renderer shadow/scale `0.5/2`.

**Side effects:**

Stack count; item-used statistic and item-on-block criterion; entity construction/add/removal;
show-bottom/beam/invulnerability/fire; placement game event; dragon respawn-stage and nearest/
health/phase joins; block/entity explosion effects; recipe output/unlock; persistence; entity
spawn/metadata/explosion/removal traffic; item glint/model/tabs and entity/base/beam rendering.

**Gates:**

Generic interaction/adventure/cooldown admission; exact obsidian or bedrock; empty lower cell;
empty entity query; server authority; infinite-material restoration; dragon-fight state and four
ritual queries; base invulnerability and non-dragon source; incoming explosion tag; current fight/
dragon/phase/player; recipe criteria; valid save/registry decode; tracking and client resources.

**State read/written:**

Reads clicked state/position, candidate air/entities, side, hand count, player abilities, current
fight/portal/dragon/respawn state, damage source/tag/entity, crystal metadata/removal, dragon
health/tick/RNG/nearest/phase, recipe/advancement/worldgen data, saved fields and client resources.
Writes exactly the stack, entity, world, fight/dragon, persistence and client projection listed
above.

**Failure behavior:**

Wrong base, a nonair lower cell or any entity in the placement box returns `FAIL` without shrink,
stat, criterion, event or fight check. Client/server occupancy disagreement follows ordinary
prediction correction. A failed entity add is ignored after construction and cannot roll back the
event, fight check or shrink. Rejected damage returns false; accepted repeated damage returns true
even when already removed. Explosion and fight callbacks have no transaction rollback.

**Boundary cases and quirks:**

A solid block may occupy the upper half of the candidate volume because only the lower block is
tested, while a tiny entity anywhere in the full volume rejects placement. The placement method
shrinks even in creative, but the enclosing server game-mode call restores the snapshot. Item
placement hides the base while generated/summoned defaults show it. Any admitted amount destroys a
crystal. Explosion damage suppresses only the crystal's secondary explosion, not fight
notification. Nonexplosion destruction removes the initiator before its explosion, so chained
victims notify first. The glint is a prototype override, not evidence of enchantments.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.EndCrystalItem#useOn`;
`net.minecraft.world.item.ItemStack#useOn`;
`net.minecraft.server.level.ServerPlayerGameMode#useItemOn`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.boss.enderdragon.EndCrystal`;
`net.minecraft.world.entity.boss.enderdragon.EnderDragon#checkCrystals`;
`net.minecraft.world.entity.boss.enderdragon.EnderDragon#onCrystalDestroyed`;
`net.minecraft.world.entity.boss.enderdragon.phases.DragonHoldingPatternPhase#onCrystalDestroyed`;
`net.minecraft.world.level.dimension.end.EnderDragonFight#tryRespawn`;
`net.minecraft.world.level.dimension.end.EnderDragonFight#onCrystalDestroyed`;
`net.minecraft.world.level.levelgen.feature.EndSpikeFeature`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.client.renderer.entity.EndCrystalRenderer`;
`net.minecraft.client.model.object.crystal.EndCrystalModel`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/end_crystal.json`;
`data/minecraft/tags/damage_type/is_explosion.json`;
`data/minecraft/recipe/end_crystal.json`;
`data/minecraft/advancement/recipes/decorations/end_crystal.json`;
`data/minecraft/advancement/end/respawn_dragon.json`;
`data/minecraft/worldgen/configured_feature/end_spike.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/items/end_crystal.json`;
`assets/minecraft/models/item/end_crystal.json`;
`assets/minecraft/textures/item/end_crystal.png`;
`assets/minecraft/textures/entity/end_crystal/{end_crystal,end_crystal_beam}.png`;
`BLK-BEDROCK-001`; `ENT-PROJECTILE-001`; `WGEN-PIPELINE-001`;
`ITM-RECIPE-001`; `ITM-ADVANCEMENT-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-036`.

**Test vectors:**

Use both hands and projections against every base/face, lower/upper block and entity-box boundary
with ordinary/infinite-material players and forced entity-add outcomes. Place outside/at each
ritual side across fight states. Create item, natural-spike, respawn-spike and command crystals;
tick across fire/portal states. Apply every invulnerability/dragon/explosion/other source with
zero/signed/nonfinite amounts, repeated hits, chains and direct kill; capture notification, dragon
heal/search/damage/phase and ordering. Persist/reload all fields and capture recipe, progression,
raw IDs, metadata, glint, item/entity/base/beam models and both tab positions.

**Limits:**

This leaf does not duplicate generic block interaction, explosion calculation/publication,
damage-source tags, entity tracking/removal, recipe matching, advancement evaluation or the
End-spike and dragon-respawn stage machine. Those remain with their cited owners; this rule fixes
the end-crystal identity and every direct join into them.
