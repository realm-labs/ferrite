# G01-P6-F009 Play Serverbound Merchant Report

## Result

Ferrite implements and verifies ID 51 in `PROTO-PLAY-SERVERBOUND-MERCHANT-001`. The transaction
reuses normalized clientbound offer/cost forms while keeping selection prediction and payment slots
menu-local.

## Verified boundaries

- The hint-zero golden, all signed VarInt endpoints, malformed, overlong, truncated and residual
  forms are covered.
- Only a current still-valid merchant menu is admitted; no idle, loaded, spectator, death or
  container-ID gate is added.
- Every admitted hint is stored and result lookup runs before auto-fill range checking.
- Positive in-range hints force one offer; zero, negative and at/beyond-size hints scan from zero.
- Normal then swapped payment lookup, out-of-stock retry, copied result/future XP and both callback
  quirks are covered.
- Invalid hints move no payment. Valid hints return payment zero then one in reverse order,
  preserving partial and second-slot failure effects.
- Auto-fill scans ascending, admits exact predicates plus extras, requires full component equality
  for merging and fills to source maximum rather than required cost count.
- Client prediction completes before send; server replay emits no direct response and converges
  only through ordinary container state.
- Seven named C3 vectors pass; the combined C3 suite is 78 tests.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_serverbound_merchant.rs`
- `docs/development/protocol-play-serverbound-merchant.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
