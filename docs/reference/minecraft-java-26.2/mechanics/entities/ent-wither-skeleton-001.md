# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-WITHER-SKELETON-001` — Wither Skeletons carry Stone Swords, inflict Wither and ignite every fired arrow

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-SKELETON-001`,
`MOB-001`, `MOB-AI-001`, `MOB-002`, `MOB-SPAWN-001`, `MOB-003`,
`MOB-DESPAWN-001`, `ITM-ARROW-AMMUNITION-001`, `ITM-BONE-001`,
`ITM-COAL-001`, `ITM-ENCHANT-001`, `ITM-ADVANCEMENT-001`,
`BLK-SKULL-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PORTAL-001`, `WGEN-STRUCTURE-FORTRESS-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `WitherSkeleton`
class, effective `AbstractSkeleton` and Mob behavior, ordinary placement,
Nether-Fortress selection, both skull-drop channels, tags, progression,
Spawn Egg, compatibility and exact scaled client projection close protocol
entity ID `147`.

**Applies when:**

`minecraft:wither_skeleton` is constructed, finalized, selected by a Nether
Fortress spawn override, spawned by Egg, spawner, command or custom code,
choosing equipment or targets, navigating Lava, attacking in melee or with
a supplied Bow, receiving fire, freezing or Wither, burning in daylight,
dying, synchronized, heard, imitated by a Parrot or rendered.

**Authoritative state:**

Protocol entity ID `147` constructs `WitherSkeleton` in the non-Peaceful
`MONSTER` category. Registration makes the type fire immune and fixes
dimensions `0.7×2.4`, eye height `2.1`, riding offset `-0.875`, client
tracking range `8` and default update interval `3`.

The raw `AbstractSkeleton` attributes are maximum health `20`, movement
speed `0.25`, attack-damage base `2` and follow range `16`. Monster
construction sets nominal XP reward `5`. Successful Wither-Skeleton
finalization changes the attack-damage base to `4`; its default Stone Sword
then contributes the item component's main-hand `+4`, producing effective
attack damage `8` before enchantment or other modifiers.

Wither Skeleton adds no synchronized metadata and no persisted subtype
field. Its complete metadata layout is inherited Entity, Living-Entity and
Mob slots `0..15`; `AbstractSkeleton` also adds no field. Generic
equipment, attributes, effects, frozen ticks and lifecycle state retain
their owners.

Construction sets the Lava path-type malus to `8`. Fire immunity does not
make Lava cost-free: path search sees that positive cost while fire-tagged
damage admission rejects the damage.

Registration also binds
`immuneTo(#minecraft:wither_skeleton_immune_to)`. The locked block tag
contains only Wither Rose. That block-danger exemption is separate from
the exact Wither-effect rejection below.

**Transition and ordering:**

### Goals, equipment and finalization

Before installing the shared skeleton goals, `registerGoals` inserts at
target priority `3` a visible-target goal for `AbstractPiglin`, covering
ordinary Piglins and Piglin Brutes. The inherited target graph then adds
Hurt-By at `1`, visible Player at `2`, and visible Iron Golem plus
baby-on-land Turtle at `3`. Same-priority insertion therefore puts the
Piglin selector before the two inherited priority-`3` selectors.

The movement graph remains Restrict Sun at priority `2`; Flee Sun and Wolf
avoidance at `3`; equipment-selected Bow or melee combat at `4`;
Water-Avoiding Random Stroll at `5`; and Player look plus random look at
`6`.

Finalization has subtype-sensitive ordering:

1. generic Monster finalization establishes its ordinary random state;
2. the virtual equipment hook installs exactly one Stone Sword in main
   hand and does not call generic armor population;
3. the virtual equipment-enchantment hook is a no-op;
4. the inherited skeleton finalizer reassesses combat, rolls loot pickup
   at `0.55*specialMultiplier`, and may add zero-drop Halloween headgear
   through the `0.25` then `0.1` draws;
5. the Wither-Skeleton finalizer changes attack-damage base `2` to `4`; and
6. it reassesses the weapon goal again.

Thus ordinary finalized instances have no random armor or equipment
enchantments, but may have the inherited Halloween head item. Raw,
conversion-like or custom construction that bypasses finalization retains
base attack `2` and receives no Stone Sword from this path.

`getPreferredWeaponType()` returns null. `canHoldItem` rejects every member
of `wither_skeleton_disliked_weapons`, whose locked values are Bow and
Crossbow, before delegating to generic admission. The inherited
`wantsToPickUp` separately rejects every Spear before generic comparison.
These are pickup/equipment-admission rules, not a prohibition on external
equipment mutation.

A command, load or custom callback can still equip a Bow. Exact Bow then
selects the cached priority-`4` ranged goal; every other held weapon selects
the speed-`1.2` melee goal. The Bow goal retains speed `1`, radius `15` and
minimum interval `20` on Hard or `40` otherwise, with the shared sight,
strafing and draw state machine from `ENT-SKELETON-001`.

### Melee Wither and effect immunity

`doHurtTarget` first executes the generic Mob attack. On a finalized
default instance, the base `4` plus Stone Sword `+4` is read through the
live attack attribute before enchantment and target admission.

If generic damage fails, the method returns false and creates no effect.
If damage succeeds and the target is a Living Entity, it attempts to add
Wither for `200` ticks, amplifier `0`, with this Wither Skeleton as source.
The returned effect-admission result is ignored. A successful attack
against a nonliving entity simply returns true without the effect attempt.

`canBeAffected` rejects an exact Wither effect before delegating all other
effects. Consequently the mob is immune to the status effect it attempts
to apply, independently of its fire immunity, undead-transitive tags and
Wither-Rose block exemption.

### Externally supplied Bow

The shared ranged method resolves the Bow hand and projectile, then calls
the virtual arrow factory. Wither Skeleton delegates to the ordinary
Mob-arrow factory and unconditionally calls `igniteForSeconds(100)` on
the returned `AbstractArrow`. Every supported arrow implementation,
not only concrete `Arrow`, receives `2,000` fire ticks.

Aim remains
`dx=targetX-selfX`, `dz=targetZ-selfZ`,
`dy=target.getY(1/3)-arrowY+sqrt(dx²+dz²)*0.20000000298023224`.
Power is `1.6`; uncertainty is `14-4*difficultyId`. The shared method then
requests Skeleton Shoot, protocol sound ID `1491`, volume `1`, pitch
`1/(0.8+0.4*nextFloat)`, even when server projectile insertion failed.

Fire-arrow behavior is reachable only through externally supplied
equipment in the locked default game: finalization gives a Stone Sword,
pickup rejects Bow and Crossbow, and preferred weapon type is null.

### Fire, daylight and freezing

Type-level fire immunity makes fire-tagged damage inadmissible and makes
`isOnFire()` false. The direct `burn_in_daylight` tag nevertheless runs the
generic sunlight transaction.

On an eligible sunlight tick, damageable headgear still spends
`nextInt(2)` durability and may break, despite the mob needing no fire
protection. Any nonempty nondamageable head item returns without ignition.
With an empty head, the path assigns an eight-second fire counter; fire
immunity suppresses the burning flag and damage, and the next Entity base
tick clears the positive counter.

Wither Skeleton has no freeze-immunity tag or override. Generic armor and
spectator gates therefore control `canFreeze`; full freezing can deal the
ordinary periodic freeze damage. The inherited
`AbstractSkeleton.isShaking()` mirrors `isFullyFrozen()`, so that state is
projected to clients.

### Placement and production

Wither Skeleton registers `ON_GROUND` with
`MOTION_BLOCKING_NO_LEAVES` and ordinary
`Monster.checkMonsterSpawnRules`. It does not use the Blaze any-light
predicate or Stray's sky-column predicate. The generic predicate owns
difficulty, spawn reason and darkness admission.

None of the `66` locked biome Monster lists contains Wither Skeleton.
Natural production comes from the Nether Fortress structure override. Its
piece-bounded Monster list contains:

| Type | Weight | Group |
|---|---:|---:|
| Blaze | `10` | `2..3` |
| Zombified Piglin | `5` | `4..4` |
| Wither Skeleton | `8` | `5..5` |
| Skeleton | `2` | `5..5` |
| Magma Cube | `3` | `4..4` |

The structure is selected through
`#minecraft:has_structure/nether_fortress`, generates at
`underground_decoration`, and uses `bounding_box=piece` for this override.
Ordinary category cap, collision, placement, cluster and despawn behavior
remain with the natural-spawn owners.

There is no locked Wither-Skeleton Trial-Spawner configuration. Exact scans
of all `1,212` structure templates find zero literal
`minecraft:wither_skeleton` or plain `wither_skeleton` entity payloads.
Commands, Eggs, conventional spawners and custom factories retain their
explicit reason, finalization and insertion owners.

### Loot, skull channels and progression

The ordinary entity loot table uses random sequence
`minecraft:entities/wither_skeleton` and three ordered one-roll pools:

1. Coal, item ID `924`, receives integer-uniform set count `-1..1`, then a
   uniform Looting increase;
2. Bone, item ID `1112`, receives integer-uniform set count `0..2`, then a
   uniform Looting increase; and
3. only on a player kill and a successful enchanted-chance draw, one Wither
   Skeleton Skull, item ID `1264`.

The skull chance is `0.025` without Looting. With Looting level `L>=1`, its
linear provider is `0.035+0.01*(L-1)`: `0.035`, `0.045`, `0.055` at the
ordinary first three levels. The player-kill and chance conditions precede
the item entry.

A powered Creeper that passes its shared one-skull transaction dispatches
an exact Wither-Skeleton victim through
`minecraft:charged_creeper/root`. The selected
`charged_creeper/wither_skeleton` table unconditionally emits one Wither
Skeleton Skull and uses its own random sequence. This channel is separate
from the ordinary entity table; Creeper admission and the
`droppedSkulls` latch remain with `ENT-DEATH-001`.

Generic eligible XP begins at `5` and can add `1+nextInt(3)` for each
qualifying equipped item. Wither Skeleton appears in the hostile OR group
of `kill_a_mob` and has its own required criterion in `kill_all_mobs`.
`sniper_duel` requires exact Skeleton and rejects Wither Skeleton.

Acquiring its skull satisfies the inventory-changed criterion for
`nether/get_wither_skull`; the same item can unlock the Skull Banner
Pattern recipe through its recipe advancement. Skull block placement,
Wither construction and banner crafting remain with their respective
owners.

The common Wither Skeleton Spawn Egg is item ID `1211`, maximum stack `64`,
with `entity_data.id=minecraft:wither_skeleton`.

### Tags, compatibility, sounds and client projection

The only direct entity-type tags are `burn_in_daylight` and `skeletons`.
`skeletons` joins `undead`, then `can_breathe_under_water`,
`ignores_poison_and_regen`, `inverted_healing_and_harm`,
`sensitive_to_smite` and `wither_friends`. Generic consumers own those
effects. The Wither-Rose block tag and disliked-weapons item tag are
separate live reload inputs.

Legacy `EntitySkeletonSplitFix` renames old `Skeleton` with
`SkeletonType=1` to `WitherSkeleton`; `EntityIdFix` maps that identifier to
`minecraft:wither_skeleton`. Schema `V705` registers the Mob shape and
maps `minecraft:wither_skeleton_spawn_egg` to Wither Skeleton. Generic UUID,
statistics, equipment and Spawn-Egg fixes retain their owners.

Locked sound joins are:

| Protocol ID | Event | English subtitle |
|---:|---|---|
| `1785` | Wither Skeleton Ambient | “Wither Skeleton rattles” |
| `1786` | Wither Skeleton Death | “Wither Skeleton dies” |
| `1787` | Wither Skeleton Hurt | “Wither Skeleton hurts” |
| `1788` | Wither Skeleton Step | none |

Inherited step playback uses volume `0.15`, pitch `1`; externally enabled
ranged release uses Skeleton Shoot. Parrot imitation maps Wither Skeleton
to event ID `1248`, subtitle “Parrot rattles”; the Parrot owns cadence,
selection, silence, volume and pitch.

`EntityRenderers` binds `WitherSkeletonRenderer`. It uses the shared
Skeleton model and humanoid armor layer with shadow radius `0.5`. The
Wither-Skeleton base and every armor model layer apply uniform mesh scaling
`1.2`; there is no Stray-style clothing layer.

Render state copies aggression, inherited fully-frozen shaking and exact
main-hand Bow identity. Only an aggressive main arm holding a main-hand Bow
takes `BOW_AND_ARROW`; the default Stone Sword follows the shared melee
animation.

The exact texture
`textures/entity/skeleton/wither_skeleton.png` is `64×32`, `496` bytes,
SHA-256
`0be837feee8359ee76df0bc4ba6f423053e5683f9c7d6d553a255fe14f2b8e1d`.
English labels are “Wither Skeleton” and “Wither Skeleton Spawn Egg”.

**Branches and aborts:**

- Failed base melee damage suppresses the Wither-effect attempt.
- Successful nonliving damage has no effect target but still returns true.
- Effect rejection does not roll back successful melee damage.
- Disliked-weapon and Spear rejection are independent pickup gates.
- External Bow equipment still selects ranged combat and ignites every
  returned arrow implementation.
- Sunlight headgear mutation still runs on this fire-immune type.
- Natural selection has zero biome rows and one Fortress override row.
- Ordinary and charged-Creeper skull tables are separate death channels.

**Invariants:**

- Wither Skeleton adds no metadata or persisted field.
- Finalization always installs a Stone Sword and never random armor or
  equipment enchantments through its virtual hooks.
- Finalized attack-damage base is `4`; the default sword makes live attack
  damage `8`.
- Every admitted melee hit on a Living Entity attempts Wither for `200`
  ticks.
- Exact Wither effects are always rejected by this mob.
- Every arrow made for an externally supplied Bow receives `2,000` fire
  ticks.
- No biome list, Trial config or structure template literally produces the
  entity outside the Fortress override.

**Constants and randomness:**

Entity/Egg IDs `147/1211`; dimensions/eye/riding
`0.7×2.4/2.1/-0.875`; range/update `8/3`; health/speed/raw attack/final
base/sword modifier/follow/XP `20/0.25/2/4/+4/16/5`; Lava malus `8`;
Piglin target priority `3`; Bow radius/interval `15/20-or-40`; shot
`1.6`, `14-4*difficultyId`, lift `0.20000000298023224`, arrow fire
`100` seconds; Wither duration `200`; daylight fire `8` seconds; Fortress
weight/group `8/5..5`; loot set counts `-1..1/0..2`, skull chance
`0.025` or `0.035+0.01*(L-1)`; render scale `1.2`; texture `64×32`.

**Side effects:**

Equipment and attack-attribute modifiers; target and combat-goal selection;
navigation cost; melee damage, Wither-effect attempts and fire-arrow state;
sunlight headgear/fire-counter mutation; freeze state; natural/spawner/Egg
entity production; ordinary and charged-Creeper loot, XP, advancements and
recipe unlock; sounds, armor and scaled model rendering.

**Gates:**

Logical side, Peaceful, NoAI and persistence; finalization and live
equipment; target class, visibility and goal arbitration; damage/effect
admission; Bow/projectile type, range, sight and draw timing; fire/freeze
state; live block/item/entity tags; Monster placement, Fortress piece and
category selectors; player kill, Looting, powered Creeper and mob loot;
resources.

**Boundary cases and quirks:**

Fire immunity does not skip the sunlight headgear transaction. A damageable
helmet may lose durability in sunlight even though the entity cannot burn;
an empty head briefly receives a positive fire counter that cannot project
as burning and is cleared on the next base tick.

The disliked-weapons tag prevents ordinary Bow and Crossbow pickup, while
the combat and renderer paths still fully support a Bow supplied by command,
load or custom code. Offhand Bow can select ranged combat through the
weapon-holding-hand helper, but the client Bow pose tests exact main-hand
Bow.

The Stone Sword modifier is part of the live attack attribute. Replacing or
removing the sword changes damage without changing the finalized base `4`.
The fixed Wither attempt follows damage admission and ignores its own
admission result.

**Failure semantics:**

Failed melee damage performs no subtype effect. Rejected Wither leaves the
already admitted hit intact. Projectile insertion failure does not undo
the arrow's fire state or suppress Skeleton Shoot. Failed natural,
spawner, Egg or custom insertion, loot insertion and advancement
persistence retain their cited owners and do not create subtype rollback.
Reloaded block/item/entity tags affect later checks without reconstructing
the mob.

**Client/server authority split:**

The server owns goals, equipment, attributes, placement, Fortress spawning,
melee and effects, projectile construction/fire state, sunlight, freezing,
loot and progression. Clients consume inherited metadata, equipment and
movement, render aggression/frozen shaking/Bow pose with scaled base and
armor models, and play sounds. There is no Wither-Skeleton-specific
metadata packet.

**Observability:**

Observe raw and finalized attributes/equipment; exact goal insertion and
weapon admission; Lava cost; melee damage/effect result; every arrow
implementation; fire/daylight/freeze boundaries; ordinary placement,
all biome rows and the Fortress override; template/Trial absence; ordered
ordinary loot and charged-Creeper dispatch; tags, criteria, Egg and
compatibility; every sound and exact scaled model/resource.

**Persistence and reload:**

Only generic entity, Mob, attribute, equipment, effect and frozen state
persists. No subtype NBT exists. Block, item and entity tags, Fortress
structure data, loot and advancements reload server-side. Language, models
and texture reload client-side. Fired-arrow fire state persists with the
projectile rather than its shooter.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.Entity`;
`net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.skeleton.AbstractSkeleton`;
`net.minecraft.world.entity.monster.skeleton.WitherSkeleton`;
`net.minecraft.world.entity.monster.piglin.AbstractPiglin`;
`net.minecraft.world.entity.projectile.arrow.AbstractArrow`;
`net.minecraft.world.entity.projectile.ProjectileUtil`;
`net.minecraft.world.entity.ai.goal.RangedBowAttackGoal`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.util.datafix.fixes.EntitySkeletonSplitFix`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.schemas.V705`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.WitherSkeletonRenderer`;
`net.minecraft.client.renderer.entity.AbstractSkeletonRenderer`;
`net.minecraft.client.renderer.entity.state.SkeletonRenderState`;
`net.minecraft.client.model.monster.skeleton.SkeletonModel`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,
loot_table,worldgen/biome,worldgen/structure,advancement}`;
`reports/minecraft/components/item/{stone_sword,
wither_skeleton_spawn_egg}.json`;
`data/minecraft/tags/block/wither_skeleton_immune_to.json`;
`data/minecraft/tags/item/wither_skeleton_disliked_weapons.json`;
`data/minecraft/tags/entity_type/{burn_in_daylight,skeletons,undead,
can_breathe_under_water,ignores_poison_and_regen,
inverted_healing_and_harm,sensitive_to_smite,wither_friends}.json`;
`data/minecraft/loot_table/{entities/wither_skeleton,
charged_creeper/root,charged_creeper/wither_skeleton}.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/worldgen/structure/fortress.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/advancement/nether/get_wither_skull.json`;
`data/minecraft/advancement/recipes/misc/skull_banner_pattern.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/textures/entity/skeleton/wither_skeleton.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-SKELETON-001`; `ENT-DEATH-001`; `BLK-SKULL-001`;
`WGEN-STRUCTURE-FORTRESS-001`; `ITM-ARROW-AMMUNITION-001`;
`ITM-BONE-001`; `ITM-COAL-001`; `ITM-ENCHANT-001`;
`ITM-ADVANCEMENT-001`; `CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-036` across raw/finalized/loaded construction, every target and
weapon branch, damage/effect admission, every arrow implementation,
fire/daylight/freeze state, placement and Fortress boundaries, both loot
channels, tags/progression/Egg/compatibility and all scaled client states.

**Limits:**

Generic lifecycle, equipment and attribute application, shared skeleton
goals/ranged state, projectile flight/hit/fire damage, status-effect
runtime, sunlight and freezing, natural/structure/spawner production,
death/XP, loot, advancements, Skull/Wither behavior, Spawn-Egg interaction
and renderer submission retain their cited owners. This leaf owns the
Wither-Skeleton selectors, overrides, constants and their exact composition.
