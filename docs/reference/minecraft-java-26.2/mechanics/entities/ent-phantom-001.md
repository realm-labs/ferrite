# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-PHANTOM-001` - Phantom size drives flight, swoop combat and client projection

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-005`, `ENT-DAMAGE-001`, `ENT-BLOCK-001`,
`ENT-DAMAGE-REDUCE-001`, `ENT-KNOCKBACK-001`, `ENT-006`,
`ENT-EFFECT-001`, `ENT-007`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-PHANTOM-SPAWN-001`, `MOB-003`,
`MOB-DESPAWN-001`, `ITM-PHANTOM-MEMBRANE-001`,
`ITM-ADVANCEMENT-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-DIMENSION-001`, `WGEN-PORTAL-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` - locked registration, the complete `Phantom` and
`PhantomSpawner` classes, generic daylight and tag consumers, the entity
loot and advancement data, six sound joins and the complete client renderer
close protocol entity ID `99`.

**Applies when:**

`minecraft:phantom` is constructed, finalized, size-changed, loaded, spawned
by the insomnia spawner or another production path, selecting a player,
circling, swooping, interrupted by a Cat, colliding, attacking, burning in
daylight, damaged, killed, synchronized, heard, imitated by a Parrot or
rendered.

**Authoritative state:**

Protocol entity ID `99` constructs `Phantom` in `MONSTER`, and registration
marks it unavailable in Peaceful. Registration fixes base width/height
`0.9x0.5`, eye height `0.175`, passenger attachment `0.3375`, riding offset
`-0.125`, client tracking range `8` and the default update interval `3`.
`shouldRenderAtSqrDistance` always returns true.

Phantom uses the generic Monster attribute supplier. It adds synchronized
metadata slot `16`, serializer `INT`, default `0`, as its size. The inherited
Entity, Living-Entity and Mob set remains slots `0..15`.
`setPhantomSize` clamps every input to `0..64`. A changed slot refreshes
dimensions and sets the attack-damage base to `6 + size`. Dimensions are the
registered dimensions uniformly scaled by `1 + 0.15*size`.

The equality guard in `SynchedEntityData#set` is observable here. A fresh
Phantom defines slot `16` as zero, and finalization calls
`setPhantomSize(0)`. Because that write is equal to the current value, it
does not call `onSyncedDataUpdated`: a never-changed default-size Phantom
retains the Monster attack-damage default `2`. Any actual size transition
sets attack damage to `6 + size`; changing a nonzero size back to zero
therefore leaves attack damage `6`, not `2`. Loading an explicitly saved
zero into a fresh zero-valued instance has the same equality behavior.

The server-only runtime fields are `moveTargetPoint`, nullable `anchorPoint`
and attack phase `CIRCLE` or `SWOOP`. Finalization sets the anchor five
blocks above the current block position and requests size zero before
delegating to generic Mob finalization. XP reward is `5`.

Persistence writes nullable `anchor_pos` through the BlockPos codec and
integer `size`. It does not save the move target, phase, goal timers,
movement-controller speed, Cat search state or circle parameters.

**Transition and ordering:**

### Goal graph and player selection

Phantom installs exactly four goals:

| Selector | Priority | Goal |
|---|---:|---|
| goal | `1` | attack strategy |
| goal | `2` | sweep attack |
| goal | `3` | circle around anchor |
| target | `1` | attack player |

The target goal starts with `reducedTickDelay(20)`, then scans every
`reducedTickDelay(60)` while idle. It queries combat-eligible players in
the Phantom bounding box inflated by `16` horizontally and `64` vertically,
sorts them by descending Y, and selects the first candidate that also
passes the Phantom's default attackability test. Continued targeting
requires that same default test.

The strategy goal starts only with a nonnull attackable target. Start sets
`nextSweepTick = adjustedTickDelay(10)`, phase `CIRCLE`, and the anchor to
the target block position plus `20 + nextInt(20)` Y, raised to sea level
plus one when necessary. While circling it decrements the timer. At zero it:

1. changes phase to `SWOOP`;
2. recomputes the target-relative anchor;
3. stores `adjustedTickDelay((8 + nextInt(4))*20)` for the next cycle; and
4. plays Phantom Swoop at volume `10` and pitch `0.95 + nextFloat*0.1`.

The strategy timer pauses during `SWOOP` and resumes after phase returns to
`CIRCLE`. When the strategy stops, a nonnull anchor is replaced by the
`MOTION_BLOCKING` heightmap position at its X/Z plus `10 + nextInt(20)` Y.

### Circling

The circle goal is usable with no target or while phase is `CIRCLE`. Start
draws radius `5 + nextFloat*10`, height field
`-4 + nextFloat*9`, direction `+1` or `-1`, then selects a point.
Selection advances angle by `direction*15` degrees and sets:

```text
anchor lower corner +
(radius*cos(angle), -4 + height, radius*sin(angle))
```

The initial selected vertical offset is therefore `[-8,1)`, not merely the
stored height field `[-4,5)`.

Each goal tick independently:

- redraws height with probability `1/adjustedTickDelay(350)`;
- increments radius by one with probability
  `1/adjustedTickDelay(250)`, wrapping values above `15` to `5` and
  reversing direction;
- redraws angle uniformly over `[0,2pi)` and immediately reselects with
  probability `1/adjustedTickDelay(450)`;
- reselects when squared distance to the current point is below `4`;
- clamps the stored height upward to at least `1` and reselects when the
  target is below the Phantom but the block below is nonempty; and
- clamps it downward to at most `-1` and reselects when the target is above
  the Phantom but the block above is nonempty.

### Swoop, Cats and attack

The sweep goal requires a target and phase `SWOOP`. It continues only while
the target is alive, is not a spectator or Creative player, the phase still
matches, and no nearby Cat has interrupted it.

When `tickCount` passes the Cat-search deadline, the goal schedules the next
search at `tickCount + 20`, queries every alive Cat in the Phantom box
inflated by `16`, and makes every result hiss. A nonempty result marks the
Phantom scared and ends the sweep. Stopping the goal clears the target and
returns phase to `CIRCLE`.

On every active sweep tick, the move target is the target's X/Z and
`getY(0.5)`. If the Phantom box inflated by
`0.20000000298023224` intersects the target box, it performs the generic Mob
melee attack, returns to `CIRCLE`, and, unless silent, emits level event
`1039` at the Phantom block position. That event plays Phantom Bite in the
hostile source at volume `0.3` and pitch `0.9 + nextFloat*0.1`. A horizontal
collision or positive `hurtTime` also returns to `CIRCLE` without clearing
the target or performing the attack.

### Flight controller and rotation

The move controller begins with speed `0.1`. Horizontal collision rotates
yaw by `180` degrees and resets speed to `0.1`. Otherwise it derives the
target vector, attenuates the horizontal components by
`1 - abs(dy*0.7)/horizontalDistance`, approaches the target yaw at at most
`4` degrees per tick, copies yaw into body yaw, and:

- approaches speed `1.8` by `0.005*(1.8/speed)` when the absolute yaw
  change is below `3` degrees; or
- approaches speed `0.2` by `0.025` otherwise.

Pitch is `-atan2(dy,horizontalDistance)` in degrees. The desired velocity
uses current speed and the normalized target components; delta movement
approaches it by adding `0.2*(desired-current)` each tick.

The look controller deliberately does nothing. The Phantom body controller
copies body yaw to head yaw, then entity yaw to body yaw. Travel uses the
shared flying path with friction/input factor `0.2`. Phantom never reports
climbable and its fall-damage hook is empty.

### Tags, spawning, loot and progression

Phantom registers `NO_RESTRICTIONS`,
`MOTION_BLOCKING_NO_LEAVES`, and `Mob.checkMobSpawnRules`; baseline biome
lists contain no Phantom row. Ordinary insomnia production is instead the
already closed `MOB-PHANTOM-SPAWN-001` transaction: it owns the pausable
timer, gamerule, sky/difficulty/rest gates, candidate position, group count,
construction and finalization.

Exactly three direct entity-type tags name Phantom:

- `burn_in_daylight` admits the generic Mob daylight check and eight-second
  ignition path;
- `fall_damage_immune` suppresses tag-selected fall damage; and
- `undead` selects the shared undead effect, damage and enchantment
  behavior.

The entity loot table has one player-kill-gated pool for Phantom Membrane,
item protocol ID `889`: integer-uniform `0..1`, plus the shared
Looting enchanted-count increase using uniform `0..1`. Its exact item,
repair, brewing and cat-gift behavior remains with
`ITM-PHANTOM-MEMBRANE-001`.

Both hostile-mob advancements contain Phantom criteria. In addition,
`adventure/two_birds_one_arrow` requires one Crossbow-fired
`killed_by_arrow` trigger whose victim list contains two separate Phantom
predicates; success grants `65` experience.

The Spawn Egg is item protocol ID `1223`, common, maximum stack `64`, with
`entity_data.id = minecraft:phantom`.

### Sounds and client projection

The locked sound joins are:

| Protocol ID | Event | English subtitle |
|---:|---|---|
| `1255` | Phantom Ambient | "Phantom screeches" |
| `1256` | Phantom Bite | "Phantom bites" |
| `1257` | Phantom Death | "Phantom dies" |
| `1258` | Phantom Flap | "Phantom flaps" |
| `1259` | Phantom Hurt | "Phantom hurts" |
| `1260` | Phantom Swoop | "Phantom swoops" |

Parrot imitation maps Phantom to sound-event ID `1231`,
`entity.parrot.imitate.phantom`, subtitle "Parrot screeches".

`TICKS_PER_FLAP` is `ceil(24.166098) = 25`; `isFlapping` is true when
`(entityId*3 + tickCount) mod 25 == 0`. On every client tick, the Phantom
computes adjacent flap cosines from `(entityId*3 + tick)*7.448451` degrees.
When the cosine crosses from positive to nonpositive it plays local Phantom
Flap with independently drawn volume and pitch in `[0.95,1.0)`.

The client also emits two Mycelium particles every tick. Their horizontal
offset is the signed pair derived from yaw and `width*1.48`; their Y is
`y + (0.3 + cosine*0.45)*height*2.5`; velocity is zero.

`PhantomRenderer` uses shadow radius `0.75`, applies the same
`1 + 0.15*size` uniform scale, translates by `(0,1.3125,0.1875)`, rotates
by entity pitch, and adds an emissive eye layer. Wing bases and tips mirror
`cos(flapTime*7.448451 degrees)*16` degrees; both tail parts use
`-(5 + cos(2*phase)*5)` degrees.

The base and eye textures are both `64x64`. The base texture is `639` bytes,
SHA-256
`97fe36ce3dcf0ec7a1a32c59379e51a959e3963e85dd43b74751d22a2fa607fa`;
the eye texture is `112` bytes, SHA-256
`7822a936cb20c677fb85789c7fbef2f15d54b41800f90723d2d00bc4b49a2102`.
English labels are "Phantom" and "Phantom Spawn Egg".

**Branches and aborts:**

- Registration rejects Phantom in Peaceful before subtype behavior.
- An equal size write changes neither dimensions nor attack damage.
- Player scans skip every candidate that fails combat or default
  attackability, after sorting the candidates by descending Y.
- Strategy start and continuation require a live attackable target.
- A Cat search hisses every nearby alive Cat before ending the sweep.
- Sweep collision with the target performs one attack; block collision or
  hurt interruption only changes phase.
- Bite level event is suppressed by silence; other attack state changes
  still occur.
- The custom insomnia spawner, generic placement and command/Egg paths keep
  their separate admission owners.
- Phantom loot is absent without player-kill attribution.

**Invariants:**

- Slot `16` is the sole subtype metadata authority and is clamped `0..64`.
- A changed size controls both authoritative dimensions and attack damage.
- Circle and sweep move goals are mutually exclusive by phase.
- Sweep interruption never persists across save/load.
- Cats interrupt only after the periodic alive-Cat query.
- Phantom takes no fall damage and never climbs.
- All baseline biome spawn lists omit Phantom.
- The renderer and server dimensions use the same linear size scale.

**Constants and randomness:**

Entity/Egg IDs `99/1223`; dimensions/eye/passenger/riding
`0.9x0.5/0.175/0.3375/-0.125`; range/update `8/3`; XP `5`; size
`0..64`, scale `1+0.15*size`, changed-size attack `6+size`, untouched
default attack `2`; phase priorities `1/2/3`, target priority `1`;
scan delays `20/60`, scan box `16/64/16`; initial sweep `10`,
cycle `(8+nextInt(4))*20`, anchor `20+nextInt(20)`, stop anchor
`10+nextInt(20)`; circle radius `5+U*10`, height `-4+U*9`, angle step
`15` degrees, reselection squared distance `4`, random delays
`350/250/450`; Cat search `20`, box inflation `16`; attack inflation
`0.20000000298023224`; move speeds `0.1/0.2/1.8`, yaw limits `4/3`,
velocity approach `0.2`; flight factor `0.2`; flap `7.448451` degrees and
period `25`; daylight ignition `8` seconds; Bite `0.3` and
`0.9+U*0.1`; Swoop `10` and `0.95+U*0.1`; Flap volume/pitch
`0.95+U*0.05`; particle width factor `1.48`; two `64x64` textures.

**Side effects:**

Slot `16`, dimensions and attack attribute; target and phase; anchor and
move target; goal timers and circle/Cat/controller state; RNG cursors;
movement, yaw and pitch; melee damage and level event `1039`; Cat hisses;
daylight fire; loot, XP and advancement progress; sounds, particles and
renderer state.

**Gates:**

Logical side, Peaceful, NoAI and persistence; metadata equality and clamp;
goal priority and phase; player combat/default targeting; spectator and
Creative state; Cat presence and search deadline; collision, hurt and
silence; generic daylight environment/light/sky/equipment checks; spawn
gamerules and insomnia transaction; player kill and Looting; resources.

**Boundary cases and quirks:**

The equal-zero size write means two size-zero Phantoms can have different
attack bases: an untouched one retains `2`, while one changed away from and
back to zero has `6`. The serialized value alone therefore cannot recover
that distinction after a fresh zero-valued load.

The strategy writes its long cooldown when entering `SWOOP` but decrements
it only while phase is `CIRCLE`. A collision or damage interruption returns
to circling with that full stored cooldown. A Cat ends the sweep more
strongly by clearing the target.

Circle selection adds `-4` to a height field that was itself initialized
from `-4 + U*9`. The first actual Y offset is consequently `[-8,1)`.
`shouldRenderAtSqrDistance` bypasses ordinary distance culling, while the
network tracking range remains `8`.

**Failure semantics:**

If generic melee admission rejects the intersecting attack, the Phantom
still returns to `CIRCLE` and emits Bite unless silent. A null or invalid
insomnia-spawn construction retains the spawner owner's nontransactional
behavior. Failed insertion, loot emission or advancement grant does not
roll back death or prior AI state.

**Client/server authority split:**

The server owns size, attributes, dimensions, targets, phases, movement,
collision, damage, fire, loot and advancements. Clients consume metadata
and movement, recompute dimensions on slot `16`, and own flap sound,
Mycelium particles, model animation, eye layer and texture projection.

**Observability:**

Observe registration and slot `16`; equal versus changed size transitions
at every clamp boundary; save/reload of `anchor_pos` and `size`; all target,
circle, sweep, Cat, collision and controller branches with controlled RNG;
daylight and all three direct tags; the linked insomnia transaction; loot,
three advancements and Egg; six sounds, Parrot imitation, flap/particle
cadence, two texture hashes and size-scaled rendering.

**Persistence and reload:**

Only `anchor_pos` and `size` supplement generic Mob data. Reload reconstructs
goals and controllers, resets phase to `CIRCLE`, move target to zero,
controller speed to `0.1`, and every strategy, circle and Cat timer/field.
The loaded size is passed through the clamp and equality guard. Loot, tags
and advancements reload through their owners; client resources reload
client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.network.syncher.SynchedEntityData`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.Phantom`;
`net.minecraft.world.entity.monster.Phantom$PhantomAttackPlayerTargetGoal`;
`net.minecraft.world.entity.monster.Phantom$PhantomAttackStrategyGoal`;
`net.minecraft.world.entity.monster.Phantom$PhantomCircleAroundAnchorGoal`;
`net.minecraft.world.entity.monster.Phantom$PhantomSweepAttackGoal`;
`net.minecraft.world.entity.monster.Phantom$PhantomMoveControl`;
`net.minecraft.world.entity.monster.Phantom$PhantomLookControl`;
`net.minecraft.world.entity.monster.Phantom$PhantomBodyRotationControl`;
`net.minecraft.world.level.levelgen.PhantomSpawner`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.animal.feline.Cat`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.data.tags.EntityTypeTagsProvider`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.advancements.packs.VanillaAdventureAdvancements`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.PhantomRenderer`;
`net.minecraft.client.renderer.entity.layers.PhantomEyesLayer`;
`net.minecraft.client.model.monster.phantom.PhantomModel`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,loot_table,
advancement,particle_type}`;
`reports/minecraft/components/item/phantom_spawn_egg.json`;
`data/minecraft/tags/entity_type/{burn_in_daylight,fall_damage_immune,
undead}.json`;
`data/minecraft/loot_table/entities/phantom.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs,
two_birds_one_arrow}.json`;
`assets/minecraft/textures/entity/phantom/{phantom,phantom_eyes}.png`;
`assets/minecraft/lang/en_us.json`;
`MOB-PHANTOM-SPAWN-001`; `ITM-PHANTOM-MEMBRANE-001`;
`ENT-DAMAGE-001`; `ENT-EFFECT-001`; `ENT-DEATH-001`; `CLI-006`.

**Test vectors:**

Run `EXP-ENT-030` across construction/metadata/NoAI/save/reload, equal and
changed size values below, inside and above `0..64`, the default attack-base
quirk, target scans and Y ordering, every circle draw and obstruction,
circle-to-sweep timing, Cat search/hiss/clear, intersection/collision/hurt
branches, move-control convergence, daylight and direct tags, linked
insomnia production, loot and three advancements, Egg, six sounds, Parrot
imitation, flap/particle cadence, two textures and render scaling.

**Limits:**

Generic lifecycle, metadata transport, movement collision, melee damage,
daylight ignition, effect/tag consumers, death, loot, advancements, Spawn
Egg interaction and client entity transport retain their cited owners.
`MOB-PHANTOM-SPAWN-001` remains the sole owner of the insomnia spawn
transaction and `ITM-PHANTOM-MEMBRANE-001` remains the sole owner of the
Membrane item after emission.
