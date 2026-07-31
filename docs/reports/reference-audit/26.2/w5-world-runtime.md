# Minecraft Java 26.2 world-runtime reference audit

## Scope and guardrails

This worker audited `WGEN-DIMENSION-001`, `WGEN-PORTAL-001`, `WGEN-BORDER-001` and the
`WGEN-PIPELINE-EQUIVALENCE-001` completion slice from clean HEAD
`feba9fac70272c8eaa4a87ea10aacb430b34294b` on `codex/ref-world-runtime`. Evidence was limited to
the repository-locked official client/server jars, bundled data, generated reports, existing
reference documents and `mc-ref`. No Ferrite runtime code or implementation disposition was
changed, and no implementation disposition is claimed Verified by this audit.

Locked artifacts inspected:

- server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`;
- client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`;
- official dimension, timeline, clock and outside-border damage/tag data inside the locked server
  jar.

## Material findings

### `WGEN-DIMENSION-001`

The dimension records, environment layering, scale direction, key/type splits, build/logical
heights, spawn RNG order and bed/anchor positional gates remained source-supported. The audit added
missing clock mutation and persistence boundaries:

- global clock saved state retains total, partial, rate and paused state;
- absolute set resets partial, signed add preserves partial and clamps the result to zero, and a
  successful marker move resets partial;
- `/time rate` admits `0.00001..1000`, while the manager setter itself stores the supplied float;
- even a missing marker runs the common broadcast/dirty/cache-invalidation wrapper before the
  command reports failure;
- an `advance_time` manager tick dirties saved data even when every clock is individually paused.

Primary bytecode entry points were
`net.minecraft.world.clock.ServerClockManager#tick()`,
`net.minecraft.world.clock.ServerClockManager#setTotalTicks(net.minecraft.core.Holder,long)`,
`net.minecraft.world.clock.ServerClockManager#addTicks(net.minecraft.core.Holder,int)`,
`net.minecraft.world.clock.ServerClockManager#moveToTimeMarker(net.minecraft.core.Holder,net.minecraft.resources.ResourceKey)`,
`net.minecraft.world.clock.ServerClockManager#modifyClock(net.minecraft.core.Holder,java.util.function.Consumer)`
and
`net.minecraft.server.commands.TimeCommand#addClockNodes(net.minecraft.commands.CommandBuildContext,com.mojang.brigadier.builder.ArgumentBuilder,net.minecraft.server.commands.TimeCommand$ClockGetter)`.

### `WGEN-PORTAL-001`

Cooldown, pre-increment wait comparison, search radii/ranking, creation geometry, safe exit
placement, gateway search, and passenger-before-root transfer order remained source-supported. The
audit made the persistence split explicit:

- entity NBT saves `PortalCooldown` but does not save the live `PortalProcessor` or accumulated
  contact time;
- ordinary cross-dimension `Entity#restoreFrom` explicitly copies both cooldown and processor
  after generic save/load restoration, while same-level and server-player transfers keep the same
  entity instance;
- a disk reload therefore retains cooldown but loses contact accumulation; gateway age, exit and
  exactness persist while its block-entity cooldown does not.

Primary bytecode entry points were
`net.minecraft.world.entity.Entity#setAsInsidePortal(net.minecraft.world.level.block.Portal,net.minecraft.core.BlockPos)`,
`net.minecraft.world.entity.Entity#handlePortal()`,
`net.minecraft.world.entity.Entity#restoreFrom(net.minecraft.world.entity.Entity)`,
`net.minecraft.world.entity.Entity#teleport(net.minecraft.world.level.portal.TeleportTransition)`,
`net.minecraft.world.entity.PortalProcessor#processPortalTeleportation(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.Entity,boolean)`,
`net.minecraft.world.level.portal.PortalForcer#findClosestPortalPosition(net.minecraft.core.BlockPos,boolean,net.minecraft.world.level.border.WorldBorder)`
and
`net.minecraft.world.level.portal.PortalForcer#createPortal(net.minecraft.core.BlockPos,net.minecraft.core.Direction$Axis)`.

### `WGEN-BORDER-001`

The tick-before-entity phase, previous/current interpolation split, geometry tolerances, collision
projection, damage formula, warning projection and client snapshots remained source-supported. The
audit added the omitted persistence exception: saved settings retain center, damage, safe zone,
warnings, current calculated size, remaining ticks and target, but not the previous geometry sample
or `absoluteMaxSize`. Direct `setAbsoluteMaxSize` neither dirties saved data nor calls listeners, so
server reload restores the default `29,999,984`; client initialization still synchronizes the live
value.

Primary bytecode entry points were
`net.minecraft.world.level.border.WorldBorder#setAbsoluteMaxSize(int)`,
`net.minecraft.world.level.border.WorldBorder#applyInitialSettings(long)`,
`net.minecraft.world.level.border.WorldBorder$Settings#<init>(net.minecraft.world.level.border.WorldBorder)`,
`net.minecraft.world.level.border.WorldBorder$MovingBorderExtent#update()` and
`net.minecraft.world.entity.LivingEntity#baseTick()`.

### `WGEN-PIPELINE-EQUIVALENCE-001`

The slice correctly remains `SourceInconclusive`. `EXP-WGEN-001` must calibrate predeclared
player-visible metrics on fixed calibration and untouched held-out populations; neither the fidelity
class nor source inspection implies same-seed block-for-block identity. `EXP-WGEN-005` and
`EXP-WGEN-006` are exact source/data conformance matrices for placed features and flat generation,
respectively, and cannot select or close the quantitative equivalence tolerances owned by
`EXP-WGEN-001`.

## Experiment disposition

All six named WGEN experiments remain explicit and unexecuted:

- `EXP-WGEN-001` remains the unresolved statistical equivalence calibration/held-out experiment;
- `EXP-WGEN-002`, `EXP-WGEN-003` and `EXP-WGEN-004` remain conformance/observation matrices for the
  three source-specified leaves;
- `EXP-WGEN-005` and `EXP-WGEN-006` remain exact source/data conformance matrices and are not
  substitutes for the `EXP-WGEN-001` equivalence decision.

No fact that required those observations was guessed or promoted from `planned` status.

## Verification

The generated report refresh completed successfully with:

```text
MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
  cargo run -q -p mc-reference --bin mc-ref -- reports
```

The complete offline verifier passed with:

```text
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
  cargo run -q -p mc-reference --bin mc-ref -- verify --offline
```

It verified 417 documentation IDs including 352 leaves; 331 completion slices; 2,789 locators
across 952 classes; 9,078 locked IDs with zero unclassified or ambiguous; all 307 experiment
definitions; all 256 protocol packets; command-root, cross-system-join and behavior-surface
coverage; and the implementation manifest. `git diff --check` also passed. SHA-1 verification of
the inspected locked jars returned:

```text
823e2250d24b3ddac457a60c92a6a941943fcd6a  target/mc-reference/26.2/server.jar
2dc72797acbc1b63fc16a11c4ac393605f453754  target/mc-reference/26.2/client.jar
```

No Rust source changed, so the documentation-only exemption in `AGENTS.md` applies to Rust format,
Clippy and crate tests. The pre-existing `wgen-pipeline-001.md` is a 7,478-line consolidated legacy
leaf; this audit replaces four lines with four lines and does not grow it. Splitting that rule would
require schema/index changes explicitly outside this worker's allowed edit scope; it should be
handled as a dedicated repository follow-up.
