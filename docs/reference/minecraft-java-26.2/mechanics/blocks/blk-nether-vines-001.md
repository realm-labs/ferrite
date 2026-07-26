# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-VINES-001` — Nether vines grow through directional head/body chains

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`, `PLY-002`, `PLY-005`, `PLY-006`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-MOVE-001`, `PLY-MOVE-SPECIAL-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-LOOT-001`, `ITM-ENCHANT-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-LIGHT-001`, `ENV-FIRE-001`, `WGEN-003`, `WGEN-PIPELINE-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and reports, all four concrete classes and
their complete growing-plant inheritance, exact loot/tag/Composter/fall-location and
worldgen consumers, all 1,212 templates and exact client resources close the family.

**Applies when:**

`minecraft:weeping_vines`, `minecraft:weeping_vines_plant`,
`minecraft:twisting_vines` or `minecraft:twisting_vines_plant` is placed, updated,
random-ticked, bonemealed, cloned, broken, exploded, composted, climbed, glided
through, selected as a fall location, generated, persisted, synchronized or rendered.

**Authoritative state:**

| Identity | Direction | State IDs | Block ID | Item ID |
|---|---|---:|---:|---:|
| Weeping head | Down | `20977..21002`, default `20977` | `878` | `282` |
| Weeping body | Down | `21003` | `879` | none |
| Twisting head | Up | `21004..21029`, default `21004` | `880` | `283` |
| Twisting body | Up | `21030` | `881` | none |

Each head is a `GrowingPlantHeadBlock` with `age=0..25`; each body is a
property-free `GrowingPlantBodyBlock`. None has a block entity or fluid property.
Heads register random ticks; bodies do not. All four are instant-breaking,
noncolliding, zero-emission blocks with piston reaction `DESTROY`, no offset and
default friction/speed/jump factors.

The authoritative outline columns are:

| Identity | X/Z range | Y range |
|---|---:|---:|
| Weeping head | `4..12` | `9..16` |
| Weeping body | `1..15` | `0..16` |
| Twisting head | `4..12` | `0..15` |
| Twisting body | `4..12` | `0..16` |

Collision is empty. The partial outline shapes do not provide full sturdy faces
or admit ordinary spawning. Weeping uses map color `NETHER`;
Twisting uses `COLOR_CYAN`. Both retain the default `HARP` note instrument.

All four registrations select `SoundType.WEEPING_VINES`, volume/pitch `1/1`, with
break/step/place/hit/fall sound IDs `1141/1142/1143/1144/1145`. The separately
constructed `TWISTING_VINES` sound type reuses those events at pitch `0.5`, but none
of these four locked registrations selects it.

Only the two heads have block items. Each is a common nondamageable 64-stack with
ordinary block-item components. All four blocks directly belong to `climbable` and
`can_glide_through`; neither item has a direct item tag.

**Transition and ordering:**

### Placement, support and chain repair

Weeping grows Down and reads support above; Twisting grows Up and reads support below.
A segment survives when the opposite-growth neighbor is its matching head/body or has
a sturdy face toward the growth direction. There is no light or identity-specific
substrate predicate.

Placement first reads the target's growth-direction neighbor. A matching head or body
selects the property-free body without RNG. Otherwise placement calls
`nextInt(25)` and selects a head aged `0..24`. Ordinary placement validation, stack
consumption, sound and neighbor publication remain with `BLK-PLACE-001`.

When a head observes another matching head/body in its growth direction, it converts
to the matching default body and discards age. When a body observes neither matching
identity in its growth direction, it converts to a head and consumes `nextInt(25)`
for age `0..24`. Thus adding a new tip turns the previous tip into body; removing a
tip turns the adjacent supported body into a newly aged tip.

A support-side update that finds an invalid segment schedules that block after one
tick. The callback rechecks survival and calls `destroyBlock(pos,true)` on failure,
ignoring the result. Removing a middle segment therefore makes the supported side
terminate as a new head while the unsupported side can destroy with drops in a
one-tick cascade.

Body clone selection always returns the matching head item. Body replacement also
refuses the matching head item whenever the inherited replacement predicate would
otherwise admit it. Rotation and mirror do not alter any state.

### Random head growth

Only heads aged `0..24` are randomly ticking. Every selected callback reads age and
consumes one `nextDouble()`; it proceeds exactly when the draw is below `0.1`.
Admission then reads one cell in the growth direction and requires `isAir()`. It
offers that cell the same head identity with age incremented by one through
`setBlockAndUpdate`, ignores the result and consumes no further Nether-vine RNG.
The helper is passed `level.getRandom()` but the shared age-cycle implementation does
not draw from it.

A successful write's neighbor updates convert the old head to body. Age `25`, a
failed probability draw or nonair target performs no write; age `25` consumes no
draw. The air predicate admits any state whose `isAir()` is true rather than checking
one registry identity.

### Bone-meal lengthening

For a head, target validity requires the next growth-direction cell to satisfy
`isAir()` and be inside build height; head age is irrelevant. For a body, the shared
scan moves in the growth direction through consecutive exact body identities and
requires a matching head immediately after them. It then applies the same air and
height test beyond that head. A missing/mismatched head rejects.

Success is unconditional. Performance on a body repeats the scan and delegates to
the live head when found; failure to find it performs no vine write. Head performance
draws the requested growth count with this exact loop:

```text
probability = 1.0
count = 0
while nextDouble() < probability:
    probability *= 0.826
    count += 1
```

The first draw always succeeds for a conforming `nextDouble`, so count is at least
one; every admitted continuation and the terminating comparison consume a draw.
There is no explicit integer cap.

Starting one cell in the growth direction, performance takes the input head age plus
one, capped at `25`. For at most the sampled count it rechecks `isAir()` and build
height, offers the original head identity at the current capped age with
`setBlockAndUpdate`, ignores the result, advances one cell and increments/caps age.
The first nonair or outside-height cell stops the loop. Rejected writes do not
otherwise stop later iterations.

Each accepted extension turns the preceding head into body through neighbor updates,
leaving one terminal head. The generic Bone Meal item consumes one stack unit for an
admitted server target. The default bonemealable type is `GROWER`, so its particle
position is `clicked.above()` for both orientations, including downward Weeping.

### Breaking, loot and item acquisition

All four one-roll tables emit the matching head item. Shears or Silk Touch level at
least one take the first alternative and yield one item without an explosion
condition. Otherwise `table_bonus` tests Fortune with chances
`0.33/0.55/0.77/1.0` for levels zero/one/two/three-or-more. The four independent
random sequences are `minecraft:blocks/<block-id>`.

There is no correct-tool requirement and no `survives_explosion` condition. Hand and
ordinary-tool breaks therefore use the `0.33` branch, while an explosion with no tool
context reaches that same conditional path. A body never drops a body item.

Both head items compost at chance `0.5`. They have no bundled recipe, furnace fuel,
trade, container-loot, food, fire-registration or other direct item consumer.

### Movement and fall-location joins

For a non-spectator living entity in either head/body state, `climbable` records the
current block position, resets fall distance and selects the shared climbing movement
limits. While fall-flying, `can_glide_through` is checked first and makes
`onClimbable()` return false, preserving glide-through behavior.

If the recorded last climbable position contains either Weeping identity,
`FallLocation` selects `weeping_vines`; either Twisting identity selects
`twisting_vines`. The English accidental-fall messages are exactly
`%1$s fell off some weeping vines` and `%1$s fell off some twisting vines`.
Movement, fall damage and message dispatch remain with their entity owners.

### World-generation joins

The exact Nether-vines feature algorithms remain fully specified by
`WGEN-PIPELINE-001`. The ordinary Twisting configuration uses spread width/height and
maximum height `8/4/8`; `twisting_vines_bonemeal` uses `3/1/2`. Both exact feature
paths require empty cells and one of Netherrack, Warped Nylium or Warped Wart Block
below, write upward bodies plus an age-`17..25` head with flags `2`, ignore write
results and return true after initial admission.

The empty Weeping configuration requires an empty origin below exact Netherrack or
Nether Wart Block. It grows a 200-attempt wart roof and then 100 downward-column
attempts, writing Weeping bodies and age-`17..25` heads with flags `2`. The Crimson
huge-fungus path can separately call the same column helper below admitted hat cells
with length `1..5`, conditional doubling and head age `23..25`.

Both ordinary placed profiles use count `10`, in-square, uniform dimension-bottom-
through-top height and biome. Crimson Forest includes Weeping and Warped Forest
includes Twisting at decoration group `9`. Warped Nylium can additionally invoke the
Twisting bonemeal profile after vegetation/sprouts on its one-in-eight branch.

All four natural/planted Crimson/Warped huge-fungus configurations include every
head/body identity in their replaceable-block predicate. An exhaustive scan of all
1,212 bundled structure templates finds zero raw cells for each identity.

**Client projection:**

Every Weeping head age selects the same untinted cross model and texture; every
Twisting head age does likewise. Each body selects its own untinted cross texture.
The shared `cross` parent disables ambient occlusion and supplies two shade-free
diagonal double-sided planes. Head item selectors use generated flat models whose
layer is the matching **body** texture.

English item/block names are `Weeping Vines` and `Twisting Vines`; body-only
translation records are `Weeping Vines Plant` and `Twisting Vines Plant`.
Natural Blocks orders Weeping then Twisting immediately after Nether Sprouts and
before Vine. State packets use the IDs above; inventory uses only item IDs `282/283`.

**Branches and aborts:**

Two colors/directions and head/body identities; every age; placement forward
identity and age draw; support face and scheduled destruction; head/body repair;
random-growth draw/air/write; direct/body bone-meal scan, height, geometric draws,
air and writes; four tool/Fortune/sequence paths; compost; ordinary/gliding climbing
and fall messages; regular/bonemeal/huge generation, replacement and 1,212-template
census; persistence and client projection are distinct.

**Constants and randomness:**

Head ages `0..25`; placement/conversion `nextInt(25)`; random growth probability
`0.1`; bonemeal decay multiplier `0.826`; shapes `8x7`, `14x16`, `8x15`, `8x16`;
sound IDs `1141..1145`; loot chances `0.33/0.55/0.77/1`; compost `0.5`;
Twisting configs `8/4/8` and `3/1/2`; placed count `10`; generated head ages
`17..25` or huge-fungus `23..25`; templates/cells `1212/0/0/0/0`.

**Side effects:**

Head/body/air writes, scheduled ticks and cascading drops; Bone Meal consumption and
particles; loot/item entities; Composter state; climbing/fall state and messages;
feature and huge-fungus writes; ordinary persistence; maps, sounds, models, names
and tab projection.

**Gates:**

Growth direction, support identity/face, block-tick survival, head age, strict
probability draw, `isAir()`, build height, connected-body/head scan, tool/Silk/
Fortune/loot context, Composter RNG, spectator/fall-flying/tags, last climbable
position, feature origin/support/configuration/biome and client resources.

**Boundary cases and quirks:**

- Ordinary placement and body-to-head repair randomize age `0..24`; default age zero
  is not guaranteed.
- Age `25` blocks random growth but not Bone Meal.
- Bone Meal length is a decreasing-threshold loop, not a fixed uniform range.
- The default Bone Meal particle position is above even for downward-growing Weeping.
- Body clone and every body loot path yield the head item.
- Fortune three guarantees the fallback drop; no branch tests explosion survival.
- All four blocks select full-pitch Weeping sounds; the half-pitch Twisting sound
  object is unused by these registrations.
- A fall-flying entity glides through rather than entering climbable movement.

**Failure semantics:**

Unsupported scheduled ticks destroy with drops. Failed placement/growth/bonemeal
writes are ignored without rollback; body execution can no-op after earlier target
admission, and a rejected bonemeal write does not itself terminate the remaining
sampled iterations. Failed loot/Composter/movement/worldgen gates commit only their
generic owner's effects.

**Client/server authority split:**

The server owns state, updates, ticks, growth, Bone Meal, loot, Composter, movement,
fall location, worldgen and persistence. Clients project committed state/age, maps,
sounds, particles, models, names, tab entries and localized fall messages.

**Observability:**

Commands/state packets, outline/collision probes, placement/update/tick traces,
controlled RNG, Bone Meal counts/particles, drops, Composter levels, entity motion,
fall messages, generated terrain and client models expose the listed branches.

**Persistence and reload:**

Heads persist identity and age; bodies persist identity only. Items persist ordinary
components. Loot, tags, biomes, configured/placed features, huge-fungus predicates
and language/assets are reload-selected. Class control flow, registrations,
Composter chance, fall-location identity order and tab order remain code-built.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`;
`OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`;
`GrowingPlantBlock#getStateForPlacement`, `#canSurvive` and `#tick`;
`GrowingPlantHeadBlock#getStateForPlacement`, `#isRandomlyTicking`, `#randomTick`,
`#updateShape`, `#isValidBonemealTarget` and `#performBonemeal`;
`GrowingPlantBodyBlock#updateShape`, `#getCloneItemStack`,
`#isValidBonemealTarget`, `#performBonemeal` and `#getHeadPos`;
`NetherVines#isValidGrowthState` and `#getBlocksToGrowWhenBonemealed`;
all four concrete Nether-vine classes; `BlockUtil#getTopConnectedBlock`;
`ComposterBlock#bootStrap`; `LivingEntity#onClimbable`;
`FallLocation#blockToFallLocation`; `CreativeModeTabs`;
`WGEN-PIPELINE-001`; both feature classes and `HugeFungusFeature`;
reports/components, four loot tables, two block tags, two biomes, three configured
and two placed feature records, four huge-fungus configurations, all four blockstate/
block-model resources, two item resources and all 1,212 templates. Complete exact-ID
and compiled-field-reference searches found no other runtime path.

**Test vectors:**

Run `EXP-BLK-103` across both directions, all four identities and every age; placement,
support loss, middle/head removal, conversion, random growth, head/body Bone Meal,
write failures, tools/Silk/Fortune/explosion, compost, climb/glide/fall selection,
regular/bonemeal/huge generation, all templates, persistence and client projection.
Assert every constant, draw/read/write order, absence claim and convergence.

**Limits:**

Generic placement/break/loot/explosion, block-update publication, Bone Meal,
Composter, entity movement/fall damage, feature placement, packet encoding and
rendering remain with their named owners. This leaf fixes the four exact identities,
directional state machine, growth distributions, acquisition, joins and projection.
