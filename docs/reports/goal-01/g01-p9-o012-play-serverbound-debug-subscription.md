# G01-P9-O012 Play Serverbound Debug-Subscription Gate Report

## Result

Ferrite explicitly gates serverbound Play ID 23 in
`PROTO-PLAY-SERVERBOUND-DEBUG-SUBSCRIPTION-001`. Diagnostic subscriptions default to disabled and
require a separately registered service. The enabled boundary replaces typed, connection-local
requested state on the level thread and never treats raw registry IDs or diagnostic projection as
simulation or persistence authority.

## Verified boundaries

- ID 23 is locked to `minecraft:debug_subscription_request`; the empty-set compression frame is
  exactly `03001700`, and the required Play decoder remains fail-closed for this optional identity.
- The encoded set admits at most 32 entries before duplicate collapse. Only the locked 16-entry
  `minecraft:debug_subscription` mapping resolves; unknown raw IDs fail rather than being retained.
- Each admitted request replaces the whole requested set. The gate produces no response or
  acknowledgement and transfers the typed replacement to the level thread.
- Unauthorized requests remain requested but produce an empty effective set. Operator permission
  or the IDE singleplayer-owner exception makes them effective on the next rebuild without a new
  request; revocation empties effective membership while preserving requested state.
- Every rebuild returns explicit synchronizer transitions: wake-and-seed when the first effective
  membership appears, sleep-and-clear when the last disappears, membership replacement for a
  changed nonempty set, and unchanged otherwise.
- Disconnect and Play-to-Configuration player removal clear both requested and effective state.
  Neither requested state nor synchronizer state survives with a replacement player object.

The optional service receives no default network wiring. Source tracking, diagnostic value
production, and end-to-end handler registration require an explicit child batch.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/serverbound/debug_subscription/`
- `crates/ferrite-protocol/tests/c4/play_serverbound_debug_subscription.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_serverbound_debug_subscription --all-features
10 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1190 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
