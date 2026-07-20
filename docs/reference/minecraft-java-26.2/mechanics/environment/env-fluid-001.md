# Environment mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENV-FLUID-001` — Fluid propagation recomputes local state through scheduled ticks

**Parent:** `ENV-001`, `ENV-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server control flow fixes all five fluid IDs, level mappings, tick
scheduling/recomputation, downward and tied horizontal selection, wall/container admission,
water/lava constants and source rules, every mixing product/direction, fast-lava and
water-evaporation attributes, lava delay/fire RNG, writes and failures. `EXP-ENV-001` is a
conformance trace, not an implementation blocker.

**Applies when:**

A liquid block is placed or receives a neighbor/shape update, a water/lava scheduled tick is
admitted, a liquid tries to enter a block/container, a lava block checks neighboring water/basalt
catalysts, lava receives a random tick, or a water-bearing bucket reaches its environment
evaporation gate.

**Authoritative state:**

The position's live block/fluid state; source versus flowing family; amount 1–8 and `falling`; block
legacy level 0–15; neighbor collision shapes/fluids; `LiquidBlockContainer` admission; scheduled
block/fluid queues; `water_source_conversion` (default true), `lava_source_conversion` (default
false); non-positional synchronized `gameplay/fast_lava` (default false); synchronized positional
`gameplay/water_evaporates` (default false); random-tick admission, gameplay RNG, chunk loading and
the fire-spread-radius gate.

**Transition and ordering:**

A liquid block placement or neighbor callback first performs the lava mixing transaction below; only
if it returns true does it schedule its current fluid type at the subtype delay. Shape updates
schedule when either the current or changed-neighbor fluid is a source. During a normally running
level tick, up to 65,536 due block ticks execute before up to 65,536 due fluid ticks. A due
nonsource fluid reads the live block state, computes `new`, computes its spread delay, then writes
air if `new` is empty or writes/schedules `new` when identity differs; source states skip
recomputation. In every case it next spreads the resulting state. Writes use update flags 3 and
ignored boolean results; callbacks caused by each write may enqueue later work but do not rewind the
current transaction.

- **State encoding and constants:** Source amount is 8. Flowing amount is its `level` 1–8; own
  height is `amount/9`, or full height 1 when the same family is above. Legacy liquid-block level is
  0 for source, otherwise `8-min(amount,8)+(falling?8:0)`; reading a block level clamps indices 8–15
  to falling amount 8. Water has drop-off 1, slope-search range 4 and tick delay 5. Lava reads
  `gameplay/fast_lava`: false gives drop-off 2/range 2/delay 30; true gives 1/4/10. If old and new
  lava are nonempty, both nonfalling, and new height is greater, one `nextInt(4)` is consumed and a
  nonzero result multiplies the delay by four.
- **Local recomputation:** Scan horizontal neighbors in `NORTH,EAST,SOUTH,WEST`, accepting
  same-family fluid only through nonoccluding joined collision faces; retain maximum amount and
  count admitted sources. With at least two sources and the matching conversion rule, return a
  source only when the block below is solid or contains a same-family source. Otherwise, admitted
  same-family fluid above produces falling amount 8. Otherwise return nonfalling
  `maxHorizontalAmount-dropOff`, or empty at zero/below. Full collision shapes reject passage
  immediately; two empty shapes pass; other static pairs use the joined-face occlusion result
  (cached only as an optimization).
- **Downward then horizontal spread:** First test below for non-source target, holdability, face
  passage and target replacement. If admitted, compute the below state's local result and commit
  downward; only when the origin has at least three horizontal source neighbors also spread
  sideways, then return. If downward is unavailable, a source spreads sideways; a nonsource does so
  only when below is not an open/same-family/holdable hole. Falling origins use side amount 7;
  others require `amount-dropOff > 0`. For each admissible horizontal target, compute its
  prospective local state and its nearest downward hole: direct holes cost 0; otherwise recursively
  scan `NORTH,EAST,SOUTH,WEST`, never immediately backtracking, through at most the subtype range.
  Keep every direction tied at the minimum. The result is stored in an `EnumMap`, so commits occur
  `NORTH,SOUTH,WEST,EAST`, with live target state reread at each commit.
- **Replacement and containers:** A destination must not already be a same-family source, must be
  generally holdable, must pass the joined collision face, must accept the incoming fluid according
  to its current fluid state's replacement rule, and if it implements `LiquidBlockContainer`, must
  also accept `canPlaceLiquid(null,...)`. The 429 registered `SimpleWaterloggedBlock` IDs accept
  only the exact `WATER` source. When not already waterlogged, server `placeLiquid` sets
  `waterlogged=true` with flags 3 and schedules that water state at delay 5; it returns true
  client-side without a write. The four intrinsically water-filled aquatic blocks (`kelp`,
  `kelp_plant`, `seagrass`, `tall_seagrass`) reject both admission and placement. A noncontainer
  target drops its block resources before water replacement, fizzes before lava replacement, then
  receives the incoming legacy liquid state; container `placeLiquid`'s boolean result is ignored
  after admission. Doors, signs, ladder, sugar cane, bubble column, both portal blocks, end gateway
  and structure void are explicitly not generally holdable even when non-motion-blocking.
- **Replacement asymmetry:** Water fluid is replaceable only from `DOWN` by a non-water fluid. Lava
  is replaceable by water from any direction only when its current height is at least `4/9`.
  Consequently water can overwrite sufficiently deep lava through the generic path without creating
  stone; this is distinct from the lava-owned reaction hooks.
- **Lava mixing transactions:** Before a lava liquid block schedules, examine
  `UP,NORTH,SOUTH,WEST,EAST` in that order (the stored direction list is
  `DOWN,SOUTH,NORTH,EAST,WEST` and each lookup uses its opposite). The first water-tagged neighbor
  replaces the current lava block with obsidian when the current lava is source, otherwise
  cobblestone, emits level event 1501 and aborts scheduling. If soul soil is directly below, the
  first blue-ice neighbor in the same scan instead creates basalt, emits 1501 and aborts. During
  actual downward lava spread into a water-tagged target, create stone only if the target block
  itself is a `LiquidBlock`; regardless, emit 1501 and return without generic placement. Horizontal
  lava-to-water does not use this stone hook. Failed block writes are not rolled back.
- **Lava random fire:** Every lava fluid state is random-ticking, but the callback first requires
  `ServerLevel#canSpreadFireAround(origin)`. Draw `k=nextInt(3)`. For `k` 1 or 2, walk cumulatively
  upward `k` times with independent `x,z=nextInt(3)-1`; an unloaded position aborts, the first air
  position with any `ignitedByLava` neighbor receives base fire and returns, and a motion-blocking
  nonair position returns. For `k=0`, make three independent horizontal offsets with the same two
  draws; each loaded base whose above block is empty and whose own state is `ignitedByLava` writes
  base fire above, so up to three fires may be placed. Unloaded probes abort the whole callback.
- **Water evaporation boundary:** `BucketItem#emptyContents` checks the positional
  `gameplay/water_evaporates` only after target/recursive placement admission and only for
  water-tagged content. True consumes two floats for extinguish-sound pitch and three floats for
  each of eight large-smoke particles, returns success, and performs no fluid write, ordinary empty
  sound or `FLUID_PLACE` event. Bucket stack/stat consequences are owned by `ITM-USE-001`; this leaf
  owns the environment predicate and no-write outcome.

**Branches and aborts:**

Source recomputation skip; unchanged identity; empty result; nonpositive side amount; same-family
source target; full/joined face occlusion; explicit unholdable block; container denial; target
replacement denial; no shortest candidate; mixing before scheduling; failed writes;
inactive/unloaded scheduled tick; fire-radius denial; unloaded random-fire probe; bucket
recursion/admission failure before evaporation.

**Constants and randomness:**

Amounts 1–8; legacy levels 0–15; water 1/4/5; ordinary lava 2/2/30; fast lava 1/4/10; tick cap
65,536 per queue; bubble-column block check delay 20; mixing event 1501. Propagation itself is
deterministic except the one lava slowdown draw. Lava random fire consumes exactly the
branch-dependent draws above; client-only ambient fluid sound/particles belong to `CLI-EFFECT-001`.

**Side effects:**

Block/fluid writes, scheduled fluid/block ticks, drops from water displacement, lava fizz events,
waterlogging, obsidian/cobblestone/basalt/stone production, bucket evaporation sound/smoke, and
lava-created fire. Entity current/collision and fluid contact effects are owned by
`PLY-MOVE-SPECIAL-001`/`ENT-EFFECT-001`; bubble-column evolution after its scheduled block callback
is block-owned.

**Gates:**

Normal level ticking, debug-world exclusion, active tick chunks, subtype and amount, collision
faces, destination block/container and target replacement rule, source-conversion rules,
fast-lava/water-evaporation attributes, liquid mixing inputs, random-tick admission, fire-spread
radius and chunk loading.

**Boundary cases and quirks:**

Empty block and empty fluid differ; waterlogged states preserve their block. Equal shortest slopes
all receive fluid, not one random winner. Candidate scan order differs from tied commit order. A
falling column can spread level 7 sideways. Two sources need solid/same-source support below.
Downward lava fizzes without stone when the water belongs to a non-`LiquidBlock` container. Water
can generically replace deep lava without a mixing product. Missed unloaded ticks resume from queued
current state and do not simulate wall time.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.material.FlowingFluid#tick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState)`,
`net.minecraft.world.level.material.FlowingFluid#getNewLiquid(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.level.material.FlowingFluid#spread(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState)`,
`net.minecraft.world.level.material.FlowingFluid#getSpread(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.level.material.FlowingFluid#spreadTo(net.minecraft.world.level.LevelAccessor,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.core.Direction,net.minecraft.world.level.material.FluidState)`,
`net.minecraft.world.level.material.WaterFluid`,
`net.minecraft.world.level.material.LavaFluid#getSpreadDelay(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.material.FluidState,net.minecraft.world.level.material.FluidState)`,
`net.minecraft.world.level.material.LavaFluid#randomTick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.material.FluidState,net.minecraft.util.RandomSource)`,
`net.minecraft.world.level.block.LiquidBlock#shouldSpreadLiquid(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.level.block.SimpleWaterloggedBlock`,
`net.minecraft.world.item.BucketItem#emptyContents(net.minecraft.world.entity.LivingEntity,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.server.level.ServerLevel#tick(java.util.function.BooleanSupplier)`,
`net.minecraft.world.level.gamerules.GameRules`,
`net.minecraft.world.attribute.EnvironmentAttributes`, and locked fluid/block/tag reports.

**Test vectors:**

Source/amount/falling matrix; two-source conversion with solid/source/open support and each rule
value; downward open/denied/container path; every equal/unequal slope and range boundary; all
waterlogged/simple/aquatic cases; lava delay draw 0/1/3 under both fast values; water
above/horizontal/below lava at each amount; source/flowing obsidian/cobble, basalt catalyst order,
downward plain/waterlogged water stone quirk; water-over-deep/shallow-lava asymmetry; lava fire
`k=0/1/2`, unloaded/radius/motion gates; evaporating bucket RNG and no-write/event boundary;
unload/reload scheduled work.
