# G01-P4-B4 Playable Topology Conformance Report

## Result

`G01-P4-B4` runs one production-backed C2 scenario through direct-local and Lattice-frame routing
and requires equal canonical committed state and exact compatible Java 26.2 packet traces. Three
process-isolated Lattice repetitions return the same structured evidence.

The contract and locked digests are recorded in
[Playable topology conformance](../../development/playable-topology-conformance.md).

## Implemented boundary

- stable bounded remoting payload codecs now carry `RegionCommand` and `EntityTransfer`;
- the Lattice player router round-trips those payloads through the pinned `EntityTell` frame,
  validates target generation, and admits only decoded Ferrite semantic types;
- the scenario executes join, terrain, loaded state, same/cross-Region movement, committed player
  transfer, recenter, accepted placement, rejected placement correction, prediction ACK, matching
  block break, and committed replication;
- final evidence uses the canonical replay world hash over loaded non-air voxels and authoritative
  player state;
- packet evidence comes from actual Java 26.2 codec bodies, not enum labels or synthetic topology
  counters.

The locked evidence is:

```text
committed ticks: 7
packet bodies:   16
state digest:    1e7c50dbf4463c858fcd779f4db59a08418e54cab7ae0e502821bba95ad0a858
trace digest:    c93f7f75ba90655bb26254a13b96a0ba05e7f49f4957ca36f33ccfca2484d809
```

## Automated evidence

Focused tests cover all command-source variants, player transfer round trips, wrong message kind,
trailing bytes, equal local/Lattice state and packet records, locked state/trace digests, terrain,
three prediction ACKs, authoritative block convergence, and the process-isolated cluster command.

The acceptance commands are:

```text
cargo test -p ferrite-region-runtime
cargo test -p ferrite-server-runtime
cargo test -p ferrite-cluster
cargo run -p ferrite-cluster -- verify-playable
cargo ferrite task check
git diff --check
```

## Coverage boundary

The process-isolated repetitions prove scenario determinism across executable boundaries but do not
claim a new gameplay Region partition implementation. The already committed Phase 2 topology suite
owns the three-node partition, barrier, fencing, recovery, and fault proof. This batch binds real C2
gameplay and protocol output to its stable semantic remoting contract.

The unmodified-client C2 smoke, adverse latency/batching, malformed input, backpressure, and
cross-Region client scenarios remain `G01-P4-B5`.
