# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-COMPARATOR-001` — Comparators cache an analog result, then expose it through a two-tick directional transaction

**Parent:** `RED-001`, `RED-003`, `SIM-003`, `BLK-003`, `PLY-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked `ComparatorBlock`, `DiodeBlock`, `ComparatorBlockEntity`, signal-query,
experimental-orientation and sound-dispatch source fixes every input branch, calculation, schedule
priority, RNG draw, state/cache write, persistence field, interaction, output face and notification;
`EXP-RED-006` is conformance-only.

**Applies when:**

A comparator is placed, receives a neighbor change or due scheduled block tick, is used
empty-handed, loads/saves its block entity, receives a block event, or is queried for
ordinary/direct signal. Generic scheduler admission, chunk activity and update-queue execution
retain `SIM-SCHEDULE-001` and `BLK-UPDATE-001`.

**Authoritative state:**

Let stored horizontal `F` point toward the rear input block. The block state owns `F`,
`mode ∈ {compare,subtract}` and `powered`; its compatible block entity owns signed integer
`OutputSignal`, default zero. The level supplies live ordinary/direct signals, block analog
interfaces, redstone-wire state, entities, scheduled-tick membership, support and write outcomes. No
subtype ticker or update packet exists.

**Rear and side sampling:**

Start rear input `I` with the level signal at `p+F` queried in direction `F`; when below 15, take
the maximum with that block's stored redstone-wire power. If the immediate block advertises analog
output, replace `I`—even a prior 15—with its analog result queried from direction `F.opposite`.
Otherwise, only when `I<15` and the immediate block is a redstone conductor, inspect `p+2F`: query
item frames intersecting that unit AABB whose attachment direction equals `F`, accepting a frame
only when exactly one matches; independently query the second block's analog output. Replace `I`
with the maximum available frame/block value when either exists, rather than taking a maximum with
the prior conductor signal. An empty sole frame contributes zero; a nonempty one contributes
`rotation mod 8 + 1`; zero or multiple matching frames contribute no frame candidate.

Side input `S` is the maximum control input at `p+F.clockwise` and `p+F.counterClockwise`, queried
in those respective directions. Each control input is 15 for a redstone block, stored wire power for
dust, direct signal for another signal source, and zero otherwise; side conductors do not relay
power. Calculation short-circuits to output `O=0` when `I=0`, without sampling sides. Otherwise it
samples `S`: `S>I` gives zero; subtract mode gives `I-S`; compare mode gives `I`. The independent
powered predicate also short-circuits false at `I=0`, then is true for `I>S`, or for `I=S` only in
compare mode. It resamples rear/sides each time called.

**Transition and ordering:**

A neighbor check first returns when this comparator is already in the scheduler's current-tick run
set. It calculates `O`, reads cached old output (zero for a missing/wrong block entity), and offers
a delay-2 tick when `O!=old`; only when equal does it additionally resample the powered predicate
and compare that with state `powered`. Priority is `HIGH` when the block at output position
`p+F.opposite` is any diode whose stored facing differs from `F.opposite`, otherwise `NORMAL`.
Generic schedule deduplication owns competing offers. Placement separately schedules delay 1 at
default priority when the initial powered predicate is true.

A due tick calls refresh with its live/captured state and ignores the tick RNG argument. Refresh
calculates `O`, then reads old zero; a compatible block entity receives `O` immediately, but its
setter neither validates the integer nor calls `setChanged`. If `old!=O` **or mode is compare**,
refresh resamples the powered predicate, optionally offers a flags-`2` state with `powered`
corrected, ignores the result, then always notifies output position `p+F.opposite`: first
`neighborChanged`, then `updateNeighborsAtExceptFromFacing` excluding `F`. With redstone experiments
disabled, both receive null orientation. When enabled, orientation initialization consumes
`level.random.nextInt(48)`, then overwrites the sampled orientation to deterministic side bias left,
up `UP`, front `F.opposite` before both calls. Subtract mode with unchanged output performs none of
those powered-state or neighbor operations, even if a loaded/mutated state is inconsistent. Compare
mode always performs them. A missing compatible block entity can power the state but exposes cached
output zero and keeps scheduling whenever calculated output is nonzero.

**Use, placement and support:**

Default state is north/compare/unpowered; placement stores the opposite of the placer's horizontal
direction. Empty-hand use by a player without build ability returns `PASS`. Otherwise both sides
cycle the captured mode, consume one sound-seed long and play comparator click at block
center/category blocks, volume `0.3`, pitch `0.55` when the intended mode is subtract and `0.5` when
compare; the client plays for its local excluded player while the server broadcasts to everyone in
range except that player. It offers the intended state with flags `2`, ignores the result, rereads
only whether the live block identity is still this comparator, and if so refreshes using the
**intended** state rather than the reread state. Thus sound and refresh survive a rejected first
write; a second powered-state offer can install the intended mode, while an unchanged subtract
output can leave live mode and cached mode calculation divergent. Return is `SUCCESS`.

The shape is a full `16×16` footprint through height `2/16`; survival requires rigid support on the
upper face below. A downward shape update without support returns air. A neighbor callback that
finds lost support captures the block entity for ordinary resource dropping, removes with
moving=false, then updates all six adjacent positions. Placement notifies the output side; removal
does so unless piston-moved. Instabreak, stone sound and piston reaction `DESTROY` are fixed block
properties.

**Signal, persistence and events:**

When unpowered, ordinary/direct output is zero. When powered, both return the raw cached signed
integer only for a query direction equal to `F`; all other directions return zero. Normal
calculations produce `0..15`, but load reads unchecked `OutputSignal` (default zero), so malformed
values can escape unclamped through both signal APIs. Save always writes that key; setter changes do
not explicitly dirty the chunk. Block events call the block superclass, then forward to any block
entity and return only the block entity's result.

**Branches and aborts:**

Current-tick schedule membership; rear zero/15; immediate analog; conductor; zero/one/multiple
matching frames; second analog; side source kind; compare/subtract and `I` versus `S`;
compatible/missing BE; old/new equal; powered consistent/inconsistent; priority diode; permission
and client/server; first/second write success/rejection/replacement; support and piston removal; raw
loaded integer.

**Constants and randomness:**

Delays `2` neighbor/`1` placement; priorities normal/high; output normal range `0..15`; frame range
`0..8`; flags `2`; shape height `2/16`; sound volume `0.3`, pitches `0.5/0.55`. Calculation and the
tick RNG argument consume no draw. Every executed output notification consumes no level RNG under
the default pack, but consumes one bounded `nextInt(48)` when redstone experiments are enabled even
though later overrides make the final orientation deterministic. Each permitted side invocation
consumes one long from the distinct level sound-seed generator.

**Side effects:**

Scheduled ticks, raw BE cache mutation/save, optional powered/mode state writes, ordered output-side
updates, placement/removal/all-adjacent support updates, click sound packets/local playback, drops
and forwarded block events.

**Gates:**

Generic chunk/freeze/scheduler gates; redstone-experiments feature flag; rigid support; current
block identity; compatible BE; player build ability; state-write admission; analog/source interfaces
and exact frame cardinality.

**Boundary cases and quirks:**

Analog output replaces rather than maximizes prior rear signal. An output mismatch short-circuits
the neighbor-time powered resample. Refresh stores output before its independent powered resample,
never explicitly dirties the BE, compare mode always notifies, unchanged subtract mode never
corrects powered, and manual refresh trusts intended mode after only a live block-identity reread.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.ComparatorBlock#calculateOutputSignal`, `#shouldTurnOn`,
`#getInputSignal`, `#getItemFrame`, `#checkTickOnNeighbor`, `#refreshOutputState`,
`#useWithoutItem`, `net.minecraft.world.level.block.DiodeBlock#getAlternateSignal`, `#getSignal`,
`#getDirectSignal`, `#updateNeighborsInFront`,
`net.minecraft.world.level.SignalGetter#getControlInputSignal`,
`net.minecraft.world.level.redstone.ExperimentalRedstoneUtils#initialOrientation`,
`net.minecraft.world.level.redstone.Orientation#random`,
`net.minecraft.world.level.block.entity.ComparatorBlockEntity`,
`net.minecraft.world.entity.decoration.ItemFrame#getAnalogOutput`,
`net.minecraft.world.level.Level#playSound`,
`net.minecraft.client.multiplayer.ClientLevel#playSeededSound`, and
`net.minecraft.server.level.ServerLevel#playSeededSound`.

**Test vectors:**

Cross all 16 states with rear ordinary/wire/analog values `0,1,14,15`; immediate conductor and
second analog; empty/rotated frame with cardinality `0,1,2`; all side source kinds and
`S=I-1/I/I+1`; current-tick and downstream-diode priorities; default/experimental update orientation
and draw counts; compatible/missing BE and raw `INT_MIN/-1/0/15/16/INT_MAX`; compare/subtract
equal-output inconsistency; client/server permission; rejected/replacing writes; support loss and
block events. Assert query order/count, schedules, cache dirtiness, flags, notifications,
gameplay/sound RNG, recipient, and face signals with `EXP-RED-006`.
