# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-VINDICATOR-001` - Vindicators latch Johnny targeting and break raid doors with replaceable Iron Axes

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`,
`MOB-002`, `MOB-SPAWN-001`, `MOB-PATROL-001`, `MOB-RAID-001`,
`MOB-003`, `MOB-DESPAWN-001`, `ITM-IRON-MATERIAL-001`,
`ITM-EMERALD-001`, `ITM-ENCHANT-001`, `ITM-ADVANCEMENT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` - locked registration, `Vindicator`, its two private
goals, inherited Raider/Illager behavior, door goals, raid and
Woodland-Mansion producers, loot, enchantment providers, advancements,
tags, migrations, sounds and the exact client renderer close protocol
entity ID `141`.

**Applies when:**

`minecraft:vindicator` is constructed, finalized, spawned by a raid,
Woodland Mansion, Spawn Egg, command, spawner or custom selector, assigned
or removed from a raid, opening or breaking a door, targeting in ordinary
or Johnny mode, renamed, saved, loaded, killed, synchronized, heard,
imitated by a Parrot or rendered.

**Authoritative state:**

Protocol entity ID `141` constructs `Vindicator` in `MONSTER`;
registration excludes it from Peaceful. Its scalable dimensions are
`0.6x1.95`, with default eye height `1.95*0.85=1.6575`, one passenger
attachment at `(0,2,0)`, riding offset `-0.6`, client tracking range `8`
and default update interval `3`. It is neither fire-immune nor
persistence-required.

Attributes start from the Monster set and fix movement speed
`0.3499999940395355`, follow range `12`, maximum health `24` and attack
damage `5`. Construction inherits XP reward `5`. A finalized nonraid
Vindicator normally holds Iron Axe item protocol ID `962`; its main-hand
`+8` attack-damage modifier makes the ordinary live attack attribute `13`
before difficulty, enchantment, defense and damage-pipeline effects.

The subtype is an `Enemy`, has no age, breeding or ordinary interaction
path and cannot be leashed through generic lead interaction. Monster
category, cluster, despawn and obstruction behavior retain their generic
owners. Patrolling-Monster and Raider state can make it persistent or
change its distance-despawn result.

Entity, Living-Entity and Mob metadata occupy slots `0..15`. Raider adds
slot `16`, serializer ID `8` (`BOOLEAN`), `celebrating=false`.
Vindicator adds no synchronized slot. Its private `isJohnny` boolean starts
false, is server-authoritative and is not sent directly to clients.

`isJohnny` saves as `Johnny=true` only when true; loading reads
`Johnny` with default false. Generic custom name saves independently.
Target, aggression, goal state, door position/progress, navigation
capabilities and raid-enchantment draw state are transient. Generic
equipment, patrol target/leader/patrolling, Raider wave/join/raid
association and persistence save through their owners.

**Transition and ordering:**

### Goal graph and targeting

The effective goal selector contains inherited registrations first, then
the nine Vindicator registrations:

| Priority | Goal and fixed inputs |
|---:|---|
| `0` | Float |
| `1` | Obtain Raid Leader Banner |
| `1` | avoid Creaking within `8`, walk speed `1`, sprint speed `1.2` |
| `2` | Vindicator door breaking |
| `3` | Pathfind to Raid |
| `3` | Raider open-door goal, not held open |
| `4` | Long Distance Patrol at leader/follower speed `0.7/0.595` |
| `4` | move through village at speed `1.0499999523162842`, distance `1` |
| `4` | Raider hold-ground attack within `10` |
| `5` | Raider Celebration |
| `5` | melee attack at speed `1`, without following an unseen target |
| `8` | random stroll at speed `0.6` |
| `9` | look at Player within `3`, probability `1` |
| `10` | look at Mob within `8`, default probability |

Goal flags and priority arbitration remain `MOB-AI-001`; patrol,
hold-ground, banner, village, raid-path and celebration algorithms remain
`MOB-PATROL-001` and `MOB-RAID-001`.

The target selector is:

1. priority `1`: retaliate when hurt, ignore Raider attackers and alert
   other eligible Vindicators;
2. priority `2`: nearest visible Player;
3. priority `3`: nearest visible Abstract Villager, registered before a
   nearest visible Iron Golem at the same priority; and
4. priority `4`: while Johnny is true, nearest attackable Living Entity,
   scan interval `0`, with sight and reach required.

Johnny broadens the last target class but does not bypass common admission.
The shared Illager attack gate still rejects baby Abstract Villagers.
Scoreboard allies remain protected; when neither side has a team, an
entity in `minecraft:illager_friends` is allied, and that tag contains
`#minecraft:illager`. The additional predicate also requires the candidate
to report `attackable=true`.

The Johnny goal competes after the Player, adult Villager and Iron-Golem
goals. Its `start` clears `noActionTime` before ordinary target pursuit.
Melee pathing, reach, cooldown, swing, held-weapon damage, blocking
disablement and equipment durability retain their cited owners.

### Johnny latch

`setCustomName` delegates to the generic setter first. It then changes
`isJohnny` from false to true only when the nonnull component's flattened
string is exactly the case-sensitive text `Johnny`. Renaming or clearing
the name cannot turn an already true latch false. A later load can clear
it only by supplying `Johnny=false` or omitting the field; saving a true
latch writes both the generic custom name and `Johnny=true`.

Because the latch is not metadata, a client learns no Johnny bit. It sees
only authoritative consequences such as the acquired target, aggression,
movement and attack animation.

### Door admission and destruction

During each admitted custom server-AI step, a non-NoAI Vindicator with
ground path navigation sets `canOpenDoors` to the live result of
`ServerLevel.isRaided(blockPosition)`, then delegates to Raider AI. Thus a
finalized Vindicator initially receives `canOpenDoors=true`, but its next
custom AI step outside a raided position sets it false. Inside a raided
position, the priority-3 open-door goal additionally requires an active
Raid.

The priority-2 break goal admits in this order:

1. require an active Raid;
2. consume `nextInt(reducedTickDelay(10))` and require zero;
3. require the shared door-interaction admission;
4. require gamerule `minecraft:mob_griefing=true`;
5. require Normal or Hard difficulty; and
6. require the door not already open.

Starting resets both break progress and `noActionTime`. Continuation
rechecks active Raid, Normal/Hard, a closed door, distance below `2` from
the door center and `breakTime<=240`; it does not reread `mobGriefing`.
The constructor passes `6`, but `BreakDoorGoal#getDoorBreakTime` returns
`max(240,6)`, so the effective break time is `240` admitted goal ticks.

Each tick has an independent `nextInt(20)==0` branch that emits level event
`1019` and swings when not already swinging. It then increments
`breakTime`, projects integer crack stage
`floor(breakTime/240*10)` when changed, and at exactly `240` removes the
door without drops, emits level event `1021`, then emits block event `2001`
using the post-removal state read at that position. Stop clears projected
cracks with stage `-1`.

Turning `mobGriefing` off after start therefore does not stop an admitted
break. Losing the active Raid, opening the door, moving out of range or
leaving Normal/Hard does stop it. Peaceful and generic effective-AI
admission remain upstream.

### Finalization and raid weapon replacement

Vindicator first invokes its superclass finalization chain. Raider sets
`CanJoinRaid=true`; then, outside `PATROL`, `EVENT` and `STRUCTURE`,
Patrolling-Monster can consume its leader draw and equip the ominous head
banner. `PATROL` instead marks the entity patrolling, after which generic
Monster/Mob finalization completes and returns.

The subtype then sets navigation `canOpenDoors=true`, reads the level RNG,
populates equipment, and applies generic equipment enchantment:

1. if `getCurrentRaid()==null`, put a fresh Iron Axe in the main hand;
2. if a Raid is already associated, leave the main hand untouched here;
3. run generic armor/weapon spawn enchantment at current regional
   difficulty.

Raid joining establishes the Raid before `EVENT` finalization, so the
ordinary equipment hook supplies no axe. `applyRaidBuffs` then always
creates a fresh Iron Axe and always installs it in the main hand. Before
installation it consumes one entity `nextFloat` and enchants when the
draw is less than or equal to the Raid enchant odds. Omen levels
`2/3/4/5` supply odds `0.1/0.25/0.5/0.75`; other levels supply zero, for
which the exact float value zero still passes the inclusive comparison.

An admitted wave `1..5` uses provider `minecraft:raid/vindicator`,
Sharpness I. Wave `6` or later uses
`minecraft:raid/vindicator_post_wave_5`, Sharpness II. The threshold is
the fixed Normal ordinary-group count `5`, even when the live Raid
difficulty has a different group count. Both providers are single,
fixed-level providers; current regional difficulty is still passed to the
generic provider call. The `isCaptain` argument is unused.

The replacement happens after finalization and discards any prior
main-hand stack and its enchantments without a drop or rollback. A custom
caller that invokes the buff without a current Raid dereferences the
missing Raid; valid raid production establishes the association first.

### Production and placement

Ordinary Raid waves `1..7` request base Vindicator counts
`0,2,0,1,4,2,5`. Difficulty then adds the source-ordered random extra
described by `MOB-RAID-001`; omen bonus groups reuse the difficulty's final
ordinary-wave index. On wave seven and later, the first Ravager rider is
an Evoker and remaining rider positions are Vindicators. Each admitted
Vindicator is created with reason `EVENT`, associated with its Raid and
wave, finalized, buffed, and offered with passengers.

Woodland Mansion's 73 templates contain exactly 20 `Warrior` DATA markers
across 14 templates:

| Template | Markers |
|---|---:|
| `1x2_a1`, `1x2_a3`, `1x2_a8`, `1x2_a9`, `1x2_b1`, `1x2_b2`, `1x2_b3`, `2x2_a1`, `2x2_a2` | `1` each |
| `1x2_c3` | `3` |
| `1x2_d3`, `2x2_b1`, `2x2_b2`, `2x2_b4` | `2` each |

Each in-box marker creates one Vindicator with reason `STRUCTURE`. A
nonnull result becomes persistence-required, is snapped to the marker at
yaw/pitch `0/0`, finalized at local difficulty with null group data and
offered with passengers. The marker is replaced by Air with flags `2`
even if entity insertion fails. Across all `1,212` templates there are
zero literal `minecraft:vindicator` or legacy
`minecraft:vindication_illager` entity identities; Mansion production is
marker-driven.

Spawn placement is `NO_RESTRICTIONS` against
`MOTION_BLOCKING_NO_LEAVES` with the standard Monster predicate.
No-restrictions itself adds no medium or support test; the predicate still
checks reason-sensitive darkness and generic Mob placement. All `66`
baseline biome records and all `28` Trial Spawner configs contain zero
Vindicator rows.

The baseline patrol spawner selects Pillagers, not Vindicators, although
custom `PATROL` finalization retains inherited state. Spawn Egg item
protocol ID `1231` has
`minecraft:entity_data={id:"minecraft:vindicator"}` and uses the generic
Egg transaction. Commands and spawners retain their generic
construction/finalization distinctions.

### Loot, tags and progression

The entity loot table has one roll and random sequence
`minecraft:entities/vindicator`. It runs only for a player kill, emits
Emerald item protocol ID `927`, sets integer-uniform count `0..1`, then
applies a uniform `0..1` Looting enchanted-count increase. Without a
player kill it emits no table item. Equipped Axe/banner drops, positive
count filtering, gamerule gates, XP `5` and item merging retain the death
and loot owners.

Vindicator is a direct member only of `minecraft:illager` and
`minecraft:raiders`; the former reaches `minecraft:illager_friends`
through nested membership. It supplies an individual
`minecraft:player_killed_entity` criterion in
`adventure/kill_a_mob` and is one required hostile type in
`adventure/kill_all_mobs`.

### Migration and schema closure

The locked compatibility pipeline names the Vindicator family in these
eight contexts:

- `ItemStackSpawnEggFix` maps legacy
  `minecraft:vindication_illager` to its legacy Spawn Egg;
- `EntityTheRenameningFix` maps both legacy entity and legacy Egg to
  `minecraft:vindicator` and `minecraft:vindicator_spawn_egg`;
- schemas `V705` and `V1460` register the legacy entity Mob shape;
- schema `V1510` moves that shape to the current entity name;
- `EntityUUIDFix` includes current Vindicator in the Mob UUID rewrite set;
- `BlockPosFormatAndRenamesFix` converts its legacy `PatrolTarget` to
  codec-shaped `patrol_target`; and
- `StatsCounterFix` maps legacy `VindicationIllager` killed/killed-by
  statistics to `minecraft:vindication_illager` before the later rename.

Schema `V705` also maps the current Spawn Egg to the current entity.
Generic entity, equipment, effects, raid and patrol migrations retain
their owners.

### Sound and client projection

Registered species sounds are Ambient `1715`, Celebrate `1716`, Death
`1717` and Hurt `1718`; Parrot imitation is `1244`. Door, melee, item and
equipment sounds remain generic.

Arm-pose selection is strict: Aggressive selects Attacking; otherwise
Celebrating slot `16=true` selects Celebrating; otherwise Crossed. In
Attacking pose an empty hand uses zombie-arm animation, while a held Axe
uses the main-arm-aware weapon-down animation. Crossed pose displays the
joined arm mesh; the other two display independent arms.

`VindicatorRenderer` uses `ModelLayers.VINDICATOR`, shared
`IllagerModel`, shadow radius `0.5`, the inherited Custom-Head layer and a
dedicated Item-in-Hand layer. That item layer submits only while render
state `isAggressive=true`; a nonaggressive celebrating Vindicator can
therefore retain an Axe without rendering it in hand. The shared model's
inflated hat cube is disabled, while head-slot equipment remains available
through the Custom-Head layer.

The entity texture
`textures/entity/illager/vindicator.png` is `64x64`, `865` bytes, SHA-256
`a82d0f7012c464b9421708bd9b4fc1d742c8149512c3be47a315bc67466d0618`.
The Spawn Egg uses its dedicated generated item model and `16x16`,
`247`-byte texture with SHA-256
`7a04b98a84248c658d028fa00fef8bf91c1f6e617bd260ad7b13b5bc95689670`.

**Gates:**

Logical side, Peaceful, NoAI, persistence and distance; goal priority and
flags; target class, age, liveness, sight, reach, team, Illager alliance
and attackability; exact custom-name text and persisted Johnny field;
ground navigation, raided position, active Raid, door state/distance,
Normal/Hard, RNG and admission-time `mobGriefing`; finalization reason,
current Raid, wave, Omen odds and enchantment provider; Raid group/rider
and Mansion marker creation/insertion; placement reason/light/Mob checks
with absent biome/Trial rows; player kill, Looting, equipment drop,
silence, metadata and client resources.

**Branches and aborts:**

- Peaceful, NoAI, generic lifecycle, persistence and distance gates.
- Goal priority/flags, target class, liveness, sight, reach, alliance,
  baby-Villager and `attackable` gates.
- Exact custom-name flattening and case-sensitive Johnny comparison.
- Active-Raid, raid-position, ground-navigation, door, distance,
  Normal/Hard, random and `mobGriefing` gates.
- Raid wave, Omen odds, current-Raid association and provider choice.
- Mansion marker box/entity creation and generic insertion.
- Placement reason/light/Mob admission with absent biome and Trial rows.
- Player-kill, Looting, equipment-drop, silence and client-resource gates.

**Invariants:**

- Johnny is a one-way live latch but an ordinary persisted boolean across
  reload.
- Johnny never bypasses Illager alliance, baby-Villager or common target
  admission.
- Only an active Raid permits either Vindicator door goal.
- Door breaking is effectively `240` goal ticks, not the constructor's
  `6`.
- `mobGriefing` is read at break admission, not during continuation.
- Every raid-buff call ends with a fresh Iron Axe, enchanted or plain.
- Raid Sharpness switches after fixed wave `5`, independently of live
  difficulty group count.
- Baseline production is raids plus 20 Mansion markers; biome, Trial,
  literal-template and patrol selectors contribute zero.

**Constants and randomness:**

Registration `141/0.6x1.95/1.6575/2/-0.6/8/3`; attributes
`24/0.3499999940395355/12/5`, finalized Axe modifier `+8`, XP `5`;
metadata slot `16`; Creaking `8/1/1.2`; hold radius `10`; melee/stroll
`1/0.6`; look `3/1` and `8`; Johnny scan `0`; break retry
`nextInt(reducedTickDelay(10))`, swing/event `nextInt(20)`, range `2`,
time `max(240,6)=240`; raid base counts `0/2/0/1/4/2/5`, enchant odds
`0/0.1/0.25/0.5/0.75`, Sharpness levels `1/2`, threshold `>5`;
Mansion markers `20`; loot `0..1` plus Looting `0..1`; texture
`64x64`, Egg `16x16`, shadow `0.5`.

**Side effects:**

Target/aggression and navigation changes; door open/break navigation,
swings, crack projection and level events; no-action resets; custom-name
and Johnny persistence; equipment replacement/enchantment and generic
melee/blocking effects; raid membership, leader/banner state and bossbar
health; Mansion entity insertion and marker clearing; loot, XP, stats and
advancement progress; sounds, Parrot imitation, arm/head/item model
submission.

**Observability:**

Entity packets and metadata; attributes, equipment and custom data; target,
path, door state, crack/event output and RNG cursor; Raid membership,
wave, leader, health and Axe enchantments; Mansion entities and consumed
markers; save records; loot/XP/stat/advancement state; sound IDs,
aggression/celebration pose, held-item visibility, model layers, textures
and hashes.

**Boundary cases and quirks:**

Naming with styled or translatable content uses its flattened string, and
only exact `Johnny` latches. A later rename does not restore ordinary
targeting. Johnny remains invisible as a direct client field. A finalized
nonraid Vindicator may have door opening enabled until its first AI step
outside a raided position. Turning `mobGriefing` off during an active
break does not cancel it. The nominal door argument `6` is ineffective
under the 240-tick floor. The destruction event reads the block after
removal. An exact random float zero can enchant at nominal Raid odds zero.
Raid buffs replace, rather than upgrade, a preexisting weapon. Celebrating
suppresses the held-item layer even though the Axe remains equipped.

**Failure semantics:**

Failed target or door admission commits only branch-local RNG already
consumed. Stopping door breaking clears cracks but does not restore
elapsed progress on a later start; start resets it. Door removal has no
rollback. Raid equipment replacement has no rollback or displaced-stack
drop. Null Mansion creation skips the entity and leaves marker handling at
that early return; failed insertion of a nonnull entity does not restore
the cleared marker or undo persistence/finalization. Loot and client
submission failures retain their generic owners.

**Client/server authority split:**

The server owns Johnny, names, targets, aggression, goals, navigation,
doors, equipment, enchantments, raids, production, persistence, damage,
loot, XP and progression. Clients consume generic state, Raider
celebration slot `16`, equipment and resources to render pose, head layer,
conditionally visible held items and sounds. Client animation or a stale
aggression/celebration projection cannot select targets, break doors or
change equipment.

**Interaction with persistence:**

Save/reload reconstructs generic state, custom name, optional
`Johnny=true`, equipment, patrol and Raider association independently.
Door progress/navigation, targets, aggression and goal state restart.
Reloaded `Johnny=true` immediately enables the broad target goal once
ordinary goal admission resumes. A missing or false field restores
ordinary targeting even when the custom name still visually reads
`Johnny`; only another custom-name setter call relatches it.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.PatrollingMonster`;
`net.minecraft.world.entity.raid.Raider`;
`net.minecraft.world.entity.raid.Raider$HoldGroundAttackGoal`;
`net.minecraft.world.entity.monster.illager.AbstractIllager`;
`net.minecraft.world.entity.monster.illager.Vindicator`;
`net.minecraft.world.entity.monster.illager.Vindicator$VindicatorBreakDoorGoal`;
`net.minecraft.world.entity.monster.illager.Vindicator$VindicatorJohnnyAttackGoal`;
`net.minecraft.world.entity.ai.goal.BreakDoorGoal`;
`net.minecraft.world.entity.ai.goal.OpenDoorGoal`;
`net.minecraft.world.entity.ai.goal.MeleeAttackGoal`;
`net.minecraft.world.entity.ai.goal.AvoidEntityGoal`;
`net.minecraft.world.entity.ai.goal.target.HurtByTargetGoal`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.entity.raid.Raid`;
`net.minecraft.world.entity.raid.Raid$RaiderType`;
`net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces$WoodlandMansionPiece`;
`net.minecraft.world.level.levelgen.PatrolSpawner`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.fixes.EntityTheRenameningFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.BlockPosFormatAndRenamesFix`;
`net.minecraft.util.datafix.fixes.StatsCounterFix`;
`net.minecraft.util.datafix.schemas.V705`;
`net.minecraft.util.datafix.schemas.V1460`;
`net.minecraft.util.datafix.schemas.V1510`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.VindicatorRenderer`;
`net.minecraft.client.renderer.entity.IllagerRenderer`;
`net.minecraft.client.renderer.entity.state.IllagerRenderState`;
`net.minecraft.client.model.monster.illager.IllagerModel`;
`net.minecraft.client.model.geom.ModelLayers`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,enchantment_provider,loot_table,advancement}`;
`reports/minecraft/components/item/{iron_axe,vindicator_spawn_egg}.json`;
`data/minecraft/tags/entity_type/{illager,illager_friends,raiders}.json`;
`data/minecraft/enchantment_provider/raid/{vindicator,vindicator_post_wave_5}.json`;
`data/minecraft/loot_table/entities/vindicator.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/trial_spawner/**/*.json`;
`data/minecraft/structure/**/*.nbt`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`assets/minecraft/{items,models/item,textures/item}/vindicator_spawn_egg.*`;
`assets/minecraft/textures/entity/illager/vindicator.png`;
`assets/minecraft/sounds.json`; `assets/minecraft/lang/en_us.json`;
`ENT-DAMAGE-001`; `ENT-BLOCK-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-SPAWN-001`;
`MOB-PATROL-001`; `MOB-RAID-001`; `MOB-DESPAWN-001`;
`ITM-IRON-MATERIAL-001`; `ITM-EMERALD-001`; `ITM-ENCHANT-001`;
`ITM-ADVANCEMENT-001`; `WGEN-STRUCTURE-WOODLAND-MANSION-001`;
`CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-040` across raw/finalized/raid/Mansion/loaded construction;
ordinary and Johnny names/fields, every goal/target/alliance priority,
door navigation/open/break admission and live-rule/difficulty/Raid
changes; all finalization reasons, current-Raid states, wave/Omen draws,
providers and preexisting equipment; every raid group/rider and all 20
Mansion markers, insertion failures, absent biome/Trial/literal-template
production; player/Looting/equipment loot, tags, criteria, migrations,
sounds and every crossed/attacking/celebrating/head/held-item render state.

**Limits:**

Generic lifecycle, metadata, equipment attributes/durability, melee and
blocking, goal arbitration, door interaction, patrol/raid orchestration,
natural spawning, structure generation, loot, advancements and rendering
retain their cited owners. Shared Patrolling-Monster/Raider/Illager
behavior is included only where Vindicator registers it, supplies exact
inputs or changes the observable result.
