# Full Goal 01 acceptance

`cargo ferrite acceptance verify` is the clean-checkout acceptance entry point for Goal 01. It
refuses a dirty Git worktree before and after execution, then runs:

1. the named cross-platform canonical/RNG/hash vector;
2. local, in-process Lattice, and process-isolated playable state/packet equivalence;
3. the same topology set through a canonical `ReplayLog` and `ReplayTarget`;
4. the exact locked Minecraft Java 26.2 client JAR SHA-1 and registry/tag fixture projection;
5. the complete repository task, including architecture, source, deployment, capacity, topology,
   protocol, workspace-test, reference, and implementation-manifest gates;
6. final diff and worktree-clean checks.

The ignored reference cache is an input, not part of the checkout. A clean worktree must provide
the locked `target/mc-reference/26.2` artifacts described by the reference lock. The acceptance
command neither downloads nor commits Mojang artifacts.

## Portable deterministic vector

The `ferrite-cross-platform-vectors-v1` integration test combines explicit little-endian fixed
width values, minimal unsigned LEB128, finite IEEE-754 bit patterns, UTF-8, the locked xoshiro
sequence, a negative-coordinate Region identity, and a canonical Region hash. Its 167-byte input
has BLAKE3 digest:

```text
11d18ab3881d50117cab7211fd9bd41355a4b7009843a908520e3ba6e4b4d1ba
```

CI runs this exact test on `ubuntu-latest`, `macos-latest`, and `windows-latest`. A platform is not
reported as passing until its job has executed successfully.

## Canonical playable replay

The replay gate records the seven-tick C2 scenario into a bounded canonical log, round-trips its
bytes, and feeds the same log to local and Lattice-backed `ReplayTarget` implementations. Three
additional process-isolated Lattice workers must return identical log and final-state evidence.
The locked current evidence is:

```text
frames=1
bytes=2586
log=a000f4dc4182c89ed2410827f4e971a30dd3a00eabffae0d61150b83b71ab7cd
state=1e7c50dbf4463c858fcd779f4db59a08418e54cab7ae0e502821bba95ad0a858
```

## Unmodified-client boundary

The unattended gate verifies the exact 39,193,383-byte client JAR with SHA-1
`2dc72797acbc1b63fc16a11c4ac393605f453754`, reconstructs the complete synchronized registry/tag
fixture, and runs independent C0-C3 protocol/session semantics. It does not label a headless client
or artifact check as a fresh graphical-client observation.

The operator-assisted `vanilla-c2-probe` remains the exact-client gate. Its prior Windows x86-64
26.2 observation is committed in the [C2 acceptance report](../reports/goal-01/g01-p4-b5-c2-acceptance-and-adversity.md).
A final rerun requires an installed official launcher/client and a valid local session; the probe
records no success evidence until the real client completes login, configuration, terrain batch,
teleport acknowledgement, player loaded, movement, and tick end.
