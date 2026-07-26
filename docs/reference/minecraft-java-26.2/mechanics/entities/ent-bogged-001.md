# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-BOGGED-001` — Bogged retain one-way shearing state and fire slow poison-arrow volleys

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`BLK-TRIAL-SPAWNER-001`, `BLK-SMALL-MUSHROOM-001`,
`ITM-ARROW-AMMUNITION-001`, `ITM-BONE-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete `Bogged`,
`AbstractSkeleton` and ranged-bow paths, generic daylight-burning and Monster
placement code, all 66 biomes, direct and transitive tags, loot and shearing
tables, four Trial-Spawner configurations, Spawn Egg, four migration
contexts, all 1,212 templates and exact client resources close protocol
entity ID `16`.

**Applies when:**

`minecraft:bogged` is constructed, naturally selected in a Swamp or Mangrove
Swamp, emitted by either poison-skeleton Trial-Spawner family, spawned by an
egg, command or custom selector, loaded, equipped, sheared, moved, targeted,
damaged, killed, synchronized or rendered.

**Authoritative state:**

Entity protocol ID `16` constructs `Bogged` in `MONSTER`. Registration makes
the type unavailable in Peaceful, with dimensions `0.6×1.99`, explicit eye
height `1.74`, riding offset `-0.7`, client tracking range `8` and default
update interval `3`. Default attributes are maximum health `16`, movement
speed `0.25`, attack damage `2` and follow range `16`. Monster construction
sets nominal XP reward `5`.

Entity/Living/Mob occupy synchronized metadata slots `0..15`.
`AbstractSkeleton` adds none; Bogged adds `BOOLEAN` slot `16`, serializer ID
`8`, default false. It is the complete subtype state: false means mushroomed
and ready for shearing, true means sheared. The setter performs no life,
equipment or range check and accepts either value.

Save always writes lowercase Boolean key `sheared`. Load reads
`getBooleanOr("sheared", false)`, so an absent or wrong-typed value restores
false. No tick, goal, spawn-finalization or other subtype path writes false:
ordinary play has no mushroom-regrowth transition. Recreating or externally
editing/resetting the entity is required to reverse shearing.

**Transition and ordering:**

### Exact shearing transaction

`readyForShearing()` is exactly `!isSheared()`; it does not independently
require life, adulthood or a particular pose. Player interaction tests an
exact Shears item and readiness. A match returns `SUCCESS` on both logical
sides, but only the server performs effects, in this order:

1. play Bogged Shear from the entity in the `PLAYERS` source, volume and
   pitch `1`;
2. evaluate `minecraft:shearing/bogged` with the supplied tool;
3. for each result, call generic `spawnAtLocation` at vertical offset equal
   to Bogged's `1.99` bounding-box height;
4. synchronize sheared true;
5. emit game event `SHEAR` with the player as source; and
6. damage the used Shears by one in the hand's equipment slot.

The public `shear` method itself does not recheck readiness. Empty loot or a
failed item-entity insertion does not undo its preceding sound, state write,
later game event or tool damage. A nonmatching or already-sheared interaction
delegates to the inherited Mob interaction path.

The shearing table has type `shearing`, random sequence
`minecraft:shearing/bogged`, and one pool with exactly two rolls. Each roll
chooses equal-weight Brown-Mushroom and Red-Mushroom entries and fixes count
one. The two selections are independent, so both equal pairs and either
ordering of the mixed pair are possible. Context supplies origin,
`THIS_ENTITY` and tool; it has no player parameter.

### Goal graph and equipment-dependent combat

The movement goal selector contains:

- priority `2`, `RestrictSunGoal`;
- priority `3`, `FleeSunGoal`, speed `1`;
- priority `3`, `AvoidEntityGoal<Wolf>`, range `6`, far speed `1` and near
  speed `1.2`;
- priority `4`, exactly one dynamically selected Bow or melee goal;
- priority `5`, `WaterAvoidingRandomStrollGoal`, speed `1`;
- priority `6`, `LookAtPlayerGoal<Player>`, range `8`; and
- priority `6`, `RandomLookAroundGoal`.

The target selector contains priority `1` `HurtByTargetGoal` without group
alert, priority `2` nearest Player, and priority `3` nearest Iron Golem and
baby Turtle on land. Both ordinary nearest-target goals require sight and use
the default ten-tick search cadence. The Turtle goal has target chance `10`,
requires sight, does not require reachability, and uses
`Turtle.BABY_ON_LAND_SELECTOR`.

The constructor creates a Bow goal at speed `1`, radius `15`, and an initial
interval later overwritten by reassessment, plus a melee goal at speed `1.2`
without long memory. Server reassessment removes both. An exact Bow held in
either hand selects the priority-four Bow goal and writes minimum interval
`50` on Hard or `70` otherwise; any other equipment selects melee. Loading
and equipment changes reassess, while the client-side method returns without
changing selectors. The melee wrapper sets aggressive true on start and
false on stop. Only an exact Bow qualifies as a non-melee weapon; the
preferred-equipment tag is `skeleton_preferred_weapons`, and pickup rejects
every item in `#spears` before generic evaluation.

Normal spawn finalization first performs generic Mob/Monster finalization,
including the optional triangular follow-range modifier and five-percent
left-handed draw. It then applies generic random armor, always overwrites the
main hand with a Bow, applies spawn-equipment enchantment chances, reassesses
combat, and sets loot pickup from
`nextFloat()<0.55*specialMultiplier`. If the head remains empty on Halloween,
a further `<0.25` draw equips a Jack o'Lantern on a nested `<0.1` draw and a
Carved Pumpkin otherwise, with head drop chance zero. Trial-spawned id-only
Bogged pass through this finalization; ominous Trial-Spawner equipment is
applied by that external owner afterward.

### Bow cadence, movement and poison projectile

The Bow goal owns Move and Look controls and ticks every update. It can use
only with a non-null target and held Bow. Start marks aggressive. Stop clears
aggression, resets `seeTime` to zero and `attackTime` to `-1`, and stops item
use.

Let `d²` be squared distance to the target's base coordinates. Line-of-sight
sign changes reset `seeTime` to zero; visible ticks increment it and hidden
ticks decrement it.

- If `d²>225` or `seeTime<20`, navigation moves toward the target at speed
  `1` and resets strafing time to `-1`.
- Otherwise navigation stops and strafing time increments. Every 20 strafing
  ticks, two independent `<0.3` floats toggle clockwise and backward state.
- Strafing forces forward movement when `d²>168.75` and backward movement
  when `d²<56.25`; equality preserves the prior direction. Forward and
  lateral inputs are each `±0.5`.
- Strafing looks at the target with limit `30` and also turns a controlled
  Mob vehicle; the nonstrafing branch uses Look Control with the same limit.

While using the Bow, `seeTime<-60` aborts use. Once visible with at least 20
use ticks, the goal stops using, attacks at the Bow charge fraction and
resets `attackTime` to `50` on Hard or `70` otherwise. While not using, it
pre-decrements `attackTime`; at zero or below and `seeTime>=-60`, it starts
using the exact Bow hand. It may therefore begin a draw after as many as 60
hidden ticks.

The attack selects a supported held projectile from the normal hand order or
uses the generic default without consuming ammunition. Generic Mob-arrow
construction preserves tipped/spectral selection. Bogged then adds Poison
duration `100` only when the result is the concrete `Arrow` class; a Spectral
Arrow result does not receive this effect.

Aim uses `dx=targetX-selfX`, `dz=targetZ-selfZ`,
horizontal `h=sqrt(dx²+dz²)`, and
`dy=target.getY(1/3)-arrowY+0.2*h`. Projectile speed is `1.6`; uncertainty is
`14-4*difficultyId`, hence `10/6/2` on Easy/Normal/Hard. The server projectile
helper owns velocity and insertion. After that conditional server spawn,
Bogged always requests Skeleton Shoot at volume `1` and pitch
`1/(0.8+0.4*nextFloat)`. Skeleton Shoot has protocol sound ID `1491`.

`rideTick` otherwise inherits Monster behavior and then copies the controlled
Pathfinder-Mob vehicle's body yaw to the Bogged. `isShaking()` is exactly
`isFullyFrozen()`; unlike concrete Skeleton, Bogged has no powder-snow
conversion path.

### Daylight, undead joins and placement

Direct `burn_in_daylight` membership invokes the generic server burn
transaction. It requires an alive entity, environment attribute
`monsters_burn`, light magic above `0.5`,
`nextFloat()*30<(lightMagic-0.4)*2`, no water/rain or current/previous Powder
Snow, and sky visibility at `(X,eyeY,Z)`. A nonempty head item prevents
ignition. Damageable headgear takes `nextInt(2)` durability and may break;
nondamageable headgear protects without that draw. An unprotected match
ignites for eight seconds.

`RestrictSunGoal` instead checks bright exterior, empty head and ground
navigation, setting navigation's avoid-sun flag while active. `FleeSunGoal`
requires no target, bright exterior, actual fire, sky visibility and empty
head. It tests at most ten candidate offsets using
`nextInt(20)-10` for X/Z and `nextInt(6)-3` for Y, accepting the first
non-sky-visible position with negative walk-target score, then moves there at
speed `1`.

Bogged's three direct entity-type tags are `burn_in_daylight`,
`no_anger_from_wind_charge` and `skeletons`. The latter joins `undead`, which
transitively joins `can_breathe_under_water`, `ignores_poison_and_regen`,
`inverted_healing_and_harm`, `sensitive_to_smite` and `wither_friends`.
Generic consumers therefore skip drowning, ignore Poison and Regeneration,
invert Healing/Harming, admit Smite sensitivity, and apply Wither
friend/target exclusions. The wind-charge tag suppresses its generic anger
attribution path; projectile behavior retains that owner.

Bogged registers `ON_GROUND`, `MOTION_BLOCKING_NO_LEAVES`, and the standard
darkness Monster predicate. The outer placement gate requires world-border,
valid support and empty candidate/above blocks. Outside spawn reasons that
ignore light, the predicate first requires sky brightness no greater than
`nextInt(32)`, then the dimension block-light limit, then raw brightness no
greater than the dimension's sampled monster-light threshold; thunder uses
the darkened local-brightness path. Generic Mob support and the non-Peaceful
type gate still apply.

Exactly two of 66 biomes select Bogged: `swamp` and `mangrove_swamp`. Each
Monster list gives it weight `30` and fixed group `4..4`. Natural spawning
then uses Monster category cap `70`, inherited per-cluster maximum `4`,
hostile/nonpersistent classification, and no-despawn/despawn distances
`32/128`.

### Trial Spawners, templates and compatibility

Four locked Trial-Spawner configurations contain Bogged:

- `trial_chamber/ranged/poison_skeleton/normal`: simultaneous `3`, added per
  player `0.5`, interval `20`;
- its ominous form: the same values plus ranged Trial-Chamber equipment with
  every slot drop chance zero and ejection weights key `3`, consumables `7`;
- `trial_chamber/slow_ranged/poison_skeleton/normal`: simultaneous `4`, added
  per player `2`, interval `160`; and
- its ominous form: those values plus the same equipment and ejection data.

Every spawn-potential list contains only id-only Bogged at weight `1`, except
for the ominous external equipment wrapper. Trial-Spawner defaults retain
total `6+2p` for `p` additional players, spawn range `4` and cooldown `36,000`;
activation, omen conversion, finalization, persistence and insertion belong
to `BLK-TRIAL-SPAWNER-001`.

Two templates contain the corresponding normal configuration keys once:
`trial_chambers/spawner/ranged/poison_skeleton.nbt` and
`trial_chambers/spawner/slow_ranged/poison_skeleton.nbt`. Exact UTF scans of
all 1,212 templates find zero `minecraft:bogged` or plain `bogged` payloads.
The filename `trial_chambers/corridor/atrium/bogged_relief.nbt` is decorative
and contains no entity identity.

Exactly four migration contexts own Bogged compatibility:

- `DataFixers` installs the version-3816 `AddNewChoices` entity fix named
  `Added Bogged`;
- schema `V3816` registers simple entity `minecraft:bogged`;
- schema `V705` maps `minecraft:bogged_spawn_egg` to the Bogged entity; and
- `TrialSpawnerConfigInRegistryFix.VanillaTrialChambers` recognizes both old
  inline poison-skeleton configurations and maps them to registry keys.

No fix rewrites `sheared`; its missing-state default remains false.

### Death, sounds and client projection

The entity loot table has type `entity`, random sequence
`minecraft:entities/bogged`, and three one-roll pools. Arrow and Bone each
receive base uniform integer count `0..2`; a living attacker with Looting
level `L>0` adds `round(L*U)` from a fresh uniform float. A
`killed_by_player` pool emits Poison Tipped Arrow with base `0..1`; its
Looting increase uses another fresh uniform float but caps the final count at
one.

Generic eligible-kill XP starts at `5` and can add
`1+nextInt(3)` for each qualifying equipped item. A normally finalized
Bogged always holds a qualifying Bow, so its minimum equipment-adjusted
range is `6..8`; armor or Halloween headgear can add further independent
increments. Exact entity conditions occur in `adventure/kill_a_mob` and
`adventure/kill_all_mobs`. `sniper_duel` tests exact
`minecraft:skeleton`, not the `skeletons` tag, so Bogged does not satisfy it.

Common Bogged Spawn Egg is raw item ID `1202`, maximum stack `64`, with
`entity_data.id=minecraft:bogged` and generic Spawn-Egg use, dispenser and
projection.

Ambient/death/hurt/shear/step sound protocol IDs are `181..185`; Skeleton
Shoot is `1491`. English subtitles exist for ambient/death/hurt as `Bogged
rattles/dies/hurts`; shear and step have no Bogged-specific subtitle.
Parrot's exact entity map selects imitation ID `1215`, subtitle `Parrot
rattles`. Generic Parrot imitation first requires its source entity alive and
nonsilent and a successful one-in-two draw, then chooses a random non-Parrot
Mob within the bounding box inflated by `20`; a silent chosen mob aborts.
Playback is at the Parrot position, volume `0.7`, Parrot pitch and the
Parrot's sound source.

`EntityRenderers` binds `BoggedRenderer`, inheriting skeleton shadow radius
`0.5`, armor layers, aggression/Bow arm pose and fully-frozen shaking. Base
texture `textures/entity/skeleton/bogged.png` is `64×32`, `817` bytes, SHA-1
`8d9a203184bdea2839526b6f381b1c4c638c5e59`. The always-present clothing
layer uses a `0.2`-deformed humanoid mesh and
`bogged_overlay.png`, `64×32`, `576` bytes, SHA-1
`25f7bc4beedd6b4de6cb3eaf9291dec21b003949`.

`BoggedRenderState` copies synchronized `isSheared`. The base skeleton model
adds a `mushrooms` child beneath the head with six zero-depth `6×4` planes:
two red planes at `(3,-8,3)` with Y rotations `pi/4,3pi/4`; two brown head
planes at `(-3,-8,-3)` with the same rotations; and two brown side planes at
`(-2,-1,4)` with X rotation `-pi/2` and Z rotations `pi/4,3pi/4`.
Texture offsets are `(50,16)`, `(50,22)` and `(50,28)` respectively.
Setup first applies complete Skeleton animation, then makes only the
mushroom group visible when not sheared. The green clothing overlay remains
visible after shearing. English names are `Bogged` and `Bogged Spawn Egg`.

**Branches and aborts:**

- Player shearing is optimistic `SUCCESS` on both sides; only the server
  changes loot, state, event and durability.
- `shear` sound and loot precede the state bit; event and durability follow
  it, and no item-insertion result rolls the transaction back.
- Exact Bow equipment selects slow ranged combat; all other equipment
  selects melee.
- Bow movement uses strict `>225`, `>168.75` and `<56.25` comparisons, with
  20 visible ticks before strafing.
- Poison is attached only to concrete `Arrow`, not every
  `AbstractArrow`.
- Daylight protection returns after headgear handling; unprotected admission
  ignites for eight seconds.
- Only two baseline biome lists and four Trial-Spawner configs select
  Bogged; templates contain configuration keys, never an embedded identity.

**Constants and randomness:**

Entity/Egg IDs `16/1202`; dimensions/eye/ride `0.6×1.99/1.74/-0.7`;
tracking/update `8/3`; health/speed/attack/follow `16/0.25/2/16`; nominal XP
`5`; metadata `0..15 inherited, 16 BOOLEAN`; shearing `2` independent
equal-weight rolls; goal priorities `2/3/3/4/5/6/6`, targets `1/2/3/3`;
Wolf `6/1/1.2`; Bow speed/radius `1/15`, use/strafe cadence `20`, hidden
cutoffs `-60`, distance squares `225/168.75/56.25`, interval `50/70`; Poison
`100`; projectile speed/uncertainty `1.6/14-4d`; fire `8` seconds; biome
weight/group `30/4..4`; category/despawn `70/32/128`; trial simultaneous/
added/interval `3/.5/20` and `4/2/160`; sounds `181..185/1215/1491`;
templates/migrations `0 exact identity of 1212/4`; shadow `0.5`; textures as
above.

**Side effects:**

Shear sound, mushroom item entities, metadata, game event and tool damage;
goal/navigation/look/aggression state; equipment and enchantment
finalization; Bow use, projectile creation, Poison and sound; daylight
headgear wear/fire; tag-selected effect and damage behavior; spawn
selection/finalization/despawn; loot, XP, criteria, Parrot imitation and
client layers.

**Gates:**

Logical side, exact held item, sheared bit, loot context and item insertion;
target/life/sight/distance, goal controls, equipment and difficulty; projectile
class; environment attribute/light/weather/headgear/sky; Peaceful/world
border/support/spawn reason/light sample; biome/category/cluster; Trial
Spawner player/omen/gamerule/collision; attacker/Looting/player kill;
migration shape and client render state.

**Boundary cases and quirks:**

Shearing is permanent in ordinary play and readiness ignores life. Failed
mushroom insertion does not prevent state or durability changes. Non-Hard
Bogged wait `70` ticks between Bow attacks, slower than Skeleton's ordinary
interval. A Spectral Arrow remains unpoisoned. The direct `skeletons` tag
grants undead behavior but does not satisfy the exact-Skeleton
`sniper_duel` criterion. The decorative Bogged relief has no Bogged entity
NBT.

**Failure semantics:**

Rejected interaction delegates without subtype change. Loot or insertion
failure leaves already ordered shearing effects committed. Rejected
placement prevents natural construction/insertion. Trial-Spawner failure
retains its owner transaction. Rejected projectile insertion still leaves
the later shoot-sound request. Rejected damage/death outputs remain under
their generic transactions.

**Client/server authority split:**

The server owns state persistence, shearing results, AI, equipment,
projectile creation, Poison, daylight burning, placement, trial spawning,
loot and XP. The client returns interaction success optimistically, receives
metadata and equipment, and projects mushrooms, overlay, animation, shaking,
sounds and item/entity models. Parrot and ordinary sound delivery remain
server-selected and recipient-rendered.

**Observability:**

Observe registration and attributes; slot-16 dirty packets and save/load;
every shearing order/failure branch and two-roll output; goal graph,
equipment reassessment, LOS/strafing/draw timing; projectile class/aim/
Poison/sound; daylight and undead-tag joins; both biome rows and all four
Trial-Spawner configs; loot/XP/criteria; template/migration closure; exact
Parrot, model, layer, texture and sheared projection.

**Persistence and reload:**

`sheared` persists explicitly; inherited entity/Mob/equipment state retains
generic persistence, while Bow-goal counters and aggression reset. Code fixes
goals, placement and migration schemas. Biomes, tags, loot, shearing,
Trial-Spawner configurations and equipment reload through their owners;
language and textures are client resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.monster.skeleton.Bogged`;
`net.minecraft.world.entity.monster.skeleton.AbstractSkeleton`;
`net.minecraft.world.entity.ai.goal.RangedBowAttackGoal`;
`net.minecraft.world.entity.ai.goal.RestrictSunGoal`;
`net.minecraft.world.entity.ai.goal.FleeSunGoal`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.projectile.ProjectileUtil`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.util.datafix.DataFixers`;
`net.minecraft.util.datafix.schemas.V3816` and `V705`;
`net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.BoggedRenderer`;
`net.minecraft.client.renderer.entity.state.BoggedRenderState`;
`net.minecraft.client.model.monster.skeleton.BoggedModel`; reports, direct
and transitive tags, loot/shearing tables, four Trial-Spawner configurations,
both biome records, advancements, all 1,212 templates, Egg components,
textures, sounds and language. Complete compiled/data identity searches find
no other direct runtime path.

**Test vectors:**

Run `EXP-ENT-012` across sheared metadata/save/reset and every interaction/
loot/insertion ordering branch, equipment finalization and combat
reassessment, all Bow LOS/range/strafe/draw/arrow-class branches, daylight
burn and undead tags, swamp placement and four Trial-Spawner configs,
despawn/death/loot/XP/advancements, Spawn Egg, templates/migrations and exact
Parrot/model/layer/texture/name projection.

**Limits:**

Generic entity lifecycle, targeting/navigation, damage/death, natural spawn,
Trial Spawner, projectile, loot evaluation, Spawn Egg, metadata packets and
rendering retain their owners. Mushroom blocks, Arrow/Bone items and
Trial-Chamber generation retain their leaves. This leaf fixes exact Bogged
dispatch and every direct join selecting it.
