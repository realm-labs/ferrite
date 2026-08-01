# G03-P2-B4 Formal Composite Region Route

## Outcome

The formal `MinecraftGateway` now owns `SessionBridge<CompositeRegionRouter>`. The router is the
only production Region command and player-transfer route, and it owns both the deterministic local
Region executor and one `CompositeProductionRegionRuntime` for every formal Region. The gateway no
longer constructs or passes `PlayerRegionLogic` into a production tick.

Every formal tick executes the low-level Region barriers and requires all active Regions to return
a composite commit for the identical tick. The returned `CompositeGatewayTickReport` retains the
local committed-command and transfer report needed by the current session acknowledgements while
also exposing each Region's service outcomes, current-generation continuity candidate,
post-commit projections, events, and replay-bound commit receipt. A missing or failed composite
Region poisons the tick instead of allowing the gateway to advance a parallel path.

## Player ownership join

The adapter synchronizes player ownership at the reconciliation barrier. Stable players newly
present in transient session state enter the composite player service through `JoinPlayer`; players
removed by disconnect or Region transfer enter through `LeavePlayer`. Both sets are ordered by
stable identity, receive bounded canonical command sequences, participate in the composite replay
identity, and are captured by the same continuity preparation as the other Region services.

The local runner remains the executor for genuine `TickPhase` barriers and required player-transfer
delivery. Its historical `PlayerRegionLogic` implementation remains usable by isolated low-level
conformance tests, but it is no longer reachable from the formal server entry.

## Verification

- `cargo test -p ferrite-server-runtime --all-features --test composite_gateway --test
  network_entry --test playable_adversity --test player_session --test block_interaction`: passed;
  formal composite join/leave, all-Region commit, socket lifecycle, backpressure, movement,
  transfer, and block regressions passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite production verify`: passed.
- `cargo ferrite source verify`: passed.
- `git diff --check`: passed.
