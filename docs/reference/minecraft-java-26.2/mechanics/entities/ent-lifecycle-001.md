# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-LIFECYCLE-001` — Entity insertion, ticking, passenger traversal, transfer, and removal have explicit ownership

**Parent:** `ENT-001`, `ENT-002`, `ENT-008`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — insertion/removal during iteration, passenger mutation, transfer flags, and
failure rollback remain unexpanded.

**Applies when:**

An entity is created, loaded, added to a level, ticked, changes dimensions, mounts/dismounts, dies,
or is discarded.

**Authoritative state:**

UUID/numeric ID, owning level/chunk section, position/velocity/rotation, removal reason,
passenger/vehicle graph, portal cooldown and tracked gameplay data.

**Transition and ordering:**

Validate unique insertion and chunk ownership; add to section/tracking callbacks; during level tick
process only non-passenger roots and recursively tick passenger trees through the vehicle; commit
section moves when coordinates cross boundaries; removal marks a reason and invokes
untracking/section callbacks exactly once. Dimension transfer removes from the old level and
creates/repositions the authoritative entity in the destination according to the transfer path.

### Anchors

`net.minecraft.server.level.ServerLevel#addEntity(net.minecraft.world.entity.Entity)`,
`net.minecraft.server.level.ServerLevel#tickNonPassenger(net.minecraft.world.entity.Entity)`, and
`net.minecraft.world.entity.Entity#remove(net.minecraft.world.entity.Entity$RemovalReason)`.

**Branches and aborts:**

Duplicate UUID; already removed; passenger handled by vehicle; destination chunk unavailable;
change-dimension denied; player-specific transfer; death versus discard/unload removal reason. An
entity removed during its callback must not receive later ordinary tick work.

**Constants and randomness:**

Entity type dimensions/tracking data come from the registry/report. Generic ownership transitions
consume no RNG. UUID creation may use randomness only at construction if not supplied.

**Side effects:**

Chunk section/tracker membership, passenger links, scoreboard/team references, leash relations,
item/XP drops on death, criteria/game events, sounds/particles and client add/remove/correction
updates.

**Gates:**

Chunk entity-ticking status, removal state, passenger root status, portal cooldown, destination
rules, peaceful/difficulty removal for mobs and entity-type feature flags.

**Boundary cases and quirks:**

Loaded is not entity-ticking. Passengers do not also run as independent roots. Unload removal must
not cause death drops. Dimension transfer identity semantics differ for players and ordinary
entities.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; locators above; unload/transfer graph `EXP-ENT-001`.

**Test vectors:**

Remove during tick; passenger tree three levels deep; unload/reload; duplicate UUID insertion; cross
chunk section during movement; portal transfer with leash/passengers and verify no duplicate ticking
or drops.
