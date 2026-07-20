# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-VEHICLE-001` — Vehicle control, physics, collision, and passenger placement are server-owned

**Parent:** `ENT-002`, `ENT-003`, `PLY-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — boat/minecart family constants, collision traversal, input transfer, and
passenger placement remain unexpanded.

**Applies when:**

A boat/raft/minecart family entity is ticked, controlled, collided, mounted, or dismounted.

**Authoritative state:**

Vehicle position/velocity/rotation/status, controlling passenger/input, passenger order,
track/fluid/ground context, damage/hurt state and interpolation target.

**Transition and ordering:**

Determine control source and environment status; apply propulsion or rail logic; move through entity
collision; resolve vehicle/entity pushes; update rotation/status; position every passenger from the
committed vehicle transform in passenger order; validate dismount location on exit.
Client-controlled prediction is reconciled to server accepted motion.

**Branches and aborts:**

No controller; non-player controller; underwater/air/land status; powered/activator rail;
derailment; collision; destroyed vehicle; passenger cannot ride; dismount pose has no collision-free
location.

**Constants and randomness:**

Acceleration, drag, max speed, buoyancy and rail projection are family/source constants with
double/float rounding. Generic movement consumes no RNG. Exact numeric trajectories are
`EXP-ENT-004`.

**Side effects:**

Vehicle/passenger movement, collisions/pushes, fall or impact consequences, block/rail callbacks,
damage/drops, sounds/particles/game events, chunk tracking and corrections.

**Gates:**

Controller identity, vehicle family/status, input, rail/fluid state, collision, riding
permission/cooldown, entity-ticking chunk and gamerules for drops.

**Boundary cases and quirks:**

Passenger position derives after vehicle motion and is not independent player movement. Dismount
searches legal poses/locations. Minecart and boat physics are different families despite shared
riding semantics.

**Evidence:**

`Confirmed` ownership/state order; numeric parity `Implementation blocker`; `OFF-SERVER-001`,
`OFF-CLIENT-001`; `EXP-ENT-004`.

**Test vectors:**

Empty/controlled boat on land/water/air; two passengers; collide with entity/wall; minecart
slopes/curves/powered rails; unload; destroy while occupied; dismount with only one legal pose.
