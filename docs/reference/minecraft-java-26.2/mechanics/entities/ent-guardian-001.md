# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-GUARDIAN-001` — Guardians swim by oscillating path control, retaliate with stationary thorns and charge a synchronized beam

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`ITM-COD-001`, `ITM-SALMON-001`, `ITM-TROPICAL-FISH-001`,
`ITM-PUFFERFISH-001`, `ITM-PRISMARINE-MATERIAL-001`,
`ITM-ENCHANT-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PORTAL-001`, `WGEN-STRUCTURE-OCEAN-MONUMENT-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Guardian` class and
its three nested goal/control classes, target-goal continuation, placement
and category code, all 66 biomes, the Ocean-Monument spawn override, four
direct entity tags, three-pool loot, both hostile-mob advancements, Spawn
Egg, ten migration/schema contexts, all 1,212 templates and exact entity,
beam and item client resources close protocol entity ID `63`.

**Applies when:**

`minecraft:guardian` is constructed, spawned naturally inside an Ocean
Monument or through an Egg, spawner, command or custom selector, restricted,
swimming, flopping, selecting or beaming a target, retaliating through
thorns, damaged, killed, synchronized, heard, imitated by a Parrot or
rendered.

**Authoritative state:**

Protocol entity ID `63` constructs `Guardian` in `MONSTER`, so it is
unavailable in Peaceful. Registration fixes width/height `0.85×0.85`, eye
height `0.425`, passenger attachment `(0,0.975,0)`, client tracking range
`8` and default update interval `3`. It is neither fire-immune nor
persistence-required.

Attributes fix maximum health `30`, movement speed `0.5`, attack damage `6`
and inherited Mob follow range `16`. Construction sets XP reward `10`,
Water path malus `0`, a Guardian movement controller and Water-Bound
navigation. It consumes one entity `nextFloat` to initialize both current
and previous client tail phase. The initial tail speed, spike phases,
client attack counter, target cache and touched-ground flag retain their
zero/false defaults.

The Monster category cap is `70`, its no-despawn/despawn distances are
`32/128`, and inherited maximum cluster size is `4`. Movement emission is
`EVENTS`, gravity is `0.08`, maximum head X/Y rotations are `180/75`, and
spawn obstruction requires only that the level report the Guardian
unobstructed. It is an `Enemy`, so generic lead interaction cannot leash
it; it has no age, breeding or subtype interaction path. Fluid-current push
is enabled. Unlike Elder Guardian, ordinary Guardian lacks direct
`cannot_be_pushed_onto_boats` membership, so that tag does not suppress its
generic Boat-push path.

Inherited Entity, Living-Entity and Mob state occupies synchronized metadata
slots `0..15`. Guardian adds slot `16`, serializer ID `8` (`BOOLEAN`),
`moving=false`, and slot `17`, serializer ID `1` (`INT`), active attack
target entity ID `0`. Neither slot persists. The Mob target, attack counter,
all client animation/cache state, controller wanted state, navigation and
goal-local state are also transient.

Generic Mob home state remains optional and persistent. Absent or negative
`home_radius` means no restriction and leaves `home_pos` at zero; a
nonnegative radius reads `home_pos`, defaulting malformed or missing
coordinates to zero. Ordinary Guardian never establishes a home itself, so
its restriction goal is inert after ordinary construction unless a caller
or loaded Mob home pair supplies one.

**Transition and ordering:**

### Goal graph and target selection

Guardian registers six goals and one target selector:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `4` | Guardian Attack, Move/Look |
| goal | `5` | Move Towards Restriction, speed `1`, Move/Look |
| goal | `7` | Random Stroll, speed `1`, interval `80`, Move/Look |
| goal | `8` | Look At Player, range `8` |
| goal | `8` | Look At Guardian, range `12`, probability `0.01` |
| goal | `9` | Random Look Around |
| target | `1` | nearest attackable Living Entity, random interval `10`, must see, need not reach |

The attack, restriction and stroll goals mutually exclude through Move/Look.
The two priority-8 goals and priority-9 goal arbitrate their Look flag in
registration/priority order. The target selector is independent of those
control flags.

Acquisition searches the follow-range-expanded box and accepts only Player,
Squid or Axolotl whose squared distance from the Guardian is strictly above
`9`; generic combat conditions additionally own liveness, alliance,
visibility and Player-mode admission. It performs the interval draw before
searching. Target continuation does not reapply the class selector or
distance-above-nine predicate: it instead uses generic attackability,
nonalliance, squared range at most `16²` and must-see unseen memory nominally
`60`, with no reach test. The Guardian Attack goal independently reapplies
the strict squared-distance-above-`9` condition at its own continuation
check.

There is no Hurt-By target goal. Direct `axolotl_always_hostiles` membership
separately makes Guardian an always-hostile target for Axolotl, while the
Guardian selector itself can target Axolotl.

`NoAI` makes Mob effective AI false, suppressing target and goal selectors,
navigation and control updates. Guardian-local Water-air reset, land flop
impulse, active-target yaw and client animation are ordered before the
inherited effective-AI gate and continue as described below.

### Path movement and Water travel

Guardian's walk-target value is
`10 + pathfindingCostFromLightLevels(position)` for Water-fluid candidates
and the Monster value otherwise.

Each movement-controller tick stops speed and clears slot `16` unless the
operation is `MOVE_TO` and navigation is not done. Otherwise it forms the
wanted-minus-current vector and length without a local zero guard, then:

- rotates yaw toward `atan2(dz,dx)*57.2957763671875-90` by at most `90`
  degrees and copies it to body yaw;
- lerps current speed by `0.125` toward
  `speedModifier*MOVEMENT_SPEED`;
- adds X/Z wave
  `sin((tickCount+entityId)*0.5)*0.05*(cosYaw,sinYaw)`;
- adds Y wave
  `sin((tickCount+entityId)*0.75)*0.05*(sinYaw+cosYaw)*0.25`
  plus `actualSpeed*normalizedY*0.1`;
- constructs a look point two blocks forward horizontally and at
  `eyeY+normalizedY`, substitutes that point for any inactive retained look
  coordinates, lerps all three coordinates toward it by `0.125`, then
  requests look limits `10/40`; and
- writes slot `16=true`.

At both direct speed-`1` movement goals, the ordinary Guardian's target
speed is therefore attribute value `0.5` before the controller lerp. A
non-done zero-length wanted vector divides by zero before downstream
movement/look projection; generic vector and floating-point behavior own
the resulting nonfinite values.

Water travel applies relative input at `0.1`, moves with `SELF`, scales the
resulting velocity by `0.9`, then adds Y `-0.005` only when slot `16` is
false and the Mob target is absent. Generic non-Water travel retains its
owner.

### Beam attack

Guardian Attack can start only with a present live target. Start writes
`attackTime=-10`, stops navigation, aims at a present target with limits
`90/90` and requests motion synchronization. It requires an update every
tick.

Continuation first requires the same live-target admission and then, for
ordinary Guardian, squared distance strictly above `9`. Every admitted tick
again stops navigation and aims at the target, then checks line of sight.
Failed sight clears the Mob target and returns before incrementing the
counter.

With sight, the counter increments first. At counter `0`, ten attack ticks
after start, the server writes the target's live entity ID to slot `17`; a
nonsilent Guardian then broadcasts entity event `21`. Silence suppresses
only this event and its client sound, not target metadata, charge, particles
or damage.

At counter `80`, the server first offers indirect-Magic damage:

- `1` on Easy or Normal;
- `3` on Hard.

The Guardian is both direct and causing entity for that source. The result
is ignored. It then independently invokes generic Mob attack with attack
attribute `6`, ignores that result and clears the Mob target. The
start-to-damage delay is therefore `90` attack ticks, while synchronized
beam charge lasts `80`.

Either offer can fail or alter/remove the target independently. Magic
failure does not suppress melee; a lethal magic offer does not cancel the
following call. The attack goal's own final `Goal.tick` has no additional
effect.

Clearing the Mob target does not invoke goal stop inline. Normal Mob
scheduling performs full selector cleanup on the
`(tickCount+entityId)` even phase and only ticks every-tick goals on the
alternate phase. Sight loss, completion or close-range entry can therefore
leave slot `17` and beam projection alive until the next full cleanup.
Close-range entry ordinarily aborts before damage when that cleanup runs,
but an alternate-phase tick can still advance once before the condition is
rechecked. Cleanup invokes stop, which writes slot `17=0`, redundantly
clears the target and triggers Random Stroll.

On slot-17 change, the client resets attack time and cached attack target.
It resolves the integer as a live entity ID once and caches it only if that
entity is Living. Client attack time increments to the `80`-tick cap and
partial scale is `(clientAttackTime+partialTick)/80`.

Entity event `21` constructs
`net.minecraft.client.resources.sounds.GuardianAttackSoundInstance`
directly in the packet handler. The instance is Hostile, looping-configured,
unattenuated and delay-free; `canPlaySound` rejects a silent Guardian. Each
sound tick follows float-rounded entity coordinates, sets
`volume=scale²` and `pitch=0.7+0.5*scale`, and stops when the Guardian is
removed or inherited `Mob.getTarget()` is null.

That sound liveness test is deliberately recorded separately from slot
`17` and `getActiveAttackTarget()`: the synchronized beam target resets and
resolves a client cache, but this class reads the unsynchronized Mob target.
The experiment therefore records both fields rather than inferring sound
lifetime from visible beam lifetime.

### Defensive thorns, breathing, flop and client animation

Incoming server damage first evaluates Guardian thorns, before generic
damage admission. If slot `16` is false, the source is neither in
`avoids_guardian_thorns` nor exactly Thorns, and the direct entity is
Living, that direct entity is offered `2` Thorns damage from the Guardian.
The result is ignored.

Projectile ownership alone is insufficient: an ordinary projectile is the
direct entity and is not Living, even when its causing entity is. A melee
Living direct entity is eligible. Thorns can commit even when the original
hit is later rejected.

Random Stroll is triggered whenever the stored goal exists, independently
of thorns eligibility. Only after that trigger is the original damage
offered to `Monster.hurtServer`, whose Boolean becomes Guardian's result.

While alive, the client-only animation block runs first:

- outside Water, tail speed becomes `2`; positive Y motion after retained
  touched-ground state plays the local flop sound at volume/pitch `1/1`
  when nonsilent, then touched-ground becomes true only for downward motion
  above a loaded standable cell;
- in Water while moving, tail speed snaps to `4` when below `0.5`, otherwise
  approaches `0.5` by `0.1`; while stationary it approaches `0.125` by
  `0.2`;
- tail phase adds that speed without wrapping;
- dry spike phase becomes one fresh entity `nextFloat`; in Water it
  approaches `0` by `0.25` while moving or `1` by `0.06` while stationary;
- moving in Water emits two local Bubble particles behind the horizontal
  view vector, each using the entity's random-X, random-Y and random-Z
  helpers; and
- with a resolved slot-17 target, client attack time advances to `80`, the
  Guardian looks at the target with `90/90` and ticks Look Control
  explicitly, then emits Bubble particles along the normalized
  eye-to-target-midpoint ray.

Beam-particle distance starts with one `nextDouble`. While it is less than
the ray length, each emitted point advances by
`1.8-scale + nextDouble*(1.7-scale)`. Requested particle velocity is zero.

After the client-only block, both logical sides reset air to `300` in Water.
Otherwise, an alive on-ground Guardian consumes three floats, adds X/Z
`(2*nextFloat-1)*0.4` and Y `0.5`, sets yaw to `nextFloat*360`, clears
on-ground state and requests motion synchronization. With an active slot-17
target it finally copies head yaw to entity yaw before inherited Monster AI.
Direct `can_breathe_under_water` membership also exempts it from generic
underwater air loss.

### Placement and natural Monument selection

Spawn Placements registers Guardian as `IN_WATER` with
`MOTION_BLOCKING_NO_LEAVES` and `checkGuardianSpawnRules`. The generic
placement type requires world-border membership, Water fluid at the
candidate and a non-redstone-conducting block above.

The Guardian predicate then consumes `nextInt(20)` first. A nonzero result
rejects only when the candidate can see sky from below Water, giving
sky-visible sites a `1/20` pass while fully covered sites pass that branch
regardless of the draw. It next requires non-Peaceful difficulty. Spawner
reason bypasses its repeated candidate-Water read; every other reason
requires candidate Water again, and the block below must always contain
Water-tag fluid. Obstruction is checked afterward by the placement owner.

All `66` locked baseline biome records contain zero
`minecraft:guardian` rows. The sole baseline selection record is Ocean
Monument's full-bounding-box Monster spawn override: weight `1`, group
`2..4`, no spawn cost. That override replaces the biome Monster list inside
the structure and independently replaces Axolotl and Underground-Water-
Creature lists with empty lists. Structure lookup, pack walks, placement
calls and cap accounting retain `WGEN-STRUCTURE-OCEAN-MONUMENT-001` and
`MOB-SPAWN-001`.

Ordinary Guardians are not persistence-required, so accepted natural
members participate in Monster distance removal at `32/128`. Group maximum
`4` does not exceed cluster maximum `4`. All `1,212` locked structure
templates contain zero exact `minecraft:guardian` or legacy `Guardian`
identity; baseline production is the data-driven Monument override rather
than template NBT or the three code-built Elder sites.

### Loot, tags, advancements and Spawn Egg

The entity loot table uses random sequence
`minecraft:entities/guardian` and evaluates three ordered, independent
one-roll pools:

1. Prismarine Shards receive integer-uniform count `0..2`, followed by
   uniform `0..1` Looting enchanted-count increase.
2. Weighted Cod `2`, Prismarine Crystals `2` or empty `1` is selected. A
   selected item starts at one and receives the same Looting increase. Cod
   then furnace-smelts when the Guardian is on fire or the direct attacker's
   main hand has an enchantment in `smelts_loot`.
3. A player kill tests rare chance `0.025` without Looting, or linear
   `0.035 + 0.01*(level-1)` with positive Looting. Success delegates once
   to Fishing/Fish: Cod/Salmon/Tropical Fish/Pufferfish weights
   `60/25/2/13`, then applies the same fire-or-direct-attacker smelting
   function.

The first two pools have no player-kill gate. A zero base Shard count or a
selected common item can become positive through Looting. The entity
supplies fixed XP `10` and has no subtype equipment producer.

Exactly four entity-type tags directly name Guardian:

- `aquatic`, which also reaches `sensitive_to_impaling` through nested
  membership;
- `axolotl_always_hostiles`;
- `can_breathe_under_water`; and
- `not_scary_for_pufferfish`.

Impaling, Axolotl targeting, breath and Pufferfish fear consumers retain
their owners. There is no direct ordinary-Guardian entry in
`cannot_be_pushed_onto_boats`.

Both hostile-mob advancements have an exact
`player_killed_entity` criterion for Guardian. `kill_a_mob` places it in one
OR requirement group with every listed hostile; `kill_all_mobs` places it
in its own required group and awards `100` experience only after all groups
complete.

The Spawn Egg is raw/protocol item ID `1222`, common, maximum stack `64`,
and its `entity_data.id` is `minecraft:guardian`. Its item definition
selects the generated model, whose sole layer selects the Egg texture.
English labels are “Guardian” and “Guardian Spawn Egg”.

### Sounds and client projection

Ambient cadence is `160`. Water/land selection uses `isInWater()` at the
sound request:

| Protocol ID | Event | Locked clips and subtitle |
|---:|---|---|
| `767` | ambient in Water | four `guardian_idle` clips; “Guardian moans” |
| `768` | ambient on land | four `land_idle` clips; “Guardian flaps” |
| `769` | beam attack | one `attack_loop` clip; “Guardian shoots” |
| `770` | death in Water | one `guardian_death` clip; “Guardian dies” |
| `771` | death on land | one `land_death` clip; same death subtitle |
| `772` | flop | four `flop` clips; “Guardian flops” |
| `773` | hurt in Water | four `guardian_hit` clips; “Guardian hurts” |
| `774` | hurt on land | four `land_hit` clips; same hurt subtitle |

Parrot imitation maps Guardian to sound event ID `1226`. It references
Guardian Water ambient as an event at volume `0.4`, pitch `1.8`, subtitle
“Parrot moans”. Parrot attempt cadence, nearby selection, silence and
playback retain that entity's owner.

`GuardianRenderer` uses shadow radius `0.5`, `ModelLayers.GUARDIAN` and
`textures/entity/guardian/guardian.png`. The entity texture is `64×64`,
`1,016` bytes, SHA-256
`c380cd88ced49e0496d293b9527dc49dce4946a358aea9eff588f4756732053d`.

The `64×64` model layer builds a composite five-box head shell, twelve
`2×9×2` spikes, one `2×2×1` eye and three nested tail sections. Spike
position uses each fixed base vector multiplied by
`1+0.01*cos(age*1.5+index)-(1-spikesAnimation)*0.55`; fixed quarter-turn
rotation arrays orient them. Head rotations convert degrees to radians.
The eye selects its active target or the camera: Y is `0` when the look
point is above and `1` otherwise, and horizontal X is
`2*sqrt(abs(sideDot))*sign(sideDot)`. Tail-section yaw is
`sin(tail)*pi*(0.05,0.1,0.15)`.

Guardian render state contains spike/tail animation, eye/look positions,
beam target midpoint, attack time and scale. The renderer keeps an otherwise
culled Guardian visible when the AABB spanning its eye and target midpoint
intersects the frustum.

The beam uses `textures/entity/guardian/guardian_beam.png`, `32×32`, `246`
bytes, SHA-256
`408a473ce3ed9130e4229ffef4817d4ae8888fc0b268f33c479073578d0d9a90`.
It starts at eye height, rotates to the target-midpoint vector and extends
to `vectorLength+1`. With `s=attackScale²`, RGB is
`(64+floor(191s),32+floor(191s),128-floor(64s))`; alpha is `255`, packed
light is `15728880`, overlay is absent and normal is `(0,1,0)`.
Cross-section radii are `0.2/0.282`, rotation phase is
`-1.5*0.05*attackTime`, and longitudinal texture offset begins at
`-1+(attackTime*0.5 mod 1)` and spans `2.5*beamLength`. Geometry uses the
cutout beam texture. Generic render submission and vertex buffering retain
`CLI-006`.

The Spawn Egg texture is `16×16`, `255` bytes, SHA-256
`2e33aa2a5c81a75e79549cce02bbf054f1ee2adf9a8afc5d3c970d1986d2b99e`.

### Migration and schema closure

Exactly ten migration/schema contexts own Guardian compatibility:

- `EntityHealthFix` recognizes legacy `Guardian` health;
- `EntityElderGuardianSplitFix` leaves legacy `Guardian` unchanged unless
  Boolean `Elder=true`, in which case it renames the entity
  `ElderGuardian` without removing that key;
- `EntityIdFix` maps `Guardian` to `minecraft:guardian`;
- `EntityUUIDFix` includes the current identity in its Mob UUID rewrite
  set;
- `ItemSpawnEggFix` maps legacy Spawn-Egg damage `68` to `Guardian`;
- `ItemStackSpawnEggFix` maps the current identity to Guardian Spawn Egg;
- `StatsCounterFix` maps legacy/current statistics identity;
- schema `V99` registers legacy simple entity `Guardian`;
- schema `V705` registers the modern Mob shape and exact Spawn-Egg mapping;
  and
- schema `V1460` registers the current Mob shape.

Runtime Guardian ignores a retained legacy `Elder` key. No moving, beam,
target, attack, animation or subtype scalar is persisted. Generic health,
effects, equipment, air and optional Mob home state retain their schema and
runtime owners.

**Branches and aborts:**

- Target acquisition aborts on its interval draw, generic combat filters,
  class predicate or squared distance at most `9`; target continuation and
  attack-goal continuation are distinct gates.
- Attack use aborts without a live target. Sight loss clears it before
  counter mutation; ordinary close-range continuation aborts on the next
  full selector check.
- Slot-17 publication and nonsilent event `21` branch at counter `0`;
  independent magic/melee offers branch at counter `80`, then target clear
  awaits selector cleanup before stop clears metadata.
- Thorns aborts while moving, for excluded damage tags/type or a non-Living
  direct entity, but no thorns branch suppresses the stroll trigger or
  original damage evaluation.
- Placement aborts at border, generic Water/above, sky RNG, difficulty,
  repeated Water/below-Water or obstruction gates. Natural identity
  selection aborts outside a Monument full box because biomes have no row.
- Loot pools abort independently on empty count/entry, player kill, rare
  chance and smelting conditions.
- Missing/nonliving slot-17 client IDs suppress target caching, beam
  particles and beam rendering. Silence separately suppresses event `21`
  and sound construction.

**Invariants:**

- Slot `16=false` means movement control is stopped, not necessarily zero
  physical velocity.
- Slot `17=0` means no synchronized beam target; a nonzero ID is transient
  and resolved only in the receiving level.
- Beam visibility begins ten ticks after attack start and normal damage
  occurs eighty attack ticks after that metadata write.
- Acquisition and attack continuation require squared distance strictly
  above `9`; generic target-selector continuation alone does not.
- Thorns precedes original damage admission and tests the direct entity.
- Water air resets to `300` independently of direct breath-tag membership.
- The Monument full-box override is the sole baseline natural selector;
  biome records and structure templates contain no Guardian identity.
- The three loot pools are independent and ordered.
- Sound-instance liveness reads Mob target, while beam projection reads
  slot `17`; neither field substitutes for the other.

**Constants and randomness:**

Entity/Egg IDs `63/1222`; dimensions/eye/attachment
`0.85×0.85/0.425/0.975`; range/update `8/3`;
health/speed/attack/follow `30/0.5/6/16`; XP `10`;
slots `16 BOOLEAN/17 INT`; stroll `80`; target interval/follow/unseen
`10/16/60`, squared distance `>9`; attack `-10/0/80`, magic `1/3`, melee
`6`, event `21`; thorns `2`; Water travel `0.1/0.9/-0.005`, air `300`;
Monster cap/distances/cluster `70/32/128/4`; placement sky chance `1/20`;
Monument weight/group `1@2..4`; loot shard `0..2`, common weights `2/2/1`,
rare `0.025/0.035/+0.01`, fish `60/25/2/13`; tags/templates/migrations
`4/0 of 1212/10`; ambient `160`; shadow `0.5`.

**Side effects:**

Metadata and optional home state; RNG cursors; targets, goal arbitration,
navigation, movement, look/yaw and velocity; thorns, magic and melee
damage; Water air and land impulse; sounds, entity event and local
particles; natural spawn insertion and accounting; loot/XP, advancement
progress and item stacks; renderer cache, frustum, model and beam state.

**Gates:**

Logical side, alive, Peaceful and NoAI; home presence; target interval,
class/liveness/distance/sight/range/alliance and Player mode; goal priority
and flags; Water/navigation/moving/ground/loaded support; incoming damage
tag/type/direct entity; border/fluid/conductor/sky RNG/spawn reason/
difficulty/obstruction; Monument full box, pack/cap/despawn; player kill,
Looting, on-fire/direct-attacker enchantment and loot randomness; silence,
Mob target versus metadata target, camera and frustum; migrations and
resource identity.

**Boundary cases and quirks:**

At exactly three blocks target acquisition and Attack continuation reject;
at exactly sixteen blocks generic target continuation remains in range.
Close-range entry can advance on an alternate selector phase before the
next cleanup. Magic and melee are separate nontransactional offers. Thorns
can hurt an attacker whose original damage is rejected. Beam metadata can
outlive sight loss or completed damage until cleanup. The sound instance is
configured to loop but tests a different target field from the visible
beam. A non-done zero-length move target divides without a local zero guard.
Sky-visible placement consumes RNG before difficulty/fluid checks.
Registration plus zero biome rows does not imply global natural spawning;
the Monument override supplies the only baseline list. A zero common-loot
count can be revived by Looting.

**Failure semantics:**

Lost sight clears the target without damage. Close-range cleanup clears
target/metadata without a partial-beam refund. Either final damage result is
ignored. Rejected thorns does not prevent original damage evaluation;
rejected original damage remains Guardian's returned result. Rejected
placement prevents insertion. Natural-spawn group failures and cap
accounting retain the generic owner. Nested loot-table and item insertion
failures retain their owners. Missing client targets suppress projection
without changing server state.

**Client/server authority split:**

The server owns optional home, targets, goal scheduling, navigation, beam
timing/metadata/event and damage, thorns, placement/natural insertion,
loot/XP and advancement criteria. Clients consume metadata, movement,
entity events and resources; they animate tail/spikes/eye, emit movement and
beam bubbles, evaluate the attack-sound instance and render entity/beam/Egg.
Client cache, animation, particle, sound or renderer state cannot alter
server authority.

**Observability:**

Observe registration/attributes and both metadata slots; constructor RNG
and absent/present home; every goal/control/target phase; movement look and
zero-length math; all beam start/sync/sight/close-range/difficulty/damage/
stop outcomes; distinct Mob/slot target state in sound and rendering;
thorns-before-damage ordering; Water travel, flop and client RNG; placement,
Monument selection, pack/cap/despawn; all loot/tag/advancement/Egg joins;
migrations and zero-template census; sound/Parrot mapping, model, textures,
beam geometry and frustum projection.

**Persistence and reload:**

Generic entity/Mob state and a caller-supplied `home_radius/home_pos` pair
persist. Moving/beam slots, targets, attack counter, navigation/controller,
client animation/cache and renderer state do not. An ordinary fresh or
default-loaded Guardian has no home and does not acquire one. Code fixes
registration, goals and migrations. Loot, tags, Monument record and
advancements reload through their owners; language, sounds, item models and
textures reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.Guardian`;
`net.minecraft.world.entity.monster.Guardian$GuardianAttackGoal`;
`net.minecraft.world.entity.monster.Guardian$GuardianMoveControl`;
`net.minecraft.world.entity.monster.Guardian$GuardianAttackSelector`;
`net.minecraft.world.entity.ai.goal.MoveTowardsRestrictionGoal`;
`net.minecraft.world.entity.ai.goal.RandomStrollGoal`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.entity.ai.goal.target.TargetGoal`;
`net.minecraft.world.level.NaturalSpawner`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.client.multiplayer.ClientPacketListener`;
`net.minecraft.client.resources.sounds.GuardianAttackSoundInstance`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.GuardianRenderer`;
`net.minecraft.client.renderer.entity.state.GuardianRenderState`;
`net.minecraft.client.model.monster.guardian.GuardianModel`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`net.minecraft.util.datafix.fixes.EntityHealthFix`;
`net.minecraft.util.datafix.fixes.EntityElderGuardianSplitFix`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.ItemSpawnEggFix`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.fixes.StatsCounterFix`;
`net.minecraft.util.datafix.schemas.V99`, `V705` and `V1460`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,loot_table,worldgen/biome,worldgen/structure,advancement}`;
`reports/minecraft/components/item/guardian_spawn_egg.json`;
`data/minecraft/tags/entity_type/{aquatic,axolotl_always_hostiles,can_breathe_under_water,not_scary_for_pufferfish,sensitive_to_impaling}.json`;
`data/minecraft/loot_table/entities/guardian.json`;
`data/minecraft/loot_table/gameplay/fishing/fish.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/worldgen/structure/monument.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/guardian_spawn_egg.*`;
`assets/minecraft/textures/entity/guardian/{guardian,guardian_beam}.png`;
`assets/minecraft/{sounds,lang/en_us}.json`;
`ENT-DAMAGE-001`; `ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`;
`MOB-AI-001`; `MOB-SPAWN-001`; `MOB-DESPAWN-001`;
`ITM-COD-001`; `ITM-SALMON-001`; `ITM-TROPICAL-FISH-001`;
`ITM-PUFFERFISH-001`; `ITM-PRISMARINE-MATERIAL-001`;
`ITM-ENCHANT-001`; `WGEN-STRUCTURE-OCEAN-MONUMENT-001`; `CLI-006`.

**Test vectors:**

Run `EXP-ENT-025` across construction/home/persistence, every goal and
movement-controller boundary, beam timing/sight/close-range/damage/cleanup,
distinct client Mob/slot targets and sound lifetime, thorns and generic
damage results, Water/dry/client animation RNG, placement/Monument/pack/cap/
despawn cases, three loot pools/XP/four tags/two advancements/Egg,
templates/ten migrations/sounds/Parrot and exact model/texture/beam/frustum
projection.

**Limits:**

Generic lifecycle, metadata, optional-home codec, goal arbitration/path
search, damage/death, natural spawning/despawn, structure-override
consumption, loot evaluation, advancement triggers, Spawn Egg interaction,
sound mixing and render submission retain their cited owners. Elder
Guardian scaling, required persistence, Mining Fatigue, apparition,
code-built production and extra loot/tag/resource joins remain
`ENT-ELDER-GUARDIAN-001`.
