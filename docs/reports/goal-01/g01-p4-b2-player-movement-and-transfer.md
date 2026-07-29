# G01-P4-B2 Player Movement and Region Transfer Report

## Result

`G01-P4-B2` implements player spawn, the four Java 26.2 player-movement variants, movement and
collision admission, terrain-ready gating, client tick-end handling, keepalive, teleport
correction and acknowledgement, Play disconnect, and explicit generation-fenced Region transfer.

The ownership and transaction contract is recorded in
[Player Movement and Region Transfer](../../development/player-movement-and-region-transfer.md).

## Source-locked behavior

The batch preserves the reviewed boundaries:

- movement omission retains current position or rotation and only flag bits 0/1 are semantic;
- invalid values are checked before the 60-tick client-load and pending-teleport gates;
- position infinities clamp while NaN disconnects; rotations must be finite and wrap in degrees;
- packet frequency above five uses multiplier one; displacement limits are `100N` and `300N`;
- the `0.0625` residual check intentionally ignores every vertical residual through the locked
  Java OR defect;
- accepted movement snaps to the packet target and installs its ground/collision flags;
- client tick end zeroes known movement only when its interval contained no accepted movement;
- floating timeout scales from 80 ticks by gravity and disables below gravity `1e-5`;
- teleport IDs wrap only `i32::MAX -> 0`, stale acknowledgements are ignored, duplicate matching
  acknowledgements disconnect, and resend age is strictly greater than 20 listener ticks;
- non-owner keepalive uses two exact 15-second windows and an exact signed-long echo;
- player ownership and chunk recenter do not change before the cross-Region tick commits.

## Automated evidence

Focused tests cover:

- load-gate expiry/idempotence, rotation wrapping, known-movement intervals, invalid values,
  pending corrections, passenger/sleep/correction mutation, clamp behavior, packet-frequency
  fallback, residual-Y compatibility, collision rejection, and floating timeout;
- deterministic full player-state transfer encoding and malformed state refusal;
- exact serverbound IDs and bodies for the four movement forms, load, tick end, feedback, and
  keepalive, including ignored high flag bits and trailing data;
- clientbound keepalive, vehicle correction, player rotation, and exceptional floats;
- Play driver movement events, pending-teleport propagation, acknowledgement, weighted latency,
  disconnect framing, and close-after-send behavior;
- Region command spawn, same-Region ECS projection, routing rollback, transfer admission,
  dual-generation fencing, typed target materialization, committed receipts, and owner switch only
  after commit.

The acceptance commands are:

```text
cargo test -p ferrite-gameplay
cargo test -p ferrite-protocol
cargo test -p ferrite-region-runtime
cargo test -p ferrite-server-runtime
cargo ferrite task check
git diff --check
```

## Coverage boundary

This fixed integration batch does not advance generated gameplay-slice or protocol-family
counters. Those counters move only when their manifest-owned batches prove every required member.
In particular, the collision-admission path consumes an authoritative geometry probe but does not
claim the complete generic `PLY-COLLISION-001` swept-shape and step algorithm.

Block interaction and prediction correction remain `G01-P4-B3`; local/Lattice trace equivalence
and an unmodified-client C2 acceptance trace remain `G01-P4-B4` and `G01-P4-B5`.
