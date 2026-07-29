# Redstone Signal, Comparator, and Daylight Runtime

`G01-P5-S013` implements the three `SourceSpecified` slices owned primarily by `RED-001`:
`RED-SIGNAL-UPDATE-001`, `RED-COMPARATOR-RUNTIME-001`, and
`RED-DAYLIGHT-DETECTOR-RUNTIME-001`. The `ferrite-gameplay::redstone` module owns
protocol-neutral decisions; Region adapters supply ordered world observations and commit the
resulting writes, schedules, notifications, sounds, and block events.

## Signal and wire ownership

Signal sampling preserves Java 26.2's fixed `Direction::ALL` order and exits as soon as strength
15 is found. A conductor combines its ordinary signal with the strongest direct signal entering
it. Control input keeps the redstone-block, wire, diode-only, and general signal-source cases
separate.

Default dust recomputation disables the wire's own signal contribution, samples the best block
signal, restores its contribution, and then walks the four horizontal routes. A conductor route
can rise only when the current block's top is open; a non-conductor route can descend. Incoming
dust loses one level, while a block signal of 15 exits before horizontal work. Power writes are
guarded by exact-state identity, use flag 2, and dispatch the seven-position unordered neighbor
set only after an accepted change.

Connection classification distinguishes `Up`, `Side`, and `None`, including repeater-axis,
observer-facing, generic signal-source, and dust-below routing. Placement normalizes an isolated
dot to a cross; player use toggles only an isolated dot/cross and respects build permission.
Placement, removal, support loss, vertical neighbors, and horizontal corners retain their audited
order. Experimental dust is selected only by the redstone-experiments gate and suppresses the
experimental evaluator's self-originated callback.

## Comparator transaction

Rear input first samples the immediate signal, conditionally samples wire below strength 15, and
then lets an immediate analog source replace that value. Only a conductor permits the second
position lookup. Exactly one correctly attached item frame contributes `rotation % 8 + 1` when it
contains an item; an empty frame contributes zero. A second-position analog source and the unique
frame candidate form the replacement input rather than being merged with the first position.

Compare and subtract calculations preserve the rear-zero short circuit and side maximum.
Neighbor checks refuse duplicate current-tick scheduling, compare calculated output against a
missing-as-zero cache, resample powered state only when output is unchanged, use delay 2, and
select high priority for the audited downstream-diode orientation. Initially powered placement
uses delay 1.

Refresh calculates output, reads and conditionally writes the compatible block entity cache,
resamples powered state, offers a flag-2 state write, and notifies the output in fixed order.
Compare mode always notifies; unchanged subtract mode does not. The block-entity setter does not
mark itself dirty. Experimental notification consumes one bounded-48 orientation draw. Powered
face queries expose the raw signed cached output only in the facing direction.

Use is permission-gated. Both sides derive the intended mode and consume the seeded click-sound
operation, but only the server offers the flag-2 state write and refreshes a still-live intended
state. Support loss captures the block entity for drops before removal and updates all six
neighbors. Non-piston removal notifies the output; loading defaults `OutputSignal` to zero; block
events return the compatible block entity result.

## Daylight detector transaction

The ticker exists only on a server dimension with skylight and admits every twentieth game tick.
Power starts from sky brightness minus sky darken. Inverted mode computes `15 - brightness`.
Non-inverted positive brightness converts the sun angle to float radians, smooths 20 percent
toward zero or two-pi, uses the Java 26.2 trigonometric lookup semantics, rounds as Java float
rounding does, and clamps to `0..15`. Nonpositive and inverted paths skip the angle work.

Periodic recomputation writes only changed power with flag 3. Player use is permission-gated:
the client returns success without prediction, while the server offers the inversion with flag 2,
emits the block-change event, and recomputes the intended state, optionally producing the second
flag-3 power write. Ordinary signal is the stored power on every face; direct signal is zero. Its
empty block entity has no persistent data, update packet, or renderer.

## Region determinism

These functions never consult wall-clock time, process identity, mailbox arrival order, or global
mutable state. Region integration must supply observations in logical tick and audited traversal
order. Cross-Region signal effects enter the normal boundary envelope and cannot reorder the
fixed local probes or comparator refresh stages. Random consumption is explicit so replay and
topology-conformance adapters can retain identical stream advancement.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/redstone/red_001.rs`. Its 16 tests cover directional
aggregation and early exit, dust routes and lifecycle, comparator sampling/calculation/scheduling/
refresh/use/persistence, and daylight ticking/formula/use/signal behavior.
