# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-EVOKER-001` — Evokers arbitrate three spells, seed raids and mansions, and create owned Vexes and fangs

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`MOB-RAID-001`, `ITM-EMERALD-001`, `PLY-AUTOJUMP-001`,
`WGEN-005`, `WGEN-PORTAL-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Evoker`,
`SpellcasterIllager`, three use-spell goals and fang-construction paths,
Raider/Patrolling-Monster joins, all 66 biomes, raid and Woodland-Mansion
producers, two direct entity tags, loot, both hostile-mob advancements,
Spawn Egg, seven migration/schema classes and exact Evoker/fang/item client
resources close protocol entity ID `46`.

**Applies when:**

`minecraft:evoker` is constructed, finalized, produced by a raid or Woodland
Mansion, spawned by an Egg, spawner, command or custom selector, patrolling,
joining or celebrating a raid, targeting or avoiding another entity,
summoning Vexes, creating Evoker Fangs, charming a Sheep, killed,
synchronized, saved, loaded, heard, imitated by a Parrot or rendered.

**Authoritative state:**

Protocol entity ID `46` constructs `Evoker` in `MONSTER`, and registration
marks it unavailable in Peaceful. Its scalable dimensions are
`0.6×1.95`, with default eye height `1.95*0.85=1.6575`, one passenger
attachment at `(0,2,0)`, registered riding offset `-0.6`, client tracking
range `8` and the default update interval `3`.

Attributes start from the Monster set and fix movement speed `0.5`, follow
range `12` and maximum health `24`; inherited attack damage remains `2`.
Construction fixes XP reward `10`. It has no direct melee goal, subtype
equipment population, age, breeding or interaction path. It is an `Enemy`,
so generic lead interaction cannot leash it.

The Monster category cap is `70`, its no-despawn/despawn distances are
`32/128`, and its inherited maximum cluster size is `4`. A raid pointer or
generic persistence blocks distance removal; otherwise Patrolling-Monster
despawn semantics apply. Movement emission is `EVENTS`, gravity is `0.08`,
maximum head Y/X rotation is `75/40`, and generic spawn obstruction applies.

Inherited Entity, Living-Entity and Mob state occupies synchronized metadata
slots `0..15`. Raider adds slot `16`, serializer ID `8` (`BOOLEAN`),
`celebrating=false`. Spellcaster Illager adds slot `17`, serializer ID `0`
(`BYTE`), spell ID `0`. Spell IDs and RGB colors are:

| ID | Spell | Entity-Effect RGB |
|---:|---|---|
| `0` | none | `0,0,0` |
| `1` | summon Vex | `0.7,0.7,0.8` |
| `2` | fangs | `0.4,0.3,0.35` |
| `3` | wololo | `0.7,0.5,0.2` |

IDs `4/5` are the inherited disappear/blindness values used by other
spellcasters; an out-of-range byte maps to spell `0`. The server decides
casting from `spellCastingTickCount>0`, while the client decides it from
slot `17>0`.

`SpellTicks` persists as an integer, default `0`. The current spell enum and
slot `17` do not persist. Each use goal's warmup and next-use tick, the
Wololo target, targets, active goals and particle phase are transient.
Raider state persists `Wave`, `CanJoinRaid` and optional `RaidId`;
Patrolling-Monster state persists optional `patrol_target`,
`PatrolLeader` and `Patrolling`. Slot `16` does not persist.

**Transition and ordering:**

### Complete goal graph, targeting and alliance

The Evoker graph includes its inherited patrol/raid goals and exact local
registrations:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `0` | Float |
| goal | `1` | Obtain Raid-Leader Banner; Evoker Casting Spell |
| goal | `2` | avoid Player within `8`, walk/sprint `0.6/1` |
| goal | `3` | Pathfind To Raid; avoid Creaking within `8`, walk/sprint `0.6/1` |
| goal | `4` | Long-Distance Patrol `0.7/0.595`; Move Through Village `1.0499999523162842`, distance `1`; summon-Vex spell |
| goal | `5` | Raider Celebration; fang-attack spell |
| goal | `6` | Wololo spell |
| goal | `8` | Random Stroll speed `0.6`, default interval `120` |
| goal | `9` | Look At Player, range `3`, probability `1` |
| goal | `10` | Look At Mob, range `8`, default probability `0.02` |
| target | `1` | Hurt By, ignoring Raider attackers, alert same-class Evokers |
| target | `2` | nearest Player, must see, unseen memory `300` |
| target | `3` | nearest Abstract Villager, need not see, unseen memory `300`; nearest Iron Golem, need not see |

The inherited Illager attack gate rejects baby Abstract Villagers even
though the target selector enumerates them. Target-goal random cadence,
reach, navigation and visibility retain `MOB-AI-001`.

Generic scoreboard alliance applies first. With no team on either side,
membership in `illager_friends` also makes another entity allied; that tag
contains `#illager`. Evoker additionally treats a Vex as allied when its
resolved root owner is this Evoker or is allied under the inherited rule.
This protects its own and allied owners' Vexes from owned fang damage.

The inherited raid goals own banner pickup, raid-center navigation, village
movement and celebration. Raider AI also attempts to join a nearby raid
every game-time multiple of `20` when `CanJoinRaid=true`, alive and not
already assigned. A Player or Iron-Golem target in an active raid resets
the inactivity counter. Evoker's `applyRaidBuffs` is deliberately empty.

### Shared spell state machine and arbitration

The summon and fang goals first require a present live combat target,
no active server casting timer and `tickCount>=nextAttackTickCount`.
Wololo instead requires no combat target, no casting timer and its own
cooldown. All three use-spell goals have no control flags, so the priority-1
casting goal can run concurrently and exclusively claims Move/Look.

On a successful start, a use goal:

1. stores `adjustedTickDelay(warmup)`; these goals do not require
   every-tick updates, so the nominal `20/40` warmups become `10/20`
   goal ticks under the alternate-phase selector;
2. sets `spellCastingTickCount` to its casting duration;
3. sets its next-use tick to current `tickCount + interval`;
4. plays its prepare sound at volume/pitch `1/1` when nonsilent; and
5. writes its spell ID to slot `17`.

The use-goal tick predecrements warmup. At exactly zero it performs the
spell, then plays Evoker Cast Spell at `1/1`. Its continuation requires a
live combat target and positive warmup; Wololo substitutes a nonnull Sheep
target. Thus the use goal ends immediately after the cast, while the
priority-1 casting goal continues for the remaining spell timer, stops
navigation, and looks first at a combat target or otherwise at the Wololo
Sheep with limits `75/40`.

Every effective server-AI tick decrements a positive spell timer. When it
expires and selector cleanup stops the casting goal, that goal writes spell
`0`. Cooldowns are measured from spell start, not completion. Registration
order tests summon at priority `4` before fangs at priority `5`; a successful
summon start makes the later fang admission see an active timer.

`NoAI` suppresses selector ticks and the custom timer decrement. Setting it
during a cast can therefore freeze both server casting state and the synced
spell byte until AI resumes or another write occurs.

### Summon-Vex spell

Summoning uses spell ID `1`, nominal warmup/casting/interval
`20/100/340`, and Evoker Prepare Summon.

After the shared admission gate, it counts every Vex—not merely owned
Vexes—returned from the Evoker AABB inflated by `16` and noncombat
targeting range `16`. This targeting ignores line of sight and invisibility
distance scaling but still requires the candidate to be visible to the
world. It then consumes `nextInt(8)` and admits only when

`nearbyVexCount < nextInt(8)+1`.

Counts `0..7` therefore admit with probability `(8-count)/8`; count `8` or
more cannot admit.

Casting performs exactly three independent attempts. Each consumes
`nextInt(5)` for X and Z and chooses
`blockPosition + (-2..2,1,-2..2)`. A nonnull Vex created with reason
`MOB_SUMMONED` is snapped to that block at yaw/pitch `0/0`, finalized at
local difficulty with null group data, assigned this Evoker as owner and
the chosen block as bound origin, and given limited life
`20*(30+nextInt(90))`, or `600..2380` ticks in steps of `20`.

If the Evoker has a scoreboard team, the Vex's scoreboard name is added to
that team before insertion. The server then offers the Vex with passengers
and emits `ENTITY_PLACE` at its block with the Evoker as context. Null
creation skips all later work for only that attempt. Vex equipment,
movement, attacks, limited-life decay, persistence and client rendering
remain the Vex subtype's separate owner.

### Fang-attack spell

The attack uses spell ID `2`, nominal warmup/casting/interval
`20/40/100`, and Evoker Prepare Attack. It does not recheck line of sight at
cast time. Let `theta=atan2(targetZ-evokerZ,targetX-evokerX)`,
`low=min(targetY,evokerY)` and
`high=max(targetY,evokerY)+1`.

At squared distance strictly below `9`, it requests:

- five fangs at radius `1.5`, angles `theta+i*pi*0.4`, delay `0`; then
- eight fangs at radius `2.5`, angles
  `theta+i*2*pi/8+1.2566371`, delay `3`.

At squared distance at least `9`, it instead requests sixteen fangs in a
line at distances `1.25*(i+1)`, angle `theta`, delay `i`.

Each request starts at `floor(x,high,z)` and scans downward until it finds a
block whose lower neighbor has a sturdy top face, including candidate Y
`floor(low)-1`. Failure suppresses only that fang. When the candidate block
is nonempty with a nonempty collision shape, the spawn Y adds that shape's
maximum Y; otherwise the offset is zero.

A successful request constructs protocol entity `47`
(`minecraft:evoker_fangs`) at that X/Y/Z, radians-to-degrees yaw, requested
warmup and this Evoker as owner. It ignores the insertion result and then
emits `ENTITY_PLACE` at the same point with the Evoker as context.

Evoker Fangs are MISC, no-loot, `0.5×0.8`, tracking range `6`, update
interval `2`, and reject all incoming server damage. Each server tick
predecrements warmup. On the first negative value it broadcasts event `4`;
at exactly `-8` it enumerates Living Entities in its AABB inflated
`(0.2,0,0.2)`. Alive, noninvulnerable, nonowner and nonallied candidates
receive ignored-result indirect-Magic damage `6`; successful server damage
also runs enchantment post-attack effects.

An ownerless loaded fang instead offers generic Magic `6`. Event `4` starts
the client attack, plays Fangs Attack unless that fang is silent, and the
server discards after its `22`-life countdown. Because constructed fangs do
not inherit Evoker silence, a silent Evoker suppresses prepare/cast sounds
but not the later fang sounds.

Fangs persist only `Warmup` and optional `Owner`. Life, sent-event and
client-start state reset on reload. A negative warmup reloaded below `-8`
therefore restarts event/life without revisiting the equality-only damage
tick.

### Wololo spell

Wololo uses spell ID `3`, nominal warmup/casting/interval `40/60/140`,
and Evoker Prepare Wololo. Admission requires `mobGriefing=true`, then
collects Sheep in the Evoker AABB inflated `(16,4,16)` through noncombat
targeting range `16`. The default targeting checks visibility-scaled range,
line of sight and world visibility, and the subtype selector accepts only
Blue Sheep. One list element is chosen with `nextInt(size)`.

Continuation checks only that the stored Sheep reference is nonnull and
warmup remains positive. At cast, a still-alive Sheep is set to Red; no
item, drop, game event or extra sound is produced. Its color is not
revalidated, and `mobGriefing` is not re-read after admission. A Sheep
recolored during warmup is still forced Red, and disabling the rule during
warmup does not cancel the cast. A dead target produces no recolor.
Stopping clears the transient Wololo reference.

### Client spell projection

Every client tick with slot `17>0` emits two local Entity-Effect particles,
protocol particle ID `28`, using the spell's RGB and zero velocity. With

`a = yBodyRot*pi/180 + cos(tickCount*0.6662)*0.25`,

the positions are opposite points
`(x +/- cos(a)*0.6*scale, y+1.8*scale,
z +/- sin(a)*0.6*scale)`. This consumes no RNG.

The arm pose is Spellcasting while the client byte is positive, otherwise
Celebrating while slot `16` is true, otherwise Crossed. Spellcasting places
the right/left arms at X `-5/+5`, Z `0`, rotates both X by
`cos(ageInTicks*0.6662)*0.25`, Z by
`+/-2.3561945`, and Y by `0`. The Evoker's item-in-hand render layer is
submitted only while casting.

`EvokerRenderer` uses `ModelLayers.EVOKER`, the shared Illager model, shadow
radius `0.5` and `textures/entity/illager/evoker.png`. The `64×64`,
`714`-byte texture has SHA-256
`34c0b8e60888982bbfb187ebd99e1b0ae70252235c6246ca767ff450c4211435`.

Fang event `4` starts progress
`1-((lifeTicks-2)-partialTick)/20`, capped at `1` when
`lifeTicks-2<=0`; progress is zero before the event. At client life `14`,
it emits twelve local Crit particles, protocol ID `13`, with exact
position/velocity RNG. Fangs Attack uses volume `1` and pitch
`0.85+nextFloat*0.2`.

The fang renderer skips exact zero progress, rotates Y by `90-yaw`, scales
`(-1,-1,1)` and translates `(0,-1.501,0)`. Its `64×32`, `391`-byte
texture has SHA-256
`ba286aa7aa1413368bc3ae2a9a9b2ba49c270d7c7ab6b51408a9ccfe21c7bbfc`.
The model eases `e=1-min(2*progress,1)^3`, rotates upper/lower jaw Z to
`pi-/+e*0.35*pi`, moves the base by
`-(progress+sin(progress*2.7))*7.2`, and during the last tenth scales the
root by `(1-progress)/0.1` while moving root Y from `4` back toward `24`.

### Production, placement and persistence

Locked raid groups select fixed Evoker counts `0,0,0,0,1,1,2` for ordinary
waves `1..7`; Evoker has no random extra count. Omen bonus groups reuse the
difficulty's final ordinary-wave index. At wave seven and later the first
Ravager rider is an Evoker and remaining riders are Vindicators. Raid
creation uses reason `EVENT`, assigns wave/raid state, may choose an Evoker
as leader when earlier eligible creations failed, applies the empty buff
hook and offers it with passengers. Full raid ordering remains
`MOB-RAID-001`.

Woodland Mansion templates contain four `Mage` DATA markers, all in
`2x2_a1`. Each in-box marker creates one Evoker with reason `STRUCTURE`;
a nonnull result becomes persistence-required, is snapped to the marker at
yaw/pitch `0/0`, finalized at local difficulty with null group data and
offered with passengers. The marker is cleared to Air even if insertion
fails. All `1,212` locked templates contain zero exact
`minecraft:evoker`/legacy Evoker entity identity; the four spawns are
marker-driven.

The separate placement registration is `NO_RESTRICTIONS` with heightmap
`MOTION_BLOCKING_NO_LEAVES` and the standard Monster predicate. It checks
darkness unless the reason ignores light, then generic Mob placement;
`NO_RESTRICTIONS` itself adds no support/medium test. All `66` locked
baseline biomes contain zero Evoker spawn rows, so the predicate does not
create baseline natural selection.

Patrolling-Monster finalization for reasons other than `PATROL`, `EVENT` or
`STRUCTURE` consumes one level `nextFloat`; below `0.06`, an eligible
Evoker becomes patrol leader, equips the ominous banner in its head slot and
sets drop chance `2`. Reason `PATROL` instead marks it patrolling, while
Raider finalization sets `CanJoinRaid=true` for every Evoker reason.
Baseline patrol production selects no Evoker, but custom/spawn-finalization
paths retain these inherited branches.

### Loot, tags, advancements and item identity

The entity loot table uses random sequence `minecraft:entities/evoker` and
evaluates two ordered one-roll pools:

1. exactly one Totem of Undying, item protocol ID `1333`, unconditionally;
2. on a player kill, Emerald item ID `927` receives integer-uniform count
   `0..1` and uniform `0..1` Looting enchanted-count increase.

The fixed XP reward is `10`. Positive-count filtering, Looting arithmetic,
mob-loot gamerule, equipment/banner drops, item merging and death ordering
retain their cited owners.

Exactly two direct entity-type tags name Evoker:

- `illager`, which also reaches `illager_friends` through nested
  membership; and
- `raiders`.

Their consumers own teamless Illager alliance, raid membership, Bell
highlighting and other tag-selected behavior. No other locked entity-type
tag directly names the identity.

Both hostile-mob advancements have an exact
`player_killed_entity` criterion for Evoker. `kill_a_mob` places it in one
OR requirement group with every listed hostile; `kill_all_mobs` places it
in its own required group and awards `100` experience only after all such
groups complete.

The Spawn Egg is raw/protocol item ID `1228`, common, maximum stack `64`,
and its `entity_data.id` is `minecraft:evoker`. Its generated model directly
selects the Egg texture. English labels are “Evoker”, “Evoker Fangs” and
“Evoker Spawn Egg”. The Egg's `16×16`, `257`-byte texture has SHA-256
`af6b7264dea1fd6f3d7d8ecabfdf53ef80e148071dabcc1d20e724450315eccc`.

### Sounds

The locked sound-event joins are:

| Protocol ID | Event | English subtitle |
|---:|---|---|
| `608` | Evoker Ambient | “Evoker murmurs” |
| `609` | Evoker Cast Spell | “Evoker casts spell” |
| `610` | Evoker Celebrate | “Evoker cheers” |
| `611` | Evoker Death | “Evoker dies” |
| `612` | Evoker Fangs Attack | “Fangs snap” |
| `613` | Evoker Hurt | “Evoker hurts” |
| `614` | Evoker Prepare Attack | “Evoker prepares attack” |
| `615` | Evoker Prepare Summon | “Evoker prepares summoning” |
| `616` | Evoker Prepare Wololo | “Evoker prepares charming” |

Ambient cadence and generic sound broadcast retain their owners. Parrot
imitation maps Evoker to sound-event ID `1224`,
`entity.parrot.imitate.evoker`, subtitle “Parrot murmurs”; the Parrot's
attempt cadence, nearby selection, silence gate and playback retain the
Parrot owner.

### Migration and schema closure

Seven exact migration/schema classes name the Evoker family:

- `ItemStackSpawnEggFix` maps legacy
  `minecraft:evocation_illager` to
  `minecraft:evocation_illager_spawn_egg`;
- `EntityTheRenameningFix` maps
  `minecraft:evocation_illager`, `minecraft:evocation_fangs` and
  `minecraft:evocation_illager_spawn_egg` to the three current Evoker
  identities;
- schema `V1510` moves the legacy Evoker and fang entity shapes to current
  names;
- schema `V705` maps the current Spawn Egg to the current entity;
- `EntityUUIDFix` includes current Evoker in the Mob UUID rewrite set and
  rewrites fang `OwnerUUID` least/most into `Owner`;
- `BlockPosFormatAndRenamesFix` includes Evoker among six patrolling mobs
  whose `PatrolTarget` becomes codec-shaped `patrol_target`; and
- `StatsCounterFix` maps legacy `EvocationIllager` statistics to
  `minecraft:evocation_illager`, before the later rename.

Generic entity, equipment, effect, raid and patrol fields retain their
schema owners.

**Branches and aborts:**

- Shared spell admission aborts on missing/dead target, active timer or
  per-goal cooldown; summon then applies its Vex-count draw, while the fang
  goal has no further admission gate.
- Summon skips only a null Vex attempt; team assignment, insertion offer and
  game event follow every nonnull creation.
- Fang pattern branches at strict squared distance `9`; each point
  independently aborts when the downward sturdy-face scan fails.
- Fang damage rejects dead, invulnerable, owner and allied candidates;
  rejected insertion/game damage is not rolled back.
- Wololo aborts on a combat target, active cast/cooldown, disabled
  `mobGriefing` or no visible Blue Sheep. Post-start continuation does not
  recheck color, rule, visibility or liveness.
- Placement has no baseline biome selector despite its registered
  predicate; Mansion and raid producers bypass that absence.
- Silence suppresses Evoker prepare/cast and ordinary entity sounds but is
  not inherited by created Vexes or fangs.

**Invariants:**

- Slot `17` is the client spell identity; `SpellTicks` is the independent
  server cast lock.
- Summon arbitration precedes fang arbitration on an otherwise shared
  target/timer boundary.
- Spell cooldowns are anchored at start and spell effects occur when the
  reduced warmup reaches zero.
- Every successful summon cast attempts three Vex creations.
- Close fangs are five radius-`1.5` plus eight delayed radius-`2.5`; far
  fangs are sixteen progressively delayed line points.
- Fangs never damage their owner or an entity allied to that owner.
- Wololo changes only a still-alive selected Sheep to Red.
- Baseline Evokers come from raids and four Mansion markers, not biome
  spawn rows.
- The Totem pool is independent of player kill; only the Emerald pool is
  gated.

**Constants and randomness:**

Entity/Egg/fang IDs `46/1228/47`; dimensions/eye/passenger/riding
`0.6×1.95/1.6575/2/-0.6`; range/update `8/3`;
health/speed/follow/attack `24/0.5/12/2`; XP `10`;
slots `16 BOOLEAN/17 BYTE`; avoid `8/0.6/1`; wander `0.6/120`;
spell IDs `1/2/3`; summon warmup/cast/interval `20/100/340`, Vex gate
`16/nextInt(8)+1`, attempts/offset/life `3/-2..2/600..2380`;
fang warmup/cast/interval `20/40/100`, distance squared `9`, rings
`5@1.5+8@2.5`, outer delay `3`, line `16@1.25`, damage/warmup/life
`6/-8/22`; Wololo `40/60/140`, AABB `16/4/16`;
raid fixed `0/0/0/0/1/1/2`; patrol-leader chance `0.06`;
Monster cap/distances/cluster `70/32/128/4`; tags/templates/markers
`2/0 of 1212/4`; shadow `0.5`.

**Side effects:**

Metadata, timers, patrol/raid/persistence and equipment state; RNG cursors;
targets, goal arbitration, navigation and look; Vex owner/bound/life/team
state and entity insertion; fang entities, magic damage and enchantment
post-attack effects; Sheep color; entity/game events, particles and sounds;
loot/XP, advancement progress and item stacks; renderer/model state.

**Gates:**

Logical side, Peaceful, NoAI and persistence; goal priority/flags and
cooldowns; target class/liveness/sight and baby-Villager exclusion; Raider,
Illager/team and Vex-owner alliance; nearby-Vex count and RNG; ground
sturdiness/collision shape; fang owner/alliance/invulnerability;
`mobGriefing`, Sheep color/visibility/range/liveness; raid wave/leader and
Mansion marker box/create state; placement reason/light/Mob predicate and
absent biome rows; player kill, Looting and mob loot; silence, metadata,
camera and resources; migrations.

**Boundary cases and quirks:**

The Creaking avoidance goal is independently registered at priority `3`.
Summon counts unrelated Vexes and becomes impossible at eight. A retained
target can be attacked through lost sight because fang casting has no
line-of-sight recheck. Close range is strictly below squared distance `9`.
Individual fang points can disappear over unsupported vertical gaps.
Silent Evokers still create audible fangs. Wololo commits after admission
even if the gamerule turns off or the Sheep stops being Blue. Persisted
positive `SpellTicks` reloads without its spell byte, creating a server-only
cast lock with no client spell pose/particles until the timer expires.
Reloaded fangs can restart their event/lifetime while missing an already
passed equality-only damage tick. Custom finalization can produce a
six-percent Evoker patrol leader even though baseline patrol selection does
not produce Evokers.

**Failure semantics:**

Interrupted use goals produce no spell effect, while their cast timer and
start-anchored cooldown continue until their separate owners clear or
expire them. Null Vex creation skips one of three attempts. Entity insertion
results do not roll back team, owner, game-event or fang construction state.
Unsupported fang points have no entity or game event. Failed fang damage is
ignored. A dead Wololo target yields no recolor. Failed raid/Mansion
insertion follows those owners' no-rollback behavior.

**Client/server authority split:**

The server owns targets, goals, timers, spell selection, Vex/fang creation,
damage, Sheep color, patrol/raid/Mansion state, loot/XP and advancements.
Clients consume slots `16/17`, entity event `4`, movement and resources;
they emit spell/fang particles, select crossed/spell/celebrate arms, play
local fang sound and render the Evoker, held items and fangs. Client
particles, animation or stale metadata cannot commit server effects.

**Observability:**

Observe registration/attributes and metadata slots; save/reload divergence;
the full inherited/local goal graph; same-tick summon-versus-fang
arbitration; all warmup/timer/cooldown edges; Vex count/RNG and three
initialization paths; both fang geometries, ground scan, event/damage/life;
Wololo gamerule/visibility/color changes; raid waves/riders and four
Mansion markers versus zero biome rows; leader/persistence branches;
loot/tags/advancements/Egg; migrations, sounds, particles, arm/model,
textures and hashes.

**Persistence and reload:**

Generic entity/Mob state, `SpellTicks`, patrol fields, wave/join/raid state,
equipment and generic persistence save. Spell ID/metadata, celebration,
warmups/cooldowns, Wololo/attack targets, active goals and client state do
not. A reloaded positive timer remains server-active with current spell
`NONE` and slot `17=0`. Vex and fang persistence remain their own entity
owners. Loot, tags, advancements and biome data reload through their
owners; raid/Mansion and spell code remain fixed. Language, models and
textures reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.PatrollingMonster`;
`net.minecraft.world.entity.raid.Raider`;
`net.minecraft.world.entity.monster.illager.AbstractIllager`;
`net.minecraft.world.entity.monster.illager.SpellcasterIllager`;
`net.minecraft.world.entity.monster.illager.SpellcasterIllager$SpellcasterCastingSpellGoal`;
`net.minecraft.world.entity.monster.illager.SpellcasterIllager$SpellcasterUseSpellGoal`;
`net.minecraft.world.entity.monster.illager.SpellcasterIllager$IllagerSpell`;
`net.minecraft.world.entity.monster.illager.Evoker`;
`net.minecraft.world.entity.monster.illager.Evoker$EvokerCastingSpellGoal`;
`net.minecraft.world.entity.monster.illager.Evoker$EvokerSummonSpellGoal`;
`net.minecraft.world.entity.monster.illager.Evoker$EvokerAttackSpellGoal`;
`net.minecraft.world.entity.monster.illager.Evoker$EvokerWololoSpellGoal`;
`net.minecraft.world.entity.projectile.EvokerFangs`;
`net.minecraft.world.entity.monster.Vex`;
`net.minecraft.world.entity.animal.sheep.Sheep`;
`net.minecraft.world.entity.ai.goal.Goal`;
`net.minecraft.world.entity.ai.goal.GoalSelector`;
`net.minecraft.world.entity.ai.targeting.TargetingConditions`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.fixes.EntityTheRenameningFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.BlockPosFormatAndRenamesFix`;
`net.minecraft.util.datafix.fixes.StatsCounterFix`;
`net.minecraft.util.datafix.schemas.V705`; `V1510`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.EvokerRenderer`;
`net.minecraft.client.renderer.entity.EvokerFangsRenderer`;
`net.minecraft.client.renderer.entity.state.EvokerRenderState`;
`net.minecraft.client.model.monster.illager.IllagerModel`;
`net.minecraft.client.model.effects.EvokerFangsModel`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,particle_type}`;
`reports/minecraft/components/item/evoker_spawn_egg.json`;
`data/minecraft/tags/entity_type/{illager,illager_friends,raiders}.json`;
`data/minecraft/loot_table/entities/evoker.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/evoker_spawn_egg.*`;
`assets/minecraft/textures/entity/illager/{evoker,evoker_fangs}.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-PROJECTILE-001`; `ENT-DAMAGE-001`; `ENT-DEATH-001`;
`MOB-AI-001`; `MOB-SPAWN-001`; `MOB-DESPAWN-001`; `MOB-RAID-001`;
`ITM-EMERALD-001`; `WGEN-STRUCTURE-WOODLAND-MANSION-001`; `CLI-006`.

**Test vectors:**

Run `EXP-ENT-023` across construction/metadata/NoAI/save/reload,
patrol/raid/persistence and the complete goal graph; all target, spell
arbitration, warmup/timer/cooldown and silence boundaries; Vex
count/RNG/create/team/life/insertion paths; close/far fang geometry,
support/collision/event/damage/reload outcomes; Wololo
gamerule/list/color/liveness changes; raid waves/riders and all Mansion
markers, placement/66-biome/cap/despawn cases; loot/XP/tags/advancements/
Egg, templates/seven migrations/sounds/Parrot and exact
particle/model/texture/fang projection.

**Limits:**

Generic lifecycle, metadata, equipment, pathfinding, target/avoidance
algorithms, damage/death, patrol/raid orchestration, natural spawning and
despawn, structure processing, loot, advancements, Spawn Egg interaction,
Sheep color storage, Vex runtime, fang projectile runtime, particles and
rendering retain their cited owners. Shared Raider/Illager/Spellcaster
algorithms are included only where the Evoker subtype registers, selects or
changes their exact inputs and observable result.
