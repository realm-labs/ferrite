# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-PILLAGER-001` - Pillagers couple crossbow state, patrol and raid production to a five-slot inventory

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`,
`MOB-002`, `MOB-SPAWN-001`, `MOB-PATROL-001`, `MOB-RAID-001`,
`MOB-003`, `MOB-DESPAWN-001`, `ITM-001`,
`ITM-ARROW-AMMUNITION-001`, `ITM-OMINOUS-BOTTLE-001`,
`ITM-ENCHANT-001`, `ITM-ADVANCEMENT-001`, `PLY-AUTOJUMP-001`,
`WGEN-005`, `WGEN-JIGSAW-OUTPOST-001`, `WGEN-PORTAL-001`,
`CLI-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` - locked registration, `Pillager`, the inherited
Patrolling-Monster/Raider/Illager chain, the complete ranged-crossbow goal
and Crossbow-Attack-Mob bridge, raid and patrol producers, structure
override, loot, enchantment providers, advancements, tags, sounds and
client renderer close protocol entity ID `103`.

**Applies when:**

`minecraft:pillager` is constructed, finalized, spawned by a patrol, raid,
outpost override, Egg or command, assigned or interrupted in crossbow
combat, moved into or out of a raid, offered an ominous banner or other
item, saved, loaded, killed, synchronized, heard, imitated by a Parrot or
rendered.

**Authoritative state:**

Protocol entity ID `103` constructs `Pillager` in `MONSTER`, excludes it
from Peaceful and permits it to spawn far from a Player. Registration fixes
width/height `0.6x1.95`, default eye height
`1.95*0.85=1.6575`, passenger attachment `2`, riding offset `-0.6`, client
tracking range `8` and default update interval `3`.

The Monster attribute supplier is overridden to movement speed
`0.3499999940395355`, maximum health `24`, attack damage `5` and follow
range `32`. Other Monster defaults remain inherited. Construction fixes XP
reward `5`, initializes an empty five-slot `SimpleContainer`, leaves loot
pickup false and installs goals only on the logical server.

Raider metadata slot `16`, serializer `BOOLEAN`, is Celebrating and defaults
false. Pillager adds slot `17`, also `BOOLEAN`, as Charging-Crossbow with
default false. Neither value supplements server persistence. Entity,
Living-Entity and Mob metadata occupy slots `0..15`.

The Pillager inventory is exposed through command slot IDs `300..304`;
other slot IDs delegate to generic equipment and container access. It saves
under `Inventory` as a list of item stacks and reloads into the same five
slots. Loading first accepts the generic `CanPickUpLoot` value and then
unconditionally rewrites it to true. A fresh or finalized Pillager
therefore has pickup disabled, while every successfully loaded Pillager has
pickup enabled even if its saved `CanPickUpLoot` was false or absent.

Patrolling-Monster persistence adds nullable `patrol_target`,
`PatrolLeader` and `Patrolling`; Raider persistence adds `Wave`,
`CanJoinRaid` and an optional `RaidId` association. Active goals, target,
sight/path counters, crossbow state/delays, charging metadata and
Celebrating are transient. Generic equipment, drop chances and
`CanPickUpLoot` persist through their owners.

**Transition and ordering:**

### Finalization and equipment

Pillager finalization uses the level RNG and regional difficulty in this
order:

1. put a fresh Crossbow in the main hand;
2. run generic armor and weapon spawn enchantment;
3. after the generic weapon path, consume `nextInt(300)` unconditionally;
4. when that draw is zero and the main-hand item is still exactly Crossbow,
   apply provider `minecraft:pillager_spawn_crossbow`, Piercing I;
5. set `CanJoinRaid=true`;
6. unless the reason is `PATROL`, `EVENT` or `STRUCTURE`, consume the
   inherited patrol-leader draw and mark a leader when
   `nextFloat<0.06`;
7. if marked leader, equip the exact ominous banner in the head slot and
   set that slot's drop chance to `2`;
8. set `Patrolling=true` for reason `PATROL`; and
9. add the permanent follow-range multiplier
   `triangle(0,0.11485000000000001)` when absent, then set left-handed from
   `nextFloat<0.05`.

The Witch-only natural-spawn exception in Raider finalization cannot select
a Pillager, so every finalized Pillager can join a raid. A raw construction
path that omits finalization has no Crossbow and cannot start its ranged
goal. The default main-hand equipment drop chance remains `0.085`.

The preferred-weapon tag is `minecraft:pillager_preferred_weapons`, whose
only member is Crossbow. `canUseNonMeleeWeapon` independently requires the
stack's item to be exactly Crossbow.

### Goal graph and targets

The effective goal selector contains the inherited registrations followed
by the Pillager registrations:

| Priority | Goal |
|---:|---|
| `0` | Float |
| `1` | Obtain Raid Leader Banner |
| `1` | avoid Creaking within `8`, near speed `1`, far speed `1.2` |
| `2` | Hold Ground Attack with radius `10` |
| `3` | Pathfind to Raid |
| `3` | ranged Crossbow attack at speed `1`, radius `8` |
| `4` | Long Distance Patrol at leader/follower speed `0.7/0.595` |
| `4` | move through village at speed `1.0499999523162842`, distance `1` |
| `5` | Raider Celebration |
| `8` | random stroll at speed `0.6` |
| `9` | look at Player within `15`, probability `1` |
| `10` | look at Mob within `15` |

Goal flags and priority arbitration remain `MOB-AI-001`; patrol, hold-ground,
banner, village, raid-path and celebration algorithms remain
`MOB-PATROL-001` and `MOB-RAID-001`.

The target selector is:

1. priority `1` Hurt-By-Target, excluding all Raiders as attackers and
   alerting all eligible allies;
2. priority `2` nearest Player with sight required;
3. priority `3` nearest Abstract Villager without a sight requirement; and
4. priority `3` nearest Iron Golem with sight required.

The shared Illager attack gate rejects baby Abstract Villagers. Raider
alliance is inherited; an entity in `minecraft:illager_friends` is also an
ally only while both entities have no scoreboard team. `getWalkTargetValue`
always returns zero and maximum spawn cluster size is one.

### Crossbow goal

`RangedCrossbowAttackGoal` owns transient states `UNCHARGED`, `CHARGING`,
`CHARGED` and `READY_TO_ATTACK`, requests Move and Look, and updates every
tick. It starts only with a live target and an exact Crossbow in either
hand. It continues while the target is live, the mob still holds a
Crossbow, and either the start predicate remains true or navigation is
still active.

Every tick tests line of sight. A visibility value different from
`seeTime>0` first resets `seeTime` to zero, then visible ticks increment it
and hidden ticks decrement it. With squared distance `d2`, the movement
predicate is

`(d2>64 || seeTime<5) && attackDelay==0`.

While that predicate is true, `updatePathDelay` decrements. At or below
zero the mob requests a path at speed `1` while `UNCHARGED`, otherwise
speed `0.5`, and samples the next delay uniformly from `20..40`. While the
predicate is false, it resets the path delay to zero and stops navigation.
It looks at the target with yaw/pitch limits `30/30`.

The weapon state then advances:

1. `UNCHARGED` starts using the Crossbow-holding hand only while the
   movement predicate is false, changes to `CHARGING` and synchronizes slot
   `17=true`.
2. `CHARGING` changes back to `UNCHARGED` if item use has stopped, but the
   method continues through the same tick and does not clear slot `17` in
   that branch.
3. Once use ticks reach `CrossbowItem.getChargeDuration`, release loads the
   projectile, changes to `CHARGED`, samples `attackDelay=20+nextInt(20)`
   and synchronizes slot `17=false`.
4. `CHARGED` predecrements the delay each tick and changes to
   `READY_TO_ATTACK` exactly at zero.
5. `READY_TO_ATTACK` waits indefinitely for line of sight, then fires at
   power argument `1` and returns to `UNCHARGED`.

Default and Piercing-I Crossbows charge in `25` ticks. The raid providers'
Quick Charge I and II reduce that to `20` and `15` ticks respectively.
The additional post-charge wait is always `20..39`.

Stopping the goal clears aggression, target and `seeTime`. Only when the
mob is currently using an item does stop also end use, clear slot `17` and
replace the use stack's Charged-Projectiles component with empty. Stop does
not reset the goal's state enum, `attackDelay` or `updatePathDelay`.
Consequently interruption while `CHARGED` or `READY_TO_ATTACK` can retain
loaded projectiles and transient state into a later run, while an
interrupted active charge is explicitly emptied.

Pillager fire delegates to Crossbow-Attack-Mob with velocity `1.6`.
The Crossbow item consumes the loaded projectile component before
constructing projectiles, shoots with inaccuracy
`14-4*difficultyId`, hence `10/6/2` on Easy/Normal/Hard, and receives the
current target for aiming. The Pillager then resets `noActionTime=0` even
if the held item failed the Crossbow-item instance check. Projectile
identity, Piercing, collision, damage, pickup and recovery remain
`ITM-ARROW-AMMUNITION-001` and `ENT-PROJECTILE-001`.

### Raid weapon replacement

`MOB-RAID-001` creates base Pillagers in waves `1..7` with counts
`4,3,3,4,4,4,2` before its difficulty bonus. The Hard final-wave Ravager
rider selector can additionally create a Pillager. A successful raid join
sets raid/wave state, finalizes with reason `EVENT`, then calls the
Pillager-specific buff before insertion.

Every buff call consumes one entity `nextFloat` and compares it at or below
the raid enchant odds. Omen levels `2/3/4/5` give odds
`0.1/0.25/0.5/0.75`; every other level gives zero. Even an odds success
does nothing in waves `1..3`. For wave `4` or `5`, it creates a fresh
Crossbow, applies `minecraft:raid/pillager_post_wave_3` (Quick Charge I)
using current regional difficulty and entity RNG, and replaces the main
hand. For wave `6` or later it instead applies
`minecraft:raid/pillager_post_wave_5` (Quick Charge II).

Replacement occurs after ordinary finalization, so it discards the
finalized Crossbow and any generic or one-in-300 Piercing enchantment. The
boolean buff argument is unused. If raid membership is absent,
`getCurrentRaid` supplies no defensive fallback; valid callers establish
the raid before invoking this method.

### Inventory and banner pickup

When the generic pickup transaction reaches Pillager:

1. every stack whose item is a `BannerItem` delegates immediately to the
   inherited Raider pickup path;
2. otherwise `wantsItem` is true only during an active raid and only for
   the exact white Banner item;
3. a wanted stack emits the pickup side effect, inserts into the five-slot
   inventory, discards the item entity if no remainder exists, or rewrites
   its count to the remainder.

All sixteen locked banner items, including White Banner, are
`BannerItem` instances. The only item accepted by `wantsItem` is therefore
always intercepted by step one. The custom five-slot insertion branch is
unreachable from ordinary locked-vanilla item pickup. Inventory contents
can still exist through commands, entity data or reload and round-trip
normally.

The inherited branch equips an exact ominous banner only while the
Pillager has an active raid, the wave has no leader and the whole stack
matches `Raid.getOminousBannerInstance`. It may drop an occupied head item
according to its generic drop chance, equips the banner, takes and discards
the source stack, installs itself as wave leader and marks Patrol-Leader.
Other banner stacks fall through to generic Mob equipment comparison.
The surrounding item scan still requires `CanPickUpLoot`, `mobGriefing`,
liveness and the generic pickup predicate; the raid banner acquisition
goal has its additional `MOB-RAID-001` gates.

### Production

Spawn placement is `ON_GROUND` against
`MOTION_BLOCKING_NO_LEAVES`. The subtype predicate first requires block
light at most `8`, then runs the shared any-light Monster rule. There are
zero Pillager rows in the `66` biome files.

The `minecraft:pillager_outpost` structure supplies a full-bounding-box
Monster override containing only Pillager at weight `1`, group `1..1`.
None of the `1,212` raw structure templates contains a Pillager entity;
outpost construction and its eleven-template payload remain
`WGEN-JIGSAW-OUTPOST-001`. Natural spawning selects the structure override,
not raw template NBT.

The independently specified patrol spawner constructs Pillagers with
reason `PATROL`; its leader is marked before finalization, receives the
ominous banner, and all members receive Crossbows and become patrolling.
Raids produce the wave and rider paths above. The Spawn Egg is protocol
item ID `1229`, has `minecraft:entity_data={id:"minecraft:pillager"}`, and
uses the generic Egg transaction. Commands and spawners retain their
generic construction/finalization distinctions.

### Loot, progression and projection

The entity loot table has one captain-only pool. When the Raider
type-specific predicate says `is_captain=true`, it emits exactly one
Ominous Bottle and sets its amplifier uniformly from `0..4`; the table has
random sequence `minecraft:entities/pillager`. Noncaptains receive no
table item. XP is `5`; equipped Crossbow and head-banner drops remain the
generic equipment transaction.

Pillager is a direct member only of `minecraft:illager` and
`minecraft:raiders`. It participates in `adventure/kill_a_mob` and
`adventure/kill_all_mobs`. `adventure/whos_the_pillager_now` additionally
requires a `minecraft:killed_by_arrow` trigger whose fired weapon is
Crossbow and whose victim list contains a Pillager; its sole criterion is
`kill_pillager`.

Species sound events are Ambient `1296`, Celebrate `1297`, Death `1298`
and Hurt `1299`. Parrot imitation is `1235`. Crossbow loading, Quick-Charge
and shooting sounds are the generic item events `484..491`; the Pillager
adds no private shoot sound.

The client selects arm pose in strict order: Crossbow-Charge while slot
`17` is true; otherwise Crossbow-Hold while either hand holds an exact
Crossbow; otherwise Attacking while aggressive; otherwise Neutral. This
override never selects Celebrating, so slot `16=true` affects inherited
raid celebration behavior but does not produce the shared Celebrating arm
pose. The stale-true charge branch can therefore display a charge pose
after item use has already stopped.

`PillagerRenderer` uses `ModelLayers.PILLAGER`, shared `IllagerModel`,
shadow radius `0.5`, inherited Custom-Head layer and a dedicated
Item-in-Hand layer. The render state includes riding, main arm, chosen arm
pose, armed stacks, attack animation, aggression, use ticks and, only for
Crossbow-Charge, the current maximum charge duration. The entity texture
`textures/entity/illager/pillager.png` is `64x64`, `761` bytes, SHA-256
`3ab515ec1aff8db061bb887d80b4757e673b66e22259be3643ea252da83bb963`.
The Spawn Egg uses its dedicated generated item model and `16x16`,
`262`-byte texture with SHA-256
`e570952d3aa129e37d9a9e0964fea9b63ce37de31e04cb04d2c49354a1a47ca0`.

**Branches and aborts:**

Peaceful, NoAI and generic lifecycle gates; target absence/death, weapon
absence, goal flags and priority conflict; distance, five-tick sight
stability, path timer, charge duration, post-charge delay and firing sight;
raid membership/wave/Omen odds; patrol reason and leader draw; pickup
enablement, `mobGriefing`, Banner subclass interception, exact ominous
stack and existing wave leader; placement light/support and absent biome
rows; captain predicate, equipment drop chance, silence and resource
availability.

**Constants and randomness:**

Dimensions/eye/passenger/riding `0.6x1.95/1.6575/2/-0.6`; range/update
`8/3`; health/speed/attack/follow `24/0.3499999940395355/5/32`; XP `5`;
metadata `16/17`; inventory slots `5` at `300..304`; Piercing draw
`nextInt(300)==0`; patrol leader/handedness `0.06/0.05`; follow triangle
`0.11485000000000001`; Creaking `8/1/1.2`; hold radius `10`; Crossbow
speed/radius `1/8`, squared radius `64`, sight gate `5`, path delay
`20..40`, path speeds `1/0.5`, look `30/30`, charge
`25/20/15`, post-charge `20..39`, shot `1.6`, inaccuracy
`14-4*difficultyId`; raid thresholds `>3/>5`, odds
`0/0.1/0.25/0.5/0.75`; outpost `1/1..1`; loot count `1`, amplifier
`0..4`; textures `64x64/16x16`; shadow `0.5`.

**Side effects:**

Equipment, enchantments, patrol/raid fields, inventory and persistence;
metadata, targets, aggression, navigation, look, item use, loaded
projectiles and RNG cursors; projectile entities, damage and pickup;
leader/banner state, wave weapon replacement and bossbar joins; loot, XP
and advancement progress; sounds, subtitles, model, held/head layers and
textures.

**Gates:**

Logical side, Peaceful, NoAI and persistence; finalization reason and RNG;
goal arbitration, live target, exact held Crossbow, sight, distance and
timers; raid association, wave and Omen; pickup and item-class/identity
tests; spawn placement, outpost bounds, patrol/raid producers; captain and
kill predicates; metadata, silence and client resources.

**Boundary cases and quirks:**

A saved false or absent `CanPickUpLoot` reloads true. The only item the
private inventory predicate wants is a White Banner, but every White Banner
is intercepted as a Banner before that predicate, making ordinary
five-slot insertion unreachable. Crossbow-goal stop resets neither its
state enum nor its delays and only empties projectiles while actively
using, so interrupted charged state can survive until a later run. A
stopped use inside `CHARGING` returns the enum to Uncharged without clearing
slot `17`, exposing a stale client charge pose. Waves one through three
consume the raid buff odds draw but can never replace the weapon. Later
successful replacement discards a possible Piercing Crossbow. Pillager
ignores Celebrating when selecting arms. A raw, nonfinalized Pillager has
no Crossbow and cannot use its main attack goal.

**Failure semantics:**

Failed path requests retain the sampled path delay. Charge interruption
does not roll back sounds or earlier projectile/component changes. A fire
call clears loaded projectiles before projectile creation, so a failed
shoot transaction does not restore ammunition; `noActionTime` still resets.
Raid odds failure or a low wave preserves the existing Crossbow, while
replacement commits without retaining its old enchantments. Partial
inventory insertion would leave the remainder on the source entity, but
the locked ordinary-pickup route cannot reach it. Rejected entity insertion
in patrol, raid or generic spawning follows the producer owner's rollback
semantics.

**Client/server authority split:**

The server owns goals, targeting, navigation, use state, projectiles,
equipment/enchantments, raid/patrol/banner state, inventory, persistence,
loot, XP and advancement triggers. Clients consume metadata slots `16/17`,
equipment/use synchronization, movement and resources to select and animate
arms, head and held-item layers. Client pose, sound or resource state
cannot load a projectile, change an inventory or join a raid.

**Observability:**

Observe registration, attributes, XP and metadata slots; every construction
and finalization reason with RNG order; false-to-true pickup reload and all
five slots; the full inherited/local goal graph and target set; every
crossbow state, sight/distance/path/charge/delay boundary and interruption;
raid odds/waves/provider replacement and patrol equipment; Banner
interception and wave-leader pickup; outpost override versus zero biome and
template rows; captain/noncaptain loot, equipment drops, three advancements,
Egg, two direct tags, thirteen sound joins and exact arm/model/texture
projection.

**Persistence and reload:**

Generic entity/Mob state, equipment, drop chances, inventory, patrol
fields, wave/join state and a resolvable Raid ID save. Loading the Pillager
inventory then forces loot pickup true. Charging and Celebrating metadata,
goal state, target, all ranged counters, active item-use state and
navigation do not save. Charged-Projectiles lives on the persisted Crossbow
stack, so it can outlive the transient goal state. Loot, tags,
enchantment providers, structure and advancement data reload through their
owners; renderer resources reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.PatrollingMonster`;
`net.minecraft.world.entity.monster.PatrollingMonster$LongDistancePatrolGoal`;
`net.minecraft.world.entity.raid.Raider`;
`net.minecraft.world.entity.raid.Raider$HoldGroundAttackGoal`;
`net.minecraft.world.entity.raid.Raider$ObtainRaidLeaderBannerGoal`;
`net.minecraft.world.entity.monster.illager.AbstractIllager`;
`net.minecraft.world.entity.monster.illager.Pillager`;
`net.minecraft.world.entity.monster.CrossbowAttackMob`;
`net.minecraft.world.entity.ai.goal.RangedCrossbowAttackGoal`;
`net.minecraft.world.entity.ai.goal.RangedCrossbowAttackGoal$CrossbowState`;
`net.minecraft.world.entity.ai.goal.AvoidEntityGoal`;
`net.minecraft.world.entity.ai.goal.target.HurtByTargetGoal`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.entity.npc.InventoryCarrier`;
`net.minecraft.world.SimpleContainer`;
`net.minecraft.world.item.CrossbowItem`;
`net.minecraft.world.item.BannerItem`;
`net.minecraft.world.entity.raid.Raid`;
`net.minecraft.world.entity.raid.Raid$RaiderType`;
`net.minecraft.world.level.levelgen.PatrolSpawner`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.advancements.packs.VanillaAdventureAdvancements`;
`net.minecraft.util.datafix.schemas.V1800`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.BlockPosFormatAndRenamesFix`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.PillagerRenderer`;
`net.minecraft.client.renderer.entity.IllagerRenderer`;
`net.minecraft.client.renderer.entity.state.IllagerRenderState`;
`net.minecraft.client.model.monster.illager.IllagerModel`;
`net.minecraft.client.model.geom.ModelLayers`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,enchantment_provider,loot_table,advancement,worldgen/structure}`;
`reports/minecraft/components/item/{crossbow,pillager_spawn_egg}.json`;
`data/minecraft/tags/entity_type/{illager,raiders}.json`;
`data/minecraft/tags/item/pillager_preferred_weapons.json`;
`data/minecraft/enchantment_provider/{pillager_spawn_crossbow,raid/pillager_post_wave_3,raid/pillager_post_wave_5}.json`;
`data/minecraft/worldgen/structure/pillager_outpost.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/structure/**/*.nbt`;
`data/minecraft/loot_table/entities/pillager.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs,whos_the_pillager_now}.json`;
`assets/minecraft/{items,models/item}/pillager_spawn_egg.json`;
`assets/minecraft/textures/{entity/illager/pillager,item/pillager_spawn_egg}.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-PROJECTILE-001`; `ENT-DAMAGE-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-SPAWN-001`;
`MOB-PATROL-001`; `MOB-RAID-001`; `MOB-DESPAWN-001`;
`ITM-ARROW-AMMUNITION-001`; `ITM-OMINOUS-BOTTLE-001`;
`ITM-ENCHANT-001`; `ITM-ADVANCEMENT-001`;
`WGEN-JIGSAW-OUTPOST-001`; `CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-032` across raw/finalized/loaded construction, every metadata
and inventory state, all goals/targets and finalization reasons; Crossbow
distance/sight/path/charge/post-charge/fire/stop states with default,
Piercing and both Quick-Charge weapons; every raid wave/Omen odds result
and patrol leader/member path; false and true pickup, all Banner classes,
exact/nonexact ominous stacks, occupied head slots and wave leadership;
outpost/biome/template production, captain loot/equipment/XP,
advancements/Egg/tags/sounds, and exact renderer/model/texture state.

**Limits:**

Generic lifecycle, metadata, equipment comparison, item use, Crossbow and
arrow internals, projectile collision/damage, goal arbitration,
patrol/raid orchestration, natural spawning, structure generation, loot,
advancements and rendering retain their cited owners. Shared
Patrolling-Monster/Raider/Illager behavior is included only where Pillager
registers it, supplies exact inputs or changes the observable result.
