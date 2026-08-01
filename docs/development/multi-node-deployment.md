# Multi-node deployment contract

`G01-P2-B6` makes process topology an explicit, versioned Ferrite contract. Local processes,
containers, and Kubernetes pods run the same `ferrite-server` executable with a TOML configuration
file. They do not select a different simulation implementation.

## Configuration schema

Schema version 1 covers every node-level concern needed before gameplay services start:

- cluster name;
- stable node ID and a fresh process incarnation;
- gateway, Region worker, coordinator-candidate, and administration roles;
- remoting bind and advertised addresses;
- development-static or Kubernetes headless-service discovery;
- Region placement capacity and required placement domains;
- storage root;
- management and Minecraft listener addresses plus the optional external 26.2 registry report;
- session, Region mailbox, and management-request bounds;
- graceful-drain timeout.

Unknown TOML fields, unsupported schema versions, duplicate roles or peers, zero limits, invalid
advertised addresses, listener collisions, and inconsistent role/capacity combinations fail before
any listener opens. Omitting `node.incarnation` creates a fresh UUID-backed Lattice incarnation for
that process. An exact non-zero incarnation remains available to deterministic harnesses.

The Kubernetes ConfigMap uses exact `${NAME}` values for the pod name, pod IP, and namespace.
Interpolation is deliberately not a general string-template language: embedded, missing, lowercase,
or malformed variable references fail closed.

## Lifecycle and probes

Every node begins in `awaiting-membership`. Discovery must reach its configured minimum cohort
before membership becomes ready. Required placement domains form a second barrier:

```text
awaiting-membership -> awaiting-placement -> ready
                                             |
                                             v
                                          draining -> drained -> stopped
```

`GET /healthz` reports process health. `GET /readyz` returns 200 only in `ready`; it returns 503
before both barriers and throughout drain. `GET /status` exposes the full bounded lifecycle
snapshot. `POST /drain` is accepted only from loopback unless `allow_remote_drain` is explicitly
enabled.

Drain closes new session admission first. Completion then waits for all three counters to reach
zero:

1. active client sessions;
2. active Region authorities, after fencing and handoff;
3. pending durable commits.

The lifecycle API is the integration boundary used by Lattice authority and persistence work.
`G01-P2-B7` supplies and faults the distributed runner that drives those counters; the process shell
cannot declare itself drained while any counter remains.

The formal Minecraft gateway drives its local Region consistency island and reports all 25
preloaded authorities through the same counters. Its detailed accept/session/tick/drain contract is
recorded in [Formal Minecraft network entry](minecraft-network-entry.md). Distributed placement of
those live gateway sessions remains behind `ferrite-region-runtime`; the packet/session adapter does
not acquire Lattice types.

## One-command development cluster

Run:

```text
cargo run -p ferrite-cluster -- dev --nodes 3
```

The launcher builds the current `ferrite-server`, creates an ephemeral directory, assigns
non-colliding listener ports, starts three child processes, and waits for every `/readyz` endpoint.
Ctrl+C posts drain to every node and requires every child to exit before the deadline. A bounded
automation run is:

```text
cargo run -q -p ferrite-cluster -- dev --nodes 3 --shutdown-after-ms 1000
```

`--base-port`, `--state-dir`, and `--server-bin` allow isolated concurrent development profiles.
An explicitly supplied state directory is retained; the default temporary directory is removed
after a clean launcher exit.

## Containers and Kubernetes

[`Dockerfile`](../../Dockerfile) produces one runtime image containing only the release
`ferrite-server` executable, certificates, and the management probe client. The Compose profile is:

```text
docker compose up --build
docker compose down
```

[`compose.yaml`](../../compose.yaml) starts three identical images with separate durable volumes and
explicit static discovery. Only node 1 publishes the Minecraft listener; all management ports are
available for inspection.

[`deploy/kubernetes/ferrite.yaml`](../../deploy/kubernetes/ferrite.yaml) defines:

- a three-replica parallel `StatefulSet`;
- headless discovery with not-ready addresses published for bootstrap;
- readiness and liveness HTTP probes;
- a loopback pre-stop drain request and 30-second termination grace period;
- rolling updates plus a `PodDisruptionBudget`;
- non-root execution and one persistent volume per node;
- a load-balanced Minecraft service.

The image reference is a deployment input and must be replaced with an immutable, published digest
before production use. Apply the contract with:

```text
kubectl apply -f deploy/kubernetes/ferrite.yaml
```

## Verification

Repository tooling rejects drift in the image, Compose topology/configuration, probe paths,
Kubernetes discovery, rolling-update, disruption, and drain contracts:

```text
cargo ferrite deployment verify
```

The batch also executes an actual three-process startup/readiness/drain/exit smoke run. Docker and
Kubernetes manifests are structurally verified in this batch; environment-backed rollout and fault
evidence belong to `G01-P2-B7` and later fault-injection batches.
