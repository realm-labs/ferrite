# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-AI-001` — Mob AI composes a phased goal arbiter, memory behaviors, path search and one-shot controls

**Parent:** `MOB-004`, `MOB-005`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — tick phasing, flag arbitration, Brain memory/activity/behavior order, sensor
cadence, visibility caching, weighted path search, waypoint/stuck rules and base controls are
explicit. Species registrations are closed dispatch through each entity's locked goals, Brain
provider/AI class and control/navigation overrides.

**Applies when:**

An entity-ticking server mob reaches effective AI. `NoAI` makes `isEffectiveAi()` false; chunk
visibility/ticking and root/passenger admission are owned by `ENT-LIFECYCLE-001`.

**Authoritative state:**

Entity/tick ID, two goal selectors and their insertion-ordered wrapped goals, disabled/locked flags,
Brain memory slots/sensors/activities/behaviors, path/navigation/node evaluator, movement/look/jump
controls, attributes, target and every species-owned predicate/timer/RNG source.

**Transition and ordering:**

**Mob phase order:**

`serverAiStep` first increments `noActionTime`, then clears the classic sensing cache. Let
`phase = tickCount + entityId`. When `tickCount > 1` and `phase` is odd, target selector then goal
selector tick only running goals that require every-tick updates. Otherwise each selector performs
full cleanup/update followed by ticking every running goal. Navigation then ticks, followed by the
species `customServerAiStep` (the usual Brain ingress), then move, look and jump controls in that
order. Thus a Brain behavior may replace navigation/control intent after navigation but before the
controls consume it. Every fifth ordinary mob tick also enables/disables selector `MOVE`, `JUMP`
and `LOOK` flags from controlling-passenger state; `JUMP` is additionally disabled in a boat.

**Classic goal arbiter:**

Full `GoalSelector#tick` has three ordered phases. Cleanup visits available goals in insertion order
and stops a running goal if any of its flags is disabled or `canContinueToUse()` is false; stale
flag locks are then removed. Update again visits insertion order. A stopped goal is eligible only if
none of its flags is disabled, every current holder is replaceable, and `canUse()` succeeds. A
holder is replaceable only when it is interruptible and the candidate's integer priority is
strictly lower. For each candidate flag, stop the current holder, assign the candidate as holder,
then start it after all assignments. Equal priorities never preempt; disjoint or empty flag sets can
run concurrently. The final phase ticks all running goals. A reduced half tick skips all cleanup,
eligibility and acquisition and ticks only running goals whose `requiresUpdateEveryTick()` is true.
`adjustedTickDelay(n)` is `n` for those goals and positive `ceil(n/2)` otherwise.

Removing goals stops matching running instances before removal. Disabling a flag does not stop its
holder until the next full cleanup; enabling only removes the disabled mark. Species registrations
are therefore an exact tuple of selector, insertion order, numeric priority, flag set and goal
implementation. Every `registerGoals`, dynamic `addGoal/removeGoal`, Brain AI class and subtype
control/navigation constructor in the locked entity hierarchy is part of this family selector; a
name-only list of goals is not a compatible substitute.

**Brain, memories and activities:**

A Brain registers the provider's memory modules, then each sensor and all memories it requires,
then behaviors by ascending integer priority. Sensors and each activity's behavior set retain
insertion order.
Unregistered memory reads through `getMemory` fail; writes to unregistered slots do nothing. Null
or an empty collection clears a registered slot. Permanent TTL is `Long.MAX_VALUE`. At the start of
each Brain tick, an expirable populated slot is cleared when its TTL is already `<= 0`; otherwise
the TTL is decremented. Sensors tick next, stopped behaviors in currently active activities are
then offered `tryStart` in ascending priority, source `HashMap` activity iteration and per-activity
insertion order. A freshly rebuilt traversal in that same container order finally calls
`tickOrStop`; equal-priority activity order has no stronger semantic guarantee than the locked map.

An ordinary `Behavior` starts only when all memory statuses and its extra predicate pass. It sets
RUNNING, samples inclusive duration `min + nextInt(max + 1 - min)`, stores
`endTimestamp = gameTime + duration`, then invokes `start`. On later ticks it stops when
`gameTime > endTimestamp` or `canStillUse` is false; equality is still active. Brain priorities do
not mutually exclude behaviors by themselves: memory/activity predicates, gates and behavior
combinators provide exclusion.

Active activity changes first erase the configured memories for every old activity other than the
new one, then replace the active set with all core activities plus the new activity. An unmet
requested activity selects the default; first-valid selection stops at the first satisfied entry.
Schedule refresh occurs only when `gameTime - lastScheduleUpdate > 20`; an environment schedule
value is selected at the mob position, or `IDLE` without a schedule. `stopAll` stops every running
behavior at the body's current level time.

**Sensing and targeting:**

Classic `Sensing` clears per-entity seen/unseen ID sets every AI step. The first line-of-sight query
per target performs the mob clip and caches the boolean; later queries that step do not re-clip.
Brain sensors independently initialize `timeToTick` uniformly in `[0, scanRate)`. Each call
pre-decrements it; at `<= 0` it resets to `scanRate`, rewrites the shared targeting-condition ranges
from the body's `FOLLOW_RANGE`, and runs the sensor. Default scan rate and default static range are
`20` and `16`. An existing `ATTACK_TARGET` changes the helper predicate to ignore invisibility for
that target; explicit helpers separately choose combat/noncombat and line-of-sight policy.

**Path creation and search:**

Path creation rejects an empty target set, a mob below level minimum, or a navigation state that
cannot update. It reuses a live path when its target is in the requested set. Otherwise it builds a
cubic `PathNavigationRegion` around the mob with radius `maxPathLength + radiusOffset` and runs the
navigation's node evaluator. Base max path length is `max(FOLLOW_RANGE, requiredPathLength)`, with
default required length `16`. The pathfinder's initial visit budget is
`floor(base FOLLOW_RANGE * 16)`; an explicit max-node update or required-length change resets it to
`floor(max(current FOLLOW_RANGE, requiredPathLength) * 16)`, and search multiplies that integer by
the current visit multiplier.

The search is A*-like: start `g=0`, initial `h` is nearest Euclidean target distance, then each
expanded neighbor uses `g + edgeDistance + costMalus` and `h = 1.5 * nearestTargetDistance`.
Expansion stops before processing count `>= adjustedVisitBudget`, at the first node within
Manhattan `reachRange` of any target, or when the heap empties; nodes at Euclidean distance
`>= maxPathLength` from start are not expanded and neighbors require walked distance strictly below
that limit. Reached alternatives choose shortest node count. If none is reached, choose least
distance to target then shortest node count. Node type, neighbor order, doors, rails, water, flying,
amphibious and species penalties are owned by the selected locked `NodeEvaluator`/navigation
subclass.

`moveTo(null, speed)` clears the path and fails. A live path is trimmed for cauldrons, must have a
node and not already be done, then records speed and resets the 100-tick stuck sample. Recompute is
allowed only when more than `20` game ticks elapsed and navigation can update; otherwise it marks a
delayed request, retried at the start of navigation ticks.

**Following, controls and failure:**

Each navigation tick increments its counter, retries delayed recompute, follows when permitted,
then writes the next entity-position waypoint to move control. A blocked update state may still
advance a same-column lower waypoint while airborne. Following advances when horizontal distances
are strictly below the width-derived tolerance and vertical distance below the subtype limit, or
when safe corner cutting/direction tests allow it. Fire-neighbor, damaging-neighbor and walkable-door
nodes cannot be corner-cut.

Every `> 100` navigation ticks, displacement is compared with
`(speed>=1 ? speed : speed^2) * 100 * 0.25`; strictly less marks stuck and clears the path. The
current node also has a game-time timeout: expected ticks are `distance / mobSpeed * 20`, and time
strictly greater than three times that limit clears the path. A node change recomputes the limit but
deliberately does not reset the accumulated timeout timer. Zero speed gives no timeout.

Base move control consumes exactly one `STRAFE` or `MOVE_TO` request and returns to `WAIT`, except a
required jump enters `JUMPING` until ground or affected liquid. Strafe normalizes to at least one,
uses `0.25 * MOVEMENT_SPEED`, and substitutes forward-only motion if the node evaluator says the
step is not walkable. Move-to turns yaw by at most `90` degrees, applies
`speedModifier * MOVEMENT_SPEED`, and jumps for a high close target or obstructing non-door,
non-fence shape. Distances below `2.5000003e-7` stop forward input. Look requests last two control
ticks, rotate by supplied head limits, otherwise turn head toward body by `10`, and clamp while
navigating. Jump control copies its one-shot flag to entity jumping and clears it. Swimming, flying,
jumping, body/look and vehicle-capable subtype controls replace these base equations where selected.

**Branches and aborts:**

Non-effective AI; reduced selector phase; disabled/conflicting flags; failed use/continue predicates;
unregistered/absent/expired memory; inactive activity; sensor not due; behavior timeout; unavailable
start/target/chunk; node/visit/path limit; delayed recompute; reached, stuck or timed-out path;
one-shot control already consumed. Subtype goals and behaviors add their locked state/RNG branches.

**Constants and randomness:**

Full goal evaluation is entity-ID phased every other tick after tick two; reduced goal delay is
positive `ceil(n/2)`. Schedule delay is strict `>20`; default sensor rate/range is `20/16`; behavior
duration is inclusive. Path heuristic factor is `1.5`, required length defaults `16`, recompute is
strict `>20`, displacement check strict `>100` with factor `0.25`, and node timeout factor is `3`.
Behavior, sensor, target, route and subtype RNG is consumed only at the reached methods described.

**Side effects:**

Goal/behavior running state, flag ownership, memories/activities/targets, sensor cache, path/debug
nodes, navigation intent, forward/strafe/jump input, rotations and all species attacks, interaction,
block mutation, sounds/events, items/entities and synced state reached through those actions.

**Gates:**

Entity/root ticking and effective AI, selector phase and flags, memory/activity requirements,
sensor cadence/range/visibility, node evaluator and loaded region, attributes/collision, controlling
passenger/leash/boat state, difficulty/gamerules and species predicates.

**Boundary cases and quirks:**

Equal-priority classic goals cannot preempt, but equal-priority Brain activities traverse a source
`HashMap`. Reduced phases never clean up a failed ordinary goal. Memory TTL zero survives until the
next Brain tick's initial expiration phase. A node change retains accumulated timeout time. A path
can be geometrically different and still compatible only inside the explicit player-visible
equivalence boundary.

**Compatibility boundary:**

Exact scheduler, memory, sensor and control state changes are required. Because this leaf is
`EquivalentPlayerVisibleBehavior`, a Ferrite pathfinder may choose a different internal route only
when it preserves reachability, timing-sensitive interaction/attack gates, stuck recovery,
block/door/fluid effects, published pose/velocity/rotation and species-visible outcomes. The locked
entity/AI registrations are the exhaustive content catalog; unknown registry activities, memories,
sensors or POIs are not silently mapped to generic behavior.

**Evidence:**

`OFF-SERVER-001`; `net.minecraft.world.entity.Mob#serverAiStep`;
`net.minecraft.world.entity.ai.goal.GoalSelector`; `WrappedGoal`; `Goal`;
`net.minecraft.world.entity.ai.Brain`; `net.minecraft.world.entity.ai.memory.MemorySlot`;
`net.minecraft.world.entity.ai.behavior.Behavior`;
`net.minecraft.world.entity.ai.sensing.Sensing`; `Sensor`;
`net.minecraft.world.entity.ai.navigation.PathNavigation`;
`net.minecraft.world.level.pathfinder.PathFinder` and all registered evaluator subclasses;
`net.minecraft.world.entity.ai.control.MoveControl`, `LookControl`, `JumpControl` and subtype
overrides; every locked entity `registerGoals`/Brain AI provider; `EXP-MOB-002`.

**Test vectors:**

Equal/lower/higher priority and interruptibility; disjoint/empty/disabled flags across full/half
phases and entity-ID parity; TTL `1/0/-1`; empty-memory write; schedule difference `20/21`;
sensor phase and repeated LOS; behavior end equality; visit/reach/length boundaries; path reuse and
delayed recompute; exact waypoint/stuck/timeouts; move/look/jump one-shots; representative ground,
water, amphibious, flying, door/rail and Brain-only species plus dynamic goal mutation.
