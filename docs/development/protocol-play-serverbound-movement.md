# Required Play Serverbound Movement Protocol

`PROTO-PLAY-SERVERBOUND-MOVEMENT-001` owns 15 Java 26.2 packets: chunk feedback, tick end,
client information, keepalive, four player movement forms, vehicle movement, paddles, abilities,
player command, player input, terrain-ready, and pong.

The four player forms preserve omitted position/rotation fields and ignore movement-flag bits above
1. Vehicle ground is an independent Boolean. Play client information reuses the exact
Configuration body codec. Ability bit 1 alone is flying; input bits 0–6 are retained and bit 7 is
ignored. All seven command actions are strict, while entity ID and non-jump data remain
connection-local.

## State and validation

The connection control projection preserves the 60-server-tick load grace, idempotent
`player_loaded`, pre-load input retention, loaded-only shift/idle application boundary, may-fly
gate, controlled-boat paddle gate, command context, hat transition, and chunk flow control. Chunk
feedback maps NaN to 0.01, clamps other values to 0.01–64, floors outstanding batches, restores
quota to one, and raises the in-flight limit to ten.

Player movement remains project-owned gameplay state: invalid values precede gates, infinities
clamp, teleport pending accepts only rotation, packet frequency scales speed checks, collision and
the locked always-zero residual-Y defect produce corrections, and gravity-scaled floating state is
persisted across Region transfer.

Vehicle validation independently orders invalid values, controlled-root admission, speed,
collision, correction, clamped success, wrapped rotation, and floating. Serverbound pong remains
separate from keepalive. Client tick end clears missing known movement; it never advances a server
tick.

## Evidence

- `crates/ferrite-protocol/tests/c2/play_serverbound_movement.rs`
- `crates/ferrite-gameplay/src/player/movement.rs`
- `crates/ferrite-gameplay/src/player/state.rs`
- `crates/ferrite-server-runtime/tests/player_session.rs`
