# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-PARCHED-001` — Parched fire slow Weakness arrows they are themselves immune to and ride Camel Husks

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`,
`ITM-ARROW-AMMUNITION-001`, `ITM-BONE-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Parched` class, the
inherited `AbstractSkeleton` goal graph and bow path, the Husk-driven
Camel-Husk jockey production, all 66 biomes with their single Desert row,
three loot pools, the one direct entity tag, the Spawn Egg, both hostile-mob
advancements and exact client resources close protocol entity ID `97`.

**Applies when:**

`minecraft:parched` is constructed, finalized, spawned naturally in the
Desert, produced as a Camel-Husk jockey, spawned by an Egg, spawner, command
or custom selector, fleeing or restricted by sunlight, avoiding a Wolf,
targeting, drawing its Bow, firing a Weakness arrow, offered Weakness
itself, damaged, killed, synchronized, heard, imitated by a Parrot or
rendered.

**Authoritative state:**

Protocol entity ID `97` constructs `Parched` in `MONSTER`, and registration
marks it unavailable in Peaceful. Registration fixes width/height
`0.6×1.99`, eye height `1.74`, riding offset `-0.7`, client tracking
range `8` and the default update interval `3`.

Attributes start from the `AbstractSkeleton` set — Monster attributes plus
movement speed `0.25` — and override maximum health to `16`. That is the
only attribute the subtype changes, so attack damage remains the Monster
default `2` and every other value is inherited.

Parched adds no synchronized metadata of its own, so its slots are exactly
the inherited Entity, Living-Entity and Mob set `0..15`. It adds no
persisted field either; only the inherited generic, equipment and skeleton
state saves.

`canBeAffected` rejects any effect instance whose effect is Weakness before
delegating. A Parched therefore cannot receive Weakness from its own arrows,
another Parched's arrows, a splash potion, a command or any other source,
while every other effect follows the inherited rules.

**Transition and ordering:**

### Inherited goal graph and weapon reassessment

`AbstractSkeleton.registerGoals` provides the whole graph; Parched
registers nothing extra:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `2` | Restrict Sun |
| goal | `3` | Flee Sun, speed `1` |
| goal | `3` | avoid Wolf within `6`, walk/sprint `1`/`1.2` |
| goal | `5` | Water-Avoiding Random Stroll, speed `1` |
| goal | `6` | Look At Player, range `8` |
| goal | `6` | Random Look Around |
| target | `1` | Hurt By, no alerted classes |
| target | `2` | nearest Player, must see |

The constructor pre-builds one ranged Bow goal at speed `1`, minimum
interval `20` and radius `15`, and one melee goal at speed `1.2` without
following unseen targets. `reassessWeaponGoal` removes both, then re-adds
the Bow goal at priority `4` when the weapon-holding hand holds a Bow and
the melee goal at priority `4` otherwise.

Before adding the Bow goal it sets that goal's minimum attack interval from
the subtype hooks: `getHardAttackInterval()` on Hard and
`getAttackInterval()` otherwise. Parched overrides these to `50` and `70`
against the shared base values of `20` and `40`, so a Parched fires roughly
half as often as an ordinary skeleton at every difficulty — `70` ticks
normally and `50` on Hard.

Reassessment runs on the server only and is re-triggered whenever equipment
changes, so removing a Parched's Bow immediately converts it to a melee
attacker at the same priority.

`populateDefaultEquipmentSlots` gives every finalized Parched a plain Bow in
the main hand before the inherited difficulty-scaled armor pass.

### Weakness arrows

`performRangedAttack` is inherited. It resolves the Bow-holding hand, reads
the projectile stack, builds the arrow through `getArrow`, and with
`dx = targetX - x`, `dy = target.getY(1/3) - arrow.getY()`,
`dz = targetZ - z` and `dist = sqrt(dx² + dz²)` shoots at
`(dx, dy + dist*0.20000000298023224, dz)` with power `1.6` and inaccuracy
`14 - 4*difficultyId`. It then plays Skeleton Shoot at volume `1` and pitch
`1/(nextFloat*0.4 + 0.8)`.

Parched overrides only `getArrow`. It delegates to the inherited factory and
then, when the result is a plain `Arrow`, adds
`MobEffectInstance(minecraft:weakness, 600)` to that arrow. The added
instance uses amplifier `0` and default ambient, visibility and icon flags,
so every ordinary Parched arrow inflicts Weakness for `600` ticks on a hit
living target.

The check is on the concrete `Arrow` class, so a Parched shooting a
projectile that resolves to any other arrow implementation adds nothing.
Because the effect is attached to the arrow rather than applied at hit time,
the arrow itself carries the effect through flight, pickup and reload, and
its item form reflects the added effect.

Parched keeps the shared Skeleton Shoot release sound rather than a species
sound.

### Production

Parched registers `ON_GROUND` placement with heightmap
`MOTION_BLOCKING_NO_LEAVES` and the generic Monster predicate. Exactly one
of the `66` locked biomes carries a Parched row: Desert, in the monster
category at weight `50` for groups of exactly `4`. The biome data generator
adds that row with the same weight and group bounds, so the Desert row is
the whole baseline biome selector.

The second production path is the Husk. When a Husk finalizes with group
data that has not yet tried a Camel Husk, and the Camel-Husk spawn box at
the Husk's block position centre is collision-free, it latches the attempt
flag and draws one `nextFloat`. Below `0.1` it:

1. gives itself an Iron Spear in the main hand;
2. creates a Camel Husk with reason `NATURAL`, positions it at the Husk's
   position, finalizes it with null group data, and mounts the Husk on it;
3. adds the Camel Husk to the level; and
4. creates a Parched with reason `NATURAL`, snaps it to the Husk position
   and yaw, finalizes it, and mounts it as a second passenger.

The attempt flag is set for spawn reasons other than `NATURAL` when the
group data is first wrapped, so only naturally spawned Husk groups can
produce the jockey, and only the first Husk of a group attempts it. The flag
is latched before the `0.1` roll, so a failed roll consumes the group's one
attempt.

### Loot, tags, advancements and item identity

The entity loot table uses random sequence `minecraft:entities/parched` and
evaluates three ordered one-roll pools:

1. Arrow, item protocol ID `923`, integer-uniform count `0..2` with uniform
   `0..1` Looting enchanted-count increase;
2. Bone, item protocol ID `1112`, integer-uniform count `0..2` with the same
   Looting increase; and
3. on a player kill only, a Tipped Arrow, item protocol ID `1323`, with the
   Weakness potion set,
   integer-uniform count `0..1` with uniform `0..1` Looting increase capped
   at limit `1`.

Only the third pool is gated on a player kill, and only that pool's Looting
increase is capped. Positive-count filtering, the mob-loot gamerule,
equipment drops and death ordering retain their cited owners.

Exactly one direct entity-type tag names the Parched: `skeletons`, whose
locked members are Skeleton, Stray, Wither Skeleton, Skeleton Horse, Bogged
and Parched. Its consumers own the undead-adjacent behavior selected by that
tag.

Both hostile-mob advancements have an exact `player_killed_entity` criterion
for Parched. `kill_a_mob` places it in one OR requirement group with every
listed hostile; `kill_all_mobs` places it in its own required group.

The Spawn Egg is raw/protocol item ID `1206`, common, maximum stack `64`,
and its `entity_data.id` is `minecraft:parched`.

### Sounds and client projection

The locked sound-event joins are:

| Protocol ID | Event | English subtitle |
|---:|---|---|
| `1205` | Parched Ambient | “Parched crackles” |
| `1206` | Parched Death | “Parched dies” |
| `1207` | Parched Hurt | “Parched hurts” |
| `1208` | Parched Step | none |

`playStepSound` is inherited and plays the subtype step event at volume
`0.15` and pitch `1`. The locked language file contains no
`subtitles.entity.parched.step` entry, so the step sound is the only Parched
sound that produces no subtitle line.

Parrot imitation maps Parched to sound-event ID `1232`,
`entity.parrot.imitate.parched`, subtitle “Parrot crackles”; the Parrot's
attempt cadence, nearby selection, silence gate and playback retain the
Parrot owner.

`ParchedRenderer` is registered as the Parched renderer and
`ModelLayers` carries its layer. The locked texture
`textures/entity/skeleton/parched.png` is `64×64`, `1,483` bytes, with
SHA-256
`276b443d5537d9c75695bbb2371f944bd5a5c4cf420bba591b4407423e5c0dbd`.
English labels are “Parched” and “Parched Spawn Egg”.

**Branches and aborts:**

- `canBeAffected` rejects Weakness outright before any inherited check.
- `getArrow` adds the effect only when the inherited factory returned a
  plain `Arrow`.
- Weapon reassessment aborts on a client level; otherwise it always removes
  both goals and re-adds exactly one.
- The Hard interval applies only when the level difficulty is exactly Hard.
- The Camel-Husk jockey aborts when the group already tried, when the spawn
  box collides, when the `0.1` roll fails, or when either creation returns
  null; the attempt flag is still latched once the box test passes.
- Placement has exactly one baseline biome row, so every other biome
  produces Parched only through spawner, command or custom paths.

**Invariants:**

- A Parched can never hold Weakness, from any source.
- Every plain Arrow a Parched fires carries Weakness for `600` ticks.
- The fire interval is `70` normally and `50` on Hard, roughly half the
  shared skeleton cadence.
- Maximum health is `16`, below the shared skeleton value.
- The Bow and melee goals are mutually exclusive at priority `4`.
- Only Desert carries a baseline Parched spawn row.
- Only a naturally spawned Husk group can produce the Camel-Husk jockey, and
  only once per group.
- Only the Tipped-Arrow pool is player-kill gated.

**Constants and randomness:**

Entity/Egg IDs `97/1206`; dimensions/eye/riding `0.6×1.99/1.74/-0.7`;
range/update `8/3`; health/speed/attack `16/0.25/2`; goals sun `2`, flee
`3/1`, Wolf avoidance `3/6/1/1.2`, stroll `5/1`, look `6/8`, bow/melee
priority `4`; bow speed/interval/radius `1/20/15`, melee `1.2`;
attack intervals `70` and Hard `50` against base `40`/`20`;
shot power `1.6`, inaccuracy `14-4*difficultyId`, aim lift
`0.20000000298023224`, target aim height `1/3`, release pitch
`1/(nextFloat*0.4+0.8)`; Weakness `600`; step volume `0.15`;
jockey chance `0.1`, Iron Spear main hand; Desert row weight `50`,
group `4/4`; biome rows `1 of 66`; tags `1`; loot `0..2/0..2/0..1` with
Looting limit `1` on the third pool; texture `64×64`.

**Side effects:**

Equipment and goal-selector membership; RNG cursors for equipment,
inaccuracy, release pitch and the jockey roll; targets, navigation and look;
arrow entities carrying an attached effect; Camel Husk and Parched creation,
mounting and insertion; Husk group-data latch; loot stacks, XP and
advancement progress; sounds and renderer state.

**Gates:**

Logical side, Peaceful, NoAI and persistence; goal priority and flags; held
Bow identity; level difficulty for the interval choice; concrete `Arrow`
class for the effect attachment; Weakness identity for the immunity; Husk
spawn reason, group latch, collision box and RNG; placement light and Mob
predicate plus the single Desert row; player kill and Looting; mob loot;
resources.

**Boundary cases and quirks:**

A Parched is immune to the exact effect it inflicts, so two Parched shooting
each other never weaken one another. The effect rides on the arrow rather
than being applied at impact, so an arrow picked up and re-fired by a Player
still carries Weakness. Removing the Bow converts a Parched to melee at the
same priority without any other state change, and returning the Bow converts
it back. The Hard interval is chosen from the level difficulty at
reassessment time, not at fire time, so changing difficulty mid-life does
not retune an already-added Bow goal until equipment changes again. The
Camel-Husk jockey consumes its group's single attempt even when the `0.1`
roll fails, so a group that rolls badly never retries. The Parched step
sound is the only one of its four events without a subtitle. Desert is the
only biome with a Parched row, so Parched are absent from every other
baseline surface.

**Failure semantics:**

A failed projectile spawn still plays the release sound. A null Camel Husk
or Parched creation aborts the remaining jockey steps without rolling back
the Iron Spear already given to the Husk or the latch already set. Failed
insertion follows the inherited no-rollback behavior. Rejected effect
application on a hit target is the arrow owner's concern and does not affect
the Parched.

**Client/server authority split:**

The server owns targets, goals, weapon reassessment, arrow creation and the
attached effect, jockey production, loot and advancements. Clients consume
inherited metadata, equipment, movement and resources; they play the ambient,
hurt, death and step sounds, render the model and held Bow, and show the
Weakness effect only once the server applies it to the hit entity.

**Observability:**

Observe registration, attributes and the absence of subtype metadata; the
inherited goal graph and both priority-`4` weapon branches across equipment
changes and every difficulty; the interval override at `70` and `50`;
`getArrow` across plain and non-plain arrow results, and the exact
`600`-tick attachment through flight, pickup and re-fire; Weakness immunity
from every source; the Desert row against all `66` biomes; the Husk group
latch, collision box, `0.1` roll and both mounted creations; all three loot
pools with their Looting and player-kill gates; the single tag, both
advancements, the Egg; four sounds with the missing step subtitle, Parrot
imitation, and exact texture and model projection.

**Persistence and reload:**

Generic entity/Mob, equipment and inherited skeleton state save; Parched
adds no field of its own. Goal-selector membership is rebuilt from the held
weapon on load through reassessment. Attached arrow effects persist with the
arrow entity, not with the Parched. The Husk group latch is spawn-time-only
and never persists. Loot, tags, advancements and biome data reload through
their owners; the arrow-effect code remains fixed. Language, models and
textures reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.skeleton.AbstractSkeleton`;
`net.minecraft.world.entity.monster.skeleton.Parched`;
`net.minecraft.world.entity.monster.zombie.Husk`;
`net.minecraft.world.entity.animal.camel.CamelHusk`;
`net.minecraft.world.entity.projectile.arrow.Arrow`;
`net.minecraft.world.entity.projectile.ProjectileUtil`;
`net.minecraft.world.entity.ai.goal.RangedBowAttackGoal`;
`net.minecraft.world.entity.ai.goal.AvoidEntityGoal`;
`net.minecraft.world.entity.ai.goal.RestrictSunGoal`;
`net.minecraft.world.entity.ai.goal.FleeSunGoal`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.data.worldgen.BiomeDefaultFeatures`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.tags.EntityTypeTagsProvider`;
`net.minecraft.data.advancements.packs.VanillaAdventureAdvancements`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.ParchedRenderer`;
`net.minecraft.client.model.geom.ModelLayers`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,mob_effect,
loot_table,worldgen/biome,advancement}`;
`reports/minecraft/components/item/parched_spawn_egg.json`;
`data/minecraft/tags/entity_type/skeletons.json`;
`data/minecraft/loot_table/entities/parched.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`assets/minecraft/textures/entity/skeleton/parched.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-PROJECTILE-001`; `ENT-DAMAGE-001`; `ENT-EFFECT-001`;
`ENT-DEATH-001`; `MOB-AI-001`; `MOB-SPAWN-001`; `MOB-DESPAWN-001`;
`ITM-ARROW-AMMUNITION-001`; `ITM-BONE-001`; `ITM-ENCHANT-001`; `CLI-006`.

**Test vectors:**

Run `EXP-ENT-029` across construction/metadata/NoAI/save/reload, the
inherited goal graph and both weapon branches across equipment changes and
all difficulties, the `70`/`50` interval override, `getArrow` across plain
and non-plain results with the exact `600`-tick attachment through flight,
pickup and re-fire, Weakness immunity from every source, the Desert row
against all `66` biomes, the Husk group latch, collision box, `0.1` roll and
both mounted creations, all three loot pools with their Looting and
player-kill gates, the single tag, both advancements, the Egg, four sounds
with the missing step subtitle, Parrot imitation, and exact texture and
model projection.

**Limits:**

Generic lifecycle, metadata, equipment population, pathfinding, sunlight
avoidance, target algorithms, damage/death, arrow projectile runtime and
effect application, Husk and Camel Husk runtime, natural spawning and
despawn, loot, advancements, Spawn Egg interaction and rendering retain
their cited owners. Shared `AbstractSkeleton` algorithms are included only
where the Parched subtype registers, selects or changes their exact inputs
and observable result.
