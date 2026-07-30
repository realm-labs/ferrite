# G01-P6-F003 Play Clientbound Merchant Report

## Result

Ferrite implements and verifies ID 52 `minecraft:merchant_offers`, the sole packet in
`PROTO-PLAY-CLIENTBOUND-MERCHANT-001`. Merchant state is an owned connection projection composed
after ordinary menu convergence; it does not introduce Region-owned packet IDs or GUI state.

## Verified boundaries

- The exact golden and structured round trips cover signed IDs, generic list counts, strict
  item/component registries, optional second costs, nonempty results, raw floats, and residual
  packet handling.
- Exact component predicates retain order and duplicates, require every expected value, allow
  candidate extras, and keep count admission separate.
- Decode fixes reward experience true, makes a true stock flag overwrite uses with maximum uses,
  and lets a false flag still derive out of stock from counts.
- First-cost pricing reproduces Java signed wrapping, float conversion, floor, clamp, NaN, and
  positive/negative infinity behavior; second cost and empty-second-input rules remain distinct.
- Client application silently ignores wrong IDs and nonmerchant menus, needs no screen, then
  replaces offers, experience, level, and flags in locked order.
- Canonical publication sends open/full content/properties before the copied offer snapshot,
  omits the offer packet for an empty list, and has no acknowledgement path.
- The end-to-end publisher-to-codec-to-client trace proves source mutation cannot alter an emitted
  offer/result snapshot.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_merchant.rs`
- `docs/development/protocol-play-clientbound-merchant.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
