# Minecraft 26.2 Required Server Connection

`G01-P3-B3` composes the independently verified C0/C1 packet families into one bounded
server-side connection driver. The driver accepts framed TCP bytes and reaches the semantic Play
installation boundary without exposing packet IDs, wire registry IDs, JSON, NBT, or compression
state to simulation.

## Ownership

`java_26_2::connection` is split by responsibility:

- `settings` owns immutable listener policy, status/configuration snapshots, disconnect
  presentation, frame limits, and initial connection-local preferences;
- `bootstrap` validates the complete 29-registry configuration projection, locked feature set,
  tag indices, known-pack provenance, and full-versus-elided registry data;
- `driver` composes framing, directional protocol states, family codecs, state owners, liveness,
  and transition callbacks;
- `output` contains bounded-driver output and the normalized events consumed by the server
  runtime;
- `error` contains structured terminal failures.

The driver has one bounded inbound frame decoder, at most 128 pending outbound frames, at most one
in-flight frame, and at most 64 pending semantic events. A protocol, codec, callback-order, queue,
or transition failure faults the connection and discards unsent frames and undelivered events.

## Directional transitions

Clientbound and serverbound protocol states are separate because terminal packets cross the two
directions at different times:

```text
Handshake
  status -> clientbound Status -> serverbound Status
  login  -> clientbound Login  -> serverbound Login

Login compression
  queue login_compression with the old envelope
  -> send completion installs compression in both directions
  -> queue login_finished with the new envelope
  -> send completion releases serverbound login acknowledgement

Login acknowledgement
  install clientbound Configuration
  -> retain normalized profile cookie
  -> install serverbound Configuration
  -> start required configuration packets

Configuration finish
  receive finish_configuration under serverbound Configuration
  -> install clientbound Play
  -> emit semantic PlayInstallationRequested
  -> server runtime performs admission and player creation
  -> completion installs serverbound Play
```

An input barrier prevents a buffered compressed acknowledgement from being decoded before the
compression packet callback, and prevents login acknowledgement from advancing before
`login_finished` is flushed. Already encoded old-state frames retain their original state and FIFO
position.

## Required configuration projection

Configuration begins with server brand, enabled features, and one known-pack offer. A response
must exactly equal the offer to enable NBT elision, and only entries whose recorded source pack is
in that offer are elided. Otherwise all available entry NBT is retained.

The snapshot constructor requires the complete locked 29-registry order, unique element IDs,
`minecraft:vanilla`, only locked 26.2 feature names, unique tag payloads, nonnegative tag members,
and valid indices for dynamic registries. Synchronization emits one registry packet per ordered
registry followed by exactly one tag update. Spawn readiness then emits terminal configuration
finish.

Client brand is ignored, the latest client information replaces earlier values, and common
configuration keepalive retains its exact 15-second challenge/timeout and latency behavior. The
latest information, normalized profile, and transfer flag are carried only in the semantic Play
installation request.

## Server-runtime boundary

This batch deliberately stops at `PlayInstallationRequested`. It installs no Region actor,
`bevy_ecs::World`, voxel storage, player entity, or Lattice reference. `G01-P3-B4` consumes the
normalized routing, duplicate-disconnection, configuration-selection, latency, and Play
installation events and connects them to Region-native session routing. Only after that semantic
work succeeds may it call `complete_play_installation`.

## Evidence

`crates/ferrite-protocol/tests/c1/server_connection.rs` covers:

- complete snapshot validation and exact known-pack elision;
- fragmented status routing, cached response, exact pong, and send-before-close;
- unavailable status and wrong-version refusal directionality;
- the integrated post-increment login timeout;
- buffered acknowledgement barriers across compression and login-finished sends;
- the complete offline login, configuration prelude, 29-registry/tag projection, latest client
  information, finish acknowledgement, and split Play installation trace;
- fail-closed early configuration finish with stale output/event removal.
