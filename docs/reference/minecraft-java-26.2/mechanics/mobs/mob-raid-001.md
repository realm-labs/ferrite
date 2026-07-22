# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-RAID-001` — The raids rule admits omen work and retires managed raids on their next raid tick

**Parent:** `SIM-002`, `SIM-006`, `MOB-001`, `WGEN-004`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — both live rule reads, omen conversion/admission, center and reuse selection,
the raid state machine, wave construction, client projection and the manager/raider persistence
join are explicit in locked source and data.

**Applies when:**

A player's Raid Omen reaches its last effect tick, or a normally advancing `ServerLevel` reaches
its raid-manager phase with one or more managed raids.

**Authoritative state:**

The level's live `raids` Boolean; positional `CAN_START_RAID`; player spectator, omen/effect and
stat state; occupied village POIs; the dimension-local `Raids` ID map, next ID and manager tick;
each `Raid` center, status, counters, omen level, health denominator, heroes, RNG and boss event;
loaded/ticking chunks, village sections and regional difficulty; live `Raider` membership,
leaders, health and persisted raid reference.

**Transition and ordering:**

**Rule and omen caller:** The `MOBS` Boolean defaults to true and has no change callback. Bad Omen
checks every effect tick, and only a nonspectator player in a non-PEACEFUL level and a village can
receive Raid Omen. It installs Raid Omen for 600 ticks with the Bad Omen amplifier and snapshots
the player's block position. Raid Omen invokes `createOrExtendRaid` only when its remaining
duration is exactly one, then clears that position and returns false to end itself regardless of
the callee's result.

`createOrExtendRaid` rejects a spectator first, reads the live rule second, and reads positional
`minecraft:gameplay/can_start_raid` at the saved position third. A false rule therefore does no
POI search or raid mutation, but the caller still consumes Raid Omen and clears its position. The
attribute's registered default is true; the locked Nether dimension type overrides it to false.
Turning the rule off or on has no retroactive effect before one of these two direct readers runs.

**Center, reuse and omen absorption:** After all three gates pass, query occupied POIs in the
`#minecraft:village` tag within radius 64 of the saved position. A nonempty result selects the
component-wise arithmetic mean of all returned positions through `BlockPos.containing`; an empty
result retains the saved position. Reuse the nearest active raid whose center has squared distance
strictly below 9,216. Otherwise construct an ongoing, active raid with a 300-tick cooldown and
zero progress; its fixed ordinary-wave count is 0/3/5/7 on PEACEFUL/EASY/NORMAL/HARD.

A new, not-started raid is inserted if the map does not already contain that object. The manager's
`nextId` starts at one and `getUniqueId` preincrements it, so a fresh manager first assigns ID two.
An already-started raid at omen level five skips absorption; every other candidate calls
`absorbRaidOmen`. A present effect adds `amplifier+1` and clamps the result to 0..5. When no wave
has spawned, it also awards `RAID_TRIGGER` and fires `RAID_OMEN`; absence of the effect returns
false. The caller ignores this Boolean, marks the manager dirty and returns the raid in all cases.

**Live rule retirement:** An admitted `Raids#tick` increments its manager tick before iterating the
current ID-map values. For every value it rereads the live rule. False calls `Raid#stop`, which sets
`active=false`, removes all bossbar players and sets status `stopped`; the same manager iteration
then removes that map entry, marks the manager dirty and never calls `Raid#tick`. It does not kill,
discard, damage or directly unlink any raider. A raid already stopped for another reason follows
the same removal path when the rule is true. Independently, every manager tick divisible by 200
marks the saved data dirty.

Consequently, disable then re-enable before the next admitted raid-manager phase preserves the
raid. Once the false read removes it, re-enabling does not reconstruct it. Loaded raiders may still
hold the stopped object in memory, but after removal their saves cannot resolve that object to a
manager ID. Freeze and other `SIM-006` phase gates delay the read rather than changing this result.

**Ongoing raid tick:** With the rule true, `Raid#tick` performs these ordered transitions:

1. Recompute `active=hasChunkAt(center)`. PEACEFUL stops and returns. An active-state change updates
   bossbar visibility; an inactive raid then returns without advancing its active counter.
2. If the center is no longer a village, scan the section cube of radius two and move to the nearest
   village section center. If none exists, zero spawned groups call `stop`; otherwise status becomes
   `loss`. This branch does not return, so the current invocation still executes its remaining
   ongoing tail before the new status is observed on the next tick.
3. Increment `ticksActive`; reaching 48,000 stops and returns. Snapshot the current tracked-raider
   count. When it is zero and waves remain, run the cooldown/spawn-position path below.
4. Every active counter multiple of 20, refresh bossbar membership, remove invalid tracked raiders,
   and show the translated remaining-raider suffix only for a pre-cleanup count of one or two.
   The local count is not recomputed after cleanup, so its effects reach wave selection next tick.
5. While cooldown is zero, no tracked raider remains, and an ordinary or bonus wave remains, choose
   a position and spawn one group. Six consecutive null position results stop the raid. A chosen
   position marks the raid started, commits a group, and emits at most one horn for this tick.
6. After the last required wave and zero tracked raiders, increment `postRaidTicks` while it is below
   40. The next eligible tick changes status to `victory` and grants stored-hero rewards. Ongoing
   invocations reaching their common tail mark the manager dirty.

When zero raiders remain but a later wave is due, a zero cooldown after at least one group resets
to 300, resets the bossbar title and returns. During a positive cooldown, an absent spawn position
is recomputed every fifth remaining tick; a present but non-entity-ticking position is recomputed
immediately. Membership refresh occurs at 300 and every remaining multiple of 20, then the counter
decrements and bossbar progress becomes `clamp((300-cooldown)/300,0,1)`.

**Spawn position and group:** Countdown probes call `findRandomSpawnPos` with eight attempts; the
wave-time fallback uses 20. Each call consumes one initial angle, then visits angles separated by
pi/8 and consumes two `nextInt(3)` jitters per attempt. The radial factor is
`0.22*(cooldown/20)-0.24`. A candidate uses `WORLD_SURFACE`, differs from center Y by at most 96,
has every chunk in its horizontal +/-10 square loaded, is entity-ticking, and passes the ravager
placement type or has snow below and air at the candidate. While `cooldown/20 > 7`, it must also be
outside a village; an integer quotient at most seven permits an in-village position.

Each group clears `totalHealth`, increments the one-based wave number, and iterates Vindicator,
Evoker, Pillager, Witch, Ravager. The fixed counts for ordinary waves 1..7 are respectively:

| type | wave counts 1..7 |
|---|---|
| Vindicator | `0, 2, 0, 1, 4, 2, 5` |
| Evoker | `0, 0, 0, 0, 1, 1, 2` |
| Pillager | `4, 3, 3, 4, 4, 4, 2` |
| Witch | `0, 0, 0, 3, 0, 0, 1` |
| Ravager | `0, 0, 1, 0, 1, 0, 2` |

Raid omen level greater than one adds one bonus group after the difficulty's ordinary 3/5/7
groups. Its fixed counts use the same table index as that ordinary group count. Difficulty and
type also add source-ordered random extras: Vindicator and Pillager use a bound of 1 on NORMAL, 2
on HARD, or an initial `nextInt(2)` on EASY, then consume `nextInt(bound+1)` only for a positive
bound; Witch uses `nextInt(2)` outside EASY after wave two except wave four; Ravager uses
`nextInt(2)` only for a non-EASY bonus group; Evoker has none.

The first created leader-capable raider receives the wave leader flag and ominous banner. Every
accepted member adds its current health to the new denominator, receives raid/wave/join state,
moves to `(x+0.5,y+1,z+0.5)`, finalizes with reason `EVENT`, applies raid buffs, becomes on-ground,
and is submitted with passengers without rollback on insertion failure. A null creation ends only
that type's loop. Ravagers on wave five receive a Pillager rider; at wave seven and later, their
first/remaining riders are Evoker/Vindicator. The group counter advances even if all
entity creations fail.

**Membership, projection and completion:** Tracked raiders are removed every 20 active ticks when
removed, in another dimension, at squared distance at least 12,544, absent by UUID after entity age
600, or after 30 qualifying checks with `noActionTime > 2400` outside a village. Removal clears the
raider's raid pointer and updates health progress. Raider death records a direct player killer as a
hero, removes a leader if needed, and removes membership without subtracting its health from the
wave denominator.

Bossbar membership is the set of alive players for which `getRaidAt(player.blockPosition())` is
this raid; that lookup again uses the nearest active raid strictly within 96 blocks. One random
long seeds every wave's horn packets. Each level player within horizontal distance 64, plus any
more-distant current bossbar member, receives neutral `RAID_HORN` at volume 64 and pitch 1 from a
point 13 blocks toward the spawn position at the player's Y.

Victory waits until the post-raid counter is already 40. Each stored UUID still resolving to a
nonspectator living entity receives Hero of the Village for 48,000 ticks with amplifier
`raidOmenLevel-1`, hidden particles and visible icon; players also receive `RAID_WIN` and its
criterion. Victory/loss then celebrates for 600 raid ticks, refreshing the visible zero-progress
victory bar or defeat title every 20; tick 600 stops, and the manager removes the raid next pass.

**Persistence and reconstruction:** `ServerLevel` obtains dimension-local saved data through
`Raids.TYPE` (`raids`, `SAVED_DATA_RAIDS`). The manager codec stores optional `raids` (default
empty), required `next_id`, and required `tick`. Every list entry stores `id` plus these required
raid fields: `started`, `active`, `ticks_active`, `raid_omen_level`, `groups_spawned`,
`cooldown_ticks`, `post_raid_ticks`, `total_health`, `group_count`, `status`, `center`, and
`heroes_of_the_village`. Absence of any parsed or partial codec result falls back to a fresh dirty
manager.

Runtime group sets, leaders, RNG, boss-event identity, cached spawn position and celebration ticks
are not in that record. Each raider entity separately stores `Wave`, `CanJoinRaid`, and optional
`RaidId` only when its current object is still in the manager map. On load, a present ID resolves
the raid, replaces equal-UUID group membership, calls `addWaveMob(..., false)` so stored total
health is not double-counted, and restores a patrol leader entry. Thus raid globals load first and
entity records reconstruct live membership; a missing/removed ID leaves the raider unattached.

**Branches and aborts:**

Spectator and absent/stale omen position; false rule or dimension attribute; empty/nonempty POIs;
new/reused/started/max-level raid and absent effect; pre-read toggle versus post-removal re-enable;
frozen, peaceful, unloaded-center and lost-village states; every cooldown/probe/loading/placement
boundary; ordinary/bonus waves, creation and insertion failure; membership invalidation, victory,
loss, timeout, celebration and all save/load reference outcomes.

**Constants and randomness:**

Rule default true; Raid Omen 600; POI radius 64; strict reuse/view radius squared 9,216; initial and
inter-wave cooldown 300; ordinary groups 3/5/7; max omen five; active timeout 48,000; relocation
section radius two; spawn probes 8/20 with six outer failures; vertical bound 96; loaded margin 10;
outside-village cutoff seven seconds; cleanup cadence 20; removal radius squared 12,544; entity-age
check 600; inactivity 2,400 and outside-check limit 30; post-raid delay 40; celebration 600; hero
duration 48,000; horn distance 64 and projected offset 13. Raid RNG owns position angles/jitters,
bonus counts, boss UUIDs and horn seed; entity finalization uses each entity's ordinary RNG.

**Side effects:**

Omen position/effect consumption; manager insertion/removal/dirty state; raid status/counters/center;
stats, criteria and Hero effect; raid/entity RNG; raider creation, equipment, membership, buffs,
riders and insertion; bossbar membership/name/progress/visibility and sound packets. Rule-off
retirement itself emits no horn, reward, death, drop or direct raider-removal effect.

**Gates:**

Normal raid phase; live rule at each direct reader; spectator, effect and positional attribute;
occupied POIs and active-radius reuse; center chunk/village/difficulty; cooldown, group count,
tracked membership, spawn-position loading/placement; entity construction; hero resolution and
saved manager/entity reference integrity.

**Boundary cases and quirks:**

Rule-off omen work is consumed rather than paused. Rule-off managed retirement is delayed until a
raid phase and leaves raider entities alive. The first fresh ID is two. The active-radius comparison
is strict, while tracked-raider removal at 12,544 is inclusive. Losing the village sets loss or stop
but still executes one ongoing tail. Cleanup uses a pre-cleanup alive snapshot. Group count advances
after all-null creation, and insertion failures do not roll back membership. The celebration
counter and cached spawn position reset on reload because they are not persisted.

**Evidence:**

`OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.world.effect.BadOmenMobEffect#applyEffectTick`;
`net.minecraft.world.effect.RaidOmenMobEffect#applyEffectTick`;
`net.minecraft.server.level.ServerLevel#tick`, `#getRaidAt`;
`net.minecraft.world.entity.raid.Raids#tick`, `#createOrExtendRaid`, `#getOrCreateRaid`,
`#getNearbyRaid`, `#getUniqueId`, `#load`;
`net.minecraft.world.entity.raid.Raids$RaidWithId`;
`net.minecraft.world.entity.raid.Raid#absorbRaidOmen`, `#stop`, `#tick`,
`#moveRaidCenterToNearbyVillageSection`, `#findRandomSpawnPos`, `#spawnGroup`, `#joinRaid`,
`#updateRaiders`, `#updatePlayers`, `#playSound`, `#addWaveMob`, `#removeFromRaid`,
`#getNumGroups`, `#getPotentialBonusSpawns`;
`net.minecraft.world.entity.raid.Raid$RaiderType`;
`net.minecraft.world.entity.raid.Raider#aiStep`, `#die`, `#addAdditionalSaveData`,
`#readAdditionalSaveData`; `net.minecraft.world.attribute.EnvironmentAttributes`;
`data/minecraft/dimension_type/{overworld,overworld_caves,the_nether,the_end}.json`;
`SIM-PIPELINE-001`; `WGEN-DIMENSION-001`; `EXP-MOB-011`.

**Test vectors:**

Cross rule toggles before omen expiry, before/after a frozen raid phase and after map removal; all
spectator/effect/attribute/POI/reuse/max-level combinations; strict radius and first-ID endpoints.
Replay active/unloaded/peaceful/lost-village/timeout states, every cooldown and probe edge, exact
wave/bonus RNG streams, null creation and rejected insertion, cleanup thresholds and reward timing.
Save/reload every status and counter with loaded/unloaded/missing/duplicate raiders and manager IDs;
compare first subsequent state, membership, bossbar, horn, stats/effects and persistence bytes to an
uninterrupted control.
