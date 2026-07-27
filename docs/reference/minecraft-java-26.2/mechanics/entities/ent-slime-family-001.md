# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-SLIME-FAMILY-001` — Cube mobs scale every attribute from one synchronized size and split into two to four children on death

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-005`, `ENT-DAMAGE-001`, `ENT-BLOCK-001`,
`ENT-DAMAGE-REDUCE-001`, `ENT-KNOCKBACK-001`, `ENT-006`,
`ENT-EFFECT-001`, `ENT-007`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`MOB-001`, `MOB-AI-001`, `MOB-002`, `MOB-SPAWN-001`, `MOB-003`,
`MOB-DESPAWN-001`, `ITM-SLIME-BALL-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration for both identities, the complete
`AbstractCubeMob` class with its five nested goal and control classes, the
`Slime` and `MagmaCube` subtypes, both spawn-rule predicates including the
slime-chunk seed, all 66 biomes, both loot tables with their Frog branches,
the direct entity tags, both Spawn Eggs, both hostile-mob advancements,
migration/schema contexts and exact client resources close protocol entity
IDs `117` and `80`.

**Applies when:**

`minecraft:slime` or `minecraft:magma_cube` is constructed, finalized,
spawned naturally in a slime chunk, a swamp band or the Nether, spawned by a
spawner, Egg, command or custom selector, jumping, squishing, changing size,
targeting, touching or pushing an entity, damaged, killed and split, eaten by
a Frog, synchronized, saved, loaded, heard or rendered.

**Authoritative state:**

Both identities register in `MONSTER` and are Peaceful-excluded. Both use
base dimensions `0.52×0.52`, eye height `0.325` and spawn-dimensions scale
`4`. Slime is protocol entity ID `117` with client tracking range `10`;
Magma Cube is protocol entity ID `80` with tracking range `8` and is
registered fire-immune. Both use the default update interval `3`.

Inherited Entity, Living-Entity and Mob state occupies synchronized metadata
slots `0..15`. `AgeableMob` adds slot `16` (`BOOLEAN`, baby) and slot `17`
(`BOOLEAN`, age locked). `AbstractCubeMob` adds slot `18`, serializer
ID `1` (`INT`), default `1`, holding the size. Both subtypes return `false`
from `canBeABaby`, so the inherited age machinery never produces a baby and
both slots `16`/`17` stay at their defaults through ordinary play.

`Size` persists as `getSize()-1` and is read back as `stored+1`, so a
missing or zero key restores size `1`. `wasOnGround` persists as a boolean,
default `false`. The squish, target-squish and previous-squish floats, the
move control's chosen yaw, jump delay and aggression flag, and the goals'
timers are transient.

Every size-derived value flows through one setter. `setSize(size, resetHealth)`
clamps the argument to `1..127`, writes slot `18`, reapplies the position,
refreshes dimensions, sets maximum health to `size*size`, sets movement
speed to `0.2 + 0.1*size`, and — only when `resetHealth` is true — sets
current health to the new maximum. Subtypes extend it:

- Slime additionally sets attack damage to `size` and XP reward to `size`;
- Magma Cube additionally sets attack damage to `size`, armor to `3*size`
  and XP reward to `size`.

Default dimensions are the registered dimensions scaled by the size, so a
size-`n` cube is `0.52n` wide and tall. `refreshDimensions` re-applies the
pre-change position afterwards, so growth and shrink keep the same feet
position rather than the same centre. The passenger attachment point is
`(0, height - 0.015625*size*partialTick, 0)`.

Magma Cube attributes additionally register movement speed
`0.20000000298023224`, but `setSize` overwrites that base value on every
size assignment, so the registered value is only observable before the first
`setSize` call. Slime registers no extra attribute at all.

`AbstractCubeMob` fixes sound source `HOSTILE`, sound volume `0.4*size`,
maximum head X rotation `0`, and sound pitch
`(1 + (nextFloat - nextFloat)*0.2) * (isTiny ? 1.4 : 0.8)`. `isTiny` is
`size <= 1`. Magma Cube overrides `isOnFire` to `false` and
`getLightLevelDependentMagicValue` to `1`.

**Transition and ordering:**

### Goal graph and targeting

`AbstractCubeMob.registerGoals` registers three shared goals and then calls
the subtype hooks, so subtype goals always register after the shared ones at
their own priorities:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `1` | Cube Mob Float, Jump/Move, every-tick |
| goal | `2` | Cube Mob Attack, Look, every-tick (both subtypes) |
| goal | `4` | Cube Mob Random Direction, Look |
| goal | `5` | Cube Mob Keep On Jumping, Jump/Move |
| target | `1` | nearest Player, interval `10`, must see, need not reach, vertical-distance selector |
| target | `3` | nearest Iron Golem, must see |

Both subtypes register exactly the same pair of target goals. The Player
selector is a subtype instance predicate that admits only candidates whose
absolute Y difference from the cube is at most `4`, so a Player far above or
below is never acquired even within follow range.

Every goal that touches movement first checks that the move control is a
`CubeMobMoveControl`, so replacing the control disables the whole graph
rather than partially driving it.

`CubeMobAttackGoal` requires a target that passes `canAttack`. It starts a
`reducedTickDelay(300)` — that is, `150` — tired timer, ticks every tick,
looks at the target with limits `10/10`, and hands the cube's current yaw
plus its `isDealsDamage` state to the move control. Continuation
predecrements the timer and ends at zero, so an attack run lasts at most
`150` goal ticks regardless of the target.

`CubeMobFloatGoal` enables navigation floating in its constructor, admits
while the cube is in water or lava, ticks every tick, jumps whenever one
`nextFloat` is below `0.8`, and sets wanted movement `1.2`.

`CubeMobKeepOnJumpingGoal` admits whenever the cube is not a passenger and
sets wanted movement `1`. Because it sits at priority `5` with the same Jump
and Move flags as the float goal at priority `1`, floating always wins while
submerged.

`CubeMobRandomDirectionGoal` admits with no target while the cube is on the
ground, in water, in lava or under Levitation. It rerolls every
`adjustedTickDelay(40 + nextInt(60))` and picks `nextInt(360)` degrees,
handing that direction to the move control with aggression `false`.

### Jumping, squishing and movement control

`CubeMobMoveControl` seeds its yaw from `180*yRot/pi` — a deliberate degree
conversion applied to a value already in degrees — and thereafter rotates
the cube toward its stored yaw with a `90`-degree limit, copying the result
into both head and body yaw so the cube always faces its travel direction.

While the operation is `MOVE_TO` and the cube is on the ground, the control
sets speed from `speedModifier * movementSpeed` and predecrements a jump
delay. At or below zero it reloads the delay from `getJumpDelay()`, divides
it by `3` when aggressive, calls the jump control, and plays the jump sound
when `doPlayJumpSound()` holds. Otherwise it zeroes both movement inputs and
the speed. Off the ground it only sets the speed.

`getJumpDelay` is `nextInt(20) + 10` for Slime and exactly four times that
for Magma Cube, so a Magma Cube waits `40..116` ticks between jumps against
a Slime's `10..29`, and the aggressive divide-by-three applies after that
multiplication.

`jumpFromGround` sets vertical velocity to the jump power for Slime, and to
`jumpPower + 0.1*size` for Magma Cube, so larger Magma Cubes leap
progressively higher. `doPlayJumpSound` requires `size > 0`, which every
clamped size satisfies.

Magma Cube overrides `jumpInLiquid` for the Lava tag only, setting vertical
velocity to `0.22 + 0.05*size` and flagging a sync; every other fluid falls
through to the inherited behavior.

`tick` first copies the squish into the previous squish and moves the squish
half-way toward the target squish. After the inherited tick, a cube that has
just landed emits `width*2*16` particles — `16` per block of doubled width —
each consuming two `nextFloat` draws for a uniform angle and a `0.5..1`
radius, plays the squish sound at the size-scaled volume and the standard
pitch formula, and sets the target squish to `-0.5`. A cube that has just
left the ground instead sets the target squish to `1`. Every tick ends by
recording the ground state and decaying the target squish by `0.6` for
Slime or `0.9` for Magma Cube.

The particle type is Item Slime, protocol particle ID `59`, for Slime and
Flame, protocol particle ID `39`, for Magma Cube.

`onSyncedDataUpdated` reacts to a size change by refreshing dimensions,
snapping both yaw values to the head yaw, and — when in water — playing a
splash effect on one `nextInt(20)` result of zero.

### Contact damage

`isDealsDamage` requires effective AI, and for Slime additionally requires
that the cube not be tiny; a Magma Cube deals damage at every size including
size `1`. `getAttackDamage` is the attack-damage attribute for Slime and
that attribute plus `2` for Magma Cube.

`dealDamage` requires a server level, a live cube, the target within melee
range and line of sight. It then applies a mob attack for the computed
amount, and on success plays Slime Attack — protocol sound ID `1493`, used
by both subtypes — at volume `1` and pitch
`1 + (nextFloat - nextFloat)*0.2`, then runs enchantment post-attack
effects.

Two independent entry points call it: `playerTouch`, which fires whenever a
Player overlaps the cube, and `push`, which fires only when the pushed
entity is an Iron Golem. There is no melee attack goal, so all cube damage
is contact damage.

### Splitting on death

`remove` runs before the inherited removal. On a server level, when the size
exceeds `1` and the entity is dead or dying, it computes the current width,
a half-width offset, the child size `size/2` and a split count of
`2 + nextInt(3)`, and captures the team.

For each child index `i` it offsets by
`((i mod 2) - 0.5) * halfWidth` on X and `((i / 2) - 0.5) * halfWidth` on Z,
then converts to the same entity type with conversion type
`SPLIT_ON_DEATH`, spawn reason `TRIGGERED`, the captured team and a callback
that sets the child's size with health reset and snaps it to the parent
position plus the offset and `+0.5` on Y at a `nextFloat*360` yaw.

Because the split count is `2..4` but the offsets repeat every four indices,
counts of two or three simply use the first two or three of the four corner
offsets. The child size is integer division, so an odd parent size loses the
remainder — a size-`3` cube produces size-`1` children.

Splitting is unconditional on the damage source and on the mob-loot gamerule;
it happens even when the parent drops nothing.

### Natural spawning

Both types register `ON_GROUND` placement with heightmap
`MOTION_BLOCKING_NO_LEAVES` and their own predicate.

`MagmaCube.checkMagmaCubeSpawnRules` is only `difficulty != PEACEFUL`. It
adds no light, height or medium test of its own, leaving the generic Mob
placement checks as the only further gate. Exactly two of the `66` locked
biomes carry a Magma Cube row: Basalt Deltas at weight `100` for groups of
`2..5`, and Nether Wastes at weight `2` for groups of exactly `4`.

`Slime.checkSlimeSpawnRules` is a three-branch predicate. Peaceful rejects
outright. A spawner reason then delegates straight to the generic Mob rules.
Otherwise:

1. **Swamp band.** When the biome is in `allows_surface_slime_spawns`, the
   Y coordinate is strictly between `50` and `70`, one `nextFloat` is below
   the `surface_slime_spawn_chance` environment attribute at that position,
   and the maximum local raw brightness is at most `nextInt(8)`, it
   delegates to the generic Mob rules.
2. **Slime chunk.** Otherwise the level must be a `WorldGenLevel`, or the
   predicate returns `false` immediately. It derives a chunk random from the
   chunk X, chunk Z, the world seed and the constant `987234911`, and
   requires `nextInt(10) == 0` from that stream, one `nextInt(10) == 0` from
   the spawn stream, and Y strictly below `40`, before delegating to the
   generic Mob rules.
3. Anything else fails.

The slime-chunk stream is seeded from the world seed and chunk coordinates
only, so it is stable for a given world and independent of the spawn
attempt; the second `nextInt(10)` is the per-attempt roll. `55` of the `66`
locked biomes carry a Slime row, all at weight `100` for groups of exactly
`4`.

`setSpawnSize` runs from `finalizeSpawn` for both types. It draws
`nextInt(3)`, and when that is below `2` it draws one more `nextFloat` and
increments the exponent if it is below `0.5 * specialMultiplier` of the
local difficulty. The size is `1 << exponent`, so natural sizes are `1`,
`2` or `4` with the larger sizes gated on regional difficulty. Both
subtypes substitute a non-baby ageable group data before delegating, which
keeps the shared group machinery from producing babies.

### Loot, tags, advancements and item identity

The Slime loot table uses random sequence `minecraft:entities/slime` and one
one-roll pool gated on the cube-mob size predicate being exactly `1`. Inside
that pool, two mutually exclusive entries select on the killer:

- not a Frog: Slime Ball, item protocol ID `1059`, integer-uniform `0..2`
  with uniform `0..1` Looting enchanted-count increase;
- a Frog: exactly one Slime Ball with no Looting.

Larger Slimes therefore drop nothing directly; all Slime Balls come from the
size-`1` children produced by splitting.

The Magma Cube loot table uses random sequence
`minecraft:entities/magma_cube` and one one-roll pool with four entries:

- not a Frog and size at least `2`: Magma Cream, item protocol ID `1154`,
  integer-uniform `-2..1` with uniform `0..1` Looting increase;
- killed by a warm Frog: one Pearlescent Froglight, item ID `1454`;
- killed by a cold Frog: one Verdant Froglight, item ID `1453`;
- killed by a temperate Frog: one Ochre Froglight, item ID `1452`.

The Magma Cream count range starts at `-2`, so a base roll produces nothing
on three of its four outcomes and positive-count filtering removes the rest;
Looting shifts the whole range upward. Unlike Slime, the Magma Cube pool
excludes size `1` rather than requiring it, so tiny Magma Cubes drop nothing
but every larger one can.

Direct entity-type tag membership differs between the two:

| Tag | Slime | Magma Cube |
|---|---|---|
| `frog_food` | yes | yes |
| `non_controlling_rider` | yes | yes |
| `immune_to_oozing` | yes | no |
| `no_anger_from_wind_charge` | yes | no |
| `fall_damage_immune` | no | yes |
| `freeze_hurts_extra_types` | no | yes |

Both hostile-mob advancements name both identities. `kill_a_mob` places each
in one OR requirement group with every listed hostile; `kill_all_mobs`
places each in its own required group.

The Spawn Eggs are raw/protocol item IDs `1225` for Slime and `1237` for
Magma Cube, both common with maximum stack `64`.

### Sounds and client projection

The locked sound-event joins are:

| Protocol ID | Event |
|---:|---|
| `1493` | Slime Attack, shared by both subtypes |
| `1494` / `1546` | Slime Death / Death Small |
| `1495` / `1547` | Slime Hurt / Hurt Small |
| `1496` / `1548` | Slime Jump / Jump Small |
| `1497` / `1549` | Slime Squish / Squish Small |
| `962` / `949` | Magma Cube Death / Death Small |
| `963` / `964` | Magma Cube Hurt / Hurt Small |
| `965` | Magma Cube Jump |
| `966` / `967` | Magma Cube Squish / Squish Small |

Every Slime hurt, death, squish and jump event selects its small variant
through `isTiny`, but Magma Cube has a single jump event with no small
variant, so a tiny Magma Cube reuses the full-size jump sound at the
tiny-scaled pitch.

`ModelLayers` registers `slime`, the `slime/outer` layer and `magma_cube`.
The two locked textures are `textures/entity/slime/slime.png`, `64×32` and
`314` bytes with SHA-256
`c724fc4eb07da3e2e10cc4b269d554bfb19271a560e2dbd4b74f71fae050f955`, and
`textures/entity/slime/magmacube.png`, `64×64` and `728` bytes with SHA-256
`a45640114edf6384b6b9e395b8ef57d7e1293ba7a54312f5b07c4ec3a69d689e`.

Because the squish interpolation and the size are both synchronized rather
than predicted, a client renders the squish from slot `18` and the two
squish floats it maintains locally from the same tick function the server
runs.

**Branches and aborts:**

- Every movement goal aborts when the move control is not a
  `CubeMobMoveControl`.
- The Player target selector rejects any candidate more than `4` blocks
  above or below the cube.
- The attack goal ends after `150` goal ticks even with a valid target.
- Contact damage aborts on a client level, a dead cube, a target outside
  melee range or without line of sight; Slime additionally aborts at size
  `1`.
- Splitting aborts on a client level, at size `1`, or when the entity is not
  dead or dying.
- Slime spawning aborts on Peaceful, on a non-worldgen level once the swamp
  branch fails, and on any of its three chained rolls; Magma Cube spawning
  only aborts on Peaceful.
- The spawn-size exponent increment aborts when the first draw is `2` or
  when the second draw is at or above half the special multiplier.

**Invariants:**

- Slot `18` is the single authority for health, speed, damage, XP, armor,
  dimensions, sound volume and pitch, and drop eligibility.
- Size is always clamped to `1..127`; natural spawning only produces `1`,
  `2` or `4`.
- Splitting always produces `2..4` children of size `size/2`, independent of
  damage source and mob-loot gamerule.
- `Size` persists one lower than it is used, so a missing key restores `1`.
- Slime drops only at size `1`; Magma Cube drops only above size `1`.
- Magma Cube deals contact damage at every size; Slime only above size `1`.
- Both subtypes register identical target goals and identical vertical-range
  selectors.
- Slime Attack is the contact sound for both identities.

**Constants and randomness:**

Entity IDs `117/80`; Egg IDs `1225/1237`; dimensions/eye/spawn-scale
`0.52×0.52/0.325/4`; tracking `10/8`; size clamp `1..127`, natural maximum
`4`; health `size²`, speed `0.2+0.1*size`, damage `size` (`+2` Magma),
armor `3*size` (Magma), XP `size`; slot `18 INT` default `1`;
attachment `0.015625*size`; volume `0.4*size`, pitch base `1.4/0.8` with
`(nextFloat-nextFloat)*0.2`; jump delay `nextInt(20)+10`, Magma `×4`,
aggressive `/3`; Magma jump lift `0.1*size`, lava jump `0.22+0.05*size`;
squish lerp `0.5`, decay `0.6/0.9`, landed `-0.5`, airborne `1`,
particles `width*2*16` with angle `2π` and radius `0.5..1`;
attack tired `reducedTickDelay(300)=150`, look `10/10`, float jump `0.8`,
float speed `1.2`, jump speed `1`, direction reroll
`adjustedTickDelay(40+nextInt(60))` over `nextInt(360)`;
target interval `10`, vertical range `4`; split `2+nextInt(3)`, child
`size/2`, offsets `±halfWidth/2`, lift `0.5`, yaw `nextFloat*360`;
slime chunk seed constant `987234911`, `nextInt(10)` twice, Y `<40`;
swamp band Y `50..70` exclusive, brightness `<= nextInt(8)`;
spawn size `nextInt(3)`, `0.5*specialMultiplier`, `1<<exponent`;
biome rows `55/2 of 66`; particles `59/39`.

**Side effects:**

Slot `18`, attribute base values, health, dimensions and position; RNG
cursors for jump delay, direction, squish particles, sound pitch, split
count and yaw, spawn size and both spawn predicates; targets, goal
arbitration, yaw and speed; contact damage and enchantment post-attack
effects; child entity creation and team assignment; loot stacks, XP and
advancement progress; particles, sounds and renderer state.

**Gates:**

Logical side, Peaceful, NoAI and persistence; move-control identity; goal
priority and flags; target vertical range, sight and `canAttack`; melee
range and line of sight; size thresholds for damage, drops and sound
variants; ground state and fluid membership; biome tag, Y band, brightness,
environment attribute, slime-chunk stream and worldgen level; local
difficulty special multiplier; killer entity type and Frog variant; Looting
and mob loot; resources.

**Boundary cases and quirks:**

The move control converts an already-degree yaw through `180/pi` when it is
first constructed, so a freshly created cube starts with a badly scaled
target yaw until the first direction assignment overwrites it. A size-`3`
cube splits into size-`1` children, losing the remainder. Split children can
overlap because two or three children reuse the first corner offsets rather
than spreading evenly. A Magma Cube jumps roughly four times less often than
a Slime but leaps higher with every size step, and its aggressive divide
applies to the already multiplied delay. Tiny Magma Cubes still deal contact
damage while tiny Slimes do not. Magma Cube has no small jump sound, so tiny
ones reuse the full-size event. Slime drops come only from the smallest
children, so killing a large Slime in one hit still yields Slime Balls only
after the split chain reaches size `1`. The Magma Cream count range starts
negative, so most unenchanted kills of a large Magma Cube drop nothing.
Magma Cube spawning has no light or biome-independent gate of its own, so
its two biome rows are the entire baseline selector. The slime-chunk branch
requires a worldgen level, so a plain runtime placement check outside
worldgen can only succeed through the swamp band.

**Failure semantics:**

A rejected contact damage result plays no sound and runs no post-attack
effects. A failed child conversion simply produces one fewer child; the
remaining indices still run. Splitting is not rolled back if the parent's
removal later fails. Spawn predicates are pure and leave no state behind
except their consumed RNG. A size assignment that clamps still writes the
clamped value to every derived attribute.

**Client/server authority split:**

The server owns size, attributes, health, targets, goals, jumping, contact
damage, splitting, spawning, loot and advancements. Clients consume slot
`18` and movement, run the same squish interpolation and landing particle
burst locally, select the size-dependent sound variants and pitch, and
render the inner and outer model layers scaled by size. Client squish state
cannot commit server effects.

**Observability:**

Observe both registrations and slot `18`; save/reload of the offset `Size`
key and `wasOnGround`; every derived attribute at sizes `1`, `2`, `3`, `4`
and `127`; the four shared goals plus both subtype target goals and the
vertical-range boundary; move-control yaw seeding, jump delay, aggression
divide and both jump overrides; squish interpolation, particle count and
both decay rates; contact damage entry points and the tiny-size split
between subtypes; split count, child size, corner offsets and team
inheritance; both spawn predicates across Peaceful, spawner, swamp band,
slime chunk and every RNG endpoint; spawn-size distribution across local
difficulty; both loot tables with every Frog and Looting branch; the six
differing tags, both advancements, both Eggs; sounds including the missing
Magma small jump; and exact texture and model projection.

**Persistence and reload:**

Generic entity/Mob state, the offset `Size` key and `wasOnGround` save.
Slot `18` is rebuilt from `Size` on load through `setSize` with health reset
disabled, so a reloaded cube keeps its stored health rather than being
restored to full. Squish state, move-control state, goal timers and targets
do not persist. Loot, tags, advancements and biome data reload through their
owners; spawn predicates and split code remain fixed. Models and textures
reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.AgeableMob`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.cubemob.AbstractCubeMob`;
`net.minecraft.world.entity.monster.cubemob.Slime`;
`net.minecraft.world.entity.monster.cubemob.MagmaCube`;
`net.minecraft.world.entity.ai.goal.Goal`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.entity.ai.control.MoveControl`;
`net.minecraft.world.entity.ConversionParams`;
`net.minecraft.world.entity.ConversionType`;
`net.minecraft.world.level.levelgen.WorldgenRandom`;
`net.minecraft.tags.BiomeTags`;
`net.minecraft.world.attribute.EnvironmentAttributes`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.tags.EntityTypeTagsProvider`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.schemas.V99`;
`net.minecraft.util.datafix.schemas.V705`;
`net.minecraft.util.datafix.schemas.V1460`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.MagmaCubeRenderer`;
`net.minecraft.client.model.monster.slime.MagmaCubeModel`;
`net.minecraft.client.model.geom.ModelLayers`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,particle_type,
loot_table,worldgen/biome,advancement}`;
`data/minecraft/tags/entity_type/{frog_food,non_controlling_rider,immune_to_oozing,
no_anger_from_wind_charge,fall_damage_immune,freeze_hurts_extra_types}.json`;
`data/minecraft/loot_table/entities/{slime,magma_cube}.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`assets/minecraft/textures/entity/slime/{slime,magmacube}.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-DAMAGE-001`; `ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`;
`MOB-AI-001`; `MOB-SPAWN-001`; `MOB-DESPAWN-001`;
`ITM-SLIME-BALL-001`; `ITM-ENCHANT-001`; `CLI-006`.

**Test vectors:**

Run `EXP-ENT-028` across construction/metadata/NoAI/save/reload at every
size endpoint; all four shared goals and both subtype target goals including
the vertical-range boundary and the tired timer; move-control yaw seeding,
jump delay for both subtypes, the aggression divide and both jump overrides;
squish interpolation, landing particle counts and both decay rates; contact
damage through both entry points at every size; split count, child size,
corner offsets, team inheritance and non-rollback; both spawn predicates
across Peaceful, spawner, swamp band, slime chunk and every RNG endpoint,
plus the spawn-size distribution against local difficulty; both loot tables
with every Frog variant and Looting branch; the six differing tags, both
advancements, both Eggs; every sound including the absent Magma small jump;
and exact model, layer and texture projection.

**Limits:**

Generic lifecycle, metadata, pathfinding, target algorithms, damage/death,
knockback, ageable machinery, natural spawning and despawn, biome data,
loot, advancements, Spawn Egg interaction, Frog runtime, particles and
rendering retain their cited owners. Shared cube-mob algorithms are included
only where the Slime and Magma Cube subtypes register, select or change
their exact inputs and observable result.
