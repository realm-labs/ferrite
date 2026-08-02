# G04-P6-B1 — Authoritative block interaction

## Outcome

`Satisfied`. The formal composite gateway no longer executes block interaction against
`RegionSimulationState::voxels`, the flat simulation shadow. It reads generated block state from the
world-service-owned chunk columns, stages all writes by their actual owning Region, and commits one
revision-fenced `SetWorldBlocks` transaction per target Region. The legacy `PlayerRegionLogic` path
retains its shadow implementation only for isolated pre-composite conformance fixtures; it is not
reachable from the formal Minecraft entry.

The client reproduction exposed two coupled defects. A command is routed by the targeted block, so
a player standing in Region A can legitimately send destroy input to Region B. The old handler then
tried to store `BlockBreakSession` in Region B's player ECS even though the player remained owned by
Region A. It also mapped every decode, source, voxel, ECS, and journal failure to the same
`ferrite:block/region_logic` identity, which caused the local runner to poison the tick and the
Minecraft gateway to close.

## Authority and commit flow

The formal handler now has a responsibility-owned boundary:

1. decode and authenticate each Region-routed block command;
2. read the hit and adjacent states from the unique world-service chunk owner in the command's
   world and dimension;
3. keep the short-lived start/abort/stop destroy session independently of the target Region ECS;
4. overlay same-tick writes so multiple interactions observe deterministic staged state;
5. group writes by the actual target Region and admit one atomic, expected-revision transaction at
   `ReconcileBoundary`;
6. publish block changes only from the committed composite projection and retain the per-player
   result/correction journal used for Java prediction convergence.

An adjacent placement may therefore read its hit in Region A and commit the placed block in Region
B. A destroy request may target Region B while the authoritative player remains in Region A. Both
paths use the generated durable column that also feeds collision, persistence, and chunk
projection.

## Failure containment and diagnostics

Unloaded, vertically invalid, or unload-busy interaction targets are ordinary gameplay rejection.
They produce a committed `Rejected` result with every available authoritative correction and do
not fail the Region tick. Invalid reconstructed hits and unreachable targets follow the same path.

Malformed internal commands, mismatched player sources, duplicate chunk authority, journal failure,
and composite service failure remain fail-closed invariants. They now retain their typed source in
`CompositeGatewayError::BlockInteraction` or `CompositeGatewayError::Composite`; the gateway no
longer replaces them with the undifferentiated block-logic semantic error.

## Regression evidence

`crates/ferrite-server-runtime/tests/composite_gateway.rs` now proves:

- missing world authority rejects one destroy command and the following tick still commits;
- a player owned by Region `(0,0)` can start and stop destroying an authoritative block in Region
  `(1,0)`, the target Region publishes air, and the gateway remains live;
- placement on block `x=127` commits its adjacent `x=128` write through Region `(1,0)` rather than
  the hit Region;
- malformed internal payloads preserve `AuthoritativeBlockError::Command` at the public gateway
  error boundary.

The complete affected-crate suite passes, including formal world persistence, portal continuity,
network entry, production replay, legacy block fixtures, and generated-world runtime tests.

The existing exact 26.2 `ferrite` client scenario also completed successfully in
`target/client-mcp-evidence/ferrite-visual-e30a35cc-e36d-4507-9ad2-e183daf33e8c`. It exercised
generated terrain, movement/collision, environment observation, framebuffer capture, graceful
flush, restart, and rejoin with no server session error. One immediately preceding attempt retained
at `ferrite-visual-f6ff73b7-2424-4879-884d-ec3b85fdf2b9` completed the entire first run but timed out
on the second client's Quick Play connection before the server admitted a session; the unchanged
retry passed. Neither run is counted as the later P6-B6 cross-Region interaction matrix.

## Verification

The batch verification commands are:

```text
cargo test -p ferrite-server-runtime --all-features
JAVA_HOME=<jdk-25> tools/ferrite-client-mcp/gradlew --no-daemon \
  -p tools/ferrite-client-mcp check build
<jdk-25>/bin/java -jar \
  tools/ferrite-client-mcp/build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-acceptance.jar \
  --workspace <workspace> --java-home <jdk-25> --mode ferrite
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p ferrite-tooling -- production verify
cargo run -p ferrite-tooling -- source verify
git diff --check
```

The production manifest now cites the formal authority handler and cross-Region composite tests for
`play/block-interaction`. Its remaining `Continuity` gap is unchanged: complete inventory-derived
placement/break semantics and their durable player state belong to Goal 05. The superseding exact
client cross-Region/restart matrix belongs to `G04-P6-B6`, after the vanilla differential and
performance batches. This batch does not claim those later gates.
