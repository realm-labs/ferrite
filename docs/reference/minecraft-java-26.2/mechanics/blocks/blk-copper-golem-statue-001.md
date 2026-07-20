# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-COPPER-GOLEM-STATUE-001` — Copper-golem statues preserve identity across pose, water, wax, weather, item, and entity transitions

**Parent:** `SIM-004`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`, `RED-003`, `ITM-001`,
`ITM-003`, `ENT-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server source, generated block/item reports, block/entity tags and all
eight block loot tables fix the complete statue state space, placement/fluid/pose/comparator
behavior, block-entity components, random weathering, waxing/scraping, golem restoration and
oxidized-golem conversion transaction. Generic interaction routing, state-write propagation, item
durability, entity admission and item-entity construction remain under their shared owners; this
rule fixes their exact statue-specific inputs, ordering and ignored results.

**Applies when:**

Any of the four unwaxed or four same-age waxed statue blocks is placed, updated, picked, broken,
interacted with, randomly ticked or transformed, when its shared block entity transfers components,
or when a server-ticking copper golem reaches the statue-conversion branch.

**Authoritative state:**

The exact IDs are `copper_golem_statue`, `exposed_copper_golem_statue`,
`weathered_copper_golem_statue`, `oxidized_copper_golem_statue` and the corresponding four `waxed_`
IDs. Every ID has horizontal `facing` × `copper_golem_pose={standing,sitting,running,star}` ×
`waterlogged`, hence 32 legal states and 256 family states; default is north/standing/false. All
eight belong to `#minecraft:copper_golem_statues` and use the one `copper_golem_statue` block-entity
type. Their age supplies copper map color; all have strength `3/6`, copper-golem-statue sound,
destroy piston reaction, no occlusion, and a centered 10-by-10 column from Y 0 through 14.

**Transition and ordering:**

Apply the placement/fluid, interaction, wax/scrape/restoration, block-weathering, block-entity/item
and golem-conversion transactions below independently at their stated callback positions. Every
block-family mapping carries common properties and retains the tagged block entity; every
entity/block/item admission result is observed or ignored exactly where stated rather than making
the family transition atomic.

**Placement, fluid and geometric behavior:**

Placement faces opposite the player's horizontal direction, starts standing, and sets waterlogged
only when the replaced fluid state is exactly `Fluids.WATER`; facing rotates/mirrors normally. A
waterlogged state exposes a nonfalling water source and every shape update schedules that water type
at its level-defined delay before delegating. The shared simple-waterlogged transaction accepts only
water as a candidate liquid; placement succeeds only while false and, server-side, writes true with
flags 3 then schedules the supplied water type. Pickup writes false with flags 3, destroys with
drops only if the resulting statue cannot survive, and returns a water bucket; an already dry pickup
returns empty. Water pathfinding is admitted exactly for the water computation type while the
state's fluid is in the water tag.

**Interaction routing and pose:**

Unless generic secondary-use bypass suppresses block use, the block handler runs before the held
item's handler, including for an empty stack. Every nonaxe stack cycles
`standing -> sitting -> running -> star -> standing`: first play `COPPER_GOLEM_BECOME_STATUE` in
`BLOCKS`, then offer the pose state with flags 3, then emit player-context `BLOCK_CHANGE`; write
failure is ignored and does not suppress the sound/event. Each pose has comparator output ordinal
plus one (`1..4`), independent of age, wax, facing, water and block-entity data; removal explicitly
refreshes analog-output neighbors.

**Wax, scrape, and restoration precedence:**

The four unwaxed blocks first require the matching block entity for their special handler. Honeycomb
then returns `PASS`, so its item transaction maps to the waxed block at the same age with all common
properties, triggers the server criterion, calls `shrink(1)`, offers flags 11, emits contextual
`BLOCK_CHANGE` and level event 3003, and ignores the write result; the generic infinite-material
caller restores the original count afterward. An axe on exposed/weathered/oxidized also returns
`PASS`; the axe item maps one age backward, plays `AXE_SCRAPE` plus event 3005 before the flags-11
write, emits contextual block change, damages one and returns success. Axe blocking-item intent can
stop these item-level transforms. Every waxed block returns `PASS` for an axe, so the item maps to
the same unwaxed age with `AXE_WAX_OFF`/3004 and the same write/event/durability transaction. A
waxed block does not special-case honeycomb: honeycomb and every other nonaxe stack cycle pose and
are not consumed. Secondary-use bypass with either hand nonempty skips the statue handler, so it
prevents unaffected-statue restoration and lets only the held item's own transform run.

On an unaffected unwaxed statue, an axe with a matching block entity calls entity creation before
touching the block. The new copper golem receives only the stored custom name, is centered at
`(x+0.5,y,z+0.5)`, takes yaw from statue facing with head/body yaw aligned and pitch zero, and plays
its spawn sound before admission. The held axe is then damaged by one regardless of creation
success. A null factory result returns `PASS` after that damage. A created entity is offered to the
current level and the statue is removed without drops; both booleans are ignored and the block
handler returns success, so admission failure still removes the block and removal failure can
coexist with the new entity. Missing/wrong block entity returns `PASS` without creation or damage.
Axes on exposed or older statues must scrape to unaffected first; a waxed unaffected statue must
first unwax. This block-level restoration precedes `AxeItem`'s blocking-intent check whenever block
use was not bypassed.

**Random block weathering:**

Only the three nonterminal unwaxed ages report randomly ticking; oxidized and all waxed IDs do not.
Each admitted callback first consumes one float and continues only below `0.05688889F`. It scans the
locked `withinManhattan(pos,4,4,4)` order through Manhattan distance four, ignoring self,
non-`ChangeOverTimeBlock` blocks, and blocks whose age enum class differs. Any younger copper-age
block aborts immediately without a second draw. Otherwise let `same` and `older` be the observed
counts and `c=(older+1)/(older+same+1)`. The second float advances only when below `c*c*0.75F` for
unaffected or `c*c` for exposed/weathered. The next unwaxed age inherits every common property and
is offered with `setBlockAndUpdate`; no rollback follows failure. All copper weathering blocks, not
only statues, contribute.

**Block-entity retention and components:**

`shouldChangedStateKeepBlockEntity` is true whenever the old state is in the eight-ID statue tag, so
pose writes and every weather/wax/scrape mapping retain the same block-entity object and component
map. The subtype adds no independent tag fields; base block-entity component persistence and its
standard data packet apply. Golem-to-statue creation merges the existing explicit component map,
replaces its custom-name component from the golem, and dirties the entity. Statue-to-golem
restoration reads only custom name; all other components remain block/item-owned.

Pick-block ignores `includeData` when the matching block entity exists: it applies every collected
block-entity component to the variant's item and then overwrites the item `block_state` component
with exactly the current pose, excluding facing and waterlogged. Without the matching subtype it
falls back to the ordinary block item. Each of the eight locked loot tables returns its own variant
only through `survives_explosion`, copies only `custom_name` from the block entity and only
`copper_golem_pose` from state, and never copies facing, waterlogged, wax/weather countdown or an
entity payload. Generic placement later applies that pose component after constructing the placement
state.

**Copper-golem conversion clock:**

Server golem ticks alone run this branch. Persisted `next_weather_age=-2` means waxed and returns
immediately; `-1` consumes one inclusive `nextIntBetweenInclusive(504000,552000)`, stores current
game time plus that duration, and returns. When due and not oxidized, advance one synced weather
age; a newly oxidized golem stores zero, otherwise add another inclusive duration to the previous
deadline rather than current time. The `isFullyOxidized` decision is captured before advancement, so
newly oxidized does not attempt conversion until its next server tick. A golem already oxidized
(after any initial `-1` setup tick) tests every server tick: it first requires air at its block
position, then consumes one level float and admits equality at `<=0.0058F`.

On admission, consume one entity-random bounded integer for one of four poses, derive horizontal
facing from body yaw, and offer the default dry unwaxed oxidized statue with flags 3; the write
result is ignored. Reread the block entity. Only a matching statue subtype copies the golem custom
name and dirties, drops and clears every equipment slot marked preserved, discards the golem, plays
`COPPER_GOLEM_BECOME_STATUE`, then resolves a remaining leash by dropping it when `entity_drops` is
true or merely removing it otherwise. A rejected/replaced write or missing subtype leaves the golem,
equipment and leash intact after consuming the pose draw; no sound follows. The conversion never
waterlogs from ambient fluid because the offered default is false.

**Branches and aborts:**

Generic interaction bypass; axe/nonaxe/empty stack; missing block entity; null entity factory;
entity/state/removal admission failure; four pose wrap; dry/source/other fluid; every weather age
and wax state; younger-neighbor early abort; explosion rejection; pick fallback; saved wax/deadline
sentinels; due equality; occupied golem position; conversion chance equality; replaced conversion
write; preserved equipment and leash rule.

**Constants and randomness:**

32 states per block; shape width/height `10/14`; comparator `1..4`; block-weather first chance
`0.05688889F`, radius 4 and unaffected factor `0.75F`; entity weather interval inclusive
`504000..552000`; conversion probability inclusive `0.0058F`; flags 3/11; axe durability one; level
events 3003/3004/3005. Block random ticks use their callback RNG, copper-golem age/chance uses level
RNG, and conversion pose uses entity RNG in that order; spawn-at-location/item-entity internals
remain entity-owned.

**Side effects:**

State/fluid writes and their generic update consequences; scheduled water ticks; block and spawn
sounds; player-context game events; comparator refresh; item shrink/durability and criteria;
block-entity component dirtiness/packet data; entity admission/removal, equipment/leash item
entities; loot/pick components. Failed writes/admissions are never transactionally reversed beyond
the explicit subtype rereads above.

**Gates:**

Current block/subtype and block-entity identity, generic interaction priority/secondary-use, item
tag/identity, water fluid identity and state property, random-tick admission, weather
age/neighborhood/floats, explosion condition, server entity ticking, saved deadline sentinels,
target air, conversion chance, post-write subtype reread, preserved-equipment flags and
`entity_drops`. Difficulty does not alter this family.

**Boundary cases and quirks:**

Empty-hand use changes pose. Honeycomb on an already waxed statue changes pose instead of
waxing/no-op. A failed unaffected-statue factory still costs axe durability, while failed golem
admission still removes the statue. Pose sound and block-change event survive a failed pose write.
Any younger copper-age block within the scan aborts aging without the second float. Newly oxidized
golems wait one tick, but already oxidized golems sample conversion every server tick regardless of
their future age deadline. A successful state write is not sufficient for conversion; only the
reread matching block entity commits the entity-side half.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.CopperGolemStatueBlock`,
`net.minecraft.world.level.block.WeatheringCopperGolemStatueBlock`,
`net.minecraft.world.level.block.entity.CopperGolemStatueBlockEntity`,
`net.minecraft.world.level.block.ChangeOverTimeBlock#changeOverTime`,
`net.minecraft.world.level.block.ChangeOverTimeBlock#getNextState`,
`net.minecraft.world.level.block.WeatheringCopper`,
`net.minecraft.world.level.block.SimpleWaterloggedBlock`, `net.minecraft.world.item.AxeItem#useOn`,
`net.minecraft.world.item.HoneycombItem#useOn`,
`net.minecraft.world.entity.animal.golem.CopperGolem#tick`,
`net.minecraft.world.entity.animal.golem.CopperGolem#updateWeathering`,
`net.minecraft.world.entity.animal.golem.CopperGolem#turnToStatue`,
`net.minecraft.world.entity.Mob#dropPreservedEquipment`; locked reports/tags and
`data/minecraft/loot_table/blocks/*copper_golem_statue.json`; `EXP-BLK-008`.

**Test vectors:**

Exhaust all 256 states and eight item/loot projections; source/flowing/dry placement plus liquid
fill/pickup and water scheduling; empty/nonaxe/honeycomb/axe interactions under normal,
secondary-use and blocking-intent routing; all poses and failed writes; missing/wrong BE, null
creation, rejected entity admission and failed removal; every wax/scrape edge with component
identity; weather scans with younger/same/older copper blocks, exact float boundaries and failed
writes; pick/loot with custom and extra components; saved golem sentinels/deadlines, newly/already
oxidized ticks, occupied/air positions, exact chance, each pose/facing, replaced state writes,
equipment and both leash-rule values. Run `EXP-BLK-008` as the executable regression matrix.
