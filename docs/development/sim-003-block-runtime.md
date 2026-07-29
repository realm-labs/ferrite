# SIM-003 Block Runtime

`G01-P5-S006` implements the 17 `SourceSpecified` block slices whose primary simulation owner is
`SIM-003`. The implementation is a protocol-neutral deterministic semantics layer in
`ferrite-gameplay`; the Phase 5 integration batches remain responsible for binding these
decisions to Region-owned state, ECS entities, scheduled queues, registry snapshots, persistence,
and client projection.

## Module ownership

| Module | Audited responsibility |
|---|---|
| `block::beacon` | Ten-cell beam scan, atomic publication, obstruction, base/effect constants, and menu selection boundaries |
| `block::brushable` | Shared brush cooldown, dust stages, delayed regression, first-result loot, completion split, and falling-data loss |
| `block::command_block` | Power/automatic scheduling, captured conditions, live dispatch rule, same-tick guard, chain decisions, and minecart throttle |
| `block::conduit` | Ordered water/frame admission, effect radius, target retention/reselection, damage, ambient clocks, and particle draw counts |
| `block::coral` | Coral block/plant IDs, ordered water checks, support precedence, drying, facing retention, water-tick duplication, and loot |
| `block::lectern` | Content/state independence, insertion, page clamping, pulse deduplication, signals, and captured-state removal |
| `block::nether` | Roots/sprouts support and loot, stem axes/stripping, wart growth/loot, wart-block joins, composting, and vegetation weights |
| `block::sculk_sensor` | State IDs, frequency map, candidate ordering, delayed travel, chunk gate, activation, directional signal, and resonance |
| `block::sign` | Side selection, editor lease, applicators, click precedence, edit updates, chaining, and render light/outline boundaries |
| `block::skull` | All 280 state IDs, neighbor power, durable/transient separation, animation, note sounds, and wither-pattern consumption |
| `block::test_block` | Mode/state divergence, edge latches, operator edit order, reset, and accept/fail/log scan precedence |
| `item::honeycomb` | Direct-shrink wax transaction and non-single copper-chest companion effects |

No module owns packet framing, command execution internals, generic loot/crafting, world generation,
mob AI, renderer execution, or distributed scheduling. Those owners consume the semantic results
without reimplementing their conditions.

## Determinism boundaries

- Callers provide already ordered observations and explicit random results. Runtime functions do
  not acquire ambient randomness.
- Failed authoritative writes do not change the semantic outcome where vanilla ignores the
  Boolean result; the integration owner must still emit the returned schedules, events, updates,
  or success result in order.
- Reloadable tag membership and registry lookup remain immutable inputs from the active content
  snapshot.
- Same-tick choices use explicit game times, strict comparison boundaries, and stable caller order.
- Server state, transient block-entity state, saved data, and client projection are not collapsed
  into a single truth when the audited source permits divergence.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/blocks/sim_003.rs`. It exercises all 17 slice owners and the
cross-owned Honeycomb leaf, including locked identities, equality boundaries, ordering,
write-failure residue, state/entity divergence, persistence defaults, and exact signal/effect
constants.
