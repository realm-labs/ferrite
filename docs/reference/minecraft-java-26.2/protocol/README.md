# Minecraft Java Edition 26.2 Protocol Compatibility Reference

This directory is the normative entry point for Ferrite's server-side wire compatibility with an
unmodified Minecraft Java Edition `26.2` client.

**Status:** C0, C1, and C2 are source-specified. C2 covers movement, terrain streaming,
chunk/biome/light mapping, readiness, liveness, disconnect, player/vehicle correction, block
prediction/convergence, interaction, and their independent golden and negative vectors. C3 entity
interaction/session feedback is source-specified; its lifecycle, effects, inventory, command and
other families plus C4 remain incomplete. Missing later-level details remain implementation
blockers and must not be inferred from another Minecraft version or from memory.

The target artifacts, hashes, Java version, legal boundaries, and report-generation procedure are
locked by the parent [source catalog](../sources.md). In particular:

- `OFF-META-001` locks the exact `26.2` version metadata;
- `OFF-SERVER-001` and `OFF-CLIENT-001` lock the official server and client jars;
- `OFF-REPORT-001` locks the reproducible `reports/packets.json` packet identity catalog.

Generated reports, jars, captures, logs, and extracted class data remain under
`target/mc-reference/26.2/` and are not committed.

## Compatibility Objective

Ferrite initially accepts exactly the locked `26.2` client protocol. An unmodified client must be
able to:

1. discover and ping the server;
2. complete offline-mode login and configuration;
3. enter a minimal world;
4. receive chunks, lighting, entities, inventories, and effects required by supported gameplay;
5. send movement and interactions;
6. converge through the version's acknowledgement and correction rules.

Online-mode authentication, encryption, secure-profile or chat requirements, transfer, cookies,
resource packs, dialogs, and other target-version features are implemented according to explicit
compatibility milestones. Unsupported optional paths must fail or degrade exactly as documented;
they are not silently treated as complete.

## Architectural Boundary

Minecraft packets are a versioned boundary format:

```text
TCP bytes
    ↓ framing, compression, encryption
26.2 connection state and packet codec
    ↓ normalized session events / gameplay requests
Ferrite server runtime and simulation
    ↓ semantic snapshots, deltas, effects
26.2 session projection and packet ordering
    ↓
TCP bytes
```

Packet structs, packet IDs, wire registry IDs, entity metadata slots, and codec-specific value types
must not enter world storage, ECS components, gameplay APIs, persistence records, or the future
Ferrite-native protocol.

## Required Specification Set

The protocol reference will be split as evidence is completed:

```text
protocol/
├── README.md
├── completion.toml
├── framing-and-primitives.md
├── handshake-and-status.md
├── login-and-configuration.md
├── play-serverbound.md
├── play-clientbound.md
├── registry-and-metadata-mappings.md
├── ordering-and-acknowledgements.md
└── conformance.md
```

[`completion.toml`](completion.toml) is the recoverable protocol work queue. Its family selectors
must partition every state/direction/identity in the locked packet report exactly once; broad
`Todo` families are placeholders that must be split before completion. Protocol readiness is
independent of gameplay readiness:

```sh
cargo run -p mc-reference --bin mc-ref -- protocol inventory
cargo run -p mc-reference --bin mc-ref -- protocol coverage
cargo run -p mc-reference --bin mc-ref -- protocol readiness
cargo run -p mc-reference --bin mc-ref -- protocol verify
```

`protocol inventory` locks packet count, state, direction, identity, numeric-ID continuity and a
sorted-entry digest. `protocol coverage` rejects missing, ambiguous or dead family selectors and
false completion metadata. `protocol readiness` additionally exits nonzero while any family is
`Todo` or `InProgress`; it never consults gameplay completion. The general offline verifier runs
protocol verification as a separate structural gate without pretending either readiness ledger is
complete.

Each packet-family specification must record:

- connection state and direction;
- namespaced packet identity and locked numeric ID;
- exact field order, encoding, conditional fields, and bounds;
- legal sender state and semantic preconditions;
- normalized Ferrite ingress or egress mapping;
- connection-local state read or changed;
- required ordering and acknowledgement relationships;
- disconnect or recovery behavior for malformed and stale input;
- primary jar symbols, official report identity, and directed experiments;
- golden bytes, boundary cases, negative cases, and end-to-end trace vectors.

## Evidence Method

Use evidence in this order:

1. Verify state, direction, packet names, and numeric IDs against the locally regenerated locked
   `packets.json` report.
2. Inspect the locked client and server codec declarations and handlers for exact field layout,
   bounds, state transitions, and side effects.
3. Cross-check registry identities and values against locked reports and bundled data.
4. Record the smallest independent specification and source locators; never commit copied generated
   tables or decompiled code.
5. Build golden vectors and a headless session test.
6. Run an unmodified `26.2` client against Ferrite.
7. When ordering or optional behavior remains ambiguous, compare a controlled packet trace with the
   locked official server and record a directed experiment.

An encode/decode round trip proves only internal symmetry. It does not prove compatibility with the
original client.

## Compatibility Levels

| Level | Required result | Status |
|---|---|---|
| `C0` | Status discovery and ping | SourceSpecified |
| `C1` | Offline-mode login, configuration, and minimal play entry | SourceSpecified |
| `C2` | Chunks, movement, correction, keepalive, and block interaction | SourceSpecified |
| `C3` | Entities, inventories, containers, effects, commands, and core survival | InProgress |
| `C4` | Online-mode security path and broad supported-gameplay conformance | Todo |

Later Minecraft versions receive sibling reference directories and adapters. They do not modify the
`26.2` packet catalog or conclusions in place.

## Test Layers

Protocol work must preserve four independent test layers:

- codec tests with golden bytes, boundaries, truncation, malformed values, compression, and fuzzing;
- session tests covering legal and illegal state transitions and acknowledgement lifecycles;
- semantic integration tests pairing packet traces with authoritative simulation state and replay
  hashes;
- unmodified-client smoke tests and directed differential traces against the locked official server.

The original client is a rapid integration tool, not the only oracle. Client prediction and cached
presentation may temporarily hide an incorrect server result.
