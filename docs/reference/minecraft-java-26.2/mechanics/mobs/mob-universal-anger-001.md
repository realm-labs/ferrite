# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-UNIVERSAL-ANGER-001` — Universal anger redirects player retaliation through two distinct models

**Parent:** `MOB-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the live Boolean rule, generic revenge-goal suppression, six classic-neutral
reset registrations, group propagation, timer RNG, Piglin target/memory writes and later target
precedence are explicit in locked source.

**Applies when:**

A mob evaluates a new `HurtByTargetGoal`, a classic `NeutralMob` tests or refreshes player anger,
or Piglin Brain ingress selects, propagates or consumes a player anger target.

**Authoritative state:**

The current level's `universal_anger` rule; last-hurt entity/timestamp and goal-local timestamp;
classic-neutral absolute anger end time, persistent entity reference and live target; selector
priority and same-class neighbors; Piglin `ANGRY_AT`, `UNIVERSAL_ANGER`, visible-player and nearby-
adult memories; player type, creative/spectator/attackability state and level difficulty.

**Transition and ordering:**

The `MOBS` Boolean rule defaults to `false` and has no change callback. Every consumer reads the
live level value at its own branch. A rule change therefore mutates no target, timer or Brain memory
by itself.

**Generic revenge gate:** `HurtByTargetGoal#canUse` first requires a last-hurt timestamp different
from its timestamp and a nonnull last attacker. If that attacker has exactly the player entity type
and the live rule is true, it returns false immediately. Ignore-class and combat-targeting checks
are not reached. Nonplayer attackers and rule-false player attackers continue through assignable
ignore classes, then combat targeting that ignores line of sight and invisibility. The goal-local
timestamp advances only in `start`, so suppression does not consume the hurt event: turning the
rule off can admit the same still-retained event later. Turning it on does not stop an already
running revenge goal. This base branch affects every registered instance/subclass; species goal
registrations and arbitration remain `MOB-AI-001`.

**Classic neutral matching:** `NeutralMob#isAngryAt` rejects a candidate that `canAttack` rejects.
For a player that is neither creative nor spectator and whose level difficulty is not PEACEFUL, it
then accepts universal anger when the live rule is true, the mob's absolute anger end time is
strictly greater than its current level game time, and its persistent anger target is null.
Otherwise it accepts only a nonnull persistent reference matching that entity. Nonplayers never
take the universal branch. Equality at the end time is not angry.

Because matching reads the rule but does not rewrite state, disabling it immediately removes the
all-player match while retaining an unexpired targetless timer; re-enabling before expiry restores
the match. The absolute `anger_end_time` and nullable `angry_at` reference use ordinary
`NeutralMob` save/load. A legacy `AngerTime` value is converted to a new relative end time on load.

**Classic neutral reset:** `ResetUniversalAngerTargetGoal#canUse` requires the live rule, a nonnull
last attacker of exactly player type and a last-hurt timestamp strictly greater than the goal's
stored timestamp. On start it stores that timestamp, then calls
`forgetCurrentTargetAndRefreshUniversalAnger`: clear last attacker, persistent reference and live
target; set end time to `-1`; then sample and install a fresh targetless anger interval. All six
registrations sample `TimeUtil.rangeOfSeconds(20,39)`, an inclusive integer range `400..780` ticks,
from that mob's RNG.

The exact target-selector registrations are Bee priority 3 with group alert, Iron Golem priority 4
without, Polar Bear priority 5 without, Wolf priority 8 with, Enderman priority 4 without, and
Zombified Piglin priority 3 with. A group-alert start queries the same runtime class in the AABB
formed by a unit cube at the starter's position inflated by `(FOLLOW_RANGE,10,FOLLOW_RANGE)`, using
`NO_SPECTATORS`. In returned-list order it excludes only the starter, then clears and resamples each
other neutral mob exactly as above. It does not require that a neighbor was hurt, attackable, idle
or previously calm, so propagation can erase another target and each refreshed mob consumes one
timer draw.

**Piglin guarded-container ingress:** `angerNearbyPiglins` queries `Piglin` only in the player's box
inflated uniformly by 16, preserves entity-list/stream order, retains idle brains and optionally
requires visibility to the triggering player. For each survivor, rule false selects the triggering
player. Rule true selects that piglin's `NEAREST_VISIBLE_ATTACKABLE_PLAYER` when present and falls
back to the trigger. The common setter first rechecks attackability ignoring line of sight, erases
`CANT_REACH_WALK_TARGET_SINCE`, and writes the selected UUID to `ANGRY_AT` for 600 ticks. When that
selected entity is a player and the rule is true at this later setter read, it additionally writes
`UNIVERSAL_ANGER=true` for 600 ticks. Existing guarded-container call admission is owned by
`ITM-ENDER-CHEST-001` and `ITM-BARREL-001`.

**Piglin retaliation and propagation:** `maybeRetaliate` returns while AVOID is active, when the
attacker is not attackable ignoring line of sight, or when it is much farther than the current
attack target under the fixed distance-4 comparison. For a player attacker with the live rule true,
the initiating piglin selects its nearest visible targetable player or falls back to the attacker,
uses the common setter, then visits its nearby-adult-piglin memory. Each adult independently uses
its own nearest visible targetable player; a peer with none receives no write. Any other branch
sets/broadcasts the exact attacker through the ordinary anger path.

The later attack-target resolver first rejects all targets while near a zombified entity. It then
prefers an attackable entity resolved from `ANGRY_AT`, even when universal memory exists. If that
exact target is absent or no longer attackable, a present `UNIVERSAL_ANGER` memory selects
`NEAREST_VISIBLE_ATTACKABLE_PLAYER`; only then do nearest nemesis and an attackable nearest
non-gold player follow. This resolver tests memory presence, not the current game rule. Disabling
the rule therefore does not clear or deactivate an already written 600-tick Piglin universal
memory; its TTL/serialization and ordinary Brain phase behavior remain `MOB-AI-001`.

**Branches and aborts:**

Rule false/true and mid-anger changes; player/nonplayer, creative/spectator/PEACEFUL and attackable
targets; new/already-consumed/running revenge events; each classic-neutral registration and group
flag; same-class neighbors and timer expiry; Piglin idle/visibility/AVOID/distance gates, nearest-
player presence, peer visibility, exact-target validity and both memory TTLs.

**Constants and randomness:**

Rule default `false`; classic interval `400..780` inclusive ticks; reset vertical alert inflation
`10`; guarded-container radius `16`; Piglin anger and universal-memory TTL `600`; retaliation
distance comparison `4`. Each classic reset consumes one bounded mob-RNG draw per refreshed mob.
The rule branches and all Piglin selection/memory writes consume no RNG.

**Side effects:**

Goal admission/suppression; classic last-attacker, target, persistent reference and absolute timer
changes; same-class alert query and ordered timer draws; Piglin reachability, UUID and Boolean Brain
memory writes; changed navigation/targeting, attacks and their ordinary entity/client projection.
The rule has no direct packet, callback, sound or particle effect.

**Gates:**

Effective AI and full selector acquisition; current level rule; hurt type/timestamp; player validity
and attackability; classic anger timer/reference; registration priority and same-class range;
Piglin activity, perception and memory presence/expiry.

**Boundary cases and quirks:**

The generic revenge goal suppresses player retaliation for every user of that goal, not only
`NeutralMob`; another goal or Brain path may still react. Classic universal anger is targetless and
live-rule-dependent, whereas Piglin universal anger keeps an exact UUID first and stores an
independent expiring marker. Group reset can overwrite a neighbor's unrelated target. No rule
toggle retroactively cancels a running goal or erases either model's stored state.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`; `net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.world.entity.NeutralMob#isAngryAt`;
`net.minecraft.world.entity.NeutralMob#isAngryAtAllPlayers`;
`net.minecraft.world.entity.NeutralMob#addPersistentAngerSaveData`;
`net.minecraft.world.entity.NeutralMob#readPersistentAngerSaveData`;
`net.minecraft.world.entity.NeutralMob#forgetCurrentTargetAndRefreshUniversalAnger`;
`net.minecraft.world.entity.NeutralMob#stopBeingAngry`;
`net.minecraft.world.entity.ai.goal.target.HurtByTargetGoal#canUse`;
`net.minecraft.world.entity.ai.goal.target.ResetUniversalAngerTargetGoal#canUse`;
`net.minecraft.world.entity.ai.goal.target.ResetUniversalAngerTargetGoal#start`;
`net.minecraft.world.entity.ai.goal.target.ResetUniversalAngerTargetGoal#getNearbyMobsOfSameType`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#angerNearbyPiglins`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#maybeRetaliate`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#broadcastUniversalAnger`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#setAngerTarget`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#findNearestValidAttackTarget`;
`net.minecraft.util.TimeUtil#rangeOfSeconds`; the `registerGoals` and
`startPersistentAngerTimer` methods on Bee, Iron Golem, Polar Bear, Wolf, Enderman and Zombified
Piglin; `MOB-AI-001`;
`ITM-ENDER-CHEST-001`; `ITM-BARREL-001`; `EXP-MOB-010`.

**Test vectors:**

Sweep the live rule before/after one retained player/nonplayer hurt event and during a running goal.
For every classic registration, test priority, player validity, exact/targetless anger, end-time
equality, save/reload and `400/780` draws; for alerting species cross self/peer class, range edge,
spectator, existing target and returned order. For Piglins cross both ingress paths, every early
return, trigger/alternate/absent nearest player, adult peer perceptions, exact-target loss, both TTL
edges and a rule disable/re-enable; record targets, memories, RNG cursor and visible attacks.
