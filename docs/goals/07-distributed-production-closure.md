# Goal 07 — Distributed Production Closure

## 1. Objective

Turn the locally integrated Minecraft 26.2 server from Goals 03–06 into the supported distributed
production system described by Ferrite's Region-first architecture. Formal gateway traffic must
route to actual Lattice-owned Region workers; membership, placement, remoting, handoff, persistence,
recovery, rolling replacement, security, observability, capacity, and the remaining required
production-manifest rows must close under real client load and faults.

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
  -> authoritative simulation and durable commit
  -> remote projection back through the gateway
  -> ownership move, node loss, or rolling replacement
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
| `G07-P0-B1` | Commit topology roles, threat model, service-level indicators, fault matrix, capacity profiles, remaining production-manifest denominator, and terminal acceptance plan. |

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
| `G07-P2-B1` | Integrate live Region handoff, durable transfer, activation, routing convergence, stale-owner fencing, and rollback refusal. |
| `G07-P2-B2` | Integrate player/entity/session continuity across Region handoff, worker loss, gateway reconnect, and ownership replacement. |
| `G07-P2-B3` | Complete snapshot/journal recovery, backups, migrations, corruption isolation, restore tooling, and disaster-recovery runbook. |
| `G07-P2-B4` | Prove graceful drain, rolling replacement, disruption, storage reattachment, and version compatibility with live gameplay. |

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
| `G07-P5-B1` | Establish real-workload capacity profiles for gateway, Region, generation, persistence, AI, tracking, and projection with collision-resistant caches. |
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

## 7. Terminal acceptance

- [ ] Formal gateway gameplay routes to actual current Lattice Region owners; no independent local production world remains.
- [ ] Remoting, membership, placement, readiness, routing, fencing, and projection are functional services rather than reachability probes.
- [ ] Region/player/entity handoff and recovery preserve single ownership, durable state, ordering, and client convergence under faults.
- [ ] Drain, worker loss, gateway reconnect, rolling replacement, storage reattachment, backup, restore, migration, and rollback policies pass live-gameplay tests.
- [ ] Every required C0–C3 production-manifest row has ingress, authority, continuity, projection, and applicable exact-client evidence.
- [ ] Every C4 gate is either provably default-closed or backed by an explicitly supported enabled implementation and acceptance.
- [ ] Security boundaries, rate limits, queue budgets, overload outcomes, secrets, permissions, and hostile-input tests pass.
- [ ] Logs, metrics, traces, readiness, health, watchdogs, alerts, and runbooks diagnose normal and faulted operation.
- [ ] Real-workload capacity profiles and multi-node fault/soak results state measured limits without unsupported player-count promises.
- [ ] Exact-client deployed-topology acceptance, universal gates, immutable-image/deployment audits, supported contracts, and clean-worktree completion pass.

Goal 07 is complete only when the supported Minecraft 26.2 production contract is both fully
integrated and proven through the real distributed server entry under gameplay load and faults.
