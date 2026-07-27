# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-ILLUSIONER-001` — Illusioner mirror images are client-only, blindness never repeats a target and its Bow is spawn equipment

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`MOB-PATROL-001`, `MOB-RAID-001`, `ITM-ARROW-AMMUNITION-001`,
`ITM-ENCHANT-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PORTAL-001`, `CLI-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Illusioner` class and
its two nested use-spell goals, the shared `SpellcasterIllager` machine, the
inherited Raider/Patrolling-Monster graph, the bow ranged path, all 66
biomes, the empty loot table, two direct entity tags, the explicit
hostile-advancement exclusion set, the absent Spawn Egg, the Villager
hostile-avoidance row, six migration/schema contexts, all 1,212 templates
and exact entity/hat/illusion client resources close protocol entity
ID `68`.

**Applies when:**

`minecraft:illusioner` is constructed, finalized, produced by a command,
spawner or custom selector, patrolling, joining or celebrating an existing
raid, targeting an entity, casting its mirror or blindness spell, drawing
and firing its Bow, avoiding a Creaking, sensed by a Villager, damaged,
killed, synchronized, saved, loaded, heard, imitated by a Parrot or
rendered.

**Authoritative state:**

Protocol entity ID `68` constructs `Illusioner` in `MONSTER`, and
registration marks it unavailable in Peaceful. Its scalable dimensions are
`0.6×1.95` with no explicit eye height, so the default
`1.95*0.85=1.6575` applies. Registration fixes one passenger attachment at
`(0,2,0)`, riding offset `-0.6`, client tracking range `8` and the default
update interval `3`. It is neither fire-immune nor persistence-required.

Attributes start from the Monster set and fix movement speed `0.5`, follow
range `18` and maximum health `32`; inherited attack damage remains the
`ATTACK_DAMAGE` default `2`, and no goal ever applies it because the
Illusioner registers no melee attack goal. Construction fixes XP reward `5`
and allocates the `2×4` client illusion-offset matrix with every entry
`Vec3.ZERO`. It is an `Enemy`, so generic lead interaction cannot leash it,
and it has no age, breeding, subtype equipment population or interaction
path.

The Monster category cap is `70`, its no-despawn/despawn distances are
`32/128`, and its inherited maximum cluster size is `4`. A raid pointer or
generic persistence blocks distance removal; otherwise Patrolling-Monster
despawn semantics apply, where a patrolling Illusioner is only removed
beyond squared distance `16384`. Movement emission is `EVENTS`, gravity is
`0.08`, maximum head Y/X rotation is `75/40`, sound source is `HOSTILE` and
the ambient interval is `80`.

Inherited Entity, Living-Entity and Mob state occupies synchronized metadata
slots `0..15`. Raider adds slot `16`, serializer ID `8` (`BOOLEAN`),
`celebrating=false`. Spellcaster Illager adds slot `17`, serializer ID `0`
(`BYTE`), spell ID `0`. The Illusioner uses two of the six shared spell
identities:

| ID | Spell | Entity-Effect RGB | Used by Illusioner |
|---:|---|---|---|
| `0` | none | `0,0,0` | yes, as the cleared value |
| `1` | summon Vex | `0.7,0.7,0.8` | no |
| `2` | fangs | `0.4,0.3,0.35` | no |
| `3` | wololo | `0.7,0.5,0.2` | no |
| `4` | disappear | `0.3,0.3,0.8` | yes, mirror spell |
| `5` | blindness | `0.1,0.1,0.2` | yes, blindness spell |

An out-of-range byte maps to spell `0`. The server decides casting from
`spellCastingTickCount>0`, while the client decides it from slot `17>0`.

`SpellTicks` persists as an integer, default `0`. The current spell enum and
slot `17` do not persist. Each use goal's warmup and next-use tick, the
blindness goal's last-target ID, targets, active goals, ranged-attack
counters and every client illusion field are transient. Raider state
persists `Wave`, `CanJoinRaid` and optional `RaidId`; Patrolling-Monster
state persists optional `patrol_target`, `PatrolLeader` and `Patrolling`.
Slot `16` does not persist.

The four illusion offsets, the three-tick transition counter and the
previous-offset row exist only on the client. No server field, metadata slot
or packet carries them, so an unmodified client reconstructs every mirror
image locally from Invisibility, hurt timing and its own RNG.

**Transition and ordering:**

### Complete goal graph, targeting and alliance

`registerGoals` calls the inherited chain first, so Monster,
Patrolling-Monster and Raider registrations precede every Illusioner-local
registration at the same priority:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `0` | Float |
| goal | `1` | Obtain Raid-Leader Banner; Spellcaster Casting Spell, Move/Look |
| goal | `3` | Pathfind To Raid; avoid Creaking within `8`, walk/sprint `1/1.2` |
| goal | `4` | Long-Distance Patrol `0.7/0.595`; Raider Move Through Village `1.0499999523162842`, distance `1`; mirror spell |
| goal | `5` | Raider Celebration; blindness spell |
| goal | `6` | Ranged Bow Attack, speed `0.5`, minimum interval `20`, radius `15`, Move/Look |
| goal | `8` | Random Stroll speed `0.6`, default interval `120` |
| goal | `9` | Look At Player, range `3`, probability `1` |
| goal | `10` | Look At Mob, range `8`, default probability `0.02` |
| target | `1` | Hurt By, ignoring Raider attackers, alert same-class Illusioners |
| target | `2` | nearest Player, must see, unseen memory `300` |
| target | `3` | nearest Abstract Villager, need not see, unseen memory `300`; nearest Iron Golem, need not see |

Unlike Evoker, the Illusioner has no Player avoidance goal, and its Creaking
avoidance uses walk/sprint `1/1.2` instead of `0.6/1`. The inherited Illager
attack gate rejects baby Abstract Villagers even though the target selector
enumerates them. Target-goal random cadence, reach, navigation and
visibility retain `MOB-AI-001`.

Generic scoreboard alliance applies first. With no team on either side,
membership in `illager_friends` also makes another entity allied; that tag
contains `#illager`, which contains the Illusioner itself. The Illusioner
adds no further alliance rule.

The inherited raid goals own banner pickup, raid-center navigation, village
movement and celebration. Raider AI also attempts to join a nearby raid
every game-time multiple of `20` when `CanJoinRaid=true`, alive and not
already assigned. A Player or Iron-Golem target in an active raid resets the
inactivity counter. `applyRaidBuffs` is deliberately empty.

### Shared spell state machine and arbitration

Both use-spell goals inherit the shared admission gate: a present live
combat target, no active server casting timer, and
`tickCount>=nextAttackTickCount`. Neither goal declares control flags, so
the priority-`1` casting goal can run concurrently and exclusively claims
Move and Look.

On a successful start, a use goal:

1. stores `adjustedTickDelay(getCastWarmupTime())`; neither goal requires
   every-tick updates, so the base warmup `20` becomes
   `positiveCeilDiv(20,2)=10` goal ticks under the alternate-phase selector;
2. sets `spellCastingTickCount` to its casting duration `20`;
3. sets its next-use tick to current `tickCount + interval`;
4. plays its prepare sound at volume/pitch `1/1` when nonsilent; and
5. writes its spell ID to slot `17`.

The use-goal tick predecrements warmup. At exactly zero it performs the
spell, then plays Illusioner Cast Spell at `1/1`. Its continuation requires
a live combat target and positive warmup, so the use goal ends immediately
after the cast while the priority-`1` casting goal continues for the
remaining spell timer, stops navigation, and looks at the combat target with
limits `75/40`.

Every effective server-AI tick decrements a positive spell timer, because
the decrement lives in `customServerAiStep` rather than in a goal. Selector
arbitration instead runs the full `GoalSelector.tick` only when
`(tickCount+entityId)%2==0` or `tickCount<=1`, and otherwise ticks only
goals that require every-tick updates. Neither use-spell goal does, so a
use goal advances exactly once per two server ticks and its first advance
happens inside the same selector pass that started it.

Both Illusioner spells therefore have a fixed shape. Starting on a
phase-aligned server tick `T` sets warmup `10` and timer `20`; the warmup
reaches zero on the tenth goal advance at `T+18`, when the timer still has
`1` left. The cast lands one server tick before the lock expires, the timer
reaches `0` at `T+19`, and the next selector cleanup at `T+20` stops the
casting goal, which writes spell `0`. Cooldowns are measured from spell
start, not completion.

Registration order tests the mirror goal at priority `4` before the
blindness goal at priority `5`; a successful mirror start makes the later
blindness admission see an active timer that same tick.

`NoAI` suppresses selector ticks and the custom timer decrement. Setting it
during a cast can therefore freeze both server casting state and the synced
spell byte until AI resumes or another write occurs.

### Mirror spell

The mirror spell uses spell ID `4` (`DISAPPEAR`), warmup/casting/interval
`20/20/340`, and Illusioner Prepare Mirror. Beyond the shared gate it
requires only that the Illusioner not already have Invisibility.

Casting adds `MobEffectInstance(minecraft:invisibility, 1200)` to the
Illusioner itself with amplifier `0`, no source entity and default ambient,
visibility and icon flags; the boolean result is discarded. Nothing else is
produced: no entity, particle, game event or extra sound. Because the
admission gate reads the effect and the effect lasts `1200` ticks while the
cooldown is `340` ticks, a live Illusioner is invisible essentially
continuously once a target exists, and the goal re-admits only in the gap
after the effect actually ends or is removed.

Invisibility is projected through ordinary Living-Entity effect
synchronization. The server sends no illusion data.

### Blindness spell

The blindness spell uses spell ID `5` (`BLINDNESS`), warmup/casting/interval
`20/20/180`, and Illusioner Prepare Blindness. Beyond the shared gate it
requires all three of:

- a nonnull current target;
- `target.getId() != lastTargetId`; and
- `level.getCurrentDifficultyAt(blockPosition()).isHarderThan(2.0)`, that is
  effective regional difficulty strictly greater than `2`.

`start` first runs the shared start, then records the current target's
entity ID in `lastTargetId` when that target is nonnull.

Casting adds `MobEffectInstance(minecraft:blindness, 400)` to the current
target with amplifier `0` and this Illusioner as the effect source; the
boolean result is discarded.

The regional-difficulty gate is exact. `getCurrentDifficultyAt` builds
`DifficultyInstance(baseDifficulty, overworldClockTime, chunkInhabitedTime,
moonBrightness)` and

- `f = 0.75 + clamp((clockTime-72000)/1440000, 0, 1)*0.25`;
- `g = clamp(inhabitedTime/3600000, 0, 1)*(hard ? 1 : 0.75)
  + clamp(moonBrightness*0.25, 0, f-0.75)`;
- Easy halves `g`; the result is `baseId*(f+g)`.

Hard therefore always exceeds `2` because its minimum is `3*0.75=2.25`.
Easy can never exceed `2` because its maximum is `1*(1+0.625)=1.625`.
Normal exceeds `2` exactly when `f+g>1`, so a freshly generated,
never-inhabited chunk at new moon early in a world does not qualify while an
inhabited or moonlit chunk does. Peaceful is unreachable because the entity
type is Peaceful-excluded.

Because `lastTargetId` is only rewritten on a successful start, the same
entity can never be blinded twice in a row by the same Illusioner. Blinding
a second entity and then returning to the first re-enables the first. The
field is transient, so reload clears it and permits an immediate repeat.

### Bow attack

`finalizeSpawn` sets `MAINHAND` to a fresh `minecraft:bow` stack before
delegating to the inherited chain, so every finalized Illusioner is armed.
The slot keeps the default drop chance `0.085`. An Illusioner created
without finalization has no bow, and the ranged goal then cannot start.

`RangedBowAttackGoal` requires a target and `isHolding(minecraft:bow)`,
requests Move/Look, and updates every tick. It marks the mob aggressive on
start and clears aggression, sight counter, attack counter and item use on
stop. It tracks `seeTime`, resets it to `0` on any visibility change, and

- navigates toward the target at speed `0.5` while squared distance exceeds
  `225` or `seeTime<20`, resetting `strafingTime` to `-1`; otherwise stops
  navigation and increments `strafingTime`;
- at `strafingTime>=20` consumes two `nextFloat` draws, flipping clockwise
  and backwards independently below `0.3`, then resets `strafingTime` to
  `0`;
- once `strafingTime>-1`, clears backwards above `225*0.75` and sets it
  below `225*0.25`, strafes `(+/-0.5, +/-0.5)`, and looks at the target
  with limits `30/30`, additionally rotating a controlled Mob vehicle;
- while using the bow, releases at `getTicksUsingItem()>=20` with power
  `BowItem.getPowerForTime(ticks)` and rearms `attackTime` to `20`, or stops
  the draw when sight is lost and `seeTime<-60`;
- otherwise counts `attackTime` down and starts the draw at or below zero
  while `seeTime>=-60`.

`performRangedAttack(target, velocity)` resolves the bow-holding hand,
reads the held stack, resolves the projectile stack, and builds a Mob arrow
from it. With
`dx=targetX-x`, `dy=target.getY(1/3)-arrow.getY()`, `dz=targetZ-z` and
`dist=sqrt(dx*dx+dz*dz)`, a server level shoots the projectile at
`(dx, dy+dist*0.20000000298023224, dz)` with power `1.6` and inaccuracy
`14-4*difficultyId`, i.e. `10/6/2` on Easy/Normal/Hard. It then plays
`minecraft:entity.skeleton.shoot` at volume `1` and pitch
`1/(nextFloat*0.4+0.8)`. The Illusioner has no arrow item of its own, so
projectile identity, pickup state, damage, enchantment transfer and
crit/power scaling remain `ITM-ARROW-AMMUNITION-001` and
`ENT-PROJECTILE-001`.

The bow goal and both spell goals hold no shared flag, so a casting
Illusioner keeps aggression from an earlier bow draw only until the casting
goal's Move/Look claim stops the ranged goal through selector arbitration at
the lower priority number.

### Client illusion projection

Every client tick the shared spellcaster tick emits two local Entity-Effect
particles, protocol particle ID `28`, using the current spell's RGB and zero
velocity, while slot `17>0`. With

`a = yBodyRot*pi/180 + cos(tickCount*0.6662)*0.25`,

the positions are the opposite points
`(x +/- cos(a)*0.6*scale, y+1.8*scale, z +/- sin(a)*0.6*scale)`. This
consumes no RNG. Mirror casts therefore show `0.3,0.3,0.8` and blindness
casts `0.1,0.1,0.2`.

The Illusioner adds a client-only illusion step in `aiStep`, evaluated after
the inherited step and only when the level is client-side and the entity is
invisible:

1. predecrement `clientSideIllusionTicks`, clamping negatives to `0`;
2. if `hurtTime==1` or `tickCount%1200==0`, shift row `1` into row `0` and
   redraw all four row-`1` offsets as
   `((-6+nextInt(13))*0.5, max(0, nextInt(6)-4), (-6+nextInt(13))*0.5)`,
   set the transition counter to `3`, emit sixteen Cloud particles,
   protocol particle ID `11`, and play a local Illusioner Mirror Move at
   `1/1` without distance delay;
3. otherwise, if `hurtTime==hurtDuration-1`, shift row `1` into row `0`, set
   every row-`1` offset to zero and set the transition counter to `3`.

Each redraw consumes exactly twelve `nextInt` draws in the order
`X, Y, Z` per illusion. X and Z land on `-3.0..3.0` in steps of `0.5`; Y is
`0` with probability `5/6` and `1` with probability `1/6`.

The sixteen Cloud particles use `getRandomX(0.5)`, `getRandomY()` and
`getZ(0.5)`. The Z coordinate is the deterministic
`z + width*0.5`, not a randomized offset, so the burst is visibly biased to
one side rather than centered.

`hurtDuration` is `10` and `hurtTime` counts down from it, so a hit first
collapses the illusions onto the real body at `hurtTime==9` and re-scatters
them eight ticks later at `hurtTime==1`. The `tickCount%1200==0` branch
re-scatters on the same period as the mirror effect duration.

`getIllusionOffsets(partialTick)` returns row `1` directly while the
transition counter is at most zero. Otherwise it eases with
`d = ((clientSideIllusionTicks - partialTick)/3)^0.25` and returns
`row1[i]*(1-d) + row0[i]*d` for each of the four entries.

The renderer copies those four offsets and the casting flag into its render
state. When the state is invisible it submits the full model once per
offset, translating each copy by

`(off.x + cos(i + ageInTicks*0.5)*0.025,
off.y + cos(i + ageInTicks*0.75)*0.0125,
off.z + cos(i + ageInTicks*0.7)*0.025)`,

and otherwise submits a single copy. `isBodyVisible` is overridden to always
return `true`, so an invisible Illusioner renders as an opaque model instead
of the usual hidden or translucent form, and never receives the
`0x26FFFFFF` invisibility tint. The culling box is the inherited box
inflated by `(3,0,3)`, matching the `3`-block illusion spread.

The arm pose is Spellcasting while the client byte is positive, otherwise
Bow-and-Arrow while the aggressive flag is set, otherwise Crossed. The
Illusioner never reports Celebrating even while slot `16` is true, so raid
celebration changes its jump behavior but not its arms. Spellcasting places
the right/left arms at X `-5/+5`, Z `0`, rotates both X by
`cos(ageInTicks*0.6662)*0.25`, sets Z to `+/-2.3561945` and Y to `0`.
Bow-and-Arrow adds head yaw `-0.1` and pitch `-1.5707964` to the right arm,
head pitch `-0.9424779` and yaw `-0.4` to the left arm, and left-arm Z
`1.5707964`. The item-in-hand layer is submitted only while casting or
aggressive.

`IllusionerRenderer` uses `ModelLayers.ILLUSIONER`, the shared Illager
model, shadow radius `0.5` and `textures/entity/illager/illusioner.png`.
The base `IllagerModel` constructor hides the `hat` cube; the Illusioner
renderer re-enables it, so the Illusioner is the only Illager that renders
the `texOffs(32,0)` `8×12×8` head cube at inflation `0.45`. The `64×64`,
`1,019`-byte texture has SHA-256
`f43b9eecec0f7c846f673297c5ceff16b091359c589b8646abf82c5308bbfa75`.

### Production, placement and persistence

The Illusioner has no baseline producer at all. It is absent from the raid
raider-type table, so raids never spawn it and it can only join one that
already exists. `PatrolSpawner` selects Pillagers only. All `66` locked
baseline biomes contain zero Illusioner spawn rows, and all `1,212` locked
structure templates contain zero exact `minecraft:illusioner` or legacy
`minecraft:illusion_illager` identity. There is no Spawn Egg item.

The separate placement registration is nevertheless present:
`NO_RESTRICTIONS` with heightmap `MOTION_BLOCKING_NO_LEAVES` and the
standard `Monster.checkMonsterSpawnRules` predicate. It checks darkness
unless the reason ignores light, then generic Mob placement;
`NO_RESTRICTIONS` itself adds no support or medium test. With no biome rows,
that registration only serves spawner, command and custom paths.

`/summon` with initialization builds the entity with reason `COMMAND` and
calls `finalizeSpawn` at the current regional difficulty with null group
data, so the command path arms the bow and runs both inherited finalization
branches.

Patrolling-Monster finalization for reasons other than `PATROL`, `EVENT` or
`STRUCTURE` consumes one level `nextFloat`; below `0.06`, an eligible
Illusioner becomes patrol leader, equips the ominous banner in its head slot
and sets drop chance `2`. Reason `PATROL` instead marks it patrolling, while
Raider finalization sets `CanJoinRaid=true` for every Illusioner reason
because the Witch exemption does not apply. Baseline patrol production never
selects an Illusioner, but every custom finalization path retains these
inherited branches.

### Loot, tags, advancements and item identity

The entity loot table uses random sequence `minecraft:entities/illusioner`
and declares no pools at all, so a killed Illusioner drops nothing from
loot. The generator confirms this by building the table with
`LootTable.lootTable()` and no `withPool` call.

The fixed XP reward is `5`. Equipment drops remain the only item source: the
main-hand Bow at the default chance `0.085`, and, for a patrol leader, the
ominous banner at the guaranteed chance `2`. Mob-loot gamerule, equipment
drop rolls, item merging and death ordering retain their cited owners.

Exactly two direct entity-type tags name the Illusioner:

- `illager`, which also reaches `illager_friends` through nested
  membership; and
- `raiders`.

Their consumers own teamless Illager alliance, raid membership, Bell
highlighting and other tag-selected behavior. No other locked entity-type
tag directly names the identity.

Neither hostile-mob advancement names the Illusioner. The advancement
generator's `EXCEPTIONS_BY_EXPECTED_CATEGORIES` maps `MONSTER` to the exact
exclusion set `{giant, illusioner, warden, sulfur_cube}`, so `kill_a_mob`
and `kill_all_mobs` deliberately omit it and completing `kill_all_mobs`
never requires killing one.

There is no `minecraft:illusioner_spawn_egg`; the locked item registry
contains no Illusioner identity. The only English label for the entity is
“Illusioner”.

`VillagerHostilesSensor` maps the Illusioner to avoidance distance `12`,
between Husk `8` and Pillager `15`. That row is the entity's only
Villager-brain join.

### Sounds

The locked sound-event joins are:

| Protocol ID | Event | English subtitle |
|---:|---|---|
| `882` | Illusioner Ambient | “Illusioner murmurs” |
| `883` | Illusioner Cast Spell | “Illusioner casts spell” |
| `884` | Illusioner Death | “Illusioner dies” |
| `885` | Illusioner Hurt | “Illusioner hurts” |
| `886` | Illusioner Mirror Move | “Illusioner displaces” |
| `887` | Illusioner Prepare Blindness | “Illusioner prepares blindness” |
| `888` | Illusioner Prepare Mirror | “Illusioner prepares mirror image” |

`getCelebrateSound` returns Illusioner Ambient, so raid celebration reuses
the ambient event rather than a dedicated one. The bow release uses
`minecraft:entity.skeleton.shoot`, protocol ID `1491`, not an Illusioner
event. Ambient cadence and generic sound broadcast retain their owners.
Parrot imitation maps the Illusioner to sound-event ID `1229`,
`entity.parrot.imitate.illusioner`, subtitle “Parrot murmurs”; the Parrot's
attempt cadence, nearby selection, silence gate and playback retain the
Parrot owner.

Illusioner Mirror Move is emitted by `playLocalSound` on the client that
owns the illusion redraw, so it is never broadcast by the server and never
reaches a client that does not currently render the entity as invisible.

### Migration and schema closure

Six exact migration/schema contexts name the Illusioner family:

- `EntityTheRenameningFix` maps `minecraft:illusion_illager` to
  `minecraft:illusioner`;
- schema `V705` registers the legacy `minecraft:illusion_illager` entity
  shape;
- schema `V1460` registers the legacy `minecraft:illusion_illager` entity
  shape for its own generation;
- schema `V1510` moves the legacy Illusioner entity shape to the current
  name;
- `EntityUUIDFix` includes current `minecraft:illusioner` in the Mob UUID
  rewrite set; and
- `BlockPosFormatAndRenamesFix` includes it among the six patrolling mobs
  whose `PatrolTarget` becomes codec-shaped `patrol_target`.

There is no Spawn-Egg migration because no such item ever existed. Generic
entity, equipment, effect, raid and patrol fields retain their schema
owners.

**Branches and aborts:**

- Shared spell admission aborts on a missing or dead target, an active
  casting timer or a per-goal cooldown.
- The mirror goal additionally aborts while Invisibility is present.
- The blindness goal additionally aborts on a null target, a target whose
  entity ID equals the recorded last target, or effective regional
  difficulty at most `2`.
- Both casts discard the `addEffect` result, so a rejected or merged effect
  produces no retry and no observable difference in timers or sound.
- The bow goal aborts without a target or without a held Bow; it also stops
  the draw once sight is lost and `seeTime<-60`.
- The client illusion step aborts entirely when the entity is not invisible,
  leaving the offset rows and transition counter untouched.
- Placement has no baseline biome selector despite its registered predicate,
  and no structure, raid or patrol producer supplies one.

**Invariants:**

- Slot `17` is the client spell identity; `SpellTicks` is the independent
  server cast lock.
- Both spells use casting time `20` and effective warmup `10` goal advances,
  so an uninterrupted cast lands one server tick before its own lock ends.
- The mirror arbitration at priority `4` always precedes the blindness
  arbitration at priority `5` on a shared target and timer boundary.
- Invisibility is applied to the Illusioner itself and blindness to its
  target; neither spell ever affects the other party.
- The four mirror images exist only on the client and are reconstructed from
  Invisibility, hurt timing and client RNG.
- The same target entity ID is never blinded twice consecutively.
- An Illusioner drops no loot; only equipment and XP `5` are observable.
- No baseline biome row, raid wave, patrol, structure template or Spawn Egg
  produces an Illusioner.
- No advancement criterion names the Illusioner.

**Constants and randomness:**

Entity ID `68`; no Egg or loot item; dimensions/eye/passenger/riding
`0.6×1.95/1.6575/2/-0.6`; range/update `8/3`;
health/speed/follow/attack `32/0.5/18/2`; XP `5`;
slots `16 BOOLEAN/17 BYTE`; Creaking avoidance `8/1/1.2`; wander `0.6/120`;
look `3/1` and `8/0.02`; unseen memory `300`; spell IDs `4/5`;
mirror warmup/cast/interval/effect `20/20/340/1200`; blindness
warmup/cast/interval/effect/gate `20/20/180/400/>2`; effective warmup `10`;
selector phase `(tickCount+entityId)%2`, cast landing `T+18` of `T+20`;
bow speed/interval/radius `0.5/20/15`, draw `20`, sight floor `-60`,
strafe `20/0.3/0.75/0.25/0.5`, look `30/30`, shot power `1.6`, inaccuracy
`14-4*difficultyId`, aim lift `0.20000000298023224`, target aim height
`1/3`; illusions `4`, transition `3`, spread `3`, redraw period `1200`,
offset draws `nextInt(13)/nextInt(6)/nextInt(13)`, cloud particles `16`,
render wobble `0.025/0.0125/0.025` at `0.5/0.75/0.7`, ease exponent `0.25`;
`hurtDuration` `10`; equipment drop `0.085`, leader banner `2`,
patrol-leader chance `0.06`; Monster cap/distances/cluster `70/32/128/4`;
patrolling despawn `16384`; Villager avoidance `12`; tags/templates/biomes
`2/0 of 1212/0 of 66`; shadow `0.5`.

**Side effects:**

Metadata, spell timer, patrol/raid/persistence and equipment state; RNG
cursors for illusion offsets, strafing and shot pitch; targets, goal
arbitration, navigation, strafing and look; Invisibility on itself and
Blindness on its target; arrow entities and their projectile state;
entity/game events, particles and sounds; XP and equipment drops;
renderer/model state and the client illusion matrix.

**Gates:**

Logical side, Peaceful, NoAI and persistence; goal priority/flags and
cooldowns; target class/liveness/sight and baby-Villager exclusion; Raider
and Illager/team alliance; present Invisibility, recorded last target and
effective regional difficulty; held Bow, sight counter and draw duration;
placement reason/light/Mob predicate and absent biome rows; equipment drop
chance and mob loot; invisibility, hurt timing and tick phase for illusions;
silence, metadata, camera and resources; migrations.

**Boundary cases and quirks:**

The mirror images have no server existence, so they cannot be hit, targeted,
or observed by any packet other than the Invisibility effect. The illusion
Cloud burst randomizes X and Y but uses the deterministic `getZ(0.5)`, so it
is offset to one side. Getting hit first collapses the images and only
re-scatters them eight ticks later. An invisible Illusioner still renders
fully opaque because `isBodyVisible` is overridden. Blindness requires
strictly more than difficulty `2`, so Easy Illusioners never blind at all
and early Normal worlds may not either. The last-target latch means a
solo player is blinded once and then never again until the Illusioner
targets something else, but a reload clears the latch. The Illusioner keeps
`getCelebrateSound` yet never shows the Celebrating arm pose. It is the only
Illager whose hat cube renders. It has no Spawn Egg, no loot table pools and
no advancement criteria, so an unmodified survival world provides no way to
obtain or complete anything from it. A finalized Illusioner carries a Bow it
can drop, while a raw `/summon` variant that skips initialization has no bow
and therefore no ranged goal. Persisted positive `SpellTicks` reloads
without its spell byte, creating a server-only cast lock with no client
spell pose or particles until the timer expires.

**Failure semantics:**

Interrupted use goals produce no spell effect, while their cast timer and
start-anchored cooldown continue until their separate owners clear or expire
them. A rejected or merged effect application is discarded, so the sound,
timer and cooldown still commit. A failed projectile spawn is not rolled
back and the release sound still plays. Client illusion state is never
reconciled with the server and simply resets on unload, reload or a
visibility change. Failed raid joins follow that owner's no-rollback
behavior.

**Client/server authority split:**

The server owns targets, goals, timers, spell selection, effect application,
bow drawing and projectile creation, patrol/raid state, XP and equipment
drops. Clients consume slots `16/17`, effect synchronization, movement and
resources; they emit spell and illusion particles, synthesize the four
mirror images and their easing, play the local mirror-move sound, choose
crossed/spell/bow arms and render the Illusioner, its hat and its held
items. Client illusions, particles or stale metadata cannot commit server
effects, and no client input can create or destroy an illusion.

**Observability:**

Observe registration/attributes and metadata slots; save/reload divergence
of `SpellTicks`, the last-target latch and the illusion matrix; the full
inherited/local goal graph and same-tick mirror-versus-blindness
arbitration; all warmup/timer/cooldown edges; the Invisibility and
Blindness gates across every difficulty and regional-difficulty endpoint;
bow acquisition, draw, strafe, inaccuracy and release; illusion redraw
periods, hurt-driven collapse and re-scatter, RNG order, Cloud asymmetry and
easing; placement across `66` biomes with zero rows; patrol-leader and
raid-join branches; zero-pool loot, equipment drops and XP; two tags,
excluded advancements, absent Egg and Villager avoidance row; six
migrations, seven sounds plus the Skeleton shot and Parrot imitation, and
exact model/hat/texture/culling projection.

**Persistence and reload:**

Generic entity/Mob state, `SpellTicks`, patrol fields, wave/join/raid state,
equipment and generic persistence save. Spell ID/metadata, celebration,
warmups/cooldowns, the blindness last-target latch, active goals, ranged
counters and every client illusion field do not. A reloaded positive timer
remains server-active with current spell `NONE` and slot `17=0`. Active
Invisibility and Blindness persist through their own effect owners, so a
reloaded Illusioner can still be invisible with a cleared latch. Loot, tags
and biome data reload through their owners; raid and spell code remain
fixed. Language, models and textures reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.DropChances`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.PatrollingMonster`;
`net.minecraft.world.entity.raid.Raider`;
`net.minecraft.world.entity.monster.illager.AbstractIllager`;
`net.minecraft.world.entity.monster.illager.AbstractIllager$IllagerArmPose`;
`net.minecraft.world.entity.monster.illager.SpellcasterIllager`;
`net.minecraft.world.entity.monster.illager.SpellcasterIllager$SpellcasterCastingSpellGoal`;
`net.minecraft.world.entity.monster.illager.SpellcasterIllager$SpellcasterUseSpellGoal`;
`net.minecraft.world.entity.monster.illager.SpellcasterIllager$IllagerSpell`;
`net.minecraft.world.entity.monster.illager.Illusioner`;
`net.minecraft.world.entity.monster.illager.Illusioner$IllusionerMirrorSpellGoal`;
`net.minecraft.world.entity.monster.illager.Illusioner$IllusionerBlindnessSpellGoal`;
`net.minecraft.world.entity.ai.goal.Goal`;
`net.minecraft.world.entity.ai.goal.GoalSelector`;
`net.minecraft.world.entity.ai.goal.RangedBowAttackGoal`;
`net.minecraft.world.entity.ai.goal.AvoidEntityGoal`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.entity.ai.sensing.VillagerHostilesSensor`;
`net.minecraft.world.entity.projectile.ProjectileUtil`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.world.entity.raid.Raid$RaiderType`;
`net.minecraft.world.level.levelgen.PatrolSpawner`;
`net.minecraft.world.DifficultyInstance`;
`net.minecraft.server.level.ServerLevel`;
`net.minecraft.server.commands.SummonCommand`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.advancements.packs.VanillaAdventureAdvancements`;
`net.minecraft.data.tags.EntityTypeTagsProvider`;
`net.minecraft.util.datafix.fixes.EntityTheRenameningFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.BlockPosFormatAndRenamesFix`;
`net.minecraft.util.datafix.schemas.V705`; `V1460`; `V1510`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.IllagerRenderer`;
`net.minecraft.client.renderer.entity.IllusionerRenderer`;
`net.minecraft.client.renderer.entity.LivingEntityRenderer`;
`net.minecraft.client.renderer.entity.state.IllagerRenderState`;
`net.minecraft.client.renderer.entity.state.IllusionerRenderState`;
`net.minecraft.client.model.monster.illager.IllagerModel`;
`net.minecraft.client.model.geom.ModelLayers`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,particle_type,mob_effect}`;
`data/minecraft/tags/entity_type/{illager,illager_friends,raiders}.json`;
`data/minecraft/loot_table/entities/illusioner.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/textures/entity/illager/illusioner.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-PROJECTILE-001`; `ENT-DAMAGE-001`; `ENT-DEATH-001`;
`ENT-EFFECT-001`; `MOB-AI-001`; `MOB-SPAWN-001`; `MOB-DESPAWN-001`;
`MOB-PATROL-001`; `MOB-RAID-001`; `ITM-ARROW-AMMUNITION-001`;
`ITM-ENCHANT-001`; `CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-026` across construction/metadata/NoAI/save/reload,
finalization/bow/patrol/raid/persistence and the complete goal graph; every
mirror and blindness admission, warmup/timer/cooldown, difficulty and
last-target boundary; bow acquisition, sight, strafe, draw, inaccuracy and
release paths; all illusion redraw, collapse, easing, RNG-order and Cloud
asymmetry cases; placement across all `66` biomes with zero rows, spawner
and command paths, cap/despawn boundaries; zero-pool loot, equipment drops
and XP; two tags, both excluded advancements, absent Egg, Villager
avoidance, all `1,212` templates, six migrations, seven sounds plus Skeleton
shot and Parrot imitation, and exact particle/model/hat/texture/culling
projection.

**Limits:**

Generic lifecycle, metadata, equipment, pathfinding, target/avoidance
algorithms, damage/death, effect application and expiry, arrow projectile
runtime, patrol/raid orchestration, natural spawning and despawn, loot,
advancements, particles and rendering retain their cited owners. Shared
Raider/Illager/Spellcaster algorithms are included only where the Illusioner
subtype registers, selects or changes their exact inputs and observable
result.
