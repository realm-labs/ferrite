# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-MOVE-SPECIAL-001` — Fluid, swimming, fall-flying and ability-flight dynamics remain separate modes

**Parent:** `PLY-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — the last audited boundary is `Player#travel`/`LivingEntity#travel` mode
dispatch. The complete `travelInFluid`, `travelFallFlying`, swimming-look steering,
depth-strider/dolphin-grace modifiers, elytra durability/collision damage and ability-flight input
branches have not yet been transcribed; this is an open source-audit item, not permission to infer
them.

**Applies when:**

The travel dispatcher selects water, lava, another standable/affecting fluid, swimming steering,
fall flying or player ability flight.

**Authoritative state:**

Medium membership/height/flow, pose/look/input, velocity, abilities, effects, attributes, equipment
and collision state.

**Transition and ordering:**

Preserve the mode dispatch described by `PLY-MOVE-001`; do not reuse ordinary-air constants for a
selected special mode. The detailed transition is intentionally blocked pending its ledger slice.

**Branches and aborts:**

Every special mode named in SourceConclusion, medium boundary crossings, grounded transitions and
equipment loss.

**Constants and randomness:**

Unknown constants are owned by the source audit; no placeholder value is normative.

**Side effects:**

Position/velocity via `PLY-COLLISION-001`, pose, fall state, equipment durability, sounds/events and
particles as selected by the audited mode.

**Gates:**

Fluid tags/heights, `isAffectedByFluids`, `canStandOnFluid`, swimming/fall-flying flags, ability
flight, effects and equipment.

**Boundary cases and quirks:**

Ordinary-air behavior must not silently receive a special-mode entity merely because its trajectory
looks similar.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`; last boundary:
`net.minecraft.world.entity.player.Player#travel(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.LivingEntity#travelInFluid(net.minecraft.world.phys.Vec3)`, and
`net.minecraft.world.entity.LivingEntity#travelFallFlying(net.minecraft.world.phys.Vec3)`.

**Test vectors:**

`EXP-PLY-004` must replay still/flowing water and lava boundaries, upward/downward swimming look,
shallow exit, depth-strider/effect levels, elytra pitch/speed/collision/equipment break, and
creative/spectator flight transitions while capturing every tick's velocity, pose and side effects.
