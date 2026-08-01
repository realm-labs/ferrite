# Entity-Service Conformance (Historical Goal 01 Phase 7)

`G01-P7-B2` closes Phase 7 with deterministic conformance over the production entity Region
runtime and the already verified entity, combat, mob, AI, spawning, and protocol partitions.

## Coverage boundary

The frozen implementation manifest assigns 56 gameplay slices and seven required C3 protocol
families to Phase 7. All are independently verified by their generated batch owners. The frozen set
of ten behavior-surface roots and 36 cross-system joins assigns no root or join owner to
`G01-P7-B2`; this batch therefore does not invent an additional denominator or relabel a later
phase's owner.

Instead, `ferrite-testkit::entity_service` supplies a dedicated stage-level conformance harness. It
drives `ferrite-server-runtime::entity_service` directly and combines the independently verified
gameplay and protocol results with executable Region lifecycle, projection, transfer, persistence,
fault, and replay evidence.

## Golden client projection trace

The golden scenario observes one entity across:

- source spawn and mutation;
- deactivation and reactivation;
- source removal during transfer preparation;
- target spawn after transfer acceptance;
- target mutation, deactivation, reactivation, and despawn.

Duplicate target delivery returns the durable applied receipt and emits no second spawn. The ten
client-facing semantic projection events are locked in exact order by BLAKE3 digest
`28deb222fdc6efac437eb4b79944dd8ebcbb7467025b2431db7c25f5e82cbaaa`.

## Property, fuzz, and transfer suites

The ordering property suite builds equivalent Regions with opposite entity insertion orders for
128 stable-ID sets. Canonical snapshot records and observer-join projections must remain identical
and ordered by stable entity ID.

The fixed-seed operation suite executes 256 cases with eight lifecycle-aware mutations,
deactivations, and activations against two independent runtimes. Every result, canonical snapshot,
and observer trace must agree.

The transfer suite executes 64 entities across adjacent Regions. Each accepted target state must
equal the directly constructed candidate state, including chunk, payload, revision, command
sequence, kind, and active lifecycle. The acknowledged source must be empty.

## Fault and replay suites

Ten fail-closed vectors cover zero limits, duplicate entity insertion, wrong Region, stale
generation, command-sequence gaps, revision mismatch, wrong chunk ownership, observer
backpressure atomicity, stale target generation, and malformed continuity records.

Eight replay frames encode fixed entity seeds in the repository replay envelope. Each frame
reconstructs production Region state, performs four deterministic mutations, and hashes canonical
snapshot records. The normal target converges and an intentionally perturbed target reports first
divergence.

## Executable entry point

The machine-owned test entry point is:

- `apps/behavior-runner/tests/entity_service_conformance.rs`.

It invokes `ferrite-testkit::entity_service::entity_conformance` and locks all counts and the golden
digest. This development-document filename and the Goal 01 phase terminology above remain stable
historical provenance; active test ownership uses responsibility names.
