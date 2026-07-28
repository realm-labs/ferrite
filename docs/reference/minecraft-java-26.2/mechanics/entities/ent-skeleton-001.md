# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-SKELETON-001` — Skeletons switch Bow and melee combat and convert to Strays in Powder Snow

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`,
`MOB-002`, `MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`,
`ITM-ARROW-AMMUNITION-001`, `ITM-BONE-001`, `ITM-ENCHANT-001`,
`ITM-ADVANCEMENT-001`, `BLK-SNOW-FAMILY-001`, `BLK-SKULL-001`,
`ENV-WEATHER-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PIPELINE-001`, `WGEN-PORTAL-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Skeleton` class,
every effective `AbstractSkeleton` override, Powder-Snow conversion, all
natural and structure spawn records, Monster-Room, Spider-jockey and
Skeleton-Trap producers, four Trial-Spawner configurations, loot,
advancements, tags, Spawn Egg, compatibility code and exact client
resources close protocol entity ID `115`.

**Applies when:**

`minecraft:skeleton` is constructed, finalized, loaded, spawned naturally,
selected by a Monster Room, Nether Fortress or Trial Spawner, created as a
Spider jockey or Skeleton-Trap rider, spawned by an Egg, command or custom
spawner, exposed to or removed from Powder Snow, converted to a Stray,
selecting or picking up a weapon, targeting, drawing or firing a Bow,
riding, burning in daylight, killed, synchronized, heard, imitated by a
Parrot or rendered.

**Authoritative state:**

Protocol entity ID `115` constructs `Skeleton` in the non-Peaceful
`MONSTER` category. Registration fixes width/height `0.6×1.99`, eye height
`1.74`, riding offset `-0.7`, client tracking range `8` and the default
update interval `3`.

Default attributes are maximum health `20`, movement speed `0.25`, attack
damage `2` and follow range `16`. Monster construction sets nominal XP
reward `5`.

Entity, Living Entity and Mob occupy synchronized metadata slots `0..15`;
`AbstractSkeleton` adds none. Concrete Skeleton adds `BOOLEAN` slot `16`,
serializer ID `8`, default false. It reports whether the Skeleton is
actively converting to a Stray and is also the exact input to
`Skeleton.isShaking()`.

Two private integers remain server-local:

- `inPowderSnowTime` is Java-initialized to `0`, increments only while
  eligible and in Powder Snow before conversion, and becomes `-1` whenever
  the eligible Skeleton is outside Powder Snow;
- `conversionTime` is the remaining conversion field. Starting conversion
  writes `300` and sets slot `16` true.

Save always writes integer `StrayConversionTime`: the current
`conversionTime` only while slot `16` is true, otherwise `-1`. Load reads an
integer with default `-1`; every other value, including negative values
other than `-1`, is copied to `conversionTime` and sets slot `16` true.
Exactly `-1` clears the metadata flag. `inPowderSnowTime` never persists.

**Transition and ordering:**

### Goals, targets and weapon selection

`AbstractSkeleton.registerGoals` installs this exact graph:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `2` | Restrict Sun |
| goal | `3` | Flee Sun at speed `1` |
| goal | `3` | avoid Wolf within `6`, walk/sprint `1`/`1.2` |
| goal | `4` | exactly one cached Bow or melee goal |
| goal | `5` | Water-Avoiding Random Stroll at speed `1` |
| goal | `6` | Look At Player within `8` |
| goal | `6` | Random Look Around |
| target | `1` | Hurt By, with no alerted classes |
| target | `2` | nearest visible Player |
| target | `3` | nearest visible Iron Golem |
| target | `3` | nearest baby Turtle on land, search interval `10` |

Construction caches a ranged Bow goal with speed `1`, minimum interval `20`
and radius `15`, and a melee goal with speed `1.2` that does not follow an
unseen target. Starting or stopping either combat goal sets or clears the
Mob aggressive bit.

`reassessWeaponGoal` returns when the level is null or client-side.
Otherwise it removes both cached combat goals. The exact Bow-holding hand is
resolved; when that hand contains exact Bow, the ranged goal is assigned
priority `4`, with minimum interval `20` on Hard and `40` on every other
difficulty. Any other equipment selects the melee goal at priority `4`.

Reassessment occurs during construction when a server level already exists,
after finalization equips/enchantments, after generic load, and after every
server equipment-change callback. The chosen interval is sampled from the
level difficulty at reassessment, so a later difficulty change alone does
not retune the cached Bow goal.

Only exact Bow passes `canUseNonMeleeWeapon`. The preferred-weapon tag is
`skeleton_preferred_weapons`, whose only locked member is Bow.
`wantsToPickUp` rejects every member of `spears` before generic Mob pickup
logic, so no Wooden through Netherite Spear is an admissible pickup even
when its other equipment comparison would win.

### Bow state machine and shot

The ranged goal requires a target and a held Bow. It can continue while the
ordinary admission remains true or navigation is still running, but in
either case still requires a held Bow. Stop clears aggression, `seeTime`,
`attackTime`, and active item use.

Each tick measures squared distance and line of sight. A change between
positive `seeTime` and current visibility first resets the counter; visible
ticks increment it and hidden ticks decrement it. Distance above `225` or
fewer than `20` consecutive visible ticks drives navigation toward the
target at speed `1` and resets strafe time to `-1`. Otherwise navigation
stops and strafe time increments.

At strafe time at least `20`, two independent `nextFloat()<0.3` draws toggle
clockwise and backwards state, then reset strafe time. While strafing,
distance above `225×0.75` forces forward and distance below `225×0.25`
forces backward. Movement uses forward `±0.5` and sideways `±0.5`; the
Skeleton and a controlled Mob vehicle both look at the target with
`30°/30°` limits.

While drawing, loss of sight for more than `60` ticks cancels use. With
sight, reaching `20` use ticks stops use, computes Bow power, performs the
ranged attack and sets `attackTime` to the selected `20` or `40` interval.
When not drawing, `attackTime` is pre-decremented; at zero or below and
`seeTime>=-60`, the Skeleton starts using the exact Bow-holding hand.

The shot resolves held ammunition without consuming it and constructs the
Mob arrow selected by the projectile stack. With
`dx=targetX-selfX`, `dz=targetZ-selfZ`,
`h=sqrt(dx²+dz²)` and
`dy=target.getY(1/3)-arrowY`, it shoots at
`(dx,dy+h*0.20000000298023224,dz)`. Power is `1.6`; uncertainty is
`14-4*difficultyId`, hence `10/6/2` on Easy/Normal/Hard. A nonserver level
skips projectile insertion, but the method still requests Skeleton Shoot
at volume `1` and pitch `1/(0.8+0.4*nextFloat)`.

`rideTick` first runs Monster riding behavior, then copies the controlled
Pathfinder-Mob vehicle's body yaw to the Skeleton.

### Finalization and daylight

`AbstractSkeleton.finalizeSpawn` first runs generic Monster finalization.
It then uses the level RNG to populate generic difficulty-scaled armor,
unconditionally places a plain Bow in the main hand, applies generic
difficulty-scaled equipment enchantments, and reassesses combat.

It next sets loot pickup from a fresh
`nextFloat()<0.55*specialMultiplier`. When the head slot remains empty and
the local calendar is Halloween, another `nextFloat()<0.25` equips headgear;
a third draw below `0.1` selects Jack o'Lantern, otherwise Carved Pumpkin.
That head slot receives drop chance zero.

Direct `burn_in_daylight` membership invokes generic daylight burning. It
requires the alive/environment/light/random/weather/Powder-Snow/sky gates
specified by `MOB-001`. Empty-headed admission ignites for eight seconds;
any head item protects, and damageable headgear takes `nextInt(2)` damage
and may break. Restrict/Flee Sun retain their cited path and shelter owners.

### Powder-Snow conversion

Before inherited ticking, a server-side, alive, AI-enabled Skeleton runs the
conversion machine:

1. While `isInPowderSnow` and slot `16` is already true,
   pre-decrement `conversionTime`; a negative result calls conversion.
2. While in Powder Snow but not converting, increment
   `inPowderSnowTime`. A result at least `140` writes conversion time `300`
   and sets slot `16` true.
3. While outside Powder Snow, write `inPowderSnowTime=-1` and clear slot
   `16`. The old `conversionTime` integer is not reset.
4. Client, dead or NoAI Skeletons skip all four mutations. Inherited tick
   runs afterward in every case.

The initial Java value `0` means first uninterrupted exposure starts
conversion on the increment reaching `140`. Leaving Powder Snow writes
`-1`, so a later fresh exposure first increments to `0` and reaches the same
threshold one eligible tick later. Leaving during conversion cancels the
metadata state rather than pausing it; reentry accumulates from `-1` and a
new threshold overwrites the stale timer with `300`.

`canFreeze()` is always false. Skeleton therefore never acquires ordinary
Powder-Snow freeze state or its speed/damage path. Its shaking projection is
only the conversion metadata, not `isFullyFrozen`.

On expiry, conversion calls `convertTo(STRAY, single(self,true,true), ...)`.
The single-entity conversion preserves equipment and `canPickUpLoot` and
carries the source team through the shared conversion owner. A successful
nonsilent conversion emits level event `1048` at the old block position;
the client plays Skeleton Converted to Stray in `HOSTILE`, volume `2`,
pitch `1+(nextFloat-nextFloat)*0.2`. A silent source suppresses only this
event.

### Production

Skeleton registers `ON_GROUND`, `MOTION_BLOCKING_NO_LEAVES` and the standard
darkness Monster predicate. Outer support, border, empty-space,
non-Peaceful, category-cap, cluster and distance rules retain
`MOB-SPAWN-001`.

Exactly `54` of the `66` locked biomes carry a Skeleton monster row.
Forty-seven use weight `100`, group `4..4`. The seven exceptions are:

| Biome | Weight | Group | Competing family row |
|---|---:|---:|---|
| Desert | `50` | `4..4` | Parched `50` |
| Ice Spikes | `20` | `4..4` | Stray `80` |
| Mangrove Swamp | `70` | `4..4` | Bogged `30` |
| Snowy Plains | `20` | `4..4` | Stray `80` |
| Soul Sand Valley | `20` | `5..5` | none in this family |
| Sulfur Caves | `50` | `2..2` | Cave Spider owns a separate row |
| Swamp | `70` | `4..4` | Bogged `30` |

The twelve absent biomes are Basalt Deltas, Crimson Forest, Deep Dark, the
five End surface biomes plus The Void, Mushroom Fields, Nether Wastes and
Warped Forest.

Nether Fortress supplies a piece-bounding-box monster override with
Skeleton weight `2`, group `5..5`, alongside Blaze `10/2..3`, Zombified
Piglin `5/4..4`, Wither Skeleton `8/5..5` and Magma Cube `3/4..4`.

Monster Room's uniform four-entry array is
`[Skeleton,Zombie,Zombie,Spider]`; selecting the origin spawner therefore
chooses Skeleton for exactly one of four indices. Spawner initialization,
room admission and write failure remain `WGEN-PIPELINE-001`.

Spider finalization independently uses its one-percent jockey branch to
construct a Skeleton with reason `JOCKEY`, finalize it and mount it on the
Spider. The exact group/RNG and insertion transaction remains
`ENT-SPIDER-001`.

A triggered Skeleton-Trap Horse creates up to four Skeleton riders. With an
alive Player within `10`, its trap goal clears trap, tames and age-resets
the original horse, creates a visual-only triggered lightning bolt, then
creates/finalizes a persistent Skeleton with `invulnerableTime=60` for the
original horse and up to three new persistent, tamed, age-zero Skeleton
Horses. Each rider receives an Iron Helmet only when its finalized head slot
is empty; both main-hand and head enchantments are cleared and reapplied
from `mob_spawn_equipment`. Additional horses receive independent X/Z
triangle pushes centered `0` with deviation `1.1485`. Null factories and
ignored insertion results do not roll back earlier trap mutations.

Four locked Trial-Spawner configurations contain only Skeleton at spawn
potential weight `1`:

- `trial_chamber/ranged/skeleton/normal`: simultaneous `3`, per-player
  addition `0.5`, interval `20`;
- its ominous form adds ranged equipment with zero slot drop chances and
  ejection weights key `3`, consumables `7`;
- `trial_chamber/slow_ranged/skeleton/normal`: simultaneous `4`,
  per-player addition `2`, interval `160`; and
- its ominous form adds the same equipment and ejection data.

The matching normal keys occur once each in
`trial_chambers/spawner/ranged/skeleton.nbt` and
`trial_chambers/spawner/slow_ranged/skeleton.nbt`. Exact UTF scans of all
`1,212` templates find zero literal `minecraft:skeleton` or plain
`skeleton` entity payloads.

### Loot, tags and progression

The entity table has sequence `minecraft:entities/skeleton` and two ordered
one-roll pools. Arrow, protocol item ID `923`, and Bone, item ID `1112`,
each receive integer-uniform base count `0..2`; Looting level `L>0` from a
living attacker adds `round(L*U)` from that pool's fresh uniform float.

A powered Creeper that passes its one-skull transaction evaluates
`charged_creeper/skeleton` for an exact Skeleton victim. That table emits
one Skeleton Skull, item ID `1263`, and then the Creeper's shared owner
latches `droppedSkulls`. Ordinary death, equipment and XP order remains
`ENT-DEATH-001`; eligible XP starts at `5` and can gain `1+nextInt(3)` per
qualifying equipped item.

Skeleton's direct entity-type tags are `burn_in_daylight`,
`no_anger_from_wind_charge` and `skeletons`. `skeletons` joins `undead`,
which transitively joins `can_breathe_under_water`,
`ignores_poison_and_regen`, `inverted_healing_and_harm`,
`sensitive_to_smite` and `wither_friends`. Their generic consumers own
drowning immunity, effect rejection/inversion, Smite and Wither relations.

`kill_a_mob` has Skeleton in the hostile OR group and `kill_all_mobs` gives
it its own required criterion. `sniper_duel` is Skeleton-specific: a
player-killed Skeleton must be at horizontal distance at least `50` and the
killing blow must carry `is_projectile`; completion awards `50` experience.

The common Skeleton Spawn Egg is item ID `1207`, stack limit `64`, with
`entity_data.id=minecraft:skeleton`.

### Compatibility, sounds and client projection

Legacy `EntitySkeletonSplitFix` keeps old `Skeleton` at type `0`, renames
type `1` to `WitherSkeleton` and type `2` to `Stray`; `EntityIdFix` then maps
`Skeleton` to `minecraft:skeleton`. Schema `V705` registers the modern Mob
shape and maps `minecraft:skeleton_spawn_egg` to Skeleton.
`TrialSpawnerConfigInRegistryFix.VanillaTrialChambers` recognizes both
legacy inline Skeleton Trial configurations and replaces them with the
normal/ominous registry-key pair. UUID, statistics, equipment and Spawn-Egg
compatibility remain with their generic fix owners.

Locked sound joins are:

| Protocol ID | Event | English subtitle |
|---:|---|---|
| `1479` | Skeleton Ambient | “Skeleton rattles” |
| `1480` | Skeleton Converted to Stray | “Skeleton converts to Stray” |
| `1481` | Skeleton Death | “Skeleton dies” |
| `1490` | Skeleton Hurt | “Skeleton hurts” |
| `1491` | Skeleton Shoot | “Skeleton shoots” |
| `1492` | Skeleton Step | none |

Inherited step playback uses volume `0.15`, pitch `1`. Parrot imitation maps
Skeleton to event ID `1239`, subtitle “Parrot rattles”; selection, cadence,
silence and pitch remain with the Parrot owner.

`EntityRenderers` binds `SkeletonRenderer`, the Skeleton and Skeleton-Armor
model layers, shadow radius `0.5` and the common humanoid armor layer.
Render state copies aggression, conversion shaking and exact main-hand Bow
identity. Only the main arm uses `BOW_AND_ARROW`, and only while aggressive
and holding that Bow; other arm poses remain generic. The shared
SkeletonModel owns thin limbs and melee/Bow animation.

The exact texture `textures/entity/skeleton/skeleton.png` is `64×32`, `477`
bytes, SHA-256
`854dd39acb1d8852db3b9e5d3d2bdc032070e37f421b53661ff48d0c1ddf290a`.
English labels are “Skeleton” and “Skeleton Spawn Egg”.

**Branches and aborts:**

- Weapon reassessment aborts for a null level or client level; otherwise it
  always removes both cached combat goals and installs exactly one.
- The Bow path requires exact Bow, while preferred pickup is separately
  tag-backed and every Spear is rejected before generic pickup.
- Conversion mutation requires server side, alive and AI enabled.
- Leaving Powder Snow clears only the conversion flag and exposure counter,
  not the stale conversion integer.
- Load treats only exact `-1` as inactive; any other integer starts active
  conversion.
- Conversion event `1048` requires a successful conversion and nonsilent
  source.
- Every producer retains its independent spawn, factory, insertion and
  equipment gates.

**Invariants:**

- Skeleton adds exactly one synchronized field, the conversion BOOLEAN at
  slot `16`.
- Ordinary freezing never applies because `canFreeze` is false.
- Bow and melee goals are mutually exclusive at priority `4`.
- A finalized ordinary Skeleton always receives a Bow after generic armor
  population.
- Every Spear pickup is rejected by the skeleton-family override.
- Only exact Skeleton satisfies `sniper_duel`.
- Literal Skeleton entity payloads occur in no locked structure template.

**Constants and randomness:**

Entity/Egg IDs `115/1207`; dimensions/eye/riding `0.6×1.99/1.74/-0.7`;
range/update `8/3`; health/speed/attack/follow/XP `20/0.25/2/16/5`;
goals sun `2`, flee/Wolf `3`, combat `4`, stroll `5`, look `6`; Bow
speed/radius/interval `1/15/20-or-40`, visibility `20/-60`, strafe cadence
`20`, toggles `0.3`, distance fractions `0.75/0.25`, movement `±0.5`;
shot power `1.6`, uncertainty `14-4*difficultyId`, lift
`0.20000000298023224`, pitch `1/(0.8+0.4F)`; pickup chance
`0.55*specialMultiplier`; Halloween `0.25`, Jack o'Lantern `0.1`;
exposure/timer `140/300`; event `1048`; biomes `54/66`; Monster Room
`1/4`; Fortress `2/5..5`; trap range `10`, riders at most `4`,
invulnerability `60`, push deviation `1.1485`; loot `0..2/0..2`;
Sniper horizontal distance `50` and reward XP `50`; texture `64×32`.

**Side effects:**

Goal-selector membership, aggression, navigation, looking and equipment;
RNG for armor, enchantments, pickup, Halloween equipment, strafing, shot
pitch and producers; arrow entities and sounds; conversion metadata, entity
replacement and level event; spawner/structure/trap entities, passengers
and insertion; loot, skulls, XP and advancement progress; client shaking,
poses, armor and texture state.

**Gates:**

Logical side, life, NoAI, Peaceful and persistence; equipment identity and
tags; goal priorities/flags, target class/visibility and Bow timing;
difficulty; Powder-Snow presence and timer; conversion factory/admission;
daylight environment and headgear; biome, placement, category and structure
selectors; Spider/trap/dungeon/Trial RNG and configuration; mob loot,
Looting, powered-Creeper latch and projectile-distance advancement;
resource reload.

**Boundary cases and quirks:**

Fresh exposure starts from `0`, but every post-exit exposure starts from
`-1`, producing a one-tick asymmetry. NoAI freezes all conversion fields
even outside Powder Snow. Loading `StrayConversionTime=-2` creates an active
conversion that expires on its next eligible in-snow tick. The timer
converts only after pre-decrement becomes negative, so a stored zero still
requires one eligible tick.

Leaving during conversion cancels the flag without zeroing the field; no
automatic resume occurs. A difficulty change does not change Bow cadence
until weapon reassessment. The ranged method can play Shoot even when its
level is not a server and no projectile was inserted. Step has a registered
sound but no subtitle. The painting variant named `skeleton` is unrelated
to the entity runtime and is not a production path.

**Failure semantics:**

If Stray creation fails, the source remains active with a negative timer and
retries on subsequent eligible in-snow ticks. Shared conversion insertion
ignores its Boolean result and retains that owner's discard semantics. A
failed projectile insertion does not suppress Shoot. Trap production
retains every mutation before a later null factory or ignored insertion;
Monster-Room/spawner and Trial failure behavior retain their owners. Loot
filters zero counts and insertion failure through the shared death owner.

**Client/server authority split:**

The server owns goals, targets, equipment reassessment, pickup, daylight
damage, projectile creation, exposure counters, conversion, entity
replacement, production, loot and advancements. Clients receive slot `16`,
equipment and movement; they use it for shaking, play event `1048` and
species sounds, and render the selected pose, armor and texture. Clients
never advance or cancel conversion themselves.

**Observability:**

Observe registration, attributes, slot `16` and both private integers across
fresh/exit/reentry/NoAI/load boundaries; the full goal/target graph and both
priority-`4` combat modes; every Bow visibility, strafe, draw, interval and
shot boundary; exact Bow/preferred/Spear decisions; finalization and
Halloween draw order; daylight; conversion factory/event behavior; all 66
biomes and every dungeon/fortress/Spider/trap/Trial producer; loot, charged
Creeper skull, XP, tags, three advancements and Egg; migrations, template
census, sounds, Parrot and exact client projection.

**Persistence and reload:**

Generic entity/Mob/equipment state persists through cited owners.
`StrayConversionTime` alone persists subtype state; `inPowderSnowTime` does
not. Load performs generic skeleton weapon reassessment before reading and
applying the subtype conversion field. Tags, biome/structure lists, Trial
configs, loot and advancements reload server-side; language, renderer
resources and texture reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.skeleton.AbstractSkeleton`;
`net.minecraft.world.entity.monster.skeleton.Skeleton`;
`net.minecraft.world.entity.monster.skeleton.Stray`;
`net.minecraft.world.entity.ai.goal.RangedBowAttackGoal`;
`net.minecraft.world.entity.projectile.ProjectileUtil`;
`net.minecraft.world.entity.ConversionParams`;
`net.minecraft.world.entity.ConversionType`;
`net.minecraft.world.level.levelgen.feature.MonsterRoomFeature`;
`net.minecraft.world.entity.monster.spider.Spider`;
`net.minecraft.world.entity.animal.equine.SkeletonTrapGoal`;
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerConfigs`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.util.datafix.fixes.EntitySkeletonSplitFix`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix`;
`net.minecraft.util.datafix.schemas.V705`;
`net.minecraft.client.renderer.LevelEventHandler`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.SkeletonRenderer`;
`net.minecraft.client.renderer.entity.AbstractSkeletonRenderer`;
`net.minecraft.client.renderer.entity.state.SkeletonRenderState`;
`net.minecraft.client.model.monster.skeleton.SkeletonModel`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,
loot_table,advancement,worldgen/biome,worldgen/structure_type}`;
`reports/minecraft/components/item/skeleton_spawn_egg.json`;
`data/minecraft/tags/entity_type/{burn_in_daylight,
no_anger_from_wind_charge,skeletons,undead,can_breathe_under_water,
ignores_poison_and_regen,inverted_healing_and_harm,sensitive_to_smite,
wither_friends}.json`;
`data/minecraft/tags/item/{skeleton_preferred_weapons,spears}.json`;
`data/minecraft/loot_table/{entities/skeleton,
charged_creeper/skeleton}.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/worldgen/structure/fortress.json`;
`data/minecraft/trial_spawner/trial_chamber/{ranged,slow_ranged}/
skeleton/{normal,ominous}.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs,
sniper_duel}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/textures/entity/skeleton/skeleton.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-SPIDER-001`; `ENT-DEATH-001`; `BLK-SNOW-FAMILY-001`;
`BLK-SKULL-001`; `ENV-WEATHER-001`; `WGEN-PIPELINE-001`;
`ITM-ARROW-AMMUNITION-001`; `ITM-BONE-001`; `ITM-ENCHANT-001`;
`ITM-ADVANCEMENT-001`; `CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-034` across raw/finalized/loaded construction, all conversion
counter/metadata/persistence states and factory outcomes, the full
goal/target graph and both weapon branches, exact ranged-state boundaries,
equipment/pickup/finalization/Halloween/daylight paths, all natural and
special producers, both loot pools plus charged-Creeper skull/XP,
tags/advancements/Egg, compatibility/template census, every sound and exact
render projection.

**Limits:**

Generic lifecycle, metadata transport, movement, navigation, target search,
melee damage, daylight transaction, projectile flight/hit, conversion copy,
natural spawning, spawner execution, Spider and Skeleton-Horse runtime,
death/equipment/XP, loot, advancements, Spawn-Egg interaction and rendering
retain their cited owners. This leaf owns the Skeleton selectors, concrete
overrides, constants, state, joins and observable composition of those
algorithms.
