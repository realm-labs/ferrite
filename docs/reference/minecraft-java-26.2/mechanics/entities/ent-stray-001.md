# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-STRAY-001` — Strays ignore Powder Snow and fire Slowness arrows under open sky

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-SKELETON-001`,
`MOB-001`, `MOB-AI-001`, `MOB-002`, `MOB-SPAWN-001`, `MOB-003`,
`MOB-DESPAWN-001`, `ITM-ARROW-AMMUNITION-001`, `ITM-BONE-001`,
`ITM-ENCHANT-001`, `ITM-ADVANCEMENT-001`, `BLK-SNOW-FAMILY-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Stray` class,
effective `AbstractSkeleton` behavior, custom placement, both immunity
channels, Skeleton conversion, all biome and Trial-Spawner records, loot,
tags, advancements, Spawn Egg, compatibility and exact two-texture client
projection close protocol entity ID `128`.

**Applies when:**

`minecraft:stray` is constructed, finalized, spawned naturally, created by
Skeleton Powder-Snow conversion, emitted by either Stray Trial-Spawner
family, spawned by Egg, spawner, command or custom selector, interacting
with Powder Snow, freezing, burning in daylight, selecting or firing a
weapon, killed, synchronized, heard, imitated by a Parrot or rendered.

**Authoritative state:**

Protocol entity ID `128` constructs `Stray` in the non-Peaceful `MONSTER`
category. Registration fixes dimensions `0.6×1.99`, eye height `1.74`,
riding offset `-0.7`, client tracking range `8` and default update interval
`3`.

Default attributes are the `AbstractSkeleton` set: maximum health `20`,
movement speed `0.25`, attack damage `2` and follow range `16`. Monster
construction sets nominal XP reward `5`.

Stray adds no synchronized metadata. Its complete layout is the inherited
Entity, Living-Entity and Mob slots `0..15`; `AbstractSkeleton` adds none.
It also adds no persisted field. Generic state, equipment and frozen ticks
retain their owners.

Registration binds `immuneTo(#minecraft:stray_immune_to)`. That block tag
contains only Powder Snow. It is a live entity-type block-danger exemption,
separate from freezing state and from the placement predicate.

The direct entity-type tag `freeze_immune_entity_types` makes generic
`canFreeze()` false. Unlike concrete Skeleton, Stray does not override the
method; it reaches the same false result through the reloadable tag.

**Transition and ordering:**

### Shared skeleton combat

`AbstractSkeleton` supplies the complete goal graph: Restrict Sun at
priority `2`; Flee Sun and Wolf avoidance at `3`; equipment-selected Bow or
melee combat at `4`; Water-Avoiding Random Stroll at `5`; Player look and
random look at `6`; Hurt-By target at `1`, visible Player at `2`, and
visible Iron Golem plus baby-on-land Turtle at `3`.

Finalization runs generic Monster equipment, unconditionally puts a Bow in
the main hand, applies difficulty-scaled enchantments, reassesses combat,
rolls loot pickup at `0.55*specialMultiplier`, and may add zero-drop
Halloween headgear through the `0.25` then `0.1` draws. Stray changes none
of that order.

Exact Bow selects the cached priority-`4` ranged goal with speed `1`, radius
`15`, and minimum interval `20` on Hard or `40` otherwise. Any other weapon
selects the speed-`1.2` melee goal. Equipment changes and load reassess on
the server. Only Bow is a usable nonmelee and preferred weapon, and every
Spear pickup is rejected before generic comparison.

The shared ranged state machine uses squared radius `225`, `20` consecutive
visible ticks before strafing, sight floor `-60`, independent
`nextFloat()<0.3` strafe toggles every `20` strafe ticks, distance fractions
`0.75/0.25`, movement components `±0.5`, and a `20`-tick draw.

### Slowness arrow

`performRangedAttack` resolves the Bow hand and held projectile, then calls
the virtual `getArrow`. Stray delegates to the shared Mob-arrow factory.
Only when that result is the concrete `Arrow` class does it append
`MobEffectInstance(minecraft:slowness,600)`. Amplifier is `0` and all
visibility flags take defaults.

Spectral or any other non-`Arrow` implementation receives no added effect.
Stray itself has no Slowness immunity, so its own or another Stray's plain
arrow can slow it.

Aim remains
`dx=targetX-selfX`, `dz=targetZ-selfZ`,
`dy=target.getY(1/3)-arrowY+sqrt(dx²+dz²)*0.20000000298023224`.
Power is `1.6`; uncertainty is `14-4*difficultyId`. The shared method then
requests Skeleton Shoot, protocol sound ID `1491`, volume `1`, pitch
`1/(0.8+0.4*nextFloat)`, even when no server projectile was inserted.

### Powder Snow and freezing

The two immunity channels have distinct effects:

- `stray_immune_to` makes Powder Snow nondangerous to the Stray entity type
  for block-danger/path consumers;
- `freeze_immune_entity_types` makes `canFreeze()` false, so Powder Snow
  cannot increase frozen ticks and full-freeze damage is rejected.

Existing positive `TicksFrozen`, including externally loaded state, still
decays by two per eligible living tick. `AbstractSkeleton.isShaking()` is
`isFullyFrozen()`, so an externally supplied fully frozen Stray may
temporarily shake while the value decays even though it cannot acquire new
freeze state or take the periodic freeze damage.

Stray has no Skeleton conversion counters or metadata and never converts
further in Powder Snow. The reverse producer belongs to
`ENT-SKELETON-001`: after Skeleton exposure `140` and its remaining-timer
expiry, single conversion creates a Stray while retaining equipment,
pickup state and team.

Direct `burn_in_daylight` membership retains the generic monsters-burn,
light/random/weather/sky and headgear transaction. Empty-headed admission
ignites for eight seconds; any head item protects, with damageable headgear
spending `nextInt(2)` durability.

### Placement and production

Stray registers `ON_GROUND` with
`MOTION_BLOCKING_NO_LEAVES` and `checkStraySpawnRules`. Given candidate
`pos`, the predicate:

1. starts a mutable position at `pos`;
2. moves it above at least once;
3. while that cell is exact Powder Snow, moves above again;
4. evaluates ordinary Monster spawn rules at the original `pos`; and
5. when the reason is not any spawner reason, requires sky visibility at
   the cell below the first non-Powder-Snow cell.

With no Powder Snow above the candidate, the sky check is therefore at
`pos`. With a contiguous column above it, the check is at the topmost
Powder-Snow cell. The predicate does not itself require any Powder Snow:
sky-visible ordinary terrain can pass. Spawner reasons bypass only the sky
test, not ordinary Monster rules.

Exactly two of the `66` locked biomes carry Stray: Ice Spikes and Snowy
Plains. Each Monster list gives it weight `80`, group `4..4`, alongside
Skeleton weight `20`, group `4..4`. Every other biome has zero Stray rows.
Natural category cap, cluster and despawn behavior remain
`MOB-SPAWN-001`/`MOB-DESPAWN-001`.

Skeleton conversion is the second ordinary producer. It does not run Stray
finalization: the shared conversion copies retained state into the new
entity instead. Commands, Eggs and custom spawners retain their explicit
reason/finalization owners.

Four locked Trial-Spawner configurations contain only Stray at spawn
potential weight `1`:

- `trial_chamber/ranged/stray/normal`: simultaneous `3`, added per player
  `0.5`, interval `20`;
- its ominous form adds ranged Trial equipment with every slot drop chance
  zero and ejection weights key `3`, consumables `7`;
- `trial_chamber/slow_ranged/stray/normal`: simultaneous `4`, added per
  player `2`, interval `160`; and
- its ominous form adds the same equipment and ejection data.

Trial defaults retain total `6+2p`, spawn range `4` and cooldown `36,000`
through the Trial-Spawner owner.

The two normal configuration keys occur exactly once each in
`trial_chambers/spawner/ranged/stray.nbt` and
`trial_chambers/spawner/slow_ranged/stray.nbt`. Exact scans of all `1,212`
templates find zero literal `minecraft:stray` or plain `stray` entity
payloads.

### Loot, tags and progression

The entity loot table uses random sequence `minecraft:entities/stray` and
three ordered one-roll pools:

1. Arrow, item ID `923`, integer-uniform count `0..2` plus uniform
   Looting increase;
2. Bone, item ID `1112`, integer-uniform count `0..2` plus uniform
   Looting increase; and
3. only on a player kill, Tipped Arrow, item ID `1323`, with the Slowness
   potion, base integer-uniform `0..1` and Looting increase capped at final
   count `1`.

Only the third pool has the player-kill gate and count cap. Generic eligible
XP begins at `5` and may add `1+nextInt(3)` for each qualifying equipped
item.

Stray's four direct entity-type tags are `burn_in_daylight`,
`freeze_immune_entity_types`, `no_anger_from_wind_charge` and `skeletons`.
`skeletons` joins `undead`, then `can_breathe_under_water`,
`ignores_poison_and_regen`, `inverted_healing_and_harm`,
`sensitive_to_smite` and `wither_friends`. Generic consumers own those
effects. The block tag is not an entity-type tag and remains a separate
registration input.

Both hostile-mob advancements have an exact Stray criterion:
`kill_a_mob` includes it in the hostile OR group and `kill_all_mobs` gives
it its own required group. `sniper_duel` tests exact Skeleton, so Stray does
not satisfy it.

The common Stray Spawn Egg is protocol item ID `1209`, maximum stack `64`,
with `entity_data.id=minecraft:stray`.

### Compatibility, sounds and client projection

Legacy `EntitySkeletonSplitFix` renames old `Skeleton` with
`SkeletonType=2` to `Stray`; `EntityIdFix` maps `Stray` to
`minecraft:stray`. Schema `V705` registers the Mob shape and maps
`minecraft:stray_spawn_egg` to Stray.
`TrialSpawnerConfigInRegistryFix.VanillaTrialChambers` recognizes both old
inline Stray Trial families and maps each normal/ominous pair to registry
keys. Generic UUID, statistics and Spawn-Egg fixes retain their owners.

Locked sounds are:

| Protocol ID | Event | English subtitle |
|---:|---|---|
| `1605` | Stray Ambient | “Stray rattles” |
| `1606` | Stray Death | “Stray dies” |
| `1607` | Stray Hurt | “Stray hurts” |
| `1608` | Stray Step | none |

Inherited step playback uses volume `0.15`, pitch `1`. Ranged release uses
Skeleton Shoot rather than a Stray event. Parrot imitation maps Stray to ID
`1242`, subtitle “Parrot rattles”; the Parrot owns cadence, selection,
silence, volume and pitch.

`EntityRenderers` binds `StrayRenderer`. The base uses the Stray and
Stray-Armor model layers, shared Skeleton model, humanoid armor layer and
shadow radius `0.5`. Aggression, inherited fully-frozen shaking and exact
main-hand Bow identity populate `SkeletonRenderState`; only an aggressive
main arm holding a main-hand Bow takes `BOW_AND_ARROW`.

An always-present `SkeletonClothingLayer` copies render state into a second
Skeleton model and submits a white cutout layer. `STRAY_OUTER_LAYER` is a
`64×32` humanoid mesh with uniform cube deformation `0.25`.

Exact textures are:

| Path | Dimensions | Bytes | SHA-256 |
|---|---:|---:|---|
| `textures/entity/skeleton/stray.png` | `64×32` | `430` | `d2f050b01ac0eb319d208db00f53081e2b87ce909be0468802d0ad5ba960b0e1` |
| `textures/entity/skeleton/stray_overlay.png` | `64×32` | `439` | `0014fbd0fda580f5d3bf97c2bfd761f2ff3d07da50a3d52766c74b0ce8c35823` |

English labels are “Stray” and “Stray Spawn Egg”.

**Branches and aborts:**

- Slowness is appended only when the Mob-arrow factory returned concrete
  `Arrow`.
- Block-danger immunity, freezing immunity and placement are three
  independent tag/code paths.
- The placement scan always reads at least the cell above the candidate.
- Nonspawner reasons require sky visibility; spawner reasons bypass that
  test only.
- Natural biome selection exists in exactly two rows.
- Only the Tipped-Arrow pool is player-kill gated and capped.
- Conversion-created Strays preserve source state instead of ordinary
  finalization.

**Invariants:**

- Stray adds no metadata or persisted field.
- Powder Snow is nondangerous and cannot newly freeze a Stray.
- Every plain `Arrow` fired by a Stray carries Slowness for `600` ticks.
- The shared Bow and melee goals remain mutually exclusive at priority `4`.
- Only Ice Spikes and Snowy Plains naturally select Stray.
- No locked structure template contains a literal Stray entity payload.
- Stray never satisfies Skeleton-only Sniper Duel.

**Constants and randomness:**

Entity/Egg IDs `128/1209`; dimensions/eye/riding `0.6×1.99/1.74/-0.7`;
range/update `8/3`; health/speed/attack/follow/XP `20/0.25/2/16/5`;
Bow radius/interval `15/20-or-40`; shot `1.6`,
`14-4*difficultyId`, lift `0.20000000298023224`, Slowness `600`;
daylight fire `8` seconds; biomes `2/66`, weight/group `80/4..4`;
Trial simultaneous/add/interval `3/.5/20` and `4/2/160`; loot
`0..2/0..2/0..1`, third-pool cap `1`; outer deformation `0.25`;
textures `64×32`.

**Side effects:**

Equipment and goal selection; targets, navigation, look and aggression; RNG
for finalization, combat and sound pitch; arrows with attached effect;
daylight headgear/fire; freeze-counter decay; natural/conversion/Trial
entity production; loot, XP and advancement progress; sounds, armor, base
and clothing render submissions.

**Gates:**

Logical side, Peaceful, NoAI and persistence; held weapon/projectile class;
difficulty, distance, sight and draw timing; live block/entity tags;
Powder-Snow column, Monster rules, spawn reason and sky visibility; biome
and Trial selectors; player kill, Looting and mob loot; resources.

**Boundary cases and quirks:**

The custom spawn predicate does not require Powder Snow. A sky-visible Stray
candidate with none above it passes the subtype sky gate at the original
cell. A tall Powder-Snow column instead moves the sky probe to its topmost
Powder cell. Spawners still evaluate Monster rules even though they bypass
the sky probe.

Freeze immunity prevents acquisition and damage but does not forcibly zero
persisted frozen ticks; those decay by two and can transiently drive the
inherited shaking render. Offhand Bow can select ranged combat through the
weapon-holding-hand helper, but the client Bow pose tests exact main-hand
Bow. Stray's step event has no subtitle, and its ranged release deliberately
sounds like Skeleton.

**Failure semantics:**

A non-`Arrow` projectile is returned unchanged. Projectile insertion failure
does not suppress Skeleton Shoot. Failed conversion insertion, natural
spawn, Trial spawn, loot insertion and advancement persistence retain their
cited owners and do not create Stray-specific rollback. Reloaded tags can
change block/freeze admission without reconstructing the entity.

**Client/server authority split:**

The server owns goals, equipment, placement, spawning, arrows and attached
Slowness, daylight, freeze-counter changes, loot and advancements. Clients
consume inherited metadata/equipment/movement, render aggression, frozen
shaking, Bow pose, armor and clothing, and play sounds. There is no
Stray-specific metadata packet.

**Observability:**

Observe registration and both immunity inputs; absence of subtype
metadata/NBT; shared goals/equipment and every projectile class; Slowness
attachment; Powder-Snow danger/freeze/decay separation; every placement
column/reason/sky boundary; both biome rows, Skeleton conversion and four
Trial configs; loot/XP/tags/criteria/Egg; migration and template censuses;
species/Skeleton-shoot/Parrot sounds; base, armor and deformed clothing
layers with exact resources.

**Persistence and reload:**

Only generic entity/Mob/equipment and frozen-tick state persists. Tags,
biomes, Trial configs, loot and advancements reload server-side.
Block-danger and freeze immunity follow the new tag snapshot. Language,
models and textures reload client-side. An arrow's attached Slowness
persists with the projectile, not the Stray.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.Entity`;
`net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.skeleton.AbstractSkeleton`;
`net.minecraft.world.entity.monster.skeleton.Skeleton`;
`net.minecraft.world.entity.monster.skeleton.Stray`;
`net.minecraft.world.entity.projectile.arrow.Arrow`;
`net.minecraft.world.entity.projectile.ProjectileUtil`;
`net.minecraft.world.entity.ai.goal.RangedBowAttackGoal`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerConfigs`;
`net.minecraft.util.datafix.fixes.EntitySkeletonSplitFix`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix`;
`net.minecraft.util.datafix.schemas.V705`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.StrayRenderer`;
`net.minecraft.client.renderer.entity.AbstractSkeletonRenderer`;
`net.minecraft.client.renderer.entity.layers.SkeletonClothingLayer`;
`net.minecraft.client.renderer.entity.state.SkeletonRenderState`;
`net.minecraft.client.model.monster.skeleton.SkeletonModel`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,
loot_table,worldgen/biome,advancement}`;
`reports/minecraft/components/item/stray_spawn_egg.json`;
`data/minecraft/tags/block/stray_immune_to.json`;
`data/minecraft/tags/entity_type/{burn_in_daylight,
freeze_immune_entity_types,no_anger_from_wind_charge,skeletons,undead,
can_breathe_under_water,ignores_poison_and_regen,
inverted_healing_and_harm,sensitive_to_smite,wither_friends}.json`;
`data/minecraft/loot_table/entities/stray.json`;
`data/minecraft/worldgen/biome/{ice_spikes,snowy_plains}.json`;
`data/minecraft/trial_spawner/trial_chamber/{ranged,slow_ranged}/
stray/{normal,ominous}.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/textures/entity/skeleton/{stray,stray_overlay}.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-SKELETON-001`; `ENT-DEATH-001`; `BLK-SNOW-FAMILY-001`;
`ITM-ARROW-AMMUNITION-001`; `ITM-BONE-001`; `ITM-ENCHANT-001`;
`ITM-ADVANCEMENT-001`; `CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-035` across raw/finalized/conversion/loaded construction,
shared goals and both weapon branches, every projectile result and exact
Slowness attachment, block-danger/freeze/decay separation, all placement
column/reason/sky boundaries, both biomes and all four Trial configs, three
loot pools, tags/criteria/Egg, compatibility/template census, every sound
and exact base/armor/clothing projection.

**Limits:**

Generic lifecycle, metadata, equipment finalization, goal and ranged
state-machine internals, projectile flight/hit/effect application, daylight,
freeze-counter runtime, Skeleton conversion, natural/Trial spawning,
death/equipment/XP, loot, advancements, Spawn-Egg interaction and renderer
submission retain their cited owners. This leaf owns the Stray selectors,
overrides, inputs, constants, resource joins and their exact composition.
