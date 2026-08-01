# Simulation Region Integration (Historical Goal 01 Phase 5)

`G01-P5-B1` composes the audited simulation, block, environment, and redstone implementations at
the server-runtime boundary. Gameplay and simulation crates retain source behavior; this layer owns
Region authority, cross-Region admission, durable continuity, bounded queues, and Java 26.2
projection.

## Ownership boundary

The active runtime is `ferrite-server-runtime::simulation`, split by responsibility:

| Module | Responsibility |
|---|---|
| `boundary` | Typed mechanic transactions, endpoint and owner validation, stable operation order |
| `budget` | Atomic capacity reservations and releases for all persistent and transient queues |
| `continuity` | Versioned scheduled-work, runtime-stream, and applied-boundary snapshot records |
| `projection` | Bounded final-state aggregation into Java block and section update packets |
| `runtime` | Region generation fencing, atomic application, queue draining, commit and handoff rules |

Protocol and persistence types do not enter `ferrite-gameplay` or `ferrite-simulation`. Conversely,
the protocol adapter never becomes world authority: it projects only committed
`BlockStateId` values supplied by the Region runtime.

## Boundary transaction

A `MechanicBoundaryTransaction` names its logical tick, source and target Region keys, both
activation generations, source sequence, mechanic, ordered block mutations, and ordered scheduled
work. Construction rejects:

- same-Region or world/dimension/mapping-incompatible endpoints;
- positions outside the target Region;
- oversized mutation or schedule sets;
- duplicate operation orders, mutation positions, or scheduled identities at one position.

Application is a fail-closed transaction:

1. Verify target Region, target generation, tick, loaded chunks, expected states, scheduled
   containers, receipt capacity, projection capacity, and every queue budget.
2. Apply all prospective block writes to cloned chunk columns. A state mismatch or chunk revision
   failure therefore leaves authoritative storage unchanged.
3. Reserve boundary, scheduled-work, mechanic-effect, and projection capacity in one atomic budget
   operation.
4. Install the staged chunks, scheduled work, ordered deferred effects, final-state projection, and
   durable source-generation-sequence receipt.
5. Release the short-lived boundary admission slot. Persistent reservations remain until their
   corresponding work is drained.

Piston and explosion writes can therefore span several target blocks without exposing a partial
shape. A full target queue returns backpressure before mutation; the existing Region transport
retains the source batch. A repeated receipt returns `AlreadyApplied`, including after a handoff.
Target-generation and tick mismatches are rejected before receipt or voxel changes.

## Queue accounting

The runtime configures independent nonzero capacities for scheduled blocks, scheduled fluids,
boundary transactions, immediate neighbors, fluids, redstone, lighting, and projection positions.
Multi-queue reservations and multi-queue releases preflight every counter before changing any
counter. Arithmetic overflow, missing configuration, over-release, and zero capacities fail
closed.

Scheduled entries hold their reservation until execution. Deferred mechanic effects retain their
reservation until the responsible service drains them. Projection counts unique positions rather than
events, so later writes replace an earlier pending state without consuming another slot.

## Continuity and handoff

Commit continuity uses bounded binary records within `RegionCommitSnapshot`:

- one runtime record stores the next sub-tick order, signed random-position stream, gameplay RNG
  algorithm, and all four gameplay RNG state words;
- one scheduled-work record per registered block or fluid chunk stores resource identity, position,
  relative delay, and priority;
- one applied-boundary record per retained receipt stores source Region, source activation
  generation, and source sequence.

Record decoding rejects bad magic, truncation, trailing bytes, unknown RNG algorithms, zero RNG
state, invalid resource identities, duplicate records, out-of-chunk ticks, invalid generations,
and configured snapshot bounds. Restore unpacks relative delays against the new authority's load
time, restores sub-tick and both random streams, re-establishes scheduled queue usage, and installs
receipts before accepting boundary replay.

Only a commit-ready state may be captured. Deferred mechanic effects and client projection are
transient outputs and must be drained first; an attempted snapshot reports their exact
remaining counts. This prevents a graceful handoff from silently dropping pre-commit work.

## Client projection

Committed block updates are aggregated in canonical `BlockPos` order. One position in a section
emits `BlockUpdate`; multiple positions emit one `SectionBlocksUpdate` with Java raw block-state
IDs resolved by the active terrain registry map. An unmapped state leaves the complete pending
buffer and its budget reservations intact, allowing retry after registry convergence.

## Verification

`crates/ferrite-server-runtime/tests/simulation_region_integration.rs` locks atomic multi-block
application, receipt idempotency, stale-generation fencing, capacity and expected-state rollback,
effect ordering, projection retry, snapshot encoding, graceful handoff, relative scheduled-delay
recovery, sub-tick continuity, and both random-stream continuations.

This filename is retained because completed Goal 01 ledgers link to it. The active module, type,
diagnostic, and test-target names are responsibility-owned. Writers use versioned
`ferrite:simulation/*_v1` continuity identities. The bounded Goal 03 compatibility path reads
valid legacy `ferrite:phase5/*_v1` records, rewrites them atomically to the current identities, and
rejects mixed or unsupported generations.
