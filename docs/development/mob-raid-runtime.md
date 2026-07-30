# MOB-RAID-001 Raid Runtime

`G01-P7-S011` implements the source-specified `MOB-RAID-001` slice as deterministic, protocol-
neutral Region transitions. The owning Region supplies live gamerules, difficulty, POI snapshots,
loaded/ticking observations, stable manager iteration order, entity lookups and explicit raid RNG
draws.

## Responsibility split

`ferrite-gameplay::mob::runtime::sim_002` has four owners:

- `omen` owns Bad Omen conversion, Raid Omen expiry, create/extend admission, center selection,
  active-raid reuse, ordinary group counts and omen absorption;
- `manager` owns preincrement IDs, live-rule retirement, dirty cadence, persisted-manager facts and
  raider reconstruction;
- `raid` owns active/Peaceful admission, village loss, active/cooldown/post-raid clocks, cleanup,
  rewards and celebration;
- `waves` owns spawn probes, fixed/random counts, member commits, ravager riders and horn
  recipients.

The transitions do not own global entity storage, chunk loading, POI search, packet emission or
random-number generation. `G01-P7-B1` composes them with Region-owned state, and protocol family
batches project the resulting bossbar, sound, entity and effect changes.

## Omen and manager admission

The `raids` rule defaults true and is read live. A nonspectator player in a village and outside
Peaceful converts Bad Omen into Raid Omen for 600 ticks, preserving the amplifier and snapshotting
the player position. At remaining duration one, Raid Omen invokes create-or-extend, clears the
snapshot and removes itself regardless of the operation result.

Create-or-extend rejects in spectator, gamerule and `CAN_START_RAID` order. The center is the
component-wise floored mean of occupied village POIs within radius 64; an empty POI set uses the
saved player position. The manager reuses the nearest active raid only at strict squared distance
below 9,216. Otherwise it creates an active ongoing raid with cooldown 300 and 0/3/5/7 ordinary
groups for Peaceful/Easy/Normal/Hard. `nextId` begins at one and preincrements, so the first assigned
ID is two.

An already-started level-five raid skips omen absorption. Other admissions add amplifier plus one
and clamp to five. Before any group, a present effect awards the trigger criterion and statistic.
The manager is marked dirty even when absorption is skipped.

Each admitted manager tick increments its clock before visiting raids. It rereads the gamerule for
every raid. A disabled rule stops the raid, hides and clears the bossbar, removes the manager entry,
marks dirty and does not tick the raid or delete its raiders. Already-stopped raids are removed the
same pass. The manager is also dirty every 200 ticks.

## Ongoing state machine

An ongoing raid derives `active` from whether its center chunk is loaded, applies bossbar visibility
changes, stops immediately in Peaceful and returns when inactive. Losing the current village either
moves the center to the nearby village section, stops before any group, or marks loss after a group;
this branch deliberately continues through the remaining tick.

Active ticks increment before the 48,000 timeout and twenty-tick cleanup test. The bossbar suffix
uses the pre-cleanup raider count only for counts one and two. Cleanup removes tracked members that
are removed, in another dimension, at squared distance at least 12,544, unresolved at age 600, or
outside the village after at least 30 checks with no-action time strictly above 2,400. Removal
clears raid membership and progress but never subtracts the historical health denominator.

When a later wave has no members at cooldown zero, the raid resets cooldown to 300, installs the
title and returns. A positive cooldown recomputes an absent cached spawn only at multiples of five,
but immediately recomputes a cached position whose chunk is not entity-ticking. Membership refresh
runs at 300 and every multiple of 20. Progress is the clamped `(300-cooldown)/300` value before the
decrement.

The ordinary wave counter increments even if every entity construction returns null. Six failed
outer spawn-position acquisitions stop the raid. After all ordinary waves, omen above one adds one
bonus group using the ordinary group count as its fixed-table index. When the final member is gone,
post-raid ticks below 40 increment; victory is entered only when the existing counter is already
40.

Victory grants living nonspectators Hero of the Village for 48,000 ticks at amplifier
`raidOmenLevel-1`, with hidden particles and a visible icon. Player recipients also receive the
statistic and criterion. Celebration stops at 600 ticks; the manager removes the stopped raid on
its next pass.

## Wave construction and projection

Normal spawn search performs eight attempts; the wave-time fallback performs twenty. A search
draws one base angle, then two jitters per attempt, steps by pi/8 and uses radial factor
`0.22*(cooldown/20)-0.24`. Candidates require vertical difference at most 96, loaded chunks through
a margin of ten, an entity-ticking position, valid placement or snow-with-air and, above cooldown
quotient seven, a position outside the village.

The five fixed rows are Vindicator `[0,2,0,1,4,2,5]`, Evoker `[0,0,0,0,1,1,2]`, Pillager
`[4,3,3,4,4,4,2]`, Witch `[0,0,0,3,0,0,1]` and Ravager `[0,0,1,0,1,0,2]`. Vindicator and Pillager
extra bounds are Easy's initial binary draw, one on Normal and two on Hard; Easy performs the
second inclusive draw only when that bound is positive. Witch uses a binary bound on non-Easy waves
above two except wave four. Ravager uses it only for a non-Easy bonus group. Evoker has no random
extra.

The first leader-capable member becomes leader and receives the banner. Health enters the
denominator before unchecked entity insertion; raid state, center-plus-one position, event
finalization, buffs and on-ground state are committed. A null construction ends only that raider
type's loop. Wave-five Ravagers carry Pillagers; from wave seven the first carries an Evoker and
the rest Vindicators.

Bossbar membership follows the nearest active raid at strict distance below 96 blocks. Horn
projection includes players within horizontal distance 64 and any distant existing bossbar member,
and points from player height toward the spawn with the audited offset. One random long supplies the
wave sound draw.

## Persistence and reconstruction

The manager persists its required saved fields. Runtime group maps, leader maps, RNG, boss event,
cached spawn position and celebration clock are reconstructed rather than persisted. A raider saves
`Wave`, `CanJoinRaid` and `RaidId` only while its raid still resolves in the manager.

Loading a resolvable raid ID replaces an equal UUID through `addWaveMob(false)`, so health is not
counted twice, and restores a saved patrol leader. A missing ID leaves the raider unattached.

## Validation

`crates/ferrite-gameplay/tests/slices/mobs/sim_002.rs` owns the slice. Its fourteen tests lock omen
gates and expiry; create order, center floor, reuse radius, group counts and IDs; absorption; manager
retirement and dirty cadence; active/village/cooldown/post-raid boundaries; cleanup and rewards;
fixed/random wave counts; spawn probes; member/rider/horn rules; and persistence reconstruction.
