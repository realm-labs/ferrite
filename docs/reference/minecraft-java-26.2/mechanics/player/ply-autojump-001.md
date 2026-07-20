# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-AUTOJUMP-001` — Obstacle geometry schedules a later synthetic jump press

**Parent:** `PLY-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — detector entry, direction fallback, look-ahead volume, ordered entity/block
shape traversal, clearance scan, height acceptance and delayed timer consumption are complete below.
No black-box result is required to choose an algorithm branch.

**Applies when:**

Any local `move(moverType,requested)` call completes and invokes the detector with actual horizontal
displacement `(float(newX-oldX), float(newZ-oldZ))`. The detector is independent of whether generic
movement reported a horizontal collision.

**Authoritative state:**

Cached auto-jump option and integer timer; on-ground/stay-on-ground/passenger state; sampled raw
movement `Vec2`, yaw/view forward, movement-speed attribute and actual displacement; current
position, AABB width/height and block jump factor; Jump Boost; ordered entity and block collision
shapes in the look-ahead volume. This is client prediction state; the later ordinary jump and server
movement validation remain authoritative as specified by `PLY-INPUT-001`, `PLY-MOVE-001` and
`PLY-MOVE-VALIDATE-001`.

**Transition and ordering:**

`LocalPlayer#move` snapshots X/Z, runs the complete generic move, casts actual X/Z differences to
floats, calls this detector, then adds walked distance. The detector returns unless all entry gates
hold: cached auto-jump enabled, `autoJumpTime<=0`, on ground, not staying on a ground surface, not a
passenger, sampled `moveVector.lengthSquared()>0`, and current block jump factor `>=1.0`.

Let current post-move position be `P`, actual horizontal float vector widened to doubles be
`V=(dx,0,dz)`, movement speed float be `S`, and `F=float(V.lengthSqr())`. If `F<=0.001f` (NaN also
takes this fallback), derive intended world motion from the **raw sampled** `moveVector`, not slowed
`xxa/zza`: `strafe=S*input.x`, `forward=S*input.y`, `sin=Mth.sin(double(float(yaw*0.017453292f)))`,
`cos=Mth.cos(...)`, and replace `V` with `(strafe*cos-forward*sin, 0, forward*cos+strafe*sin)` after
float arithmetic. Recompute `F`; return unless it is strictly greater than `0.001f`. Let float
`I=invSqrt(F)` and direction `D=V*double(I)`. Dot only X/Z of `D` with the player's
three-dimensional `getForward()` vector and return when the float result is `<-0.15f`.

Before searching ahead, let `headPos=BlockPos.containing(x,boundingBox.maxY,z)`. Return if the
contextual collision shape at `headPos` is nonempty; replace it by `headPos.above()` and again
return if that shape is nonempty. Let maximum accepted rise `H=1.2f`, plus
`0.75f*(jumpBoostAmplifier+1)` when Jump Boost exists. Let look-ahead distance
`R=max(S*7.0f, 1.0f/I)` in float operations; because `I` is the inverse square root, the second term
is the selected motion magnitude modulo float rounding.

Construct `start=P`, `baseEnd=P+(dx,0,dz)`, and `end=baseEnd+D*R`. Let width `W` and height `B` be
current AABB dimensions. Query collisions in the AABB spanning `start` and `end+(0,B,0)`, inflated
by `(W,0,W)`. Raise both line endpoints by `0.5099999904632568`; let perpendicular
`N=D cross (0,1,0)` and half-width offset `O=N*(W*0.5f)`. The two probes are the line segments
`start-O -> end-O` and `start+O -> end+O`.

Collision iteration is entity collision shapes first, followed by block collision shapes; each shape
is flattened in its `toAabbs()` order. Initialize `obstacleTop=Float.MIN_VALUE` (the smallest
positive float). For every AABB intersected by either probe segment:

1. Overwrite `obstacleTop=float(box.maxY)`; this is assignment, not maximum accumulation.
2. Let `centerPos=BlockPos.containing(box.center)`. For integer `i=1` while `float(i)<H`, read the
   contextual collision shape at `centerPos.above(i)`. If nonempty, overwrite
   `obstacleTop=float(blockY)+float(shape.max(Y))`; if `double(obstacleTop)-playerY > double(H)`,
   abort the entire detector.
3. For every such `i>1`, also advance the retained player-head column position by one and abort if
   its contextual collision shape is nonempty. The initial two head-column checks therefore cover
   ordinary height, while larger Jump Boost ranges extend this vertical-clearance check.

After all candidates, return if the sentinel was never replaced. Compute
`rise=float(double(obstacleTop)-playerY)`. Store `autoJumpTime=1` only when `rise>0.5f && rise<=H`;
otherwise return. On the next loaded local `aiStep`, `PLY-INPUT-001` decrements the timer to zero
and changes that tick's sampled jump boolean to true. The detector itself never applies an impulse
or sends a message.

**Branches and aborts:**

Every entry gate; actual-displacement versus raw-input fallback; squared motion at/equal/above
`0.001f`; backward-view dot at `-0.15f`; two initial head blocks; absent/present Jump Boost; search
volume with entity then block shapes; neither/either probe intersection; every `i<H` overhead shape;
excessive candidate rise or player-column obstruction; no candidate; final rise at `0.5f` and `H`.
There is no explicit manual-jump, fluid, climbable, flight, mover-type, `step_height` or
`jump_strength` test beyond the stated entry state and inputs.

**Constants and randomness:**

Actual displacement is narrowed to float. Motion threshold is strict `>0.001f`; yaw conversion is
`0.017453292f`; view-dot lower bound is inclusive `-0.15f`; base rise is `1.2f`; Jump Boost adds
`0.75f` per amplifier-plus-one; look-ahead multiplier is `7.0f`; line height is double
`0.5099999904632568`; lateral offset is float `W*0.5f`; sentinel is `Float.MIN_VALUE`; minimum rise
is strict `>0.5f`; maximum is inclusive `<=H`; scheduled timer is exactly one. The contextual
shapes, AABB operations and line intersection use doubles after the marked float conversions. No RNG
is consumed.

**Side effects:**

On success, only `autoJumpTime=1`; the next input pass produces the synthetic jump and all
downstream movement/messages. Collision queries are read-only. `LocalPlayer#move` separately records
walked distance after detection.

**Gates:**

Cached client option, existing timer, on-ground/stay-on-surface/passenger flags, nonzero raw input,
block jump factor, movement speed, actual/fallback direction, yaw/pitch forward vector, Jump Boost,
pose dimensions and entity/block collision geometry. The cache is initialized to true and refreshed
at the tail of controlled non-passenger `sendPosition`, after movement; an option change can
therefore affect detection one eligible movement tick later, and is not refreshed while riding or
while the player is not the camera entity.

**Boundary cases and quirks:**

The detector does not require horizontal collision and can see collidable entities. A newly
constructed local player can therefore use the initial true cache before the first eligible
post-move refresh even when the saved option is false. Slow actual motion uses unmodified raw input
times movement speed for direction/look-ahead, ignoring item/sneak multipliers. The gaze comparison
ignores forward Y but `getForward`'s horizontal magnitude still shrinks with pitch. Search
broad-phase inflates by the full width while the two narrow probes use half width. Candidate order
matters: every intersecting box overwrites `obstacleTop`, so the last iterated match, not the
highest match, supplies the final rise unless an overhead block aborts first. A half-block rise is
rejected exactly; ordinary step logic owns it. Scheduling occurs after movement and consumption on
the next input pass.

**Evidence:**

`OFF-CLIENT-001`. Anchors:
`net.minecraft.client.player.LocalPlayer#move(net.minecraft.world.entity.MoverType,net.minecraft.world.phys.Vec3)`,
`net.minecraft.client.player.LocalPlayer#updateAutoJump(float,float)`,
`net.minecraft.client.player.LocalPlayer#canAutoJump()`,
`net.minecraft.client.player.LocalPlayer#isMoving()`,
`net.minecraft.world.level.CollisionGetter#getCollisions(net.minecraft.world.entity.Entity,net.minecraft.world.phys.AABB)`,
`net.minecraft.world.phys.shapes.VoxelShape#toAabbs()`, and
`net.minecraft.world.phys.AABB#intersects(net.minecraft.world.phys.Vec3,net.minecraft.world.phys.Vec3)`.
Data anchors: `reports/blocks.json`, `reports/registries.json#minecraft:entity_type`,
`reports/registries.json#minecraft:attribute/minecraft:movement_speed`, and
`reports/registries.json#minecraft:mob_effect/minecraft:jump_boost`.

**Test vectors:**

(1) Toggle every entry gate independently, including option changes before/after cache refresh,
timer `0/1`, stay-on-surface, raw opposing keys and block jump factor around one. (2) Actual squared
motion immediately below/equal/above `0.001f`, fallback cardinal/diagonal input, varied
speed/yaw/pitch and dot immediately below/equal/above `-0.15f`. (3) Empty/nonempty contextual shapes
in both initial head positions and every extended Jump Boost head column. (4) Single and multiple
entity/block shapes intersecting neither/left/right/both probes, with deliberately different ordered
tops to assert last-match rather than maximum behavior. (5) Rise immediately below/equal/above
`0.5f` and `H` for no Jump Boost and several amplifiers; include overhead shapes that overwrite
within versus abort above `H`. (6) Assert successful movement tick writes one without jumping, and
exactly the next input pass decrements it and forces jump. `EXP-PLY-007` is the executable
regression probe.
