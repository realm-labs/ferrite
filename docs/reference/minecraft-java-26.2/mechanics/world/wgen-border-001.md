# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-BORDER-001` — World border is a tick-interpolated geometry used by independent mechanics

**Parent:** `WGEN-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server/client source fixes the executed-tick countdown, phase order,
geometry predicates, damage and warning formulas, packets, reconnect snapshot and render
interpolation; `EXP-WGEN-004` is a conformance probe.

**Applies when:**

A level's border is loaded or mutated, normally ticks, synchronizes to a client, or supplies
geometry to collision, clipping, interaction, spawning, respawn, portals, damage, HUD warning and
rendering. The audited registry endpoint is `minecraft:outside_border`; unrelated callers retain
their own transaction rules.

**Authoritative state:**

Each level owns center `(centerX,centerZ)`, absolute maximum coordinate, a static or moving extent,
damage rate/safe zone, warning distance/time and listeners. A moving extent owns `from`, `to`,
original double duration, signed-long remaining ticks, previous/current size and begin/end game-time
metadata. After initial settings, defaults are center `(0,0)`, size `59,999,968`, absolute maximum
`29,999,984`, damage `0.2`, safe zone `5`, warning distance `5`, warning time `300` ticks and no
active lerp. Server geometry/damage is authoritative; the client owns a synchronized presentation
copy.

**Transition and ordering:**

`setSize(s)` immediately replaces the extent with a static one, marks saved data dirty, then
notifies a copied listener list. `lerpSizeBetween(from,to,duration,begin)` creates a static target
when `from==to`; otherwise it initializes remaining=`duration` and both current/previous size from
progress zero. It then marks dirty and emits one lerp notification even for equal endpoints. Each
normally running `ServerLevel#tick` calls border tick before weather, clocks, chunks and entities.
Moving `update` decrements remaining **first**, copies current to previous, computes
`p=(duration-remaining)/duration`, sets current to `lerp(p,from,to)` while `p<1` else `to`, marks
dirty, and becomes static when remaining `<=0`. Thus positive duration `D` calculates the target on
the Dth executed border update; overload, server freeze and saved-world downtime consume no steps.
On updates `1..D-1`, ordinary geometry still reads copied previous size, while `getSize()` exposes
calculated current size. Update D returns a static target and discards that lag sample, so geometry
changes from step `D-2` directly to the target (for D>1) before that tick's entity phase. Saving
records calculated current size, target and remaining ticks, so reload resumes a fresh interpolation
over exactly that remainder and resets the lag history to its snapshot.

The world-border command's time argument is already an integer tick count: suffix multipliers are
none/`t`=`1`, `s`=`20`, `d`=`24,000`, with float multiplication then `Math.round`. `set ... 0` is
immediate; positive values start from the current size. `add ... time` targets `current+delta` and
uses `currentRemaining+parsedTime`, so it extends/restarts an in-progress motion from its current
size. Command target size must be `[1,59,999,968]`. A direct unequal-endpoint lerp with duration `0`
constructs a moving extent already exposing target (NaN progress selects the target) and becomes
static next tick; a negative duration exposes `from` until its first update returns static target.
Equal endpoints are static immediately for every duration.

**Geometry:**

At partial `q`, moving half-size is `lerp(q,previous,current)/2`. Every no-argument min/max getter
passes `q=0`; all authoritative containment, distance, clamp and collision methods therefore use
**previous** size during an intermediate moving update. Only `getSize()` returns calculated current
size. The force-field extractor passes frame partial tick. Every edge is clamped to
`[-absoluteMax,+absoluteMax]`. Point/radius containment is
`x>=minX-r && x<maxX+r && z>=minZ-r && z<maxZ+r`: minimum inclusive, maximum exclusive. `BlockPos`
tests its integer X/Z origin. A chunk requires both its min and max block origins. An AABB requires
`(minX,minZ)` and `(maxX-ε,maxZ-ε)` with `ε=0.000009999999747378752`, so an exact maximum face is
accepted while its minimum still cannot begin at the exclusive maximum. Distance is the minimum of
`z-minZ`, `maxZ-z`, `x-minX`, `maxX-x`, positive inside and negative outside. Vector clamp preserves
Y and clamps X/Z to `[min,max-ε]`; block clamp then floors through `BlockPos.containing`.

The collision shape is infinity minus the vertical interior box bounded by `floor(minX/minZ)` and
`ceil(maxX/maxZ)`. It is added only when `d=max(1,max(abs(AABB width),abs(AABB depth)))`, center
distance is `<2d`, and the entity center is within the border expanded by `d`; therefore a nearby
outside entity can collide, while a far outside entity receives no automatic wall recovery.
`clipIncludingBorder` changes a ray result only when the ray starts inside and that result ends
outside: it clamps the result, derives the approximate face from travel, and returns a block hit
marked `worldBorderHit`. `findFreePosition`, generic entity collision/path regions and dismount
searches reuse these shapes/predicates. Block interaction, natural placement/spawn types, player
respawn-radius limiting, default-spawn correction, beds, dragon eggs, pistons and portals explicitly
call point/distance/clamp APIs; the border does not globally prohibit every outside block/entity
action.

**Damage and warning:**

On each alive server living-entity base tick, an in-wall hit takes precedence and skips the border
branch. Otherwise only a `Player` whose AABB is not contained continues. Let
`o=distance(playerCenter)+safeZone`. If `o<0` and damage rate `r>0`, submit
`minecraft:outside_border` damage amount `float(max(1,floor(-o*r)))`; normal damage admission may
still reject it. Intermediate ticks use the pre-update geometry despite `getSize()` already
advancing, so the first calculated shrink step past a player does **not** by itself damage. A later
tick can expose that prior edge, while completion exposes the static target immediately and can
damage in its same entity phase. The damage type has exhaustion `0`, message `outsideBorder`,
scaling `when_caused_by_living_non_player`, and tags `bypasses_armor`, `bypasses_wolf_armor`,
`no_knockback`.

The client HUD narrows center distance from the no-argument, previous-edge geometry to float, but
computes `projected=min(lerpSpeed*warningTime,abs(target-currentSize))` from calculated current
size; `threshold=max(warningBlocks,projected)`. When `distance<threshold`, red warning intensity is
`1-distance/threshold`, later clamped to `[0,1]` while blending with the environmental vignette;
outside distances therefore saturate. This is presentation only and does not gate damage.

**Synchronization and rendering:**

Per-level listeners broadcast immediate size, lerp, center and warning packets only to players in
that dimension; damage/safe-zone changes send no client packet. A level-info/reconnect packet
snapshots calculated current size as old size, target, remaining ticks, center, absolute maximum and
warnings; mid-lerp this deliberately omits the server extent's previous geometry sample. The handler
starts the client copy at that snapshot using client game time only as begin metadata.
`ClientLevel#tick` advances its border exactly once only while its tick-rate manager runs normally;
packet delay or independent freeze can make its presentation differ from server truth until another
snapshot/mutation. Force-field extraction uses partial-tick min/max, but its alpha uses no-argument
previous-edge distance: `clamp((1-distance/renderDistance)^4,0,1)` when the camera is in the render
band. The HUD likewise mixes previous-edge distance with calculated current size.

**Branches and aborts:**

Static/equal/moving extent; positive/nonpositive duration; server/client normal-run gate;
min/max/absolute clamp; point/chunk/AABB/radius overload; near/far collision; ray start/end;
player/non-player, in-wall, AABB, safe-zone, positive-rate and damage admission; warning
zero/nonzero threshold; dimension-specific packet audience.

**Constants and randomness:**

`59,999,968`, `29,999,984`, ε above, defaults `0.2/5/5/300`, tick suffix multipliers `1/20/24,000`,
force-field exponent `4`. Geometry/interpolation use Java doubles except HUD distance/intensity
narrowing to float and damage's final int-to-float conversion. Signed-long remaining can underflow
only through unsupported direct inputs. No branch consumes RNG.

**Side effects:**

Saved-data dirtiness, listener packets, command feedback, client border replacement/ticking,
collision/ray results, interaction/spawn/portal rejection or clamp at caller-owned layers, damage
submission, HUD vignette and rendered wall. Border ticking itself neither broadcasts per-step
packets nor deletes outside objects.

**Gates:**

Per-level normal tick, exact caller opt-in, command permission/input bounds, client packet/tick,
player and damage admission. Gamerules and difficulty do not alter border geometry or formula.

**Boundary cases and quirks:**

Intermediate authoritative geometry is one calculated sample behind `getSize()`, but completion
skips the last lagged sample and installs target immediately. AABB exact maximum faces are inside
due to ε while a point at maximum is outside. Clamp never returns the exact maximum. Collision walls
use floor/ceil rather than the precise double edge. `add` during a lerp adds remaining time, not
original duration. Save/reload and reconnect restart interpolation from calculated current size with
the remaining count and discard prior geometry history; wall-clock time is irrelevant. Client
partial geometry, previous-distance alpha/HUD and calculated size can describe different instants.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`. Anchors:
`net.minecraft.world.level.border.WorldBorder`, both extent implementations,
`net.minecraft.server.level.ServerLevel#tick(java.util.function.BooleanSupplier)`,
`net.minecraft.world.entity.LivingEntity#baseTick()`,
`net.minecraft.world.entity.Entity#collectCollidersIgnoringWorldBorder`,
`net.minecraft.world.level.CollisionGetter#clipIncludingBorder`,
`net.minecraft.server.players.PlayerList#sendLevelInfo`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleInitializeBorder`,
`net.minecraft.client.gui.Hud#extractVignette`, and
`net.minecraft.client.renderer.WorldBorderRenderer#extract`.

**Test vectors:**

(1) Resize over D=`1/2/20`, assert calculated size versus ordinary geometry before every entity
phase, including intermediate lag and the Dth static-target jump; repeat during freeze, overload
delay and save/reload. (2) Test point, radius, BlockPos, chunk and AABB at `min`, `max`, `max±ε`,
absolute clamps and nonintegral edges. (3) Approach each integer-rounded collision wall from
inside/outside and ray-clip across it. (4) Sweep damage at AABB, center-distance, safe-zone, floor
and rate-zero boundaries, including in-wall precedence, an intermediate sweep and completion jump.
(5) Trace warning thresholds for static/growing/shrinking/zero cases and compare partial force-field
geometry with previous-edge HUD/alpha. (6) Join/reconnect/change dimension mid-lerp and assert
snapshot history reset plus dimension-scoped listener behavior. Run `EXP-WGEN-004` as the executable
observation matrix.
