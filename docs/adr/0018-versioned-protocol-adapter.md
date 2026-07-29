# ADR-0018: Keep Wire Protocols Behind Versioned Adapters

## Status

Accepted

## Context

Minecraft wire IDs and state machines are version-specific, while gameplay, persistence, Region
routing, and replay need stable semantic contracts.

## Decision

The protocol responsibility has three layers:

1. project-owned semantic session events, commands, snapshots, deltas, acknowledgements, and effects;
2. the exact `minecraft_java_26_2` framing, connection states, packet catalog, codecs, registry
   projection, and acknowledgement machinery;
3. separately versioned future adapters, including a Ferrite-native protocol if later approved.

Only the 26.2 adapter knows packet numeric IDs. Every decoder is bounded, rejects malformed or
trailing data according to its family contract, and performs connection-state validation before
producing semantic ingress. Semantic commands are routed by stable world/Region identity.
Clientbound packets are projections of committed semantic state.

The locked packet catalog is generated locally from verified official artifacts. Mojang reports and
JARs are not production dependencies or committed generated data.

## Consequences

- Simulation and saves do not change when wire IDs change.
- Protocol conformance can test golden bytes independently from gameplay.
- Translation and projection layers add explicit code and memory bounds.
- Unsupported packets/states fail according to audited refusal behavior rather than leaking through.

## Alternatives Considered

- Use packet structs as gameplay commands: rejected because versioned wire details would spread.
- Expose Lattice messages directly to clients: rejected because the external compatibility contract
  is Minecraft 26.2.
- Decode permissively and ignore residual fields: rejected because ambiguity is unsafe and breaks
  conformance.

## Migration or Reversal Plan

Add a sibling adapter with its own packet inventory and acceptance suite. Any semantic API change
requires callers and replay implications to be reviewed independently of the wire version.
