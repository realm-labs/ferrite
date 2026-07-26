# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-GHAST-001` — Ghasts float through collision sweeps, charge large fireballs and admit reflected-fireball kills

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`,
`MOB-002`, `MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`,
`MOB-005`, `MOB-BREED-001`, `ITM-GHAST-TEAR-001`,
`ITM-GUNPOWDER-001`, `ITM-JUKEBOX-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`,
`WGEN-STRUCTURE-FORTRESS-001`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Ghast` class and
four nested AI/control classes, Large-Fireball construction and reflected
damage join, all 66 biomes, the sole direct entity tag, three-pool loot,
five advancement records, Wolf exclusion, Spawn Egg, nine migration/schema
classes and exact model/texture/sound resources close protocol entity ID
`57`.

**Applies when:**

`minecraft:ghast` is constructed, moved, given or deprived of a Player
target, charging or firing, hit by an ordinary or reflected projectile,
spawned naturally or through a generic producer, saved, loaded, leashed
through custom state, used as a leash holder, killed, observed through a
Spyglass, heard, imitated by a Parrot, synchronized or rendered.

**Authoritative state:**

Protocol entity ID `57` constructs `Ghast` in `MONSTER`. Registration is
fire-immune and unavailable in Peaceful, fixes dimensions `4×4`, eye
height `2.6`, one passenger attachment at `(0,4.0625,0)`, riding-offset
argument `0.5`, client tracking range `10` and default update interval
`3`.

The explicit attributes are maximum health `10`, follow range `100`,
camera distance `8` and flying speed `0.06`; other registered Living/Mob
attributes retain their defaults. Construction sets XP reward `5` and
explosion power `1`. Ghast is an `Enemy`, so ordinary lead interaction
cannot leash it and it has no breeding, age, equipment, melee or direct
item-interaction path.

The Monster cap is `70`, no-despawn/despawn distances are `32/128`, and
Ghast overrides maximum spawn cluster size to `1`. It uses Hostile sound
source with local voice volume `5`. Movement emission, persistence,
generic obstruction and generic distance removal retain their owners.

Inherited Entity, Living-Entity and Mob state occupies synchronized slots
`0..15`. Ghast adds slot `16`, serializer ID `8` (`BOOLEAN`),
`charging=false`. `ExplosionPower` persists as a byte and is loaded into
an integer with default `1`; values are not clamped, so the authoritative
loaded range is signed byte `-128..127`. Charging metadata, target,
attack charge/cooldown, float-controller countdown and wanted position do
not persist.

Ghast has three independent fall/movement declarations: entity-type
`fireImmune`, direct membership in `fall_damage_immune`, and an empty
`checkFallDamage` override. `onClimbable()` is always false. Its travel
path calls flying travel with input acceleration `0.02`, moves by current
velocity, then scales velocity by `0.800000011920929` in Water, `0.5` in Lava or
`0.9100000262260437` otherwise, without the ordinary gravity branch.

Ghast cannot normally be the leashed subject, but it advertises
quad-leash support when it is another entity's holder. Its own forced or
custom leash state uses elastic distance `10` and snap distance `16`
instead of generic `6/12`.

**Transition and ordering:**

### Complete goal and target graph

| Selector | Priority | Goal and exact configuration |
|---|---:|---|
| goal | `5` | Random Float Around; Move flag |
| goal | `7` | Ghast Look; Look flag, every tick |
| goal | `7` | Shoot Fireball; no flags, every tick |
| target | `1` | nearest Player; random interval `10`, must see, need not reach, acquisition `abs(playerY-ghastY)<=4` |

The target search uses follow range `100`, combat targeting and ordinary
Player visibility/alliance gates. Its vertical selector applies to
acquisition. Continuation instead uses the generic target goal: live/
attackable/nonallied target, range at most `100`, and nominal unseen
memory `60`; it does not reapply the four-block vertical selector.

Random Float starts when Move Control has no wanted point, or the current
wanted displacement has squared length below `1` or above `3600`. It
cannot continue as an active goal: each start only assigns a new
destination at speed parameter `1`, while Move Control retains that
destination afterward.

The default distance-to-blocks argument is zero. Position selection draws
X, Y and Z independently as

`current + (nextFloat()*2-1)*16`.

Without a home, the first candidate is accepted. With a home, at most
`64` candidates are drawn until one is within home; exhausting only null
home-rejected candidates falls back to one unrestricted three-float
candidate. The zero distance-to-blocks argument makes the surrounding-air
test succeed without block reads.

After selection, the goal reads `MOTION_BLOCKING` height at candidate
X/Z. When that height is below candidate block Y but above level minimum, it
replaces candidate Y with
`currentY-abs(currentY-candidateY)`. An upward candidate is thereby
reflected downward by the same delta; an already-downward candidate is
unchanged. Other height relations retain the original Y.

The Look goal always runs. With no target it sets entity and body yaw to
`-atan2(deltaX,deltaZ)*57.295776`. With a target at squared distance
strictly below `4096`, it instead faces the target's horizontal delta.
A retained target at exactly or beyond `64` blocks suppresses both
branches and leaves yaw unchanged.

No goal uses navigation. The random Move goal, Look goal and flagless
Shoot goal can run concurrently. `NoAI` suppresses selector and control
updates; it therefore starts no new destination or attack cycle and can
leave charging metadata/wanted movement state frozen until effective AI
resumes.

### Float movement controller

Construction installs `GhastMoveControl` with `careful=false` and an
always-false stop supplier. On each `MOVE_TO` controller tick it
postdecrements `floatDuration`. A nonpositive old value adds
`nextInt(5)+2`, giving a countdown `2..6` and evaluation gaps `3..7`
controller ticks.

At an admitted evaluation, let `d=wanted-position`. The controller adds

`normalize(d) * flyingSpeed * 5/3`

to current velocity: with the default attribute this is magnitude `0.1`.
It does not use the Move goal's speed parameter. If the full displacement
fails its block-intersection sweep, operation becomes `WAIT` and no
acceleration is added.

The sweep admits Air, empty collision shapes and a noncolliding swept
shape. Because ordinary Ghast uses `careful=false`, it skips the
inflated-destination precheck and returns after collision evaluation:
`happy_ghast_avoids`, Water/Lava continuity and other fluid state do not
gate this subtype's movement. Those later checks belong only to careful
users of the shared controller.

### Fireball charge and launch

The flagless Shoot goal can start whenever `getTarget()` is nonnull and
resets `chargeTime=0`. Each every-tick update distinguishes a valid firing
window:

`target distance squared < 4096 && hasLineOfSight(target)`.

Inside that window, it increments charge first. At `10`, a nonsilent
Ghast broadcasts level event `1015` at its block. At `20`, it optionally
broadcasts level event `1016`, creates one protocol entity `52`
(`minecraft:fireball`), offers it to the level with ignored insertion
result and sets charge to `-40`.

Outside the firing window, a positive charge decrements by one. Zero and
negative values do not change, so losing sight/range freezes the
post-shot negative cooldown but drains an incomplete positive charge
toward zero. Target-goal removal stops the Shoot goal, clears slot `16`,
and a later start resets charge to zero.

After every update, slot `16` is true exactly when `chargeTime>10`.
Consequently warning event `1015` occurs while the normal texture remains
selected; ticks `11..19` select the shooting texture; the shot tick resets
the byte to false. During an invalid positive tail the texture remains
charged only down through `11`.

For a shot, view vector `v` is sampled at partial tick `1`. The launch
point is four blocks ahead horizontally:

`(ghastX+4*vX, ghastY(0.5)+0.5, ghastZ+4*vZ)`.

Its aim vector is target X/Z and target `getY(0.5)` minus that launch
point, normalized by the projectile constructor. The new Large Fireball
owns this Ghast and copies the Ghast's current signed `ExplosionPower`;
later changes affect only later shots. Generic projectile flight,
deflection, entity damage `6`, enchantment post-attack effects,
mobGriefing-selected block destruction, explosion radius and persistence
remain `ENT-PROJECTILE-001`.

Client level events `1015/1016` play Ghast Warn/Shoot locally in Hostile
source at volume `10` and pitch
`1+(nextFloat()-nextFloat())*0.2`. Silence prevents the server event, so
it also prevents these client draws and sounds.

### Reflected-fireball damage

A reflected fireball is defined narrowly: the direct damage entity is a
`LargeFireball` and the causing entity is a `Player`. The Fireball damage
type is both `is_fire` and `is_projectile`. Ghast's invulnerability hook
exempts this exact source from inherited fire immunity and other generic
Mob immunity, while the explicit invulnerable entity flag still retains
its ordinary gate unless the source bypasses it.

`hurtServer` tests this source first and offers damage `1000` through the
base Mob transaction, ignores that transaction's boolean and returns true.
All other sources run the explicit invulnerability check and then ordinary
Mob damage. Thus a normal Player-reflected Large Fireball overwhelmingly
kills a non-invulnerable Ghast and credits the Player; the special return
value alone does not prove health changed.

Deflection, owner reassignment and fireball motion remain projectile
owners. A Large Fireball attributed to a non-Player does not take the
special branch, while a Player-owned projectile of another class is also
ordinary damage.

### Production and placement

The explicit placement record is `ON_GROUND`,
`MOTION_BLOCKING_NO_LEAVES`, with `checkGhastSpawnRules`. In Peaceful it
returns false without a Ghast-local draw. Otherwise it consumes
`nextInt(20)` first and admits only zero, then evaluates generic Mob spawn
rules. Medium/support adjustment, collision/obstruction, player distance,
caps and biome selection retain `MOB-SPAWN-001`.

Exactly three of the 66 locked baseline biomes select Ghast:

| Biome | Monster weight | Declared group | Spawn cost |
|---|---:|---:|---|
| Basalt Deltas | `40` | `1..1` | none |
| Nether Wastes | `50` | `4..4` | none |
| Soul Sand Valley | `50` | `4..4` | charge `0.7`, energy budget `0.15` |

The fixed-four rows can offer four candidate attempts in one selected
group, but the first accepted Ghast reaches subtype cluster size `1` and
ends all three pack walks. A valid Nether-Fortress spawn override contains
no Ghast row, so it replaces rather than augments these biome lists above
Nether Bricks.

No structure producer creates Ghasts. Exact UTF scanning of all `1,212`
locked structure templates finds zero current `minecraft:ghast` and zero
legacy `Ghast` entity identity. Spawn Eggs, spawners, commands and custom
data remain generic producers; Dried-Ghast hydration produces a Ghastling/
Happy Ghast, not this hostile identity.

### Loot, cross-entity consumers and advancement joins

The loot table uses random sequence `minecraft:entities/ghast` and
evaluates three ordered one-roll pools:

1. Ghast Tear item ID `1146`, integer-uniform base `0..1`, then Looting
   enchanted-count uniform `0..1`;
2. Gunpowder item ID `978`, integer-uniform base `0..2`, then the same
   Looting increase; and
3. exactly one Music Disc Tears item ID `1360` only when the direct damage
   entity is protocol fireball `52`, its damage type is `is_projectile`,
   and a Player owns kill credit.

The first two pools have no player-kill gate; zero counts are removed only
by generic empty-stack filtering and a positive Looting bonus can revive a
zero base. The disc conditions are independent of the reflected-damage
implementation, though the ordinary Player-reflected kill satisfies their
shape. XP is fixed `5`.

Exactly one direct entity-type tag names Ghast:
`fall_damage_immune`. No aquatic, raider, illager, undead or other locked
entity tag directly includes it.

Tamed-Wolf owner retaliation has an explicit class exclusion: a Wolf's
`wantsToAttack` returns false for any Ghast, as it also does for Creepers
and Armor Stands. This does not make Ghast allied and does not stop
unrelated targeting or direct Player attacks.

Five advancement records select Ghast:

- `adventure/kill_a_mob` includes its exact Player-kill criterion in the
  shared OR group;
- `adventure/kill_all_mobs` requires its exact criterion independently and
  awards `100` XP only after the full hostile set;
- `adventure/spyglass_at_ghast` requires active Spyglass use while the
  Player type-specific looking-at predicate resolves a Ghast;
- `nether/return_to_sender` requires a Player-killed Ghast whose killing
  blow has direct fireball entity and `is_projectile`, awarding `50` XP;
  and
- `nether/uneasy_alliance` requires a Player-killed Ghast whose entity
  location is in the Overworld, awarding `100` XP. It does not constrain
  the Player's dimension separately.

### Migration and schema closure

Exactly nine migration/schema classes select Ghast identity:

- `EntityHealthFix` recognizes legacy `Ghast`;
- `EntityIdFix` maps `Ghast` to `minecraft:ghast`;
- `EntityUUIDFix` includes current Ghast in the Mob UUID rewrite set;
- `ItemSpawnEggFix` maps legacy generic Spawn Egg damage `56` to `Ghast`;
- `ItemStackSpawnEggFix` maps current Ghast to
  `minecraft:ghast_spawn_egg`;
- `StatsCounterFix` recognizes legacy Ghast statistics;
- schema `V99` registers the legacy simple entity; and
- schemas `V705` and `V1460` register the modern Mob/Spawn-Egg shapes.

The legacy Egg damage `56` is unrelated to current protocol entity ID
`57`. No fix clamps or rewrites `ExplosionPower`; its missing/default and
signed-byte behavior is authoritative. Ghast Tear and Music Disc item
migrations retain their item owners.

### Sound and client projection

The six registered Ghast sound-event IDs are:

| ID | Event | Direct Ghast path |
|---:|---|---|
| `705` | Ambient | inherited ambient cadence, subtitle “Ghast cries” |
| `706` | Death | admitted death, “Ghast dies” |
| `707` | Hurt | admitted hurt, “Ghast hurts” |
| `708` | Scream | no direct call in the locked Ghast classes |
| `709` | Shoot | client level event `1016`, “Ghast shoots” |
| `710` | Warn | client level event `1015`, shoot subtitle family |

Parrot imitation maps Ghast to event ID `1225`, subtitle “Parrot cries”.
Parrot cadence, nearby selection and playback retain their owner.

`EntityRenderers` binds Ghast to `GhastRenderer`, model layer `GHAST` and
shadow radius `1.5`. Slot `16=false` selects
`textures/entity/ghast/ghast.png`; true selects
`ghast_shooting.png`. Both are `128×64`: the ordinary texture is
`1,045` bytes with SHA-256
`50bb277c7bd76b95501845b30d4245bd51c8169e86b819edf47627efe104bd4a`,
and shooting is `1,188` bytes with SHA-256
`dc8e796a7df956f6cd559ec739c01c4c550776a297b18ba5162614ba7046ac56`.

The model bakes a `64×32` logical atlas and scales the layer by `4.5`. Its
body is one `16×16×16` cube rooted at Y `17.6`. Nine `2×L×2`
tentacles root at Y `24.6` in the staggered three-by-three layout; fixed
Single-Threaded-Random seed `1660` gives lengths
`8,13,9,11,11,10,12,9,12` for indices `0..8`. Every frame tentacle `i`
sets

`xRot = 0.2*sin(ageInTicks*0.3+i)+0.4`.

The Spawn Egg is common, maximum stack `64`, raw/protocol item ID `1234`,
and has `entity_data.id=minecraft:ghast`. Its generated model directly
selects a `16×16`, `193`-byte texture with SHA-256
`bde96f5d5a5af12c18cafcb4ae99e0e0733f0588091ae2c0be73f510a2e67e91`.
English names are “Ghast” and “Ghast Spawn Egg”.

**Branches and aborts:**

- Float reassignment branches on absent, nearer-than-one or farther-than-60
  wanted displacement; home rejection can consume up to 64 candidate
  triples before unrestricted fallback.
- Move acceleration waits for its countdown and aborts to `WAIT` on a
  rejected collision sweep.
- Player acquisition consumes `nextInt(10)` before search and requires
  vertical difference at most four; continuation uses different gates.
- Firing increments only under strict distance `64` plus line of sight;
  invalid positive charge drains, while invalid nonpositive cooldown
  freezes.
- Silence suppresses both level events; shot construction and cooldown
  still occur.
- Reflected damage requires both exact Large-Fireball class and Player
  causing entity; its fixed return ignores the underlying damage result.
- Natural admission consumes its `1/20` draw before generic rules only
  outside Peaceful.
- Loot Tear/Gunpowder pools are unconditional after table admission; Disc
  requires projectile-shaped direct fireball plus Player kill.

**Invariants:**

- The only persistent Ghast-local value is signed-byte
  `ExplosionPower`.
- All three ordinary goals can coexist because their control flags do not
  conflict.
- A retained target from `64..100` blocks prevents movement-facing but
  cannot advance firing charge.
- Charge texture is true only for counter values strictly above `10`.
- Each successful fire cycle creates at most one owned Large Fireball and
  resets counter to `-40` regardless of insertion success.
- Player-reflected Large Fireballs bypass Ghast fire immunity and are
  offered as `1000` damage.
- At most one Ghast can succeed in a natural pack despite fixed-four biome
  group records.

**Constants and randomness:**

Entity/Egg/fireball IDs `57/1234/52`; dimensions/eye/passenger/riding
`4×4/2.6/4.0625/0.5`; tracking/update `10/3`; health/follow/camera/flying
`10/100/8/0.06`; XP `5`; metadata `16 BOOLEAN`; explosion default `1`;
float position `[-16,16)` per axis and attempts `64`; wanted squared
thresholds `1/3600`; control countdown `nextInt(5)+2`, acceleration `0.1`;
target interval/vertical/range `10/4/100`; fire range/charge/warn/shot/
reset `64/10/20/-40`; launch offset `4`; reflected damage `1000`;
travel damping `0.800000011920929/0.5/0.9100000262260437`;
spawn `nextInt(20)==0`, cluster `1`; cap/distances `70/32/128`; biome
weights/groups `40@1,50@4,50@4`; Soul cost `0.7/0.15`; loot bases
Tear `0..1`, Gunpowder `0..2`, Disc `1`; voice/event volume `5/10`.

**Side effects:**

Metadata, target and goal/control state; RNG cursors; yaw, wanted position
and velocity; level events and client sounds; owned Large Fireball
construction/insertion/explosion; damage, death, loot/XP and advancement
progress; generic leash forces; client texture/model state.

**Gates:**

Logical side, Peaceful, NoAI and silence; target RNG, Player status,
vertical/range/sight/alliance; goal flags and selector cadence; wanted
distance, home and collision sweep; charge counter and strict 64-block
window; exact projectile class/owner and invulnerability; placement,
biome/fortress list, spawn cost, cap/cluster/player distance; loot context,
Looting, direct entity/damage tags and Player credit; dimension, using
item and looking-at predicates; metadata and resources.

**Boundary cases and quirks:**

At exactly four vertical blocks Player acquisition is allowed; at exactly
64 blocks aiming/firing is not, while target continuation can remain
through exactly 100. A target beyond firing range freezes negative
cooldown and yaw, but drains a positive partial charge. Event `1015`
precedes the charged texture by one tick. A silent Ghast still launches
and explodes fireballs. Signed negative Explosion Power is passed
unvalidated to future Large Fireballs. The reflected branch returns true
even when the underlying base damage transaction changes no health.
Biome group size four affects failed attempts, but cluster one ends all
walks on the first success. Forced leash data can exercise `10/16`
distances even though ordinary Player lead admission rejects Enemy.

**Failure semantics:**

No home-valid candidate after 64 triples falls back outside home. A blocked
movement sweep drops the wanted operation to `WAIT`. Invalid sight/range
does not stop the Shoot goal while a target reference remains. Fireball
insertion failure does not restore charge or undo level event `1016`.
Ordinary rejected damage returns false; reflected damage returns true
after an ignored-result base offer. Spawn failure reserves no subtype
state before natural-spawner accounting. Zero-count loot emits nothing,
while failed advancement predicates do not roll back death or loot.

**Client/server authority split:**

The server owns target selection, goals, movement acceleration,
charge counter/metadata, fireball construction, damage, spawning,
loot/XP and advancement progress. Clients consume slot `16`, movement,
level events and resources; they choose the two textures, animate
tentacles and play randomized Warn/Shoot sounds. Client visuals, stale
charging state or sounds cannot create a projectile or commit damage.

**Observability:**

Observe registration/attributes, slot `16`, signed NBT and reload;
goal/control sets, destination draws, height reflection and collision
sweep; target acquisition/continuation, yaw and every charge/range/sight/
silence edge; fireball owner/position/direction/power/insertion and
reflected damage result; exact three-biome selection, Soul cost,
fixed-four attempts and cluster-one termination; loot/XP/tag/Wolf and
five advancement joins; Egg, templates, nine migrations, sounds,
textures, tentacle geometry and animation.

**Persistence and reload:**

Generic Entity/Mob state and byte `ExplosionPower` save. Charging byte,
target, charge/cooldown, goals, controller countdown, wanted position and
client state do not. Reload therefore begins uncharged with attack start
resetting to zero, while future shots inherit the signed loaded power.
Already-created Large Fireballs persist their own owner, movement and
power independently. Biome, loot, tag and advancement data reload through
their owners; registration/AI code does not. Models, textures and
language reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.monster.Ghast`;
`net.minecraft.world.entity.monster.Ghast$RandomFloatAroundGoal`;
`net.minecraft.world.entity.monster.Ghast$GhastMoveControl`;
`net.minecraft.world.entity.monster.Ghast$GhastLookGoal`;
`net.minecraft.world.entity.monster.Ghast$GhastShootFireballGoal`;
`net.minecraft.world.entity.projectile.hurtingprojectile.LargeFireball`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.entity.ai.goal.target.TargetGoal`;
`net.minecraft.world.entity.animal.wolf.Wolf`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.client.renderer.LevelEventHandler`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.GhastRenderer`;
`net.minecraft.client.renderer.entity.state.GhastRenderState`;
`net.minecraft.client.model.monster.ghast.GhastModel`;
`net.minecraft.util.datafix.fixes.EntityHealthFix`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.ItemSpawnEggFix`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.fixes.StatsCounterFix`;
`net.minecraft.util.datafix.schemas.V99`, `V705` and `V1460`;
`reports/registries.json#minecraft:{entity_type,item,sound_event}`;
`reports/minecraft/components/item/ghast_spawn_egg.json`;
`data/minecraft/tags/entity_type/fall_damage_immune.json`;
`data/minecraft/loot_table/entities/ghast.json`;
`data/minecraft/worldgen/biome/{basalt_deltas,nether_wastes,soul_sand_valley}.json`;
`data/minecraft/advancement/{adventure/{kill_a_mob,kill_all_mobs,spyglass_at_ghast},nether/{return_to_sender,uneasy_alliance}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/ghast_spawn_egg.*`;
`assets/minecraft/textures/entity/ghast/{ghast,ghast_shooting}.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-PROJECTILE-001`; `ENT-DAMAGE-001`; `ENT-DEATH-001`;
`MOB-AI-001`; `MOB-SPAWN-001`; `MOB-DESPAWN-001`;
`ITM-GHAST-TEAR-001`; `ITM-GUNPOWDER-001`; `ITM-JUKEBOX-001`;
`WGEN-STRUCTURE-FORTRESS-001`; `CLI-006`.

**Test vectors:**

Run `EXP-ENT-024` across construction/fire/fall/travel/leash/metadata/NBT/
reload; all goal flags, selector phases, target acquisition/continuation
and NoAI; destination/home/RNG/height/collision/countdown/velocity cases;
all charge/range/sight/silence/event/fireball create/position/power/
insertion paths; ordinary, reflected, invulnerable and failed damage;
placement/three-biome/fortress/cost/group/cluster/cap/despawn cases; three
loot pools, XP/tag/Wolf/five advancements/Egg/templates/nine migrations;
sounds/Parrot and exact texture/model/tentacle projection.

**Limits:**

Generic lifecycle, metadata, targeting, scheduler, collision primitives,
damage/death, natural spawning, biome potential, despawn, leash physics,
Large-Fireball flight/deflection/hit/explosion, loot, items, advancements,
Spawn Egg interaction, Wolf ownership, Parrot imitation and rendering
retain their cited owners. Shared Ghast movement helpers used by Happy
Ghast are included only for the ordinary Ghast constructor's
`careful=false`, never-stop and distance-to-blocks-zero inputs.
