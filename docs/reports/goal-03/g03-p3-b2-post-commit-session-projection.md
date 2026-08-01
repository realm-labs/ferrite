# G03-P3-B2 Post-Commit Session Projection

## Outcome

The formal gateway now consumes every `CompositeGatewayTickReport` projection only after its
Region commit receipt exists. Projection decoding fails closed on an unknown owner/kind,
zero sequence, malformed stable audience, or malformed semantic payload. Block updates are scoped
to sessions owned by the producing Region; player and entity updates retain their stable targeted
audiences.

Every network session owns an atomic fixed-capacity queue. The route preflights all applicable
records before enqueueing any of them, preserves canonical Region/projection order, and drains at
most 32 records per server tick into the protocol driver's independently bounded 128-frame queue.
A full session queue terminates only that slow session. Registry conversion failure occurs before
the semantic prefix is removed.

Current block projections become exact 26.2 `BlockUpdate` packets through the installed registry
map. Player inventory/menu and entity projections are parsed, targeted, and recorded as explicitly
deferred because their clientbound packet implementations belong to Goals 05 and 06; they do not
produce false success packets.

## Verification

- `cargo test -p ferrite-server-runtime --all-features --test composite_projection --test
  composite_gateway --test network_entry --test block_interaction --test player_session`: passed;
  audience decoding, Region scoping, atomic overflow, bounded prefix delivery, exact block packet
  conversion, malformed-record rejection, formal tick, and existing projection regressions pass.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite production verify`: passed.
- `cargo ferrite source verify`: passed.
- `git diff --check`: passed.
