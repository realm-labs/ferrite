# String and Tripwire Runtime

`G01-P6-S011` implements `ITM-STRING-RUNTIME-001`, the audited String item slice primarily owned
by `SIM-003`. It closes the item identity and data joins together with the Tripwire behavior that
produces and consumes String.

## Responsibility boundary

`item::runtime::string` owns:

- the `minecraft:string` identity, stack limit, and imported `string-runtime` family closure;
- the 17 direct acquisition tables and the audited looting bonus arithmetic;
- all nine String-consuming recipe joins and the distinct nine-recipe unlock set;
- Fisherman and Fletcher purchase offers, fishing-junk denominators, and the one decoded
  structure placement.

Loot-table evaluation, recipe matching, merchant offer selection, fishing category selection, and
structure placement remain with their generic owners. This module supplies exact, ordered inputs
for those systems and does not duplicate their execution.

`block::tripwire` owns:

- the seven-property wire state and its 128-state catalog range;
- horizontal placement connectivity, rotation, mirrors, and attached/unattached contact shapes;
- the source-side South/West scan and the two-ended Hook line transaction;
- server-only entity contact, 10-tick rescans, release scheduling, sounds, and redstone output;
- the shears-before-removal disarm contract without suppressing the String drop.

World mutation, scheduled-tick storage, collision queries, event dispatch, loot execution, and
Region command delivery remain at their existing boundaries. Tripwire functions return
deterministic transition descriptions for those owners to apply.

## Line transaction

A Hook scans at most 41 positions in its facing direction. An opposite-facing Hook is valid only
at distance 2 through 41, allowing one through forty intervening wires. Every intervening wire
must remain armed before the line can attach, and only an attached line may become powered.
Removing or disarming the changed wire forces the line to detach and suppresses power.

When attachment changes, every still-present intervening wire is rewritten with the new attached
state. The opposite Hook is written when found; the origin Hook is omitted only when it is the
removed block. Entity contact is authoritative only on the server and is ignored while the wire
is already powered or already has a pending tick. A new press powers the wire and schedules a
rescan after ten ticks; a live press repeats that delay, while release clears power and requests a
zero-delay Hook recalculation.

## Determinism and Region ownership

The owning Region resolves the encountered cell sequence and triggering-entity predicate. Runtime
functions use only those ordered inputs and explicit random draws, so topology and process
identity cannot affect a result. Hook sound selection preserves source priority, and the detach
pitch accepts the already-consumed random value rather than drawing from ambient state.

## Validation

`crates/ferrite-gameplay/tests/slices/items/sim_003.rs` verifies the imported family, exact item
joins, state transforms, shapes, source scan, line-length boundaries, arming and power
suppression, attachment rewrites, contact scheduling, Hook signals and sounds, and shears
disarming.
