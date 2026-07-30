# MOB-004 AI and Universal Anger Runtime

`G01-P7-S009` implements `MOB-AI-NAVIGATION-001` and `MOB-UNIVERSAL-ANGER-001` as protocol-neutral
Region transitions. The owning Region supplies ordered goal/behavior collections, memory and path
state, perception observations, live gamerules and explicit entity RNG draws.

## Responsibility split

`ferrite-gameplay::mob::runtime::mob_004` has five owners:

- `selector` owns effective-AI phasing, passenger flag refresh and classic cleanup/acquisition/tick
  arbitration;
- `brain` owns memory write/TTL, behavior duration and stopping, activity changes, schedule,
  sensors and line-of-sight caching;
- `navigation` owns path admission/reuse, visit/length/reach limits, alternative selection,
  delayed recomputation, waypoint following and both stuck detectors;
- `controls` owns base one-shot strafe/move-to/jump/look consumption;
- `anger` owns generic revenge suppression, classic neutral matching/reset/group propagation and
  Piglin memory selection/precedence.

## AI phase and schedulers

Effective AI increments inactivity and clears classic sensing before selectors. After tick one, odd
`tickCount+entityId` phases tick only running goals that require every-tick updates; full phases run
cleanup, acquisition and all running ticks for target then ordinary selectors. Navigation precedes
species Brain ingress, and move/look/jump controls follow. Every fifth tick refreshes MOVE/JUMP/LOOK
disablement from controlling passenger state, with boat JUMP disabled independently.

Full cleanup is insertion ordered. Disabled flags or failed continuation stop a running goal before
stale locks are removed. Acquisition is insertion ordered; every flag holder must be interruptible
and the candidate priority strictly lower. Equal priority never preempts, while disjoint and empty
flag sets can run together. Reduced phases perform no cleanup or acquisition. Non-every-tick delay
is positive `ceil(n/2)`.

Brain memory TTL at or below zero expires at the start of the next Brain tick; otherwise it
decrements, while permanent memory has no finite TTL. Unregistered writes are ignored and empty
writes clear. Sensors run after memory expiration, then stopped behaviors are offered by priority/
container order, and running behaviors tick-or-stop last. Behavior duration is inclusive and
`gameTime == endTimestamp` remains active. Activity changes erase configured old memories, restore
all core activities plus requested/default, and first-valid selection stops at the first match.
Schedule refresh is strict after twenty ticks. Sensors predecrement and reset to their scan rate;
classic sight clips once per target per AI step.

## Navigation and controls

Path creation rejects empty targets, below-minimum bodies and blocked update states before reusing a
live requested target or searching. Maximum length is `max(FOLLOW_RANGE, required)`. Initial visit
budget uses base follow range times sixteen; a required-length/max-node reset uses the maximum, and
search applies the visit multiplier. Expansion is strict below visit and length budgets, while
Manhattan reach equality succeeds. Reached alternatives minimize node count; unreached alternatives
minimize target distance then node count.

`moveTo(null)` clears/fails; a usable path trims cauldrons, records speed and resets the 100-tick
sample. Recompute is strict after twenty game ticks and otherwise remains delayed. Waypoint
horizontal/vertical tolerances are strict. Stuck displacement checks only after 100 navigation
ticks using `(speed>=1 ? speed : speed²)*100*0.25`. Node timeout is strict above three times
`distance/speed*20`; zero speed disables it, and node changes recompute the limit without resetting
accumulated time. Unsafe fire/damage/door nodes prohibit corner cuts.

Base strafe normalizes to at least one, applies `0.25*MOVEMENT_SPEED`, and falls back to forward
when unwalkable. Move-to consumes the request, turns within ninety degrees, uses
`speedModifier*MOVEMENT_SPEED`, stops below `2.5000003e-7`, or enters JUMPING for the audited height/
shape gates. Look requests last two control ticks and otherwise turn ten degrees toward the body;
jump copies then clears its one-shot request.

## Universal anger

The rule defaults false and has no callback. Generic revenge first checks new timestamp and attacker,
then suppresses an exact player under the live rule before ignore-class/combat checks. Suppression
does not advance the goal timestamp, and a rule toggle does not stop a running goal.

Classic neutral universal anger requires an attackable noncreative/nonspectator player outside
Peaceful, live rule, targetless state and `angerEndTime > gameTime`; exact persistent references are
the separate fallback. Reset goals require a newer player hurt event, clear attacker/persistent/
live targets, pass through end time `-1`, then sample 400..780 ticks. The six exact priority/group
registrations are encoded. Group alert queries the same runtime class at follow-range/10/follow-
range, excludes spectators and only the starter, then resets every returned peer in order with one
peer RNG draw each.

Piglin guarded-container ingress filters idle brains and optional visibility inside radius sixteen.
Rule false chooses the trigger; rule true prefers each Piglin's visible attackable player. The
common setter rechecks attackability, erases reach failure, writes `ANGRY_AT` for 600 and—after a
second live rule read—writes player universal memory for 600. Retaliation preserves AVOID,
attackability and distance-four gates. Later target resolution ignores the current rule: nearby
zombified suppression precedes exact `ANGRY_AT`, then universal nearest player, nemesis and
non-gold player.

## Validation

`crates/ferrite-gameplay/tests/slices/mobs/mob_004.rs` owns both source-specified slices. Its
seventeen tests lock phase parity, selector conflicts, memory/behavior/activity/sensor boundaries,
path budgets/search/recompute/stuck/timeouts, one-shot controls, revenge retention, classic timers
and registrations, group facts, Piglin ingress/retaliation/writes and final target precedence.
`G01-P7-B1` remains responsible for composing these transitions with Region entity state, loaded
navigation observations and protocol projection.
