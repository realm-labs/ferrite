# Goal 07 — Distributed Production Closure

## 1. Objective

Turn the locally integrated Minecraft 26.2 server from Goals 03–06 into the supported distributed
production system described by Ferrite's Region-first architecture. Formal gateway traffic must
route to actual Lattice-owned Region workers; membership, placement, remoting, handoff, persistence,
location-independent durable storage, recovery, rolling replacement, security, observability,
capacity, and the remaining required production-manifest rows must close under real client load and
faults.

This Goal is the final compatibility and production-readiness closure for the planned Minecraft
26.2 server. It does not claim plugin compatibility, other Minecraft versions, or enabled optional
services that remain explicitly default-closed outside the supported contract.

## 2. Production closure contract

A distributed claim must prove:

```text
exact client(s)
  -> production gateway process
  -> authenticated bounded remoting
  -> current Lattice Region owner
  -> authoritative simulation and storage-fenced durable commit
  -> remote projection back through the gateway
  -> ownership move, permanent node/local-disk loss, or rolling replacement
  -> continued or explicitly bounded client outcome
  -> converged recovered state and observable operations evidence
```

A TCP reachability probe, reserved listener, in-process topology, synthetic actor benchmark, or
library-only Lattice test is not production distributed completion.

## 3. Scope boundary

### In scope

- real Lattice startup, authenticated remoting, discovery, membership, placement, ownership,
  generation fencing, and shutdown in `ferrite-server`;
- role-correct Gateway, RegionWorker, CoordinatorCandidate, and Administration process behavior;
- gateway-to-owner semantic command routing and owner-to-gateway projection delivery;
- session/player continuity across Region movement, ownership handoff, worker replacement, gateway
  disconnect/reconnect, and rolling deployment;
- durable Region recovery, journal/snapshot selection, content/config compatibility, backups,
  migrations, corruption isolation, and disaster-recovery procedures;
- a logically dedicated, location-independent Region storage layer with immutable payloads,
  linearizable fenced Region/checkpoint heads, backend-independent APIs, credentials, quotas,
  retention, and garbage collection;
- Kubernetes and local multi-process discovery, readiness, drain, disruption, storage, and immutable
  image behavior backed by live gameplay rather than endpoint probes alone;
- remaining C0–C3 production-manifest protocol and gameplay integration rows not closed by Goals
  03–06;
- explicit enabled implementations or documented default-closed treatment for every supported C4
  gate;
- configurable rate limits, body limits, queue budgets, timeouts, authentication boundaries,
  operator permissions, denial-of-service resistance, and secret handling;
- structured logs, metrics, tracing, readiness, health, status, watchdogs, alerts, capacity profiles,
  and troubleshooting/runbook evidence;
- exact-client single- and multi-client soak, fault, handoff, restart, rolling-upgrade, overload, and
  visual convergence scenarios;
- clean-checkout, image, deployment, source, dependency, license, security, format, compatibility,
  and completion audits.

### Out of scope

- Bukkit/Spigot/Paper or another plugin API;
- Forge/Fabric server mods or custom client protocol extensions;
- cross-version translation or compatibility beyond locked Minecraft Java 26.2;
- global multi-datacenter consensus or zero-latency migration promises;
- player-count promises for unmeasured hardware or workloads;
- enabling optional C4 services without their complete configured implementation and acceptance.

## 4. Distributed ownership rules

- A Gateway never owns an independent local copy of the production world when Region routing is
  enabled.
- Every command, transfer, persistence commit, and projection is fenced by current ownership and
  activation generation.
- Compute-node filesystems and per-node volumes are never the only durable world authority. Local
  storage is a disposable cache or explicit migration source in distributed mode.
- The durable storage metadata plane independently rejects stale activation generations and
  publishes Region and cross-Region checkpoint heads atomically; placement fencing alone is not a
  storage commit.
- Membership readiness requires functioning remoting and placement participation, not socket
  reachability alone.
- One Region has one authoritative owner; handoff has explicit source, durable transfer, target
  activation, routing publication, and stale-owner rejection stages.
- A client-visible outcome is emitted only from committed owner state and carries enough identity to
  reject stale or duplicate delivery.
- Drain stops admission, moves or flushes authority, completes durable work, and then closes
  sessions according to the documented bounded policy.
- Overload is bounded at every ingress, mailbox, generation, persistence, AI, tracking, and
  projection queue and has observable, tested behavior.

## 5. Phased batches

### Phase 0 — Freeze production closure truth

| Batch | Outcome |
|---|---|
| `G07-P0-B1` | Commit topology roles, formal-production durable-storage consistency/availability contract and backend choice, threat model, service-level indicators, fault matrix, inherited Goals 04–06 performance envelopes, distributed-overhead budgets, remaining production-manifest denominator, and terminal acceptance plan. The local reference profile remains MinIO plus etcd regardless of that production choice. |

### Phase 1 — Real remoting and placement

| Batch | Outcome |
|---|---|
| `G07-P1-B1` | Replace remoting reservation/probe membership with real Lattice runtime, authenticated envelopes, discovery, and lifecycle integration. |
| `G07-P1-B2` | Install role-correct placement participation, ownership claims, generation fencing, routing publication, and readiness. |
| `G07-P1-B3` | Route gateway semantic commands to current remote Region owners with bounded retry, stale-route rejection, and backpressure. |
| `G07-P1-B4` | Route committed projections and session effects from owners to gateways with ordering, duplicate suppression, and disconnect cleanup. |

### Phase 2 — Handoff, persistence, and recovery

| Batch | Outcome |
|---|---|
| `G07-P2-B1` | Implement the backend-neutral Region durable-store contract; add the MinIO-plus-etcd local/CI conformance adapter, then the production shared backend/service selected by P0-B1. |
| `G07-P2-B2` | Import validated local stores and integrate live Region handoff through published durable commit identities, activation, routing convergence, stale-owner rejection, and rollback refusal. |
| `G07-P2-B3` | Integrate player/entity/session continuity across Region handoff, permanent worker/local-disk loss, gateway reconnect, and ownership replacement. |
| `G07-P2-B4` | Complete checkpoint recovery, backup/restore, migrations, corruption isolation, retention/garbage collection, credentials, and disaster-recovery tooling/runbook. |
| `G07-P2-B5` | Prove graceful drain, rolling replacement, storage outage/recovery, and version compatibility with live gameplay on newly selected workers. |

### Phase 3 — Required service closure

| Batch | Outcome |
|---|---|
| `G07-P3-B1` | Audit every C0–C3 serverbound packet and clientbound effect against the production manifest; materialize concrete remaining batches. |
| `G07-P3-B2` | Close remaining required session, configuration, command, content, lifecycle, world, persistence, and projection service rows. |
| `G07-P3-B3` | Audit every C4 gate; keep unsupported services provably default-closed or add separately accepted enabled implementations. |
| `G07-P3-B4` | Run full exact-client reference-differential traces where permitted and resolve all required production gaps without guessed behavior. |

### Phase 4 — Security and operations

| Batch | Outcome |
|---|---|
| `G07-P4-B1` | Complete network/remoting/management authentication, authorization, rate limits, request bounds, queue budgets, secret handling, and abuse tests. |
| `G07-P4-B2` | Add structured logs, metrics, traces, watchdogs, readiness reasons, alerts, dashboards/runbooks, and bounded diagnostics. |
| `G07-P4-B3` | Validate immutable images, Kubernetes/local deployment drift, non-root operation, volumes, backups, upgrade, and rollback policy. |

### Phase 5 — Capacity, soak, faults, and completion

| Batch | Outcome |
|---|---|
| `G07-P5-B1` | Validate distributed scaling and overhead against the frozen Goals 04–06 real workloads for gateway, Region, generation, persistence, AI, tracking, and projection; publish hardware/topology-specific capacity limits with collision-resistant caches. |
| `G07-P5-B2` | Run multi-node fault injection covering loss, partition, duplication, delay, reordering, stale ownership, overload, disk faults, and rolling replacement. |
| `G07-P5-B3` | Run exact-client single/multi-client soak, handoff, restart, gameplay, visual, overload, and recovery acceptance against the deployed topology. |
| `G07-P5-B4` | Close every required production-manifest row, publish supported/unsupported contracts and capacity limits, run clean-checkout/image/deployment audits, and record completion. |

## 6. Required verification

Every Rust batch runs focused affected-crate tests plus:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Distributed batches require actual multi-process tests and may not substitute an in-process
topology. Deployment batches run immutable-image and Kubernetes/local drift gates. Player-visible
batches run Goal 02 exact-client scenarios against the formal deployed topology. Capacity and CI
work use isolated workspace-owned target directories and guarded cache policy from `AGENTS.md`.
Storage batches additionally kill the source worker, make its local state unavailable, activate the
Region elsewhere, inject stale writers and partial object/head publication, and verify the exact
published recovery point through the real configured backend.
The local and CI version of that matrix runs against MinIO plus etcd. Passing it proves adapter and
protocol conformance only; the selected production backend must pass the same matrix separately.
Goal 07 consumes and extends the [performance engineering contract](../development/performance-engineering.md);
it does not establish local generation or gameplay performance for the first time. Every remoting,
storage, handoff, and projection batch declares and measures its distributed overhead against the
same frozen pre-distributed workload.

## 7. Terminal acceptance

- [ ] Formal gateway gameplay routes to actual current Lattice Region owners; no independent local production world remains.
- [ ] Remoting, membership, placement, readiness, routing, fencing, and projection are functional services rather than reachability probes.
- [ ] Region/player/entity handoff and recovery preserve single ownership, durable state, ordering, and client convergence under faults.
- [ ] Killing a Region worker and making its local disk permanently unavailable still allows another
  eligible worker to recover the published Region commit without data loss or dual authority.
- [ ] Storage-side fencing rejects stale writers; partial object/head publication, metadata outage,
  lost receipts, retries, retention, and garbage collection cannot publish or delete live state.
- [ ] Drain, worker loss, gateway reconnect, rolling replacement, backup, restore, migration, and rollback policies pass live-gameplay tests.
- [ ] Every required C0–C3 production-manifest row has ingress, authority, continuity, projection, and applicable exact-client evidence.
- [ ] Every C4 gate is either provably default-closed or backed by an explicitly supported enabled implementation and acceptance.
- [ ] Security boundaries, rate limits, queue budgets, overload outcomes, secrets, permissions, and hostile-input tests pass.
- [ ] Logs, metrics, traces, readiness, health, watchdogs, alerts, and runbooks diagnose normal and faulted operation.
- [ ] Real-workload capacity profiles and multi-node fault/soak results state measured limits without unsupported player-count promises.
- [ ] Exact-client deployed-topology acceptance, universal gates, immutable-image/deployment audits, supported contracts, and clean-worktree completion pass.

Goal 07 is complete only when the supported Minecraft 26.2 production contract is both fully
integrated and proven through the real distributed server entry under gameplay load and faults.
