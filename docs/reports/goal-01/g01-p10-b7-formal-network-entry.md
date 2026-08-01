# G01-P10-B7 — Formal Minecraft network entry remediation

The earlier completion record proved the protocol and gameplay partitions but left the production
`ferrite-server` Minecraft listener as a bound port with no accept/session loop. This remediation
closes that integration gap. The immutable server binary now owns the network entry described by
the [formal network entry contract](../../development/minecraft-network-entry.md).

## Implemented path

- A nonblocking listener accepts bounded work and preserves partial outbound writes.
- Every accepted socket registers with `NodeLifecycle`, enters `SessionBridge`, and is removed at
  the next uncommitted Region tick.
- `ServerConnection` owns handshake, status, offline login, configuration, Play installation, and
  packet state; malformed or failed connections are isolated to their session.
- Structured Play entry packets and `JavaPlayerConnection` project initial and continuing terrain,
  movement, block results, recentering, and chunk-flow feedback.
- A 20 TPS local consistency island starts 25 Regions around spawn and exposes both session and
  authority counts through management status.
- Drain closes admission, disconnects sessions, commits semantic leave, releases Region authority,
  and only then allows the process to stop.

The optional `minecraft.registry_report` path supplies ignored, locked Mojang registry and tag data
for an exact client. It does not add Mojang artifacts to the repository.

## Exact-client findings

The locked 39,193,383-byte Java 26.2 client has SHA-1
`2dc72797acbc1b63fc16a11c4ac393605f453754`. Driving this client through HMCL exposed three defects
that the static terrain fixture could not prove:

1. Java 26.2 `PalettedContainer` uses fixed-size long-array reads and writes. Ferrite still emitted
   the older VarInt storage-length prefix, causing `ClientboundLevelChunkWithLightPacket` to fail in
   the client. The codec now derives the exact word count from palette bits and writes no prefix.
2. A connection entering `Closing` could still receive tick projection. That session error escaped
   the gateway and stopped the process. Tick projection is now Play-stage gated and any remaining
   projection failure terminates only its owning session.
3. The formal entry used `NoCollision` against a solid flat world, so normal standing eventually
   triggered the flying timeout. Movement now uses `FlatWorldCollision` at the projected surface
   (`ground_y = 64`) and the Play world is declared flat.

After those corrections, the unmodified HMCL-launched client completed login, configuration, Play
installation, chunk decoding, and remained in Play for more than 30 seconds. During the observation
the management snapshot remained `ready`, `active_sessions` remained 1,
`active_region_authorities` remained 25, and the client log recorded no new packet or disconnect
error. An unrelated offline-profile certificate request returned 401 and did not affect the server
session.

## Verification

The focused evidence is:

```text
cargo test -p ferrite-server-runtime --test network_entry -- --nocapture
cargo test -p ferrite-protocol --test c2 --all-features
target/debug/protocol-conformance connect-c2 127.0.0.1:25565
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --no-fail-fast
cargo run -q -p mc-reference --bin mc-ref -- readiness
cargo run -q -p mc-reference --bin mc-ref -- protocol readiness
cargo run -q -p mc-reference --bin mc-ref -- verify --offline
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
```

The external C2 command reported login, Play acknowledgement, batch feedback, loaded, movement,
and tick-end success against the already-running formal binary. The network-entry integration test
performs framed status/ping, holds a connection across process polls, validates lifecycle counts,
and proves session/Region drain. Final repository, reference, format, Clippy, and workspace-test
gates are run in this report's containing commit.

## Completion boundary

This batch integrates a single-node local Region consistency island with the production Minecraft
entry. Distributed placement remains owned by `ferrite-region-runtime`; this report does not claim
that live gateway sessions are already placed through Lattice. It adds no compatibility claim for
other client versions, plugins, modified clients, or unmeasured scale.
