# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-GIANT-001` — Giants are goal-free Monsters with latent oversized combat attributes and no baseline spawn selector

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked entity/attribute/spawn bootstraps, the complete
three-method `Giant` class, inherited Monster/Mob behavior, registry/tag/loot/
biome data, all `1,212` templates, seven exact migration/schema contexts and
client renderer/model/texture/language resources close protocol entity ID
`59`. Giant is not a Zombie server subtype and no baseline spawn list selects
it.

**Applies when:**

`minecraft:giant` is explicitly constructed, loaded, ticked, moved, targeted
or navigated by an external caller, damaged, killed, despawned, synchronized
or rendered, or when reloadable biome data is modified to request Giant
spawning.

**Authoritative state:**

Entity protocol ID `59` constructs `Giant` directly in category `MONSTER`.
Registration fixes width/height `3.6/12.0`, eye height `10.44`, riding offset
`-3.75`, client tracking range `10` and the builder-default update interval
`3`. `notInPeaceful` makes the type disallowed in Peaceful. It otherwise
retains ordinary summon/save, loot, fire and far-spawn defaults.

The registered attribute supplier is the Mob/Monster base with:

- maximum health `100`;
- movement speed `0.5`;
- attack damage `50`;
- camera distance `16`; and
- inherited follow range `16`.

`Monster` construction sets XP reward `5`. Giant declares no synchronized
data accessor, so its wire metadata is exactly the inherited Entity, Living
and Mob slots `0..15`; it adds no subtype NBT field or transient subtype state.
Generic entity state, attributes, equipment, effects, leash/passenger data
and persistence retain their owners.

Despite its appearance, Giant extends `Monster` directly. It does not inherit
Zombie AI, sunlight burning, reinforcement, conversion, baby, door-breaking,
equipment-finalization, sound or persistence behavior.

**Transition and ordering:**

### Empty AI and latent combat

`Giant` does not override `registerGoals`; neither does `Monster`, and the Mob
base implementation is empty. Both goal and target selectors therefore begin
empty. The ordinary server AI pass still increments `noActionTime`, ticks
sensing/selectors/navigation/controllers and applies generic movement, but no
baseline selector acquires a target, starts navigation, looks around or calls
melee attack.

Attack damage `50` is consequently latent rather than autonomous. A custom
goal, command/mod integration or other caller can set state, request
navigation or invoke the inherited Mob attack transaction, in which case
generic attack admission, enchantment, knockback, fire, damage and event
ordering use the stored attribute. Merely assigning a target does not install
an attack goal.

Giant's only behavioral override returns
`level.getPathfindingCostFromLightLevels(position)` directly as its walk-target
score. Ordinary `Monster` negates that value. Callers that compare candidate
walk positions therefore make Giant favor the larger, brighter-light score,
the inverse of the ordinary hostile preference. Empty baseline selectors do
not themselves request such a candidate.

`Monster.aiStep` updates swing time, then reads its light-dependent value; when
that value is greater than `0.5`, it increments `noActionTime` by an additional
`2` before delegating to PathfinderMob. The generic Mob server-AI increment
remains separate. This can accelerate random despawn eligibility in bright
conditions even though no goal runs.

### Hostile, sound and rest behavior

Giant inherits sound source `HOSTILE` and has no ambient-sound override.
Swimming, splashing, hurt, death, small-fall and big-fall select registered
Hostile sound IDs `876`, `875`, `873`, `872`, `874` and `871` respectively.
Pitch, volume, playback admission and fall thresholds remain generic. The
English generic fall/hurt/death/swim/splash subtitles are used where the sound
definitions request them; there is no Giant-specific subtitle or sound
record.

The inherited Monster rest predicate returns true. The player sleep owner
still supplies spatial lookup and all other admission checks; Giant does not
globally prevent sleep from arbitrary distance.

Damage, blocking, armor/effects, knockback, equipment, death protection and
death timing are wholly generic. `Monster.shouldDropLoot` reads live game rule
`mob_drops`; `shouldDropExperience` is always true, while the generic
player/recent-hit and death pipeline decides whether reward `5` is emitted.

### Despawn and Peaceful

On a Mob despawn check, Peaceful difficulty is tested first. Because Giant's
type is not allowed in Peaceful, it is discarded immediately before
persistence and distance checks. Construction is not itself rejected; the
generic despawn pass performs removal.

At other difficulties a persistence-required or custom-persistent Giant
resets `noActionTime` and returns. Otherwise the nearest-player squared
distance is compared with the MONSTER distances: beyond `128` blocks it is
discarded when `removeWhenFarAway` admits; after `noActionTime>600`, each
eligible check draws `nextInt(800)` and can discard beyond `32` blocks;
inside `32` blocks it resets `noActionTime`. MONSTER cap is `70`, friendly
and persistent category flags are both false, and inherited maximum spawn
cluster size is `4`.

### Placement predicate without a baseline selector

Giant nevertheless registers spawn placement `ON_GROUND`, heightmap
`MOTION_BLOCKING_NO_LEAVES`, with `Monster.checkMonsterSpawnRules`. When a
spawn reason does not ignore light, its darkness gate runs in this order:

1. read sky brightness and reject when it is greater than `nextInt(32)`;
2. read the dimension's `monster_spawn_block_light_limit`; when that limit is
   below `15`, reject if block light exceeds it;
3. obtain maximum local raw brightness with sky darken forced to `10` during
   thunder, otherwise the normal value; and
4. sample the dimension's `monster_spawn_light_test` with the supplied RNG
   and accept darkness only when local brightness is no greater than the
   sample.

On darkness success, `Mob.checkMobSpawnRules` accepts a spawner reason or
otherwise requires the state below to be valid spawn support for the type at
the candidate position. Every rejection short-circuits the later reads and
draws.

This registered predicate is not a natural-spawn request. Exhaustive inspection
of all `66` locked biome records finds zero Giant entry in any category, and
no structure/trial-spawner or other locked data record selects exact entity
identity `minecraft:giant`. Consequently the baseline natural/chunk-generation
spawners never choose Giant and its MONSTER cap/group mechanics receive no
vanilla Giant candidate. A data pack that adds Giant to a reloadable biome
spawn list makes the ordinary natural-spawn owner compose that row, cap,
candidate walk, placement predicate, cluster limit and insertion.

### Loot, tags, items and templates

The entity loot table has type `entity`, sequence
`minecraft:entities/giant`, and no pools. With `mob_drops` enabled it therefore
rolls zero item entries; with the rule disabled the inherited gate skips the
table. Eligible death can still emit XP reward `5`.

The entity has zero direct entity-type tag memberships. No item named
`minecraft:giant_spawn_egg` exists in the locked item registry, so Giant has
no dedicated creative/generic Spawn Egg path. Explicit entity construction,
commands, spawners carrying entity data and custom content retain their
generic owners.

Exact decoded UTF scanning of all `1,212` structure templates finds zero
`minecraft:giant` occurrence. The three filenames
`ruined_portal/giant_portal_{1,2,3}.nbt` describe template size, not Giant
entity identity; likewise `minecraft:giant_trunk_placer` is a worldgen
registry name and not an entity selection.

### Legacy migration

Seven exact identity-bearing fix/schema contexts are relevant:

- `EntityHealthFix` recognizes legacy entity name `Giant`;
- `EntityIdFix` maps legacy `Giant` to `minecraft:giant`;
- `EntityUUIDFix` includes `minecraft:giant` in generic mob UUID migration;
- `ItemSpawnEggFix` maps legacy generic Spawn Egg damage value `53` to
  EntityTag identity `Giant`;
- schema `V99` registers the legacy simple `Giant` shape; and
- schemas `V705` and `V1460` register modern `minecraft:giant` as a mob shape.

The later `ItemStackSpawnEggFix` has no Giant-to-dedicated-egg mapping, matching
the absent current item. It uses its generic fallback when converting that
historical item identity; the entity payload remains migration-owned rather
than creating a current Giant Spawn Egg. Substring matches in biome, height
and generator migration classes refer to unrelated “giant” worldgen names.

### Client projection

`EntityRenderers` binds Giant to `GiantMobRenderer` with scale argument `6`.
`LayerDefinitions` applies a `6.0` mesh transformer to the ordinary humanoid
zombie body layer and to the Giant armor set. The renderer constructs
`GiantZombieModel`, sets shadow radius `0.5*6 = 3`, and adds both
`ItemInHandLayer` and `HumanoidArmorLayer`. Custom equipment can therefore
render even though baseline Giant initialization equips nothing.

The renderer extracts generic Mob plus humanoid render state and always uses
`textures/entity/zombie/zombie.png`. That texture is exact `64×64`, `700`
bytes, SHA-1 `3d28d27b5388a7c2a296586649e98d068a795c2d`. Its zombie-shaped model
and texture are client projection only; they do not confer Zombie server
behavior. English entity name is `Giant`. There is no item name for a Giant
Spawn Egg.

**Branches and aborts:**

- Empty selectors perform no autonomous target, navigation or attack work.
- Positive light scoring matters only when a caller requests candidate
  selection.
- Peaceful discard precedes every persistence/distance branch.
- Persistent Giants return before nearest-player/RNG despawn.
- Hard `128`-block discard precedes the `noActionTime>600`, one-in-800
  random branch; distance below `32` resets inactivity.
- Spawn sky, block-light, local-light sample and support checks short-circuit
  in order, but no baseline biome row reaches them for Giant.
- Empty loot and XP reward `5` are distinct death outputs.

**Constants and randomness:**

Entity ID `59`; dimensions/eye/riding `3.6×12/10.44/-3.75`; tracking/update
`10/3`; health/speed/attack/camera/follow `100/0.5/50/16/16`; metadata
`0..15 inherited`; XP `5`; MONSTER cap/flags/distances
`70/false,false/32,128`; cluster `4`; bright inactivity increment `+2`;
random despawn `noActionTime>600`, `1/800`; spawn sky draw `nextInt(32)`;
block-light limit `<15`; thunder sky darken `10`; biome selectors `0/66`;
tags/items/template occurrences `0/0/0 of 1212`; legacy/schema contexts `7`;
sounds `871..876`; model/shadow/texture
`6/3/64×64,700,3d28d27b5388a7c2a296586649e98d068a795c2d`.

**Side effects:**

Generic metadata/save/equipment/effect/leash/passenger mutation; gravity,
collision and externally requested navigation/attack; inactivity/despawn;
damage/death/loot/XP; hostile sound and sleep query; injected-data spawn
candidate reads/RNG/insertion; synchronization and scaled rendering.

**Gates:**

Construction and logical side; external goal/target/navigation/attack caller;
path candidate light; difficulty/persistence/player distance/inactivity/RNG;
damage/death attribution and `mob_drops`; reloadable biome selection, spawn
reason/light/support/caps/cluster/insertion; client resource validity.

**Boundary cases and quirks:**

The server Giant is not a Zombie even though the client uses zombie geometry
and texture. Attack damage `50` does not imply autonomous attacks. Its positive
light score inverts the normal Monster walk preference but has no baseline goal
consumer. A registered spawn predicate does not make it naturally spawnable:
the locked biome selector census is empty. An old damage-53 generic Spawn Egg
migration does not imply a current dedicated item.

**Failure semantics:**

Generic lifecycle, navigation, attack, damage, death, spawn and despawn owners
retain commit/rollback rules. Peaceful and distance removal discard the entity;
spawn predicate failure prevents construction/insertion by its caller; empty
loot commits no item. Client resource failure affects projection, never server
identity.

**Client/server authority split:**

The server owns type, attributes, inherited metadata/state, motion, explicit
AI/navigation/attack requests, spawn admission, damage, despawn and death.
The client receives ordinary entity/living/mob state and projects a sixfold
humanoid zombie-shaped model, equipment layers, shadow, texture and name.

**Observability:**

Observe registry/dimensions/attributes, inherited metadata and save payload,
selector contents, path-score sign, explicit attack, inactivity/despawn
ordering, spawn light/support draws and zero selector census, interaction/
death outputs, exact tag/item/template/fix census, packet tracking and exact
renderer/model/equipment/texture/name projection.

**Persistence and reload:**

Only generic entity/Mob state persists; Giant adds no field. Entity type,
dimensions, attributes, metadata layout, empty goal registration, placement
predicate and migrations are code-built. Biome spawn lists, dimension light
settings and loot reload through their owners; language/texture are client
resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.Giant`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.GiantMobRenderer`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`net.minecraft.client.model.monster.zombie.GiantZombieModel`; four fix classes
and three schema classes; entity/sound reports, empty loot, all 66 biomes, all
`1,212` templates, item/tag absence, 64×64 texture and language. Complete
compiled/data/fix/NBT exact-identity searches exclude unrelated giant worldgen
names.

**Test vectors:**

Run `EXP-ENT-008` across construction/inherited state, empty selectors,
external navigation/attack, both light-score signs, all spawn light/support/
reason paths under an injected biome selector, all baseline biome/template
absence, cap/cluster/Persistence/Peaceful/distance despawn, loot/XP/sounds/
rest, exact migrations and scaled client projection.

**Limits:**

Generic entity lifecycle, movement/collision, navigation, combat, effects,
death, spawn engine, despawn, metadata packets and humanoid rendering retain
their owners. Reloaded custom biome rows and external goal injection are
supported boundary probes, not claims about baseline vanilla content. This
leaf fixes exact Giant and every direct join selecting it.
