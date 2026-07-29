# BLK-003 Update and Runtime Partition

`G01-P5-S003` implements the six `SourceSpecified` slices grouped under `BLK-003`. The code remains
protocol-neutral and operates on project-owned semantic inputs so the same mechanics can execute in
local and Lattice-backed Region runners.

## Block update kernel

`ferrite-gameplay::block::update` separates four responsibilities:

- `flags` defines all ten independent 26.2 update bits and the exact `260`, `3`, `11`, and `816`
  masks;
- `lifecycle` orders storage, lighting/heightmap, block-entity removal/retention, removal and
  placement callbacks, publication, ordinary/comparator/shape work, and POI replacement;
- `neighbor` executes work depth-first by callback layer and FIFO within one added layer while
  counting submitted work items against the independent chain cap;
- `event` provides insertion-order deduplication, same-drain callback requeue, current-type checks,
  and inactive isolation;
- `ticker` preserves wrapper position on rebind, prunes invalid wrappers even while frozen, and
  defers block entities created during iteration until the next phase entry.

The generic shape depth remains a signed caller budget with default `512`; the neighbor collector
default remains `1,000,000` and accepts a negative unlimited configuration. Region mutation and
cross-boundary delivery are composed in `G01-P5-B1`; this partition supplies their exact local
ordering contract.

## Area commands

`block::command_area` performs inclusive signed-long volume precharge before selection, locks clone
overlap/load/debug validation order, and produces deterministic source-clear, barrier, category,
block-entity, explicit-neighbor, and whole-box scheduled-tick phases. Fill decisions distinguish
replace, outline, hollow, and destroy and count destroy-or-place at most once. Fillbiome rounds
every endpoint down to a multiple of four, preflights all FULL chunks, counts matching quart cells
including unchanged targets, and dirties/resends every collected chunk even at zero matches.

## Ticking block entities

The three dynamic block-entity modules expose their source-owned state machines:

- `block::spawner` freezes before the live rule when no eligible player exists, retains arbitrary
  signed-short configuration, distinguishes retry-only from delay-reset attempt failures, and
  closes ordinary/trial spawn-egg edit ordering;
- `block::trial_spawner` implements six states, phased scans, strict tracking, pre-scan target
  snapshots, retry timing, float reward boundaries, fixed-table ejection counts, and ominous item
  selection-before-timer behavior;
- `block::vault` implements strict `4`/`4.5` hysteresis helpers, key-validation precedence,
  15-tick failure buffering, 14/20-tick transitions, reverse-list ejection, FIFO rewarded eviction,
  and the load setter's intentional omission of the decoded total.

Entity construction, loot evaluation, player effects, and client presentation retain their
separate generated owners. These modules fix the exact gates, parameters, mutation order, counters,
and RNG call positions supplied to those joins.

## Vines

`block::vine` represents all 32 five-face states, including the invalid all-false full-outline
fallback. It implements ordered placement, direct/inherited support repair, rotation/mirror,
four-versus-five local density admission, horizontal branch priority and strict `0.05` fallback,
and the opposite four-Boolean semantics of upward and downward copying. Every admitted candidate
uses flags `2`; rejected writes have no retry candidate.

## Validation

The test owner `crates/ferrite-gameplay/tests/slices/blocks/blk_003.rs` locks the six slice-to-module
owners and exercises flag isolation, callback aborts, neighbor/event/ticker ordering, command
precharge and partial phases, spawner retry/reset gates, trial/vault time boundaries, all vine
states, density edges, and branch-local RNG cardinality.
