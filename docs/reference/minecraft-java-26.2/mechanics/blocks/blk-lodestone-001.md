# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-LODESTONE-001` — Lodestone binds Compasses through a zero-ticket POI

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-USE-001`, `ITM-CONTAINER-001`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`,
`MOB-001`, `MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FLUID-001`, `ENV-FIRE-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-BASTION-001`,
`WGEN-STRUCTURE-RUINED-PORTAL-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, `CompassItem`,
`LodestoneTracker`, POI and data-component bytecode, the complete recipe,
advancement, loot and tag set, every compiled exact-name consumer, all
`1,212` decoded structures and exact client resources close Lodestone.
The block has no subclass callback; its special runtime is the exact
Compass interaction, a one-state zero-ticket POI, lazy tracker validation,
two generated-container sources and component-aware client projection.

**Applies when:**

`minecraft:lodestone` is placed, removed, mined, exploded, used by a
Compass, registered or queried as a POI, crafted, generated as chest loot,
migrated, persisted, synchronized or rendered; and when a Compass carrying
`minecraft:lodestone_tracker` ticks, names, glints or selects its needle
model.

**Authoritative state:**

Lodestone is a property-free base `Block`, has no block entity and has sole
block-state ID `21830`. Its block protocol ID is `923`; its ordinary
stack-64 block-item raw ID is `1414`.

Registration fixes map color `METAL`, default note instrument `HARP`,
hardness/resistance `3.5/3.5`, correct-tool-required drops, Lodestone
sounds, friction `0.6`, speed/jump factors `1`, emission `0`, light
dampening `15`, shade brightness `0.2` and piston reaction `BLOCK`.
Outline, collision, visual, support and occlusion shapes are full unit
cubes. Every face is sturdy; the state is an ordinary redstone conductor,
view blocker, suffocation state and valid-spawn support.

It has no placement-state, survival, use, attack, random/scheduled tick,
shape-update, neighbor, entity-contact, fall, signal, comparator, fluid or
block-event override. Land, water and air pathfinding return false through
the ordinary full-solid path. Rotation and mirror preserve state `21830`.

The Lodestone break/step/place/hit/fall sound-event IDs are
`950/951/952/953/954`; the profile has volume/pitch `1/1`.

**Transition and ordering:**

### Placement, piston admission, mining and self loot

Ordinary item placement, component placement, commands and structure writes
select state `21830` without a support predicate. Successful generic
state insertion/removal updates the POI section through the shared block
state/POI lifecycle.

Piston resolution rejects the structure when it reaches Lodestone:
`BLOCK` neither moves nor piston-destroys it. Ordinary explosions can
still remove it according to resistance and the explosion owner.

The sole direct block tag is `mineable/pickaxe`; no incorrect-tier tag
contains Lodestone. A Pickaxe of any material is therefore correct for
drops. Hand and every non-Pickaxe can break it at their generic speeds but
the player correct-tool gate suppresses self loot. A correct Pickaxe,
including one with Silk Touch or Fortune, reaches the same one-roll table:
one Lodestone behind `survives_explosion`, using random sequence
`minecraft:blocks/lodestone`. Enchantments do not change count.
Explosion-origin loot evaluates that same decay condition without a
player-tool gate.

Lodestone has no Fire bootstrap row, lava-ignition property, fuel time,
Composter entry, merchant record, non-block entity loot or item tag.

### Lodestone POI registration

POI protocol ID `18`, `minecraft:lodestone`, registers the complete
possible-state set `{21830}` with maximum tickets `0` and valid range `1`.
It cannot be acquired or claimed through the ordinary ticket API; the
Compass tracker makes the exact non-ticketed
`PoiManager.existsAtPosition(PoiTypes.LODESTONE, target)` query.

Successful block writes and removals add or remove the state-to-POI
mapping through the generic section update. Chunk POI scanning and
persistence can reconstruct the same mapping. No Lodestone callback
searches for Compasses, and removing the block does not eagerly mutate
stacks.

### Exact Compass binding transaction

The ordinary Compass is raw item ID `1063`, a stack-64 `CompassItem`.
Using any other item on Lodestone follows that item's/block's normal
interaction path. Using a Compass runs this exact item callback:

1. It reads the clicked block and delegates unless it is exact Lodestone.
2. It plays `item.lodestone_compass.lock`, sound-event ID `955`, in
   `PLAYERS` at the clicked position with volume/pitch `1/1`.
3. It creates `LodestoneTracker(Optional.of(GlobalPos(current dimension,
   clicked position)), true)`.
4. If the player lacks infinite materials and the held count is exactly
   one, it sets that component on the existing stack.
5. Otherwise it transmutes a one-count copy to Compass while preserving
   the source component patch, consumes one source Compass, overwrites the
   copy's tracker, tries to add it to the player inventory, and drops it
   with `throwRandomly=false` when insertion returns false.
6. It returns `SUCCESS`.

For a non-infinite stacked hand, exactly one input is consumed and the
bound copy is inserted or dropped. For a non-infinite one-stack hand, the
same stack is rebound in place. An infinite-material player keeps the
source stack and receives or drops a separate one-count bound copy, even
when the source count was one. Inventory/drop failure does not undo the
sound, input consumption or tracker construction; the drop result is
ignored.

`ItemStack.useOn` awards `ITEM_USED` for the original Compass after this
item-interaction success. `ServerPlayerGameMode` then triggers
`ITEM_USED_ON_BLOCK` with a pre-callback copy of the Compass and the
clicked position. Thus component mutation, splitting or creative copying
does not prevent the exact Compass criterion from observing the use.

Client interaction runs the same immediate success/prediction path; the
server remains authoritative for inventory, component, advancement and
sound publication.

### Bound-Compass inventory validation

On each server `CompassItem.inventoryTick`, an absent tracker makes no
change. For a present tracker, `LodestoneTracker.tick` applies these gates
in order:

1. `tracked=false` or an empty target returns the same record instance.
2. A target dimension other than the current `ServerLevel` returns the
   same record; no bounds or POI query occurs.
3. In the target dimension, an out-of-world-bounds position or absent
   exact Lodestone POI returns a new
   `LodestoneTracker(Optional.empty(), true)`.
4. An in-bounds existing Lodestone POI returns the same record.

`CompassItem` writes the result only when record identity changed.
Consequently a broken target remains stored while the Compass ticks in a
different dimension, then clears on its first qualifying tick in the
target dimension. Clearing removes only `target`: the component remains
present with `tracked=true`, never reacquires a rebuilt Lodestone, and
continues to select Lodestone-Compass name, glint and model routing.
`tracked=false` deliberately disables validation while leaving any target
available to the client needle.

Dropped stacks do not cause an eager target check through this callback.
Placement/removal, POI maintenance and inventory iteration retain their
generic owners.

### Tracker component persistence and wire form

Data-component protocol ID `67`, `minecraft:lodestone_tracker`, is
persistent, network-synchronized and encoding-cached. Its record has:

- optional `target`, encoded as `GlobalPos` with a dimension key and
  `BlockPos`; and
- Boolean `tracked`, omitted from persistent data when true.

The persistent codec defaults `tracked` to true. The stream codec writes
optional `GlobalPos` followed by a Boolean. Component presence is distinct
from target presence, and arbitrary commands/data operations can construct
empty-target or untracked records. Ordinary Lodestone block stacks have no
default tracker component.

### Recipe and advancements

The sole Lodestone recipe is shaped and fixed:

```text
SSS
S#S
SSS
```

`S` is exact Chiseled Stone Bricks and `#` is exact Iron Ingot. It returns
one default Lodestone. Mirrored or translated placement follows shaped
recipe rules; missing, extra or wrong occupied cells reject the match.
Input component patches and remainders follow the generic crafting owner,
and no recipe consumes Lodestone or produces a bound Compass.

The recipe advancement has one OR requirement across existing knowledge
of `lodestone`, possession of exact Iron Ingot and possession of exact
Lodestone. It awards the recipe and has no other reward.

Adventure advancement `use_lodestone` has one criterion: at the trigger
position a location-check must find exact Lodestone and the pre-use tool
must be exact Compass. It displays a Lodestone icon, awards no explicit
reward and sends a telemetry event. Its English title/description are
`Country Lode, Take Me Home` and `Use a Compass on a Lodestone`.

### Generated-container acquisition

`chests/bastion_bridge` pool zero has one unconditional item entry, makes
one roll and emits exactly one Lodestone. The table uses random sequence
`minecraft:chests/bastion_bridge`; later pools continue the same cursor.
Exactly one decoded Bastion template,
`bastion/bridge/starting_pieces/entrance`, stores this table on a chest.
That rigid template is a weight-one member, alongside weight-one
`entrance_face`, of the reachable `bridge/starting_pieces` pool and uses
`entrance_replacement`. Template selection, connector admission,
processing, clipping, chest creation and loot seed remain with
`WGEN-JIGSAW-BASTION-001`; a materialized bridge-table chest guarantees
the one Lodestone.

`chests/ruined_portal` pool one makes one roll between Empty weight `1`
and Lodestone weight `2`. Lodestone therefore wins with probability `2/3`
and emits a uniformly integral count `1..2`. The table uses random
sequence `minecraft:chests/ruined_portal`. Each of the 13 decoded Ruined
Portal templates—`portal_1..10` and `giant_portal_1..3`—contains exactly
one chest reference to this table. Structure selection, placement,
processors, chest creation and seed installation retain
`WGEN-STRUCTURE-RUINED-PORTAL-001`.

Complete recipe, loot, trade and code-built source searches find no other
direct Lodestone acquisition. Self loot, crafting, these two chest tables,
creative publication and commands are the closed source set.

### Structure-template identity census

An exhaustive decoded state scan of all `1,212` templates finds zero raw
Lodestone cells and zero Lodestone block NBT. Exact decompressed-string
scanning likewise finds no `minecraft:lodestone` occurrence. The fourteen
relevant templates contain chest loot-table identifiers, not a Lodestone
state or stack; loot materializes only when their named tables evaluate.

### Compass-component migration

Lodestone block/state/item identity is stable through the applicable
post-flattening fixes; complete compiled exact-name search finds no
block-specific remap. Three Compass-data migrations are exact:

- `BlockPosFormatAndRenamesFix` upgrades legacy `LodestonePos` through the
  shared block-position format conversion.
- `ItemStackComponentizationFix` removes `LodestonePos` and
  `LodestoneDimension`. When either exists it also removes
  `LodestoneTracked` (default true), creates
  `minecraft:lodestone_tracker`, nests `target` only when both position
  and dimension exist, and writes `tracked` only when false. When both
  location fields are absent it returns before consuming
  `LodestoneTracked`.
- `LodestoneCompassComponentFix` renames
  `minecraft:lodestone_target` to `minecraft:lodestone_tracker`, removes
  its top-level `pos` and `dimension`, and nests both under `target` only
  when both were present; other component remainder fields survive.

Malformed one-sided targets therefore migrate to a present tracker with no
target rather than fabricating a dimension or position.

### Client projection

The sole blockstate variant selects `minecraft:block/lodestone`. Its
`cube_column` model maps the four sides to `block/lodestone_side` and
up/down to `block/lodestone_top`. Both textures are static, untinted,
fully opaque palette PNGs of size 16×16. The Lodestone item definition
points directly to the block model. Its English name is `Lodestone`.

The Compass item definition first tests presence of
`minecraft:lodestone_tracker`. Absence selects a scale-32 Compass range
dispatch targeting spawn; presence selects an otherwise identical
scale-32 dispatch targeting Lodestone. Both have 33 thresholds:
`0 -> compass_16`, half-integers `0.5..14.5 -> compass_17..31`,
`15.5..30.5 -> compass_00..15`, and `31.5 -> compass_16`.

For the Lodestone target, the client reads optional `GlobalPos` from the
component and ignores `tracked`. A target is valid only when present, in
the owner's current dimension and at squared center distance at least
`9.999999747378752E-6`. A valid target angle is
`atan2(targetCenter.z-owner.z, targetCenter.x-owner.x) /
6.2831854820251465`, joined with wrapped visual yaw. A local player while
ticks run normally uses the damping-`0.8` wobbler; other owners use the
direct wrapped formula. An invalid target uses the separate damping-`0.8`
random wobbler plus `hash(seed)=seed*1327217883`, modulo one.

Any tracker component—not merely a nonempty target—also makes
`CompassItem.isFoil` true and changes its name to `Lodestone Compass`.
Thus a lazily cleared Compass continues glinting and naming as bound while
its needle spins randomly.

Functional Blocks publishes Lodestone exactly once, locally after Conduit
and before Ladder. The ordinary Compass remains in its own creative
positions; binding does not create a separately registered item identity.

**Branches and aborts:**

Non-Lodestone Compass use delegates before sound, tracker construction or
inventory mutation. Adventure-mode placement permission and the ordinary
block-use-first route can prevent the Compass callback through their shared
owners. Within binding, only the single survival stack uses in-place
mutation; every stacked or infinite-material case takes the copy/
consume/add/drop branch. Tracker validation aborts unchanged for untracked,
empty-target and other-dimension records, and replaces only after the
matching-dimension bounds/POI test fails. Wrong tools abort player self
loot before table evaluation; rejected chest or block writes prevent their
local effects without rolling back earlier structure work.

**Constants and randomness:**

Fixed identities are state/block/item `21830/923/1414`, Compass item
`1063`, POI `18`, tracker component `67`, block sounds `950..954` and lock
sound `955`. Strength is `3.5`, POI tickets/range are `0/1`, all sound
volume/pitch values are `1/1`, and binding uses no RNG. The self table uses
the named block sequence only for explosion survival. Bastion bridge emits
one deterministically from pool zero; Ruined Portal draws weight `2/3`
then uniform integer `1..2`. Client invalid-target animation draws one
float on each no-target-wobbler update and adds the fixed multiplicative
hash; valid local animation has no random draw.

**Side effects:**

Successful state writes alter the POI section and ordinary block-update
surfaces. Binding plays one lock sound, mutates or creates one tracker
component, may consume a Compass, may insert or drop the bound copy, awards
the original Compass use statistic and can award/telemeter the advancement.
Inventory validation may replace the component value but emits no sound,
particle, event, statistic or eager rescan. Crafting, loot and generated
chests expose only their generic inventory/knowledge/container effects.

**Gates:**

State survival has no local gate. Player self loot requires a correct
Pickaxe and then explosion survival when applicable. POI existence
requires the exact registered state at the exact position. Binding
requires exact clicked Lodestone after interaction routing and exact
Compass dispatch. Validation requires component presence, `tracked=true`,
nonempty target, matching current dimension, world bounds and exact
Lodestone POI. Recipe and advancement predicates, Bastion connector/
template admission, Ruined Portal placement and all client component/
dimension/distance tests apply in the orders specified above.

**Invariants:**

- Lodestone block state is always property-free ID `21830`.
- A successful binding always stores the clicked dimension and position
  with `tracked=true`.
- Binding never consumes more than one Compass and never consumes the
  source count for an infinite-material player.
- Tracker validation never queries a POI outside the target dimension.
- Invalid tracked targets become empty targets; the component is not
  removed.
- A component with `tracked=false` is never invalidated by server ticks.
- Bastion bridge table pool zero always emits one Lodestone; Ruined Portal
  pool one emits zero or `1..2`.
- No bundled structure contains a raw Lodestone state.

**Boundary cases and quirks:**

The block is piston-immovable but explosion-breakable. Any Pickaxe tier is
correct despite the explicit correct-tool requirement. A full stacked
inventory turns split binding into a dropped bound Compass after consuming
the source. Creative binding leaves the original untouched and creates a
new copy. Cross-dimension inventory ticks preserve a stale target.
Rebuilding a broken target cannot restore a tracker whose optional target
has already been cleared. Empty-target and untracked components still
select the Lodestone Compass presentation.

**Failure semantics:**

Rejected non-Lodestone use delegates without the lock sound or tracker.
The binding callback ignores inventory-add and drop return details after
its specified fallback. Wrong-tool player mining suppresses self loot;
explosion decay may suppress it independently. Rejected structure/chest
writes prevent that acquisition instance. POI write, scan and persistence
failures retain their generic owners; a later same-dimension Compass tick
observes the resulting absence.

**Client/server authority split:**

The server owns block/POI state, inventory disposition, tracker validation,
crafting, loot, advancements, generation, persistence and migration. The
client predicts the successful use and lock sound, consumes synchronized
stack components and selects name, foil and needle/block/item models.
Only the server can clear a tracked target from POI absence.

**Observability:**

Observe block/item/POI/component/sound IDs; all physical, tool, piston,
loot and explosion branches; POI add/remove/existence state; exact binding
sound, component, split/creative/inventory/drop and stat/criterion order;
every tracker gate and identity-preserving versus replacing tick; codec and
wire fields; recipe/unlock/use advancement; both chest tables and fourteen
references; zero raw-cell census; migrations; and exact block/Compass
client projection.

**Persistence and reload:**

Chunks persist property-free state `21830` plus generic POI sections; the
block has no entity payload. Lodestone stacks use generic components.
Compass tracker target/tracked fields persist and synchronize through
component ID `67`. Loot, recipe and advancements reload independently.
Registration, POI type, Compass callbacks, component codecs, migrations
and creative ordering are code-built. Reload does not repair already
cleared targets.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.entity.ai.village.poi.PoiTypes`;
`net.minecraft.world.entity.ai.village.poi.PoiManager`;
`net.minecraft.world.item.CompassItem`;
`net.minecraft.world.item.ItemStack#transmuteCopy`;
`net.minecraft.world.item.component.LodestoneTracker`;
`net.minecraft.core.component.DataComponents`;
`net.minecraft.server.level.ServerPlayerGameMode`;
`net.minecraft.util.datafix.fixes.BlockPosFormatAndRenamesFix`;
`net.minecraft.util.datafix.fixes.ItemStackComponentizationFix`;
`net.minecraft.util.datafix.fixes.LodestoneCompassComponentFix`;
`net.minecraft.client.renderer.item.properties.numeric.CompassAngleState`;
`net.minecraft.world.item.CreativeModeTabs`; block/item/POI/component/
sound reports; block/Bastion/Ruined-Portal loot, recipe and both
advancements; the complete block/item tag corpus; all Bastion/Ruined
Portal pools and processors; all `1,212` decoded templates and
decompressed strings; exact blockstate/model/item/texture/language
resources. Complete compiled exact-name and bundled-data searches find no
other Lodestone-specific runtime or source.

**Test vectors:**

Run `EXP-BLK-120` across state/registry identity, every shape/support/path/
redstone/piston/tool/explosion branch, POI add/remove/rebuild/persistence,
single/stacked/full-inventory/infinite-material Compass binding, component
patches and advancement/stat order, every target/tracked/dimension/bounds/
POI inventory-tick boundary, persistent/stream codecs, the recipe and both
advancements, every Bastion and Ruined-Portal table/template path, all
`1,212` raw templates, all three migrations, reload and every client model/
angle/name/foil branch. Assert IDs, ordering, constants, absences, census
and vanilla convergence.

**Limits:**

Generic block lifecycle/mining, piston, POI storage, item-use routing,
inventory insertion/drop, crafting, loot, advancement, Jigsaw/Ruined
Portal, packet and renderer algorithms retain their named owners. Chiseled
Stone Bricks, Iron Ingot, the general Compass identity and both structures
retain their catalog families. This leaf fixes exact Lodestone and the
component/POI paths selected specifically by it.
