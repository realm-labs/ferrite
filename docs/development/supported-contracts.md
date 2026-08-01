# Goal 01 supported contracts

This document freezes Ferrite's supported interfaces at the Goal 01 completion boundary. The
snapshot is workspace version `0.1.0`, Minecraft Java Edition 26.2, server configuration schema 1,
Region mapping version 1, and placement domain `ferrite-region-v1`.

Changing a versioned format requires a new version plus a fail-closed migration or compatibility
path. Changing a supported command, configuration field, management endpoint, or deployment
behavior requires updated documentation, focused tests, and the complete acceptance gate.

## Operator CLI

The supported server process command is:

```text
ferrite-server --config <PATH> [--check-config]
```

`--config` is mandatory. `--check-config` parses, expands, and validates the configuration without
opening listeners. Unknown arguments and missing values fail. SIGINT begins the same bounded drain
as the management endpoint and exits only after sessions, Region authority, and pending durable
commits reach zero.

The supported local-cluster command is:

```text
ferrite-cluster dev --nodes <N> [--base-port <PORT>] [--state-dir <PATH>]
  [--server-bin <PATH>] [--shutdown-after-ms <MILLIS>]
```

It starts the requested processes, waits for every readiness barrier, and drains every node on
Ctrl+C or the optional automation deadline. The `capacity` and `verify-*` commands are supported
repository evidence interfaces. Commands ending in `-worker` are private subprocess protocols and
are not operator compatibility surfaces.

The supported offline recovery inspection command is:

```text
world-inspector <STORE_DIRECTORY> <WORLD_ID_HEX> <DIMENSION> <REGION_X> <REGION_Z>
```

`behavior-runner`, `protocol-conformance`, `mc-ref`, and `cargo ferrite` are reproducibility and
repository-maintenance interfaces. Their committed help and runbooks define the Goal 01 snapshot,
but they are not server deployment APIs.

## Server configuration schema 1

The version-1 TOML sections are `cluster`, `node`, `remoting`, `discovery`, `placement`, `storage`,
`management`, `minecraft`, `limits`, and `shutdown`. The concrete examples in
[`deploy/compose`](../../deploy/compose) and the Kubernetes ConfigMap are canonical inputs.

The schema is closed: unknown fields, unsupported versions, duplicate roles or peers, zero or
colliding listener ports, invalid discovery cohorts, inconsistent Gateway/Minecraft or
RegionWorker/capacity pairs, missing placement domains, empty storage roots, zero limits, and an
invalid drain deadline fail before listeners open. The only supported environment interpolation is
one complete uppercase `${NAME}` value in the explicitly templated Kubernetes fields.

Omitting `node.incarnation` generates a fresh process incarnation. Supplying it is reserved for
deterministic harnesses; stable node ID does not weaken generation fencing.

The optional `minecraft.registry_report` path selects the external locked 26.2 registry report and
matching extracted data tree used for exact unmodified-client registry/tag projection. Those
Mojang-generated inputs are deployment data and remain ignored. The formal listener, continuous
session, Region tick, projection, and drain behavior is specified by the
[Minecraft network entry contract](minecraft-network-entry.md).

## Management and lifecycle

The supported HTTP surface is:

- `GET /healthz` for process health;
- `GET /readyz`, which returns success only after membership and all required placement domains;
- `GET /status` for the bounded lifecycle snapshot;
- `POST /drain`, restricted to loopback unless `allow_remote_drain` is explicitly enabled.

The lifecycle is `awaiting-membership -> awaiting-placement -> ready -> draining -> drained ->
stopped`. Drain closes admission before moving authority and flushing durable work. A process may
not report readiness during drain or claim completion while any owned work remains.

## Rust library boundary

All workspace packages are version `0.1.0`, `publish = false`, and deny unsafe code at their crate
roots. They are supported as one source-built workspace, not as independently versioned crates.io
libraries. The stable architectural contract is responsibility and type ownership:

- `ferrite-foundation`, `ferrite-registry`, and `ferrite-replay` own topology-independent identity,
  content identity, canonical encoding, hashes, and replay;
- `ferrite-world`, `ferrite-simulation`, and `ferrite-gameplay` own Region-local authoritative
  state and semantic behavior without packet, executor, or Lattice types;
- `ferrite-persistence` owns versioned recovery formats and durable commit ordering;
- `ferrite-region-runtime` exclusively owns Lattice integration, routing, fencing, and handoff;
- `ferrite-protocol` exclusively owns the Minecraft Java 26.2 wire adapter and exposes semantic
  session commands and effects inward;
- `ferrite-server-runtime` composes sessions, lifecycle, admission, persistence, Region routing,
  client projection, and the formal nonblocking Minecraft listener.

Public Rust items exist for workspace composition and testing. Their exact item-level signatures
are not an external SemVer commitment; persisted schemas, canonical codecs, protocol identity,
configuration schema, and the ownership/type boundaries above are the compatibility surfaces.

## Deployment contract

The same `ferrite-server` binary and schema-2 configuration run locally, in the immutable runtime
image, under three-node Compose, and in the three-replica Kubernetes StatefulSet. The supported
deployment behavior includes two-stage readiness, loopback pre-stop drain, rolling replacement,
headless discovery, one durable volume per node, a disruption budget, non-root execution, and the
Minecraft service.

Production deployment must replace the example image tag with an immutable published digest. The
named capacity profiles are reproducible synthetic regression evidence, not a player-count promise
or a claim about unmeasured hardware, workloads, topologies, or platforms.

## Compatibility boundary

Goal 01 supports an unmodified Minecraft Java Edition 26.2 client through the audited C0-C3
baseline and the formal `ferrite-server` entry. It does not claim compatibility with plugins,
client modifications, other Minecraft versions, enabled implementations for services whose C4
contract is only a gate, or behavior beyond the locked catalog and source evidence. The four
source-inconclusive observations remain explicit `DeferredExperiment` records and are not filled by
guessed compatibility behavior.
