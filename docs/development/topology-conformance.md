# Region topology and fault conformance

`G01-P2-B7` closes the Region-runtime phase with executable evidence that deployment topology does
not alter semantic Region state.

## Locked scenario

The conformance scenario uses twelve Regions in a canonical ring, three node partitions, bounded
mailboxes, and one required cross-Region boundary contribution per Region per logical tick. The
contribution depends only on stable Region state, key, and tick. Activation generation participates
in admission fencing and durable recovery but never in gameplay calculation or the semantic hash.
Every owned Region installs a real claim in the pinned Lattice `PlacementAuthority`; emit and receive
paths require that exact generation's claim to remain open.

All modes run exactly 10,000 committed ticks:

| Mode | Execution boundary | Message path |
|---|---|---|
| Local | All Regions on one partition | Ferrite semantic Region envelope |
| In-process distributed | Regions round-robin over three partitions | Lattice adapter encode/decode, deliberately reversed delivery |
| Multi-process | Three persistent `ferrite-cluster topology-worker` children | The same bounded Lattice transport payload over a JSON-lines control pipe |

The control pipe transports already encoded Region payload bytes; it does not define another
gameplay or Region protocol. The parent coordinates the required phase barrier, routes by the locked
layout, and hashes worker-owned snapshots only after every partition commits.

The locked final digest is:

```text
02ae8ad8bb897c569339b725bc3f44ed8ea49db653a25adf8d244ca68bf27685
```

Run the complete proof with:

```text
cargo ferrite topology verify
```

## Admission and fault outcomes

The topology inbox validates tick, message kind, source sequence, source and target Region keys,
both activation generations, expected ring predecessor, target partition, payload shape, duplicate
identity, and capacity before admitting a message.

| Fault | Required outcome |
|---|---|
| Reordering | Canonical admission and commit produce the same digest |
| Exact duplicate | Idempotent duplicate outcome; one stored message |
| Conflicting duplicate | Reject without replacing the accepted message |
| Missing required message | Global preflight blocks every partition commit; retry can complete the same tick |
| Corrupt transport payload | Lattice adapter rejects before inbox mutation |
| Stale source or target generation | Reject before authoritative mutation |
| Full mailbox | Return bounded overload while retaining previously accepted work |
| Node loss | Restore the latest encoded and decoded `RegionRecoveryPoint`, verify handoff digest, advance generation, and continue with the uninterrupted semantic digest |

Global commit performs a read-only preflight across every partition before mutating any Region.
This is the in-process representation of the required distributed phase barrier: one partition
cannot advance the logical tick merely because its own message arrived.

## Recovery boundary

The node-loss test does not copy live actor memory. Each affected Region is lowered to a project-owned
`RegionRecoveryPoint`, encoded, decoded, checksum-verified through `RegionHandoffState`, installed
under a strictly newer generation, and then placed on the survivor. Messages emitted by the failed
generation are rejected after recovery. The recovered and uninterrupted clusters converge because
generation is fencing metadata rather than semantic gameplay input.

## Scope

This workload proves the runtime substrate, transport, barrier, recovery, and overload contracts. It
does not claim that the later Minecraft gameplay denominator is implemented. Phase 3 and subsequent
generated implementation batches put their protocol and gameplay semantics through these already
locked Region boundaries.
