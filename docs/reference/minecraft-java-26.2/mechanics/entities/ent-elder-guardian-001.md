# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-ELDER-GUARDIAN-001` — Elder Guardians anchor monuments, charge a synchronized beam and pulse Mining Fatigue

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`BLK-SPONGE-001`, `ITM-COD-001`, `ITM-SALMON-001`,
`ITM-TROPICAL-FISH-001`, `ITM-PUFFERFISH-001`,
`ITM-PRISMARINE-MATERIAL-001`, `ITM-SMITHING-TEMPLATE-001`,
`ITM-ENCHANT-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PORTAL-001`, `WGEN-STRUCTURE-OCEAN-MONUMENT-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete `Guardian`,
`GuardianAttackGoal` and `ElderGuardian` paths, fatigue helper and client
packet handler, placement/category code, all 66 biomes, Ocean-Monument
construction, five direct entity tags, loot, both hostile-mob advancements,
Spawn Egg, eight migration/schema contexts, all 1,212 templates and exact
entity, beam, effect and item client resources close protocol entity ID `40`.

**Applies when:**

`minecraft:elder_guardian` is constructed, placed by an Ocean Monument,
spawned by an Egg, spawner, command or custom selector, loaded, restricted,
swimming, flopping, selecting or beaming a target, retaliating through
thorns, pulsing Mining Fatigue, killed, synchronized, heard, imitated by a
Parrot or rendered.

**Authoritative state:**

Protocol entity ID `40` constructs `ElderGuardian` in `MONSTER`, so it is
unavailable in Peaceful. Registration fixes width/height
`1.9975×1.9975`, eye height `0.99875`, passenger attachment
`(0,2.350625,0)`, client tracking range `10` and update interval `3`.
Those dimensions are the Guardian registration's `0.85×0.85`, eye
`0.425` multiplied by the runtime-derived elder scale `2.35`; the passenger
attachment is independently registered as `2.350625` rather than scaling
the Guardian attachment `0.975`.

Attributes inherit Guardian follow range `16`, then fix maximum health `80`,
movement speed `0.30000001192092896` and attack damage `8`. Construction
sets XP reward `10`, Water path malus `0`, a Guardian movement controller,
Water-Bound navigation and required persistence. It consumes one entity
`nextFloat` to seed the client tail-animation speed and changes its inherited
Random-Stroll interval from `80` to `400`.

The Monster category cap is `70`, its no-despawn/despawn distances are
`32/128`, and the inherited maximum cluster size is `4`. Required
persistence blocks distance despawn, but does not override the Peaceful
removal gate. Movement emission is `EVENTS`, gravity is `0.08`, maximum head
X/Y rotations are `180/75`, and spawn obstruction requires only that the
level report the Elder Guardian unobstructed. It is an `Enemy`, so generic
lead interaction cannot leash it; it has no age, breeding or subtype
interaction path. Generic fluid-current push remains enabled, while its
direct entity tag separately prevents Boat pushing.

Inherited Entity, Living-Entity and Mob state occupies synchronized metadata
slots `0..15`. Guardian adds slot `16`, serializer ID `8` (`BOOLEAN`),
`moving=false`, and slot `17`, serializer ID `1` (`INT`), active attack
target entity ID `0`. Neither Guardian slot persists. Client attack time and
cached target, all animation values, goal counters and targets are likewise
transient.

Generic Mob home state does persist: absent or negative `home_radius`
produces no home and leaves `home_pos` at zero; a nonnegative radius reads
`home_pos`, defaulting malformed/missing data to zero. Elder Guardian
establishes its current block position with radius `16` on its first custom
server-AI tick if no home exists, and writes both keys thereafter.

**Transition and ordering:**

### Goal graph, target selection and movement

The inherited Guardian graph is exact:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `4` | Guardian Attack, Move/Look |
| goal | `5` | Move Towards Restriction, speed `1`, Move/Look |
| goal | `7` | Random Stroll, speed `1`, Elder interval `400`, Move/Look |
| goal | `8` | Look At Player, range `8`; Look At Guardian, range `12`, probability `0.01` |
| goal | `9` | Random Look Around |
| target | `1` | nearest attackable Living Entity, random interval `10`, must see, need not reach |

The target predicate accepts only Player, Squid or Axolotl and only at
squared distance strictly above `9`. The direct graph has no Hurt-By target
goal. Goal arbitration, nearest-target enumeration, sight and navigation
retain `MOB-AI-001`.

Guardian's walk-target value is `10 + pathfindingCostFromLightLevels(pos)`
for a Water-fluid candidate and the Monster value otherwise. Each
Guardian-move tick either stops speed and clears slot `16` when not moving
or navigation is done, or normalizes the wanted-position delta and:

- rotates yaw toward `atan2(dz,dx)*57.2957763671875-90` by at most `90`
  degrees and copies it to body yaw;
- lerps speed by `0.125` toward
  `speedModifier*MOVEMENT_SPEED`;
- adds X/Z wave
  `sin((tickCount+entityId)*0.5)*0.05*(cosYaw,sinYaw)`;
- adds Y wave
  `sin((tickCount+entityId)*0.75)*0.05*(sinYaw+cosYaw)*0.25`
  plus `actualSpeed*normalizedY*0.1`;
- computes a look point two blocks forward horizontally and at
  `eyeY + normalizedY`, lerps the retained look coordinates toward it by
  `0.125`, then looks with limits `10/40`; and
- sets slot `16=true`.

Water travel applies relative input at `0.1`, moves `SELF`, scales resulting
velocity by `0.9`, and adds Y `-0.005` only while not moving and without an
attack target. Generic non-Water travel retains its owner.

### Beam attack

Guardian Attack starts from `attackTime=-10`, stops navigation, looks at a
present target with limits `90/90`, marks motion for synchronization and
then updates every tick. It remains usable while the target exists and is
alive. Unlike an ordinary Guardian, an Elder Guardian has no continuation
distance-`3` exclusion, so it can finish a beam after the target enters that
radius.

Every attack tick stops navigation, looks at the target and first checks line
of sight. Failed sight clears the Mob target before the counter changes.
Otherwise the counter increments. At counter `0`, ten ticks after start, the
server writes the target's live entity ID into slot `17`; a nonsilent
Guardian then broadcasts entity event `21`.

At counter `60`, the Elder Guardian first offers indirect-Magic damage `3`
on Easy/Normal or `5` on Hard: base `1`, plus `2` for Elder and another `2`
on Hard, with the Guardian as both direct and causing entity. It ignores
that damage result, then independently calls generic Mob attack with
attribute damage `8`, ignores that result too, and clears the Mob target.
The start-to-damage delay is therefore `70` AI ticks, while synced beam
charge lasts `60`.

Clearing the Mob target does not call goal stop inline. Normal Mob scheduling
performs full selector cleanup on the `(tickCount+entityId)` even phase and
only ticks every-tick goals on the alternate phase. A post-sync sight loss
or completed attack can therefore leave slot `17` and its client beam
projection alive for one or two more server-AI ticks. Cleanup invokes stop,
which writes active target ID `0`, redundantly clears the Mob target and
triggers Random Stroll. A target can consequently receive the second attack
even when it rejected the magic branch, and either damage pipeline can independently
change or remove the target before the other completes.

On slot-17 change, the client resets attack time and cached target. It
resolves the integer as a live entity ID once and caches it only if it is a
Living Entity. Client attack time increments to the `60`-tick cap and
partial scale is `(clientAttackTime+partialTick)/60`.

Entity event `21` constructs `GuardianAttackSoundInstance`, not the generic
entity-event handler. The hostile sound instance is looping-configured,
unattenuated and delay-free; it starts only for a nonsilent Guardian,
follows float-rounded entity coordinates, sets volume to `scale²` and pitch
to `0.7+0.5*scale`, and stops once the Guardian is removed or inherited
`Mob.getTarget()` is null. That liveness read is distinct from the
synchronized slot-17 target and its client cache, so sound lifetime must not
be inferred from visible beam lifetime.

### Defensive thorns, swimming and animation

Guardian's incoming-damage override runs before generic damage admission. If
slot `16` is false, the source is neither tagged
`avoids_guardian_thorns` nor Thorns itself, and the direct entity is Living,
that direct entity is offered `2` Thorns damage. The result is ignored.
Random Stroll is then triggered when present, and only then is the original
damage offered to `Monster.hurtServer`.

Thus thorns can commit even if the incoming hit is later rejected.
Projectile ownership alone is insufficient: the direct entity, not merely
an indirect attacker, must be Living.

Each Guardian AI step performs its client-only animation block first. A dry
client sets tail speed `2`; positive Y motion after a retained touched-ground
state plays the local flop sound if nonsilent, then touched-ground becomes
true only for downward motion above a loaded standable cell. In Water,
moving tail speed snaps to `4` below `0.5` or approaches `0.5` by `0.1`;
stationary speed approaches `0.125` by `0.2`. Tail phase adds that speed
without wrapping.

Dry spikes become one fresh entity float every client tick. In Water they
approach `0` by `0.25` while moving or `1` by `0.06` while stationary.
Moving in Water emits two local Bubble particles behind the horizontal view
vector per tick, consuming the entity's random X/Y/Z helpers for both.

With a resolved beam target, the client increments attack time, looks at it
with `90/90`, explicitly ticks Look Control, then emits local Bubble
particles along the normalized eye-to-target-midpoint ray. The first
distance is one `nextDouble`; each following step advances by
`1.8-scale + nextDouble*(1.7-scale)`.

After that client block, both logical sides reset air to `300` in Water.
Otherwise an on-ground Guardian consumes three floats, adds X/Z
`(2*nextFloat-1)*0.4` and Y `0.5`, sets yaw to
`nextFloat*360`, clears on-ground state and requests motion sync. An active
attack target finally copies head yaw to body yaw before Monster AI
continues. The direct `can_breathe_under_water` tag also exempts the entity
from generic underwater air loss.

`NoAI` makes Mob effective-AI false, so goal scheduling, navigation/control
ticks, fatigue cadence and first-home acquisition do not run. The
Guardian-local animation, Water-air reset, land flop and active-target yaw
work above precedes that inherited effective-AI gate and still runs.

### Mining Fatigue pulse and home

After inherited Guardian custom AI, a server tick for which

`(tickCount + entityId) % 1200 == 0`

constructs Mining Fatigue for `6000` ticks at amplifier `2`. The surrounding
player helper considers Server Players whose `GameType.isSurvival()` is
true—Survival or Adventure, but not Creative or Spectator—rejects allies,
and uses strict Euclidean distance below `50`.

An otherwise eligible player is selected when Mining Fatigue is absent, its
amplifier is below `2`, or its amplifier is at least `2` and it ends within
`1199` ticks. The helper clones the effect, calls `addEffect(clone, elder)`
and ignores the Boolean merge result. It returns the preselected list, so a
stronger short-lived effect can reject the weaker amplifier-2 merge while
the player still receives the following visual packet.

Every selected player receives `GUARDIAN_ELDER_EFFECT` with parameter `0`
when the source is silent and `1` otherwise. The client rounds the parameter
with `floor(value+0.5)`, always creates one Elder-Guardian particle at the
local player's position with zero requested velocity, and plays Elder
Guardian Curse at hostile volume/pitch `1/1` only for rounded value `1`.
Silence therefore suppresses the curse sound, not the effect or apparition.

Mining Fatigue is harmful effect protocol ID `3`, color `4866583`. Its
Attack-Speed modifier is identity `effect.mining_fatigue`, amount
`-0.10000000149011612`, operation `ADD_MULTIPLIED_TOTAL`. The generic
amplifier-plus-one scaling therefore creates amount
`-0.30000000447034836` for this amplifier-2 pulse. Its `18×18`, `181`-byte
icon has SHA-256
`5a40d5e5e5e94cb450aa6e7b4df99459e700a118e4dd8893606adc6b5065c4f6`.

After the cadence branch, the same custom tick establishes current
block-position home radius `16` if absent. Consequently a first tick that
also matches the cadence pulses before it acquires home.

### Production, placement and natural selection

Ocean Monuments own the intended three-Elder production graph: one in each
wing and one in the penthouse. When a fixed Elder point is inside the
current processing box, the piece creates protocol type `40` with reason
`STRUCTURE`; a nonnull instance is healed to maximum health, positioned at
the cell center with exact Y and yaw/pitch zero, finalized with local
difficulty and null group data, then offered through `addFreshEntityWithPassengers`.
The insertion result is ignored. There is no per-Elder latch, so later
reprocessing can offer duplicates; structure placement owns those calls and
failure semantics.

The server separately registers Elder Guardian placement as `IN_WATER` with
heightmap `MOTION_BLOCKING_NO_LEAVES` and the same predicate as Guardian.
The generic placement gate requires world border, candidate Water fluid and
a non-redstone-conducting block above. The Guardian predicate then consumes
`nextInt(20)` first: when the result is nonzero and the candidate can see sky
from below Water, it rejects. It next requires non-Peaceful difficulty;
Spawner reason bypasses its repeated candidate-Water test, while every other
reason requires it, and the cell below must always contain Water-tag fluid.
Obstruction is checked afterward by its owner.

All `66` locked baseline biome records contain zero
`minecraft:elder_guardian` spawn rows. Therefore the registered predicate
does not itself make Elder Guardians naturally selectable in the locked
baseline. The Monster cap `70`, distances `32/128` and cluster `4` remain
inputs for custom data that does select them, though required persistence
prevents their distance removal after insertion.

All `1,212` locked structure templates contain zero exact
`minecraft:elder_guardian` or legacy `ElderGuardian` identity. Monument
production is code-built rather than template NBT.

### Loot, tags, advancements and identity joins

The entity loot table uses random sequence
`minecraft:entities/elder_guardian` and evaluates five ordered, independent
one-roll pools:

1. Prismarine Shards receive integer-uniform count `0..2`, then the uniform
   `0..1` Looting enchanted-count increase.
2. Weighted Cod `3`, Prismarine Crystals `2` or empty `1` is selected. A
   selected item starts at one and receives the same Looting increase. Cod
   then furnace-smelts when the Elder Guardian is on fire or the direct
   attacker's main hand has an enchantment in `smelts_loot`.
3. A player kill emits exactly one Wet Sponge.
4. A player kill also tests rare chance `0.025` without Looting, or linear
   `0.035 + 0.01*(level-1)` with positive Looting. Success delegates once to
   Fishing/Fish: Cod/Salmon/Tropical Fish/Pufferfish weights
   `60/25/2/13`, then applies the same fire-or-direct-attacker smelting
   function to the selected fish.
5. Unconditionally, weighted empty `4` versus Tide Armor Trim Smithing
   Template `1` gives template probability `1/5`.

The entity supplies fixed XP `10`. It has no subtype equipment producer.
Looting count arithmetic, killed-by-player state, nested-table RNG,
smelting recipes, item merging and death ordering retain their cited
owners.

Exactly five direct entity-type tags name Elder Guardian:

- `aquatic`, which also reaches `sensitive_to_impaling` through nested tag
  membership;
- `axolotl_always_hostiles`;
- `can_breathe_under_water`;
- `cannot_be_pushed_onto_boats`; and
- `not_scary_for_pufferfish`.

Their consumers own Impaling, Axolotl targeting, breath, Boat push and
Pufferfish fear behavior. No other locked entity-type tag directly names
the identity.

Both hostile-mob advancements have an exact
`player_killed_entity` criterion for Elder Guardian. `kill_a_mob` places it
in one OR requirement group with every listed hostile; `kill_all_mobs`
places it in its own required group and awards `100` experience only after
all such groups complete.

The Spawn Egg is raw/protocol item ID `1221`, common, maximum stack `64`,
and its `entity_data.id` is `minecraft:elder_guardian`. Its generated item
model directly selects the Egg texture. English labels are “Elder Guardian”
and “Elder Guardian Spawn Egg”.

### Sounds and client projection

The eight Elder sound events are:

| Protocol ID | Event | Locked clips and subtitle |
|---:|---|---|
| `573` | ambient in Water | four `elder_idle` clips; “Elder Guardian moans” |
| `574` | ambient on land | four shared Guardian `land_idle` clips; “Elder Guardian flaps” |
| `575` | curse | one `curse` clip; “Elder Guardian curses” |
| `576` | death in Water | one `elder_death` clip; “Elder Guardian dies” |
| `577` | death on land | one shared `land_death` clip; same death subtitle |
| `578` | flop | four `flop` clips; “Elder Guardian flops” |
| `579` | hurt in Water | four `elder_hit` clips; “Elder Guardian hurts” |
| `580` | hurt on land | four shared `land_hit` clips; same hurt subtitle |

Ambient cadence is inherited `160`. The beam sound is Guardian Attack event
ID `769`, one `attack_loop` clip with subtitle “Guardian shoots”. Parrot
imitation maps this entity type to sound event ID `1221`, which references
Elder Guardian land ambient at volume `0.7`, pitch `1.8`, subtitle
“Parrot moans”. Parrot's own once-per-400-AI-tick attempt, nearby selection,
silence gate and playback retain the Parrot owner.

`ElderGuardianRenderer` uses shadow radius `1.2`,
`ModelLayers.ELDER_GUARDIAN` and
`textures/entity/guardian/guardian_elder.png`. The `64×64`, `965`-byte
texture has SHA-256
`6aa3da2530a53c4c9d1a63ada991773e56260090d57bee5aad643f73fc11739a`.
Its layer is the Guardian body layer transformed uniformly by `2.35`.

Guardian render state carries spike/tail animation, eye and look positions,
beam target position, attack time and scale. The renderer keeps an otherwise
culled Elder Guardian visible when the AABB spanning its eye and the target
midpoint intersects the frustum. It renders the beam from eye height through
the shared `32×32`, `246`-byte texture whose SHA-256 is
`408a473ce3ed9130e4229ffef4817d4ae8888fc0b268f33c479073578d0d9a90`;
beam geometry and ordinary lighting remain the Guardian renderer's owner.

The Guardian model rotates head degrees to radians, drives twelve spikes by
`(1-spikesAnimation)*0.55`, points the eye toward the active target or
camera, and rotates the three tail sections by
`sin(tail)*pi*(0.05,0.1,0.15)`. Eye Y is `0` when the look target is above
and `1` otherwise; horizontal eye X is
`2*sqrt(abs(sideDot))*sign(sideDot)`.

Elder-Guardian particle protocol ID `24` ignores requested velocity, has
gravity `0` and lifetime `30`, and renders the translucent Elder texture
with the same baked scaled model. At progress
`q=(age+partialTick)/30`, alpha is `0.05+0.5*sin(q*pi)` with white RGB.
The camera-facing model rotates X by `60-150*q` degrees, scales
`(0.42553192,-0.42553192,-0.42553192)` and translates
`(0,-0.56,3.5)`. This is a camera apparition, not a world entity.
The particle type is registered with `overrideLimiter=true`, so it bypasses
the ordinary distance and particle-setting limiter before joining the
dedicated `ELDER_GUARDIANS` queue. Queue admission is unconditional below
`12,288`, then uses probability
`((16,384-currentSize)/4,096)^2`, and rejects at `16,384`. Render-state
extraction maps every queued apparition and does not use its frustum
argument.

The Spawn Egg's `16×16`, `257`-byte texture has SHA-256
`7235534d20e0eb19d0954e43adbd8bdc5c9d999999a930378a6c7b64b30bb3ba`.

### Migration and schema closure

Eight exact migration/schema contexts name the identity:

- schema `V700` registers legacy simple entity `ElderGuardian`;
- `EntityElderGuardianSplitFix`, installed at version `700`, rewrites an
  entity named `Guardian` with Boolean `Elder=true` to
  `ElderGuardian` without removing the old Boolean;
- `EntityIdFix` maps `ElderGuardian` to
  `minecraft:elder_guardian`;
- schema `V705` registers the modern Mob shape and exact Spawn-Egg mapping;
- schema `V1460` registers the current Mob shape;
- `EntityUUIDFix` includes the current entity in its Mob UUID rewrite set;
- `ItemStackSpawnEggFix` maps current entity identity to the Elder Guardian
  Spawn Egg; and
- `StatsCounterFix` maps the legacy/current statistics identity.

The runtime ignores the retained legacy `Elder` key. No Elder-specific
metadata or attack/fatigue counter is persisted. Generic health, effects,
equipment, air, persistence-required state and the Mob home pair retain
their schema and runtime owners.

**Branches and aborts:**

- Goal admission aborts without a live target; sight loss clears it before
  beam-counter mutation, while an Elder target entering range `3` does not
  abort continuation.
- Slot-17 publication and nonsilent event `21` branch at counter `0`;
  independent magic/melee offers branch at counter `60`, then Mob-target
  clear awaits the next full selector cleanup before stop clears slot `17`.
- Thorns aborts on moving, excluded source tags/types or a non-Living direct
  entity, but none of those branches skips the later stroll trigger or
  generic incoming-damage call.
- Fatigue aborts outside the modulo tick, then per player on game mode,
  alliance, strict radius or sufficient retained effect. Packet emission
  follows helper selection rather than effect-merge success.
- Monument placement aborts per site outside the processing box or after
  null creation; placement admission aborts at border, generic Water/above,
  sky RNG, difficulty, repeated Water/below Water or obstruction gates.
- Loot pools abort independently on their own conditions; no failed
  player-kill or rare branch suppresses unconditional template selection.
- Missing/wrong client target IDs suppress beam target projection; silence
  separately suppresses beam/curse sounds.

**Invariants:**

- Slot `17=0` means no client beam target; a nonzero ID is transient and
  resolved only in the receiving level.
- Beam visibility begins ten ticks after goal start and damage occurs sixty
  ticks after that metadata write.
- Elder continuation deliberately lacks the ordinary Guardian's
  distance-squared-above-`9` condition.
- Thorns precede incoming damage admission and test the direct entity.
- Fatigue cadence is phase-shifted by the runtime entity ID.
- The helper's packet-recipient decision precedes and is independent of the
  effect merge result.
- Silence changes the curse/beam sounds, not target, damage, effect or
  apparition; beam-sound liveness separately reads Mob target rather than
  slot `17`.
- Locked baseline natural spawn selection is empty; Monument code is the
  intended baseline producer.
- The five loot pools remain independent and ordered.

**Constants and randomness:**

Entity/Egg/effect/particle IDs `40/1221/3/24`; dimensions/eye/attachment
`1.9975×1.9975/0.99875/2.350625`; range/update `10/3`;
health/speed/attack/follow `80/0.30000001192092896/8/16`; XP `10`;
slots `16 BOOLEAN/17 INT`; home `16`; stroll `400`; target interval `10`,
distance squared `>9`; attack `-10/0/60`, damage `3/5 + 8`, event `21`;
thorns `2`; water air `300`; fatigue interval/radius/duration/amplifier/
display limit `1200/50/6000/2/1200`; Monster cap/distances/cluster
`70/32/128/4`; placement sky chance `1/20`; loot shard `0..2`, common
weights `3/2/1`, rare `0.025/0.035/+0.01`, fish `60/25/2/13`, template
`1/5`; tags/templates `5/0 of 1212`; ambient `160`; apparition reservoir/
cap `12288/16384`; shadow `1.2`.

**Side effects:**

Metadata, home and persistence state; RNG cursors; targets, goal arbitration,
navigation, movement, look/yaw and velocity; thorns, magic and melee damage;
effects and attribute modifiers; per-player game-event packets; sounds,
entity events and local particles; structure and custom spawn insertion;
loot/XP, advancement progress and item stacks; renderer cache, frustum,
model and beam state.

**Gates:**

Logical side, Peaceful and required persistence; target class/liveness/
distance/sight; goal priority/flags; Water/navigation/moving/ground/
loaded support; incoming damage tag/type/direct entity; pulse modulo,
Survival-or-Adventure classification, alliance, strict distance and existing
effect; processing box/create/insertion; border/fluid/conductor/sky RNG/spawn
reason/difficulty/obstruction; absent biome selection; player kill, Looting,
on-fire/direct attacker enchantment and random loot; silence, metadata
resolution, camera and frustum; migrations and resource identity.

**Boundary cases and quirks:**

The beam can finish inside three blocks, but target acquisition still
requires strictly greater distance. Magic and melee are separate
nontransactional offers. Thorns can hurt an attacker whose own damage is
rejected. Beam metadata can outlive sight loss or completed damage by one or
two server-AI ticks, while sound liveness independently reads Mob target. A
non-done zero-length move target divides without a local zero guard. A
stronger short-lived Mining Fatigue can yield apparition without effect
replacement. Home is acquired after any
same-tick pulse. Sky-visible placement consumes RNG before difficulty/fluid
checks. Placement registration does not imply baseline natural production.
Monument reprocessing has no per-Elder latch. Tide-template chance is
unconditional on player kill.

**Failure semantics:**

Lost beam sight clears the target without damage; either damage result is
ignored. Rejected thorns does not prevent incoming-damage evaluation.
Rejected effect merge does not retract the game-event packet. Null Monument
creation performs no later step; rejected entity insertion is not rolled
back and has no latch. Rejected placement prevents natural insertion.
Nested loot-table and item insertion behavior retain their owners.

**Client/server authority split:**

The server owns home/persistence, targets, goal scheduling, navigation,
beam timing and damage, thorns, fatigue selection/effect/packets, structure
and spawn insertion, loot/XP and advancement criteria. Clients consume
metadata, movement, entity/game events and resources; they animate tail,
spikes and eye, emit movement/beam bubbles, evaluate the looping-configured
beam sound, display the fatigue apparition and render the entity/beam/Egg.
Client cache or resource state cannot alter server authority.

**Observability:**

Observe registration/scale/attributes and both metadata slots; constructor
RNG and persistence; the complete goal/control graph; every beam
start/sync/sight/damage/stop outcome; thorns-before-damage ordering; water
travel, flop and client animation RNG; cadence/player/effect/packet/home
boundaries; three Monument positions and reprocessing; registered placement
versus zero biome rows; all loot pools/tags/advancements/Egg; migrations and
zero-template census; sound/Parrot mapping, particle apparition, model,
texture, beam and frustum projection.

**Persistence and reload:**

Generic entity/Mob state plus required persistence and acquired
`home_radius/home_pos` persist. Moving/beam slots, attack/fatigue cadence,
targets, goal counters, client animation/cache and renderer state do not.
Because runtime entity ID does not persist, reload can rephase the
`tickCount+entityId` fatigue schedule.
Code fixes registration, goals, pulse and migrations. Loot, tags, biomes and
advancements reload through their owners; Monument structure code remains
fixed. Language, sounds, item models and textures reload client-side.

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
`net.minecraft.world.entity.monster.ElderGuardian`;
`net.minecraft.world.entity.ai.control.MoveControl`;
`net.minecraft.world.entity.ai.goal.MoveTowardsRestrictionGoal`;
`net.minecraft.world.entity.ai.goal.RandomStrollGoal`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.effect.MobEffectUtil`;
`net.minecraft.world.level.NaturalSpawner`;
`net.minecraft.world.level.levelgen.structure.structures.OceanMonumentPieces$OceanMonumentPiece`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.sounds.SoundEvents`;
`net.minecraft.util.datafix.fixes.EntityElderGuardianSplitFix`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.fixes.StatsCounterFix`;
`net.minecraft.util.datafix.schemas.V700`, `V705` and `V1460`;
`net.minecraft.client.multiplayer.ClientPacketListener`;
`net.minecraft.client.resources.sounds.GuardianAttackSoundInstance`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.ElderGuardianRenderer`;
`net.minecraft.client.renderer.entity.GuardianRenderer`;
`net.minecraft.client.renderer.entity.state.GuardianRenderState`;
`net.minecraft.client.model.monster.guardian.GuardianModel`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`net.minecraft.client.particle.ElderGuardianParticle`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,particle_type,mob_effect}`;
`reports/minecraft/components/item/elder_guardian_spawn_egg.json`;
`data/minecraft/tags/entity_type/{aquatic,axolotl_always_hostiles,can_breathe_under_water,cannot_be_pushed_onto_boats,not_scary_for_pufferfish,sensitive_to_impaling}.json`;
`data/minecraft/loot_table/entities/elder_guardian.json`;
`data/minecraft/loot_table/gameplay/fishing/fish.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/elder_guardian_spawn_egg.*`;
`assets/minecraft/textures/entity/guardian/{guardian_elder,guardian_beam}.png`;
`assets/minecraft/textures/mob_effect/mining_fatigue.png`;
`assets/minecraft/{sounds,lang/en_us}.json`;
`ENT-DAMAGE-001`; `ENT-EFFECT-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-SPAWN-001`;
`MOB-DESPAWN-001`; `BLK-SPONGE-001`; `ITM-COD-001`;
`ITM-SALMON-001`; `ITM-TROPICAL-FISH-001`; `ITM-PUFFERFISH-001`;
`ITM-PRISMARINE-MATERIAL-001`; `ITM-SMITHING-TEMPLATE-001`;
`ITM-ENCHANT-001`; `WGEN-STRUCTURE-OCEAN-MONUMENT-001`;
`CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-022` across construction/home/persistence, every goal and
movement-controller boundary, beam timing/sight/distance/damage/stop state,
thorns and generic damage results, water/dry/client animation RNG,
fatigue cadence/player/effect/packet/silence cases, all three Monument
positions and reprocessing, placement/66-biome/cap/despawn cases, five loot
pools/XP/tags/advancements/Egg, templates/eight migrations/sounds/Parrot and
exact particle/model/texture/beam/frustum projection.

**Limits:**

Generic lifecycle, metadata, home codec, goal arbitration/path search,
damage/effect/death, natural spawning/despawn, structure processing, loot
evaluation, advancement triggers, Spawn Egg interaction, sounds, particles
and rendering retain their cited owners. Guardian-common movement, attack
and model algorithms are included only where the Elder subtype selects,
scales or changes their exact inputs and observable result.
