# G01-P6-F001 Play Clientbound Container Convergence Report

## Result

Ferrite implements and verifies all seven required packets in
`PROTO-PLAY-CLIENTBOUND-CONTAINER-CONVERGENCE-001`. Wire-only IDs, raw registry numbers, client
screens, state counters, and remote snapshots remain inside the Java 26.2 adapter; normalized item
and menu authority remains owned by the Region runtime.

## Verified boundaries

- IDs 17, 18, 19, 20, 59, 96, and 108 have exact golden bodies and round trips.
- Signed container/state VarInts and signed slot/property shorts retain their full wire domains.
- Optional stacks enforce packet-bounded allocation, strict item/component registries, typed
  component consumption, last-write/removal-wins patch semantics, signed count-capacity faults,
  nonpositive emptiness, and canonical AIR egress.
- Unknown menu IDs, malformed trusted component NBT, overlong VarInts, impossible list counts,
  truncation, and residual bytes fail decoding.
- Open, missing-screen, ID-agnostic close, exact/zero container targeting, verbatim state
  installation, short-list retention, and prefix-before-fault behavior match the locked client.
- Tutorial ordering, hotbar pop time, creative forced-slot/broadcast and cursor suppression, plus
  all ordinary/equipment inventory destinations are covered.
- Canonical server opening emits close/open/full/data in order; deltas scan slots, cursor, then data,
  and state/container counters wrap at 32,767 and 100 respectively.
- A complete publisher-to-codec-to-client trace converges slots, cursor, data, and state.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_container_convergence.rs`
- `docs/development/protocol-play-clientbound-container-convergence.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
