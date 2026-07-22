# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BEACON-001` — Beacons incrementally publish a colored sky beam before refreshing pyramid effects

**Parent:** `SIM-003`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`,
`PLY-005`, `ITM-002`, `ITM-CONTAINER-001`, `ITM-CONTAINER-CONTROL-001`,
`ITM-CONTAINER-CLOSE-001`, `ENT-006`, `CLI-001`, `CLI-005`, `CLI-006`, `ENV-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server and client classes, block/registry/item reports, bundled tags,
loot, advancement and model assets, and the already specified beacon control packet fix the
incremental beam scan, pyramid refresh, effects, menu transaction, persistence and renderer
branches. Generic container clicks/close, effect merging, tag reload publication and sound/packet
audiences remain owned by their existing leaves.

**Applies when:**

A beacon is placed, ticked, obstructed, supplied with a base, opened, paid, configured, removed,
saved, loaded, synchronized or rendered; or its active pyramid refresh applies one or two effects
and triggers the construct-beacon criterion.

**Authoritative state:**

The block has one propertyless state, state ID `9980`. Its block-entity protocol ID is `15`. The
block entity begins with empty published and in-progress beam-section lists, level `0`, null
primary and secondary powers, no custom name, no lock, and a scan cursor initialized by
`setLevel` to `level.minY - 1`.

The block emits light `15`, has diamond map color, hat note-block instrument, hardness and blast
resistance `3`, no occlusion, a full-cube shape and an explicitly false redstone-conductor
predicate. It does not require a correct tool. Its loot table unconditionally returns one beacon
and copies only `custom_name` from the block entity. The item stacks to 64 and has rare rarity.

The six valid powers are tiered as speed/haste at required level 1, resistance/jump boost at 2,
strength at 3, and regeneration at 4. Published beam sections and scan cursor are transient.
Primary/secondary power, custom name and lock are persistent. `levels` is written but deliberately
not loaded; it, both beam lists and scan progress therefore reconstruct from zero/empty state.

**Transition and ordering:**

Every admitted client or server block-entity tick first advances at most ten vertical scan cells.
At `gameTime % 80 == 0`, the prior published beam controls pyramid rescan, effect refresh and
ambient sound. Only afterward can a scan that reached the heightmap boundary replace the published
sections. On that same completion path, the server compares the level value captured at tick entry
with its current value to emit activation/deactivation and advancement effects.

Menu ingress is separate: a server interaction opens the ordinary beacon menu, then a dedicated
serverbound control request validates the current level and payment before storing powers,
consuming one payment and dirtying the chunk. The canonical client sends that request before its
independent close request, as specified by `PROTO-PLAY-SERVERBOUND-ANVIL-BEACON-001`.

**Incremental beam scan:**

When `lastCheckY < beaconY`, the next tick clears the in-progress list, starts at the beacon cell
and sets `lastCheckY=beaconY-1`. Otherwise it resumes at `lastCheckY+1`. The upper endpoint is
`Level.getHeight(WORLD_SURFACE,x,z)`, inclusive. At most ten cells are consumed per admitted tick;
there is no catch-up for unloaded or inactive time.

The beacon itself implements `BeaconBeamBlock` with white color. Stained-glass blocks and panes
are the other implementations and contribute their texture diffuse color. A colored block always
creates a new section while the list has at most one section; afterward it increments the current
section when colors match, or appends a section whose color is the ARGB average of the current and
new colors when they differ. Each new section starts at height one.

An ordinary non-beam block extends the current section when its light dampening is below 15 or it
is bedrock. With no current section, or with dampening 15 from any non-bedrock block, the scan
clears every in-progress section, jumps `lastCheckY` to the heightmap endpoint and stops that
tick. Thus bedrock is an explicit transparent exception, while tinted glass blocks because it is
not a beam block and has dampening 15.

After each admitted cell the cursor and position advance by one. Once `lastCheckY >= height`, the
entity resets the cursor to `minY-1` and atomically replaces the published list with the completed
in-progress list. Partial scans never leak into rendering or the 80-tick gameplay refresh.

**Pyramid refresh and active residue:**

Only an 80-tick boundary with a nonempty previously published beam list calls the base scanner.
It tests complete square layers one through four below the beacon: side lengths 3, 5, 7 and 9,
requiring 9, 25, 49 and 81 cells respectively. Every cell must currently be in the reloadable
`minecraft:beacon_base_blocks` tag; a layer may mix netherite, emerald, diamond, gold and iron
blocks. The first incomplete or below-minY layer stops the scan, returning the number of complete
layers `0..4`.

If the published beam list is empty, base scanning is skipped and the old level value is retained.
Beam obstruction therefore immediately publishes no beam after its incremental scan completes
and suppresses effects/rendering, but does not by itself clear the remembered pyramid level or
emit the level-based deactivation sound. A base mutation while the beam is obstructed also remains
unobserved until a nonempty beam is published and a later 80-tick boundary scans it.

Activation/deactivation is not a direct beam-list comparison. At scan completion on the server,
the implementation compares `oldLevel > 0` from tick entry with current `levels > 0`. A same-tick
zero-to-positive transition plays activate sound and triggers `construct_beacon(level)` for every
server player in the normalized box from `(x,y,z)` to `(x,y-4,z)`, inflated by `(10,5,10)`; its
final extents are X/Z `x/z ± 10` and Y `y-9..y+5`. A positive-to-zero transition plays deactivate.
If an unusually tall colored column delays scan completion beyond the 80-tick level change, that
later completion sees no transition and does not replay the sound or criterion.

Server-side block-entity removal separately plays the deactivate sound unconditionally, including
when the retained level is zero or the beam is already obstructed. The matching client-side call
does not locally play it because `ClientLevel` ignores the null-excluded-player sound path.

**Effect refresh:**

At an 80-tick boundary, effects and ambient sound require both `levels > 0` and a nonempty
published beam. Effect application additionally requires a nonnull primary and a server level.
For level `L`, the player query box begins as the beacon cell AABB, inflates by
`r = 10L + 10` on all axes, then expands upward by the level's total height. There is no spherical
distance, wetness, game-mode or line-of-sight filter.

Every returned player receives primary for `d = (9 + 2L) × 20` ticks, hence levels 1..4 give
durations 220, 260, 300 and 340 and radii 20, 30, 40 and 50. Primary amplifier is 1 only at level 4
when secondary equals primary; otherwise it is 0. Ambient and particle visibility flags are true.
At level 4, a distinct nonnull secondary is then applied to the same player list for the same
duration at amplifier 0. When secondary equals primary it is not applied a second time.

The ambient beacon sound follows the successful level/beam gate every 80 ticks even when primary
is null; effect application has already returned in that case. All beacon sounds use block source,
volume 1 and pitch 1. Client-side calls with null excluded player do not locally play through
`ClientLevel`, so the authoritative server sound packet is the observable path.

**Interaction, payment and selection:**

Empty-hand block interaction always returns success. On the server, a matching block entity asks
the player to open its menu and then awards `interact_with_beacon`, even if a lock rejects menu
creation. Lock rejection sends the ordinary locked-container notification at block center.

The menu contains payment slot 0, player main slots 1..27 and hotbar slots 28..36. Payment accepts
only the reloadable `minecraft:beacon_payment_items` tag and has maximum size one. The locked tag
contains netherite ingot, emerald, diamond, gold ingot and iron ingot. Exact quick-move routes and
close-time payment return remain owned by `ITM-CONTAINER-MOVE-001` and
`ITM-CONTAINER-CLOSE-001`.

The three synchronized menu data values are level, primary and secondary. Effect data encodes null
as zero and otherwise the built-in mob-effect ID plus one; this is distinct from the packet's two
optional configured-holder raw IDs. Setting primary on the server plays power-select before the
assignment whenever the published beam list is nonempty, even at level zero and even if the value
does not change. Both setters filter decoded values to the six valid powers.

Control admission first requires a current, still-valid `BeaconMenu`. `updateEffects` then checks
only that payment slot zero is nonempty; it does not recheck tag membership, maximum size or count.
It validates current level `L`: nonnull secondary requires `L>=4`; both powers' required levels
must be at most `L`; primary must require less than four; and a tier-1..3 secondary must equal
primary. Regeneration is therefore secondary-only. Both absent is accepted with any nonnegative
level, and level four accepts absent primary plus regeneration. Absent primary plus a tier-1..3
secondary reaches the locked null `equals` fault.

Success writes primary then secondary, removes exactly one payment and calls
`Level.blockEntityChanged`, marking the loaded chunk unsaved. It does not send a block update or
immediate block-entity-data packet. A false result causes the handler's generic disconnect; wrong
or invalid current menus are ignored as specified by the protocol/control owners.

**Persistence and synchronization:**

Full save and the ordinary update tag write valid nonnull powers as `primary_effect` and
`secondary_effect` names, always write integer `Levels`, optionally write `CustomName`, and encode
the lock through its shared `lock` field. Loading accepts only registered members of the six-power
set; missing, malformed or other effects become null. It restores custom name and lock but never
reads `Levels`.

Consequently a loaded server or client starts with level zero, empty beam lists and a fresh
`minY-1` cursor regardless of the saved `Levels` value. The first admitted tick starts a new scan;
the first completed scan publishes it; only a later eligible 80-tick boundary with that published
list can reconstruct the pyramid level. Powers remain available throughout that delay but cannot
apply while level is zero.

The update packet is the ordinary block-entity-data packet for protocol type 15. Although the
update tag includes `Levels`, receiving it does not install that value. Menu data remains the
authoritative live level/power projection for an open menu. Ordinary payment success dirties for
later save but does not itself request a block-entity update.

Breaking loot copies custom name but not lock or selected powers. Implicit component application
and collection can carry custom name and a nondefault lock through generic block-entity component
paths; `removeComponentsFromTag` removes `CustomName` and `lock` from the residual tag.

**Client rendering:**

The client runs the same incremental scan and 80-tick base derivation independently; server beam
sections and levels are not synchronized into its runtime fields. `getBeamSections` returns empty
whenever local level is zero, otherwise the locally published list. The ordinary block model draws
the full glass shell, obsidian base and inner beacon, and the item uses that same block model.

The block-entity renderer copies each published color/height section. Earlier sections use their
stored height; the final section is forced to height 2048. It draws an opaque rotating core with
radius `0.2 × s` and a translucent glow with radius `0.25 × s`, where scoping forces `s=1` and
otherwise `s=max(1,horizontalCameraDistance/96)`. Animation time is
`floorMod(gameTime,40)+partialTick`. Rendering is allowed off-screen, ignores vertical distance,
and uses a horizontal view distance of the effective chunk render distance times 16.

**Branches and aborts:**

Fresh/reloaded and mid-cycle cursors; scan heights below/at/above ten cells; beacon, all 32 stained
glass block/pane colors, air, bedrock, dampening 14/15 and tinted-glass obstruction; equal/different
adjacent colors; empty/partial/complete layers 1..4; every 80-tick phase; obstructed-level residue;
primary null/all tiers/invalid, secondary null/same/regeneration/invalid and the null-equality fault;
empty, ordinary and forged payment; lock success/failure; save/update tags with every malformed
field; tall-column transition loss; client levels, sections, scoping and distance scales.

**Constants and randomness:**

State ID `9980`; block-entity protocol ID `15`; scan budget `10` cells/tick; base/effect period
`80`; maximum levels `4`; base side lengths `3,5,7,9`; base cell total `164`; radii
`20,30,40,50`; durations `220,260,300,340`; primary amplifier `0/1`; activation criterion box
X/Z ±10 and Y -9/+5; final render height `2048`; renderer radii `0.2/0.25`; distance scale divisor
`96`; animation modulus `40`; light `15`; strength `3`; payment maximum `1`.

The beacon runtime consumes no gameplay RNG. Generic sound transport chooses its ordinary sound
seed, but scan, layer validation, effects, menu validation, payment and renderer geometry have no
branch-local random draw.

**Side effects:**

Published/in-progress beam and level mutation; player effects; activate, deactivate, ambient and
power-select sounds; construct-beacon criterion; menu open/stat/locked notification; payment
consumption or close return; chunk dirtiness; block-entity-data and menu-data projection; colored
beam rendering. The beacon creates no scheduled block tick, random tick, redstone output,
comparator output, particle emission, entity damage or game event.

**Gates:**

Matching block-entity type and admitted ticker; incremental unobstructed beam; nonempty published
beam at the 80-tick boundary; complete tag-selected base layers; nonnull valid primary for effects;
matching current valid menu, nonempty payment, level/tier/equality validation for selection; generic
lock and menu-distance rules. Difficulty and game rules do not gate scanning, levels or effects.

**Boundary cases and quirks:**

Beam publication can lag a tall colored column by multiple ticks. The 80-tick refresh reads the
previous published list because publication occurs later in the tick. Bedrock passes a fully
opaque scan while tinted glass blocks it. The first filter always begins a separate section.
Obstruction suppresses output without clearing level. Level-transition sound/criterion can be lost
when incremental completion does not coincide with the level-changing 80-tick tick. Saved
`Levels` is ignored on load and in update tags. Submit-time payment validation is deliberately
weaker than slot insertion. Power-select depends on beam-list nonemptiness rather than level or
value change. Selection dirties but does not immediately project block-entity data. The final beam
section ignores its stored height and renders to 2048.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`. Anchors:
`net.minecraft.world.level.block.BeaconBlock#getTicker`,
`net.minecraft.world.level.block.BeaconBlock#useWithoutItem`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#tick`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#updateBase`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#validateEffects`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#applyEffects`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#loadAdditional`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#saveAdditional`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#getUpdateTag`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#createMenu`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#setRemoved`,
`net.minecraft.world.inventory.BeaconMenu#updateEffects`,
`net.minecraft.world.inventory.BeaconMenu#quickMoveStack`,
`net.minecraft.world.inventory.BeaconMenu#removed`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetBeaconPacket`,
`net.minecraft.client.renderer.blockentity.BeaconRenderer#extract`,
`net.minecraft.client.renderer.blockentity.BeaconRenderer#submit`; locked block, block-entity and
item reports; bundled base/payment tags, loot/advancement/model assets; existing
`PROTO-PLAY-SERVERBOUND-ANVIL-BEACON-001`; `EXP-BLK-024`.

**Test vectors:**

Drive the ten-cell cursor through every transparent/color/opaque branch and scan height; mutate
base and obstruction before/at/after 80-tick boundaries, including a tall colored column. Exhaust
all 164 base positions, mixed tag members, effect boxes/durations/amplifiers, activation criterion
audience and sound order. Cross menu open/lock/payment/selection/control/close races and every
valid, rejected or faulting tuple. Save/load/update all fields around dirty and first-reconstruction
boundaries, then compare independent client sections and exact renderer geometry. Run
`EXP-BLK-024` as the executable matrix.
