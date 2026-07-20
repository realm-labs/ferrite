# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-DESPAWN-001` — Persistence and distance checks choose immediate removal, random removal, or retention

**Parent:** `MOB-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — exact thresholds, random cadence, and every persistence override remain
unexpanded.

**Applies when:**

A mob's server tick reaches despawn checking.

**Authoritative state:**

Persistence-required flag, custom persistence conditions, nearest eligible player distance,
category/type despawn distances, `NoAI`/special flags, difficulty and RNG.

**Transition and ordering:**

If peaceful removal applies, discard through that path; otherwise test persistence; find nearest
relevant player; if beyond hard distance and type may despawn, discard immediately; if beyond random
distance and inactivity/random check succeeds, discard; if near enough, reset no-action timer.

**Branches and aborts:**

No player; persistent because named/tamed/leashed/passenger/picked-up or subtype rule; type cannot
despawn; hard distance; random-distance roll; near reset; special dimension/difficulty removal.

**Constants and randomness:**

Distances and random interval/chance are mob-category/type methods, measured by squared distance.
Random despawn consumes mob RNG only when that branch is reached. Exact thresholds and no-player
behavior are `EXP-MOB-003`.

**Side effects:**

Removal/untracking without death loot, reset inactivity timer, passenger/leash cleanup and client
removal.

**Gates:**

Entity ticking, persistence, nearest eligible non-spectator player, type despawn policy, distance,
difficulty and special states.

**Boundary cases and quirks:**

Despawn is discard, not death. Horizontal and 3D distance must follow the source calculation. A
loaded but non-ticking mob does not run this check.

**Evidence:**

`Confirmed` branch model; thresholds `Cross-checked`; `OFF-SERVER-001`; `Mob#checkDespawn()` family;
`EXP-MOB-003`.

**Test vectors:**

Exact squared-distance boundaries, named/tamed/leashed/passenger, no players, spectator only,
peaceful hostile, inactive chunk and fixed-RNG random despawn.
