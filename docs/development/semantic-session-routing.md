# Semantic Session-to-Region Routing

`G01-P3-B4` connects the Minecraft Java 26.2 connection driver to Region-native simulation
without allowing packet IDs, Java packet structs, registry wire IDs, or Lattice types to cross the
session boundary.

## Boundary

`ferrite-protocol::semantic` defines the version-independent contract:

- a nonzero connection-local `SessionId`;
- normalized profile identity, virtual host, and client settings;
- routing, duplicate-login, configuration, latency, join, and close ingress;
- admission denial, duplicate disconnect, and successful Play installation egress.

The Java 26.2 normalizer is the only server-runtime module that accepts versioned connection
events. Known-pack and registry selection remains connection-local because it has no simulation
meaning. The Region router accepts only `RegionCommand`, so both the local runner and a future
distributed adapter implement the same topology-independent interface.

## Routing and admission

Every accepted socket is first registered against the node lifecycle's bounded admission counter.
The terminal handshake virtual host and port select an exact route or the configured fallback.
Host matching is case-sensitive and includes the port; route tables have an explicit nonzero
capacity and reject duplicates before reporting capacity exhaustion.

An initial route owns:

- stable world identity;
- stable dimension identity;
- spawn chunk;
- versioned `RegionMapping`.

The spawn chunk is converted with the mapping's Euclidean rules, including negative coordinates,
to a `SimulationRegionKey`. No network address or node identity is embedded in that key.

Admission policy runs once during login, where a bounded denial can still use the login
disconnect packet, and again immediately before Region command admission. The second check closes
the time-of-check/time-of-use gap. Duplicate ownership is checked again at that boundary.
Policy denial, duplicate ownership, invalid stable identity, encoding failure, and Region routing
failure leave the session in Configuration and do not request Play installation.

## Region command

Successful admission emits one deterministic `ferrite:session/join` command for the selected
Region and requested tick. The project-owned `FSJ1` payload contains:

- session, player, and profile identities;
- transfer status;
- profile name;
- normalized client settings.

Strings use bounded `u16` byte lengths, booleans and enums reject unknown values, and decoders
reject truncation, malformed UTF-8, invalid magic, and trailing bytes. The command source is the
stable player identity and its sequence is per session. Sequence exhaustion is checked before
routing, so a successful route cannot be followed by a failed local state transition.

Only after the Region router accepts the command does the bridge claim profile ownership, move the
session to Play, and return `CompletePlayInstallation`. The connection owner may then install
serverbound Play. Closing or explicitly unregistering a connection releases profile ownership and
the node lifecycle counter.

## Evidence

`crates/ferrite-server-runtime/tests/session_routing.rs` covers:

- payload round-trip, semantic command projection, malformed booleans, trailing bytes, and
  oversized strings;
- exact and fallback virtual-host routing across worlds, dimensions, ports, and negative chunks;
- Java 26.2 event normalization and connection-local registry selection;
- two-stage admission, duplicate ownership, denial, retry after unavailable routing, and
  fail-without-state-advance behavior;
- an actual local Region runner consuming the normalized join command during Ingress;
- latency propagation, close cleanup, and lifecycle session accounting.
