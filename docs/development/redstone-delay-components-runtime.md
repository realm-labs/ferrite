# Redstone Delay Components Runtime

`G01-P5-S014` implements the `SourceSpecified` `RED-DELAY-COMPONENTS-001` slice. The
`ferrite-gameplay::redstone::delay` module owns protocol-neutral diode, repeater, observer, and
redstone-torch decisions. Region adapters supply ordered block/signal/schedule observations and
commit state writes, scheduled ticks, neighbor notifications, level events, and presentation.

## Shared diode transaction

A diode samples the block in its facing direction through ordinary/conductor signal semantics and
then, unless that value is already 15, takes the maximum with dust power from the same block.
Unlocked neighbor checks compare current power with current input and reject a duplicate tick due
in the same game tick. A changed edge schedules after the component delay:

- an output-side diode not facing back selects `ExtremelyHigh`;
- a currently powered falling edge selects `VeryHigh`;
- every other edge selects `High`.

The due callback resamples both lock and input. A powered diode turns off only when input is absent.
An unpowered diode always turns on; if its input disappeared before execution, it also schedules a
`VeryHigh` fall after the same delay. State writes use flag 2. Placement with live input schedules
one default-priority tick after one game tick.

Placement and non-piston removal notify the output-side block, first through `neighborChanged` and
then through every face except the diode-facing side. Experimental redstone consumes one
bounded-48 orientation draw and fixes front to the output direction and up to `Up`; default mode
has no orientation. Lost rigid support drops resources, removes with `moving=false`, and updates
all six adjacent positions.

## Repeater

The closed delay type maps properties 1, 2, 3, and 4 to 2, 4, 6, and 8 game ticks and cycles back
from 4 to 1. The two perpendicular inputs use diode-only control sampling, so non-diode sources
including a redstone block and dust cannot lock it. Placement faces opposite the player's
horizontal direction and captures the current side lock.

Server shape updates resample `LOCKED` whenever the changed direction's axis differs from the
repeater facing axis; loss of rigid support below becomes Air first. Build-authorized use cycles
delay even while locked and offers the flag-3 write on both client and server. A powered repeater
publishes 15 only in its facing output query direction.

## Observer

An observer starts work only on the server, only for its watched `FACING` direction, only while
unpowered, and only when no tick is already scheduled. The first due tick writes powered with flag
2 and schedules another tick after two game ticks. The second writes unpowered, also with flag 2.
Every due edge then notifies the position opposite `FACING` in the same direct-then-except-facing
order as a diode.

Replacing a different block with a server-side powered observer that has no pending tick clears it
with flag 18 and notifies. Same-block placement, client placement, and a pending tick leave it
alone. Removing powered state with a pending tick emits the unpowered output notification.
Experimental notifications consume one bounded-48 draw, fix front to the output direction, and do
not fix up. Its default is south-facing and unpowered; placement retains the nearest looking
direction. Powered face signal is 15 only for the facing query direction.

## Redstone torches

Floor input samples the block below toward `Down`. Wall input samples its supporting block in the
direction opposite wall-torch facing. A neighbor callback schedules exactly one tick after two
when `LIT == hasNeighborSignal`; equality with a tick already due does not duplicate work.

Each Region-owned level keeps one chronological toggle history shared by all torch positions. A
due tick removes entries only when their age is greater than 60, so age 60 remains and age 61 is
purged. A lit torch with input offers the flag-3 unlit write and appends its position/time. The
eighth retained entry for that position emits level event 1502 and schedules the current live block
at that position after 160 ticks. An unlit torch without input relights only below the threshold.
A restart tick with retained history can remain dark without scheduling another retry; later
ordinary neighbor activity may enqueue one.

Both torch forms default lit; wall torch defaults north-facing. Floor weak signal excludes `Up`;
wall weak signal excludes its `FACING`; direct signal is the weak result only toward `Down`.
Support-facing shape loss becomes Air. Placement and non-piston removal update all six adjacent
positions. Experimental mode consumes one bounded-48 orientation draw per notification
transaction, fixes up to `Up`, and for a wall torch also fixes front opposite its facing.

## Region determinism

Toggle history belongs to the authoritative level/Region boundary and is not process-global.
Schedules carry logical time and priority; mailbox arrival order cannot replace tick ordering.
Every experimental orientation draw is explicit and must use the Region's named deterministic
gameplay stream. Cross-Region notifications retain the source transaction order in boundary
envelopes.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/redstone/red_003.rs`. Its 13 tests cover input aggregation,
all diode priorities and due edges, support/notification behavior, all repeater delays and lock/use
paths, the complete observer pulse/replacement/removal lifecycle, torch support and signal faces,
age 60/61 history, the seventh/eighth toggle boundary, restart behavior, and experimental
orientation consumption.
