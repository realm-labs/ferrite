# G04-P3-B1 — Authoritative voxel collision

## Outcome

The formal Minecraft gateway no longer injects `FlatWorldCollision`. Before a player movement
packet can mutate or route session state, it captures an immutable collision scene for the swept
player volume from the same committed, `FULL`, accessible `ChunkColumn` snapshots used by Java
terrain projection. The capture finishes before the router is mutably borrowed, so movement
validation observes one stable pre-command authority view.

The shared gameplay collision kernel now exposes a scene-backed `CollisionWorld`. It derives the
0.6-by-1.8 player box, clips Y before horizontal axes, selects improving steps up to 0.6 blocks,
detects support and nearby blocks below, and distinguishes newly introduced intersections. The
existing movement validator then preserves its speed, residual, floating, correction, command,
and Region-transfer ordering.

## Shape and failure boundary

The current formal overworld generator has a closed three-state vocabulary: air has no collision;
stone and grass block use full-cube shapes. Any future non-air state without a narrower shape is
conservatively a full cube until its state-specific shape is integrated. This prevents new content
from silently becoming passable.

Collision capture is bounded to 65,536 queried cells. Non-finite or oversized sweeps, missing or
inconsistent authoritative chunks, and movement outside the configured build-height range produce
an unavailable collision probe. That probe admits no displacement and forces normal movement
correction rather than interpreting absent authority as air.

Entity and world-border shapes remain later Goal 05/06 consumers of the same ordered scene; P3-B1
closes the Goal 04 voxel/block-state responsibility.

## Verification

- Gameplay tests prove falling, support, wall clipping, introduced-collision detection, and the
  existing Y-first/step selection behavior.
- Runtime adapter tests prove generated state lookup, wall clipping, missing-chunk closure, and
  pre-iteration rejection of oversized or non-finite queries.
- Player-session, serverbound dispatch, formal network-entry, adversity, and protocol-conformance
  regressions pass with the authoritative adapter installed.
- Universal Rust, source-policy, production-manifest, and diff gates run before commit.
