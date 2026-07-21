# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-DESPAWN-001` — Despawn is a strict-distance discard with persistence and subtype policy overrides

**Parent:** `MOB-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — invocation order, persistence reset, nearest-player selection, all thresholds,
RNG cadence and every `removeWhenFarAway`/`requiresCustomPersistence` override are explicit in the
locked entity hierarchy.

**Applies when:**

The server root entity loop reaches a non-frozen, non-removed mob. `checkDespawn` runs before the
current-chunk entity-ticking admission; a loaded visible mob can therefore be discarded even when
its ordinary tick would be skipped. A valid passenger is not independently root-checked.

**Authoritative state:**

Difficulty/type peaceful policy, stored `persistenceRequired`, subtype custom persistence,
category distances, nearest player, 3D squared distance, `noActionTime`, mob RNG and subtype
`removeWhenFarAway(distanceSquared)`.

**Transition and ordering:**

Peaceful difficulty first discards a type not allowed in Peaceful, with no persistence or distance
test. Otherwise, if stored or custom persistence is true, set `noActionTime = 0` and stop. If both
are false, find the nearest player with unlimited range through `getNearestPlayer(mob, -1)`; this
targeting path excludes spectators. No player leaves the timer unchanged and consumes no despawn
RNG.

With a player, compute ordinary 3D `player.distanceToSqr(mob)`. The hard threshold is the type's
category despawn distance squared: `128^2` for every category except `WATER_AMBIENT = 64^2`. If
distance is strictly greater and `removeWhenFarAway` returns true, discard immediately. Source then
continues through the soft branch; discard state is not an explicit early return.

The soft threshold is always `32^2`. Only when `noActionTime > 600` does the expression evaluate
`random.nextInt(800)`. A zero result followed by distance strictly greater than `32^2` and a true
subtype predicate discards. Otherwise, distance strictly below `32^2` resets `noActionTime` to zero.
Equality at either threshold neither qualifies as farther nor as the near reset. `serverAiStep`
increments `noActionTime` before goal/Brain work whenever effective AI runs.

**Persistence and removal-policy catalog:**

Base custom persistence is passenger or leashed. The stored flag is serialized, can be set by
commands/subtypes, and becomes true when a mob equips a picked-up item; age locking also sets it
when locking succeeds. A custom name and tame state are not generic base persistence predicates:
they matter only through a stored flag or the following overrides.

- `Animal`, `AbstractGolem`, `Villager`, `WanderingTrader`, `Allay` and `Warden` return false from
  `removeWhenFarAway`; ordinary animals therefore never distance-despawn. Chicken alone returns its
  chicken-jockey flag. Cat and ocelot return true only when respectively untamed/untrusting and
  `tickCount > 2400`.
- `AbstractFish` and axolotl add `fromBucket` to custom persistence and allow far removal only when
  not from a bucket and not custom-named. `AbstractNautilus` always permits far removal but adds
  tame state to custom persistence. Sulfur cube adds body item or bucket origin to custom
  persistence. Enderman adds a carried block.
- Raider adds current raid to custom persistence and refuses far removal during a raid.
  `PatrollingMonster` permits removal when not patrolling or when squared distance is strictly
  greater than `16384`. Piglin permits it when the stored persistence flag is false. Hoglin,
  camel husk and zombie horse always permit it.
- Zombie villager permits removal only when not converting and villager XP is zero. All other
  locked entity types inherit their nearest superclass policy; this inheritance closure is the
  exhaustive subtype catalog.

`removeWhenFarAway` does not itself make an entity persistent: it is called only at reached hard or
random-distance branches, and persistent/custom-persistent mobs bypass it.

**Branches and aborts:**

Peaceful removal; stored/custom persistence; no eligible player; hard strict distance; inactivity
not over `600`; 1-in-800 roll miss; soft equality/inside/outside; subtype predicate false. `NoAI`
can prevent further `noActionTime` increments but does not itself appear in `checkDespawn`.

**Constants and randomness:**

No-despawn distance `32`; hard distance `128`, except water ambient `64`; inactivity condition
strictly `> 600`; one `nextInt(800) == 0` trial per reached check. The RNG call occurs before the
soft-distance and subtype tests because of left-to-right conjunction order.

**Side effects:**

`discard()` removes without death damage, loot or XP; entity removal still performs passenger,
leash, tracking and section cleanup. Retained persistent or near mobs reset inactivity as stated.

**Gates:**

Root-entity admission, peaceful policy, stored/custom persistence, nearest nonspectator player,
category distance, inactivity, mob RNG and inherited subtype removal predicate.

**Boundary cases and quirks:**

The hard-discard branch does not return before evaluating the soft branch. Both distance comparisons
are strict and use full 3D squared distance. The random draw precedes distance/policy testing once
the inactivity gate passes. No player does not reset inactivity. Tame/name are not universal
predicates; only stored state and the audited subtype closure count.

**Evidence:**

`OFF-SERVER-001`; `net.minecraft.server.level.ServerLevel#tickNonPassenger`;
`net.minecraft.world.entity.Mob#checkDespawn`, `#serverAiStep`, `#requiresCustomPersistence`,
`#removeWhenFarAway`; `net.minecraft.world.entity.MobCategory`; every locked override in `Raider`,
`Animal`, fish/axolotl/nautilus, chicken/cat/ocelot, golem/allay, villager/trader,
patrolling/piglin/hoglin/zombie-villager/warden/enderman/sulfur-cube families; `EXP-MOB-003`.

**Test vectors:**

Peaceful persistent hostile; no/spectator-only player; squared distance one below/equal/one above
`32^2`, `64^2`, `128^2` and `16384`; timer `600/601`; roll hit/miss ordering; passenger/leashed/
picked-up/age-locked; bucket/name/tame/trust/raid/patrol/conversion/carried-block/body-item states;
visible non-entity-ticking root versus passenger; repeat removal cleanup.
