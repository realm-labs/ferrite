# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-VEX-001` — Vexes phase through collision, inherit an owner's target and starve after an optional lifetime

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-005`, `ENT-DAMAGE-001`, `ENT-BLOCK-001`,
`ENT-DAMAGE-REDUCE-001`, `ENT-KNOCKBACK-001`, `ENT-006`,
`ENT-EFFECT-001`, `ENT-007`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-EVOKER-001`, `MOB-001`,
`MOB-AI-001`, `MOB-002`, `MOB-SPAWN-001`, `MOB-003`,
`MOB-DESPAWN-001`, `MOB-RAID-001`, `ITM-001`,
`ITM-IRON-MATERIAL-001`, `ITM-ENCHANT-001`,
`ITM-ADVANCEMENT-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PORTAL-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`,
`CLI-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete `Vex` and nested goal/
control implementations, Evoker summon spell, effective Mob/Monster
finalization and despawn behavior, loot, progression, Spawn Egg,
compatibility and exact client projection close protocol entity ID `139`.

**Applies when:**

`minecraft:vex` is created by an Evoker summon spell, Spawn Egg, spawner,
command or custom code, finalized with equipment, resolving an owner,
copying targets, retaliating, selecting a Player, moving around a bound
origin, charging, phasing through collision, aging under a limited life,
dying, persisting, synchronized, heard, imitated by a Parrot or rendered.

**Authoritative state:**

Protocol entity ID `139` constructs `Vex` in the non-Peaceful `MONSTER`
category. Registration makes the type fire immune and fixes dimensions
`0.4x0.8`, eye height `0.51875`, passenger attachment Y `0.7375`, riding
offset `0.04`, client tracking range `8` and default update interval `3`.

Attributes are maximum health `14`, raw movement speed `0.7`, raw attack
damage `4` and follow range `16`. Construction sets XP reward `3`.
Movement speed is registered but the subtype's custom Move Control uses
the requested speed modifier directly rather than reading that attribute.

Finalized default equipment is one Iron Sword, raw item ID `959`, in the
main hand with drop chance `0`. Its main-hand attack modifier is `+5`, so
an unenchanted finalized Vex has effective attack damage `9`. Raw or
unfinalized instances retain attack `4` and no sword.

Inherited Entity, Living-Entity and Mob state occupies synchronized metadata
slots `0..15`. Vex adds slot `16`, serializer ID `0` (`BYTE`), default
`0`. Bit `0x01` is the charging flag. Reads test that bit; writes OR `1`
or AND the inverse while preserving all other bits. Charging is not saved.

Subtype server state is:

| Field | Fresh default | Save form |
|---|---:|---|
| owner | null `EntityReference<LivingEntity>` | optional `owner` reference |
| bound origin | null `BlockPos` | nullable codec field `bound_pos` |
| has limited life | false | inferred from presence of `life_ticks` |
| limited-life ticks | `0` | signed integer `life_ticks` only when limited |

Loading `life_ticks` calls `setLimitedLife(value)` and therefore makes the
life limited even for zero or negative values. Absence explicitly clears
`hasLimitedLife`; the numeric field is then irrelevant. `restoreFrom`
copies the owner reference from another Vex after generic state restoration
but does not copy bound origin or limited-life state.

`getOwner` resolves the reference through the current level. An unresolved,
unloaded, removed or cross-context owner therefore returns null without
clearing the stored reference. Owner identity alone does not create alliance;
the Evoker producer separately copies its scoreboard team.

**Transition and ordering:**

### Complete goal and target graph

`registerGoals` first calls the empty Monster goal registration, then
installs:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `0` | Float |
| goal | `4` | Vex Charge Attack |
| goal | `8` | Vex Random Move |
| goal | `9` | Look At Player, range `3`, probability `1` |
| goal | `10` | Look At Mob, range `8`, default probability |
| target | `1` | Hurt By, rejecting Raider attackers, alert same-class Vexes |
| target | `2` | Copy Owner Target |
| target | `3` | nearest Player, must see |

Hurt-By excludes any attacker assignable to `Raider`. Calling
`setAlertOthers()` with an empty ignore list alerts other exact-class Vexes
inside the generic follow-range X/Z and `10`-vertical box when they have no
target and pass alliance admission. The shared Hurt-By owner also suppresses
Player retaliation while `universalAnger` is true.

Copy Owner Target requires a resolved owner implementing `Targeting`, a
nonnull owner target and ordinary Vex attack admission. Its noncombat
conditions ignore line of sight and invisibility testing. Start re-resolves
the owner and copies its then-current target; a disappearing or changed owner
can therefore yield null at start after admission.

The priority-`3` Player selector provides independent hostile targeting when
retaliation and owner copying do not win. Owner, target, active goals and
movement request are transient.

### Reduced-delay move admission

Both custom Move goals call `nextInt(reducedTickDelay(7))`, and
`reducedTickDelay(7)` is `positiveCeilDiv(7,2)=4`. Each admission therefore
uses a one-in-four draw when its Move Control has no wanted position.

Charge is priority `4`, so an admitted charge occupies the Move flag before
priority-`8` random movement. If charge rejects, random movement performs its
own independent one-in-four draw. Both draws are branch-local.

### Bound-centered random movement

The random goal cannot continue; its one tick tries at most three candidates.
Its center is `boundOrigin` when nonnull, otherwise the Vex's current block
position at execution time. Each candidate independently draws:

```text
x offset = nextInt(15) - 7
y offset = nextInt(11) - 5
z offset = nextInt(15) - 7
```

The first candidate whose live block is empty becomes a wanted block center
at speed modifier `0.25`. If there is no target, Look Control also points at
that center with yaw/pitch limits `180/20`. The method stops after the first
accepted candidate whether or not a target exists. Three nonempty candidates
leave Move Control waiting.

Bound origin is only a random-flight center. It is not a leash, range limit,
teleport anchor, targeting gate or despawn exemption.

### Charge state machine

Charge admission requires all of:

1. a nonnull alive target;
2. no current wanted position;
3. the one-in-four reduced-delay draw; and
4. squared distance strictly greater than `4`.

Start snapshots the target's eye position into Move Control at speed modifier
`1`, sets charging bit `0x01`, and plays Vex Charge, protocol sound ID
`1692`, volume/pitch `1/1`.

The goal updates every tick. If Vex and target bounding boxes intersect, it
calls generic `doHurtTarget`, ignores the Boolean damage result, and clears
the charging bit. It does not directly cancel the current Move-Control
request. Otherwise, only when current squared distance is strictly below
`9`, it replaces the wanted point with the target's current eye position at
speed `1`. At distance at least `3`, the original eye-position snapshot
remains the destination.

Continuation requires a wanted position, charging bit true and a nonnull
alive target. Stop only clears charging. Thus collision ends visual charging
even when damage is rejected, while the stale movement request can continue
until the Move Control reaches its own arrival boundary.

### Collision-free Move Control

At the start of every Vex tick, `noPhysics=true`; the complete inherited
Monster tick then runs with collision resolution bypassed. The subtype resets
`noPhysics=false` immediately afterward and calls `setNoGravity(true)`.
Fresh or loaded state therefore receives the no-gravity write after, not
before, its first inherited tick.

`isAffectedByBlocks` returns true whenever the entity is not removed.
Collision-free travel consequently does not globally opt the Vex out of
block-contact processing. Fire damage is independently rejected by its
fire-immune entity type.

When Move Control is not `MOVE_TO`, its tick does nothing. Otherwise it
builds the wanted-minus-current vector and length. The arrival boundary is
strictly below `boundingBox.getSize()`, the arithmetic mean of dimensions:
`(0.4+0.8+0.4)/3 = 0.5333333333333333` at default scale. Arrival switches
to `WAIT` and halves current velocity.

Outside that boundary it adds:

```text
delta += direction * (speedModifier * 0.05 / distance)
```

With no target, yaw follows current velocity:
`-atan2(deltaX,deltaZ)*57.295776`. With a target, yaw instead follows
target-minus-self X/Z using the same conversion. Body yaw is then assigned
the resulting entity yaw. Neither branch uses path navigation or tests
intervening blocks.

### Optional lifetime and starvation

After physics/no-gravity ordering, a limited-life Vex predecrements
`limitedLifeTicks`. A result greater than zero does nothing. A result at or
below zero resets the counter to `20` and attempts Starve damage `1`; the
damage result is ignored.

Positive value `N` therefore makes the first Starve attempt on the `N`th
Vex tick, followed by one attempt every `20` ticks. Zero and negative loaded
values attempt damage on the next tick. With health `14` and no healing or
damage rejection, fourteen admitted attempts kill it, placing the final
attempt `260` ticks after the first.

Limited life does not set `PersistenceRequired`. Generic Monster far-distance
despawn and Peaceful removal can therefore precede starvation. Conversely an
unlimited Vex has no subtype age-out clock.

### Equipment and finalization

Vex finalization intentionally runs subtype equipment before generic Mob
spawn state:

1. install one Iron Sword and set main-hand drop chance `0`;
2. run generic equipment enchantment for the local difficulty;
3. call inherited Mob finalization, adding the follow-range triangle
   modifier when absent; and
4. roll handedness, left on `nextFloat()<0.05`.

The sword can therefore be enchanted before handedness is selected. A
five-percent left-handed result changes the main arm used for display but
does not move the stack out of the main-hand equipment slot.

### Evoker summon production

`ENT-EVOKER-001` owns common spell arbitration and its warmup/casting state.
The Vex summon spell additionally admits only when the base spell can start
and:

1. counts Vexes within the Evoker box inflated by `16` using range `16`,
   noncombat targeting, ignored line of sight and ignored invisibility; then
2. draws `nextInt(8)+1` and requires that value to be strictly greater than
   the count.

For counts `0..7`, admission probabilities are `(8-count)/8`; a count of
at least `8` always rejects after consuming the draw. Casting time is `100`,
interval `340`, preparation sound is Evoker Prepare Summon and spell state is
`SUMMON_VEX`.

On spell execution it caches the Evoker's scoreboard team and performs three
independent attempts. Each consumes X/Z offsets `-2+nextInt(5)` and uses
Y offset `+1`. A null Vex factory skips only that attempt. For each nonnull
Vex, source order is:

1. snap to the chosen block position with yaw/pitch `0`;
2. finalize with reason `MOB_SUMMONED` and local difficulty;
3. set owner to the Evoker;
4. set bound origin to the chosen position;
5. set limited life to `20*(30+nextInt(90))`, one of
   `600,620,...,2380` ticks;
6. if the Evoker has a team, add the Vex scoreboard name to it;
7. call `addFreshEntityWithPassengers`; and
8. emit `GameEvent.ENTITY_PLACE` at the chosen position with the Evoker as
   context source.

Insertion has no caller-visible Boolean and does not gate the game event.
Finalization, ownership, lifetime and team mutation all precede insertion.
The three attempts do not require empty space because phasing is intrinsic.

**Other production and spawning:**

Spawn placement registers `NO_RESTRICTIONS`, heightmap
`MOTION_BLOCKING_NO_LEAVES` and the generic Mob spawn predicate. All 66
bundled biome spawn lists nevertheless contain zero Vex rows. No Trial
Spawner configuration names Vex, and exact namespaced plus legacy scans of
all 1,212 locked structure templates find zero literal Vex payloads.

Evoker spells are the only dedicated baseline producer; raids and Woodland
Mansions produce Evokers rather than embedded Vexes. Generic commands,
spawners, custom code and the Spawn Egg remain possible. Vex Spawn Egg is
raw item ID `1232`, common stack size `64`, with
`entity_data.id=minecraft:vex`.

**Loot, XP, tags and progression:**

Entity loot table `minecraft:entities/vex`, with random sequence of the same
key, contains zero pools. It emits no table loot. The finalized Iron Sword
has drop chance `0`, so baseline Vex death does not yield it. XP reward is
`3`, subject to generic Monster experience admission.

Vex has zero direct entity-type tag memberships. It is explicitly selected
by exactly two bundled hostile-kill advancements, `kill_a_mob` and
`kill_all_mobs`. The similarly named Vex armor-trim template, trim pattern
and recipes are item/content identities, not Vex entity behavior.

**Compatibility:**

Legacy entity ID migration maps `Vex` to `minecraft:vex`.
Schemas V705 and V1460 register the namespaced mob. Spawn-egg migration maps
`minecraft:vex` to `minecraft:vex_spawn_egg`. Stats migration includes Vex
in the mob identity set.

`InlineBlockPosFormatFix` renames legacy `LifeTicks` to `life_ticks` and
combines `BoundX`, `BoundY` and `BoundZ` into `bound_pos`. Generic UUID
migration and the current `EntityReference` owner codec own owner-reference
compatibility.

**Client projection:**

`VexRenderer` uses `VexModel`, model layer `minecraft:vex`, shadow radius
`0.3`, one Item-In-Hand layer and forced block light `15`. Render-state
extraction copies armed state and charging bit.

Charging selects
`textures/entity/illager/vex_charging.png`; all other states select
`textures/entity/illager/vex.png`. Both are 32x32. Normal is 363 bytes,
SHA-256
`692f8906bf07f01aba5e628489542d794fb9d993f44cf3f37942a7c2eeb46e74`;
charging is 421 bytes, SHA-256
`3f9b436f3146c2c53b4ac47468c8d4465d632bd8cd455f0f6da1f967853dc8bd`.

The 32x32 model contains a `5x5x5` head, a layered narrow body, two arms and
two zero-width `5x8` wings. Setup converts head yaw/pitch directly to
radians. Arm idle roll oscillation is
`0.1*cos(ageInTicks*5.5 degrees)`. Noncharging body pitch is `9` degrees;
charging body pitch is `0`.

While charging, an empty pair of rendered hands sweeps both arms backward.
Otherwise each nonempty hand independently applies its charging pose, so
the finalized sword follows the selected main arm. Hand submission follows
root, body and chosen-arm transforms, scales by `0.55`, then offsets
`(+/-0.046875,-0.15625,0.078125)`.

Left-wing yaw is:

```text
1.0995574 + cos(ageInTicks*45.836624 degrees)
              * 0.017453292 * 16.2
```

Right-wing yaw is its negative. Both wing X rotations are `0.47123888`;
wing Z rotations are `-0.47123888` left and `+0.47123888` right. Server
`isFlapping` returns true when `tickCount % 4 == 0`; client wing motion uses
continuous render age rather than that Boolean.

Species sound protocol IDs are Ambient `1691`, Charge `1692`, Death `1693`
and Hurt `1694`. Their English subtitles are `Vex vexes`, `Vex shrieks`,
`Vex dies` and `Vex hurts`. Parrot imitation uses protocol sound ID `1243`.
The English entity and Egg labels are `Vex` and `Vex Spawn Egg`.

**Constants and randomness:**

Entity/Egg `139/1232`; dimensions/eye/passenger/riding
`0.4x0.8/0.51875/0.7375/0.04`; tracking/update `8/3`;
health/raw-speed/raw-attack/follow/XP `14/0.7/4/16/3`;
Iron-Sword modifier/live damage/drop `+5/9/0`; charging slot/bit
`16/1`; admission denominator `4`; charge distance squared `4/9`;
random offsets `[-7,7]/[-5,5]/[-7,7]`, attempts `3`, speed `0.25`;
charge speed `1`; acceleration factor `0.05`; arrival mean
`0.5333333333333333`; life interval/damage `20/1`; Evoker count/draw
`16/8`, attempts `3`, life `600..2380` step `20`; left-handed chance
`0.05`; shadow `0.3`; flap period `4`.

Server randomness includes both move admissions, up to nine random-position
draws, equipment enchantment, follow-range/handedness finalization, summon
count admission, six attempt-position draws, three life draws, ordinary
combat and XP. Client wing motion is deterministic from render age.

**Side effects:**

Target copying and same-class alerting; wanted position, velocity, yaw,
collision/gravity flags and charging metadata; melee damage; charge and
species sounds; starvation damage; equipment, enchantment, attributes and
handedness; owner/bound/life/team persistence; entity insertion, placement
game events, XP, criteria and two-texture wing/item client projection.

**Gates:**

Logical side, Peaceful, NoAI and generic despawn; owner resolution and owner
target; Raider attacker exclusion, universal anger and alliance; target life,
Move flag, reduced-delay RNG and distance; Air candidates; bounding-box
intersection and damage admission; limited-life presence and damage admission;
spell base admission, nearby count/draw, factory and team; zero natural/
Trial/template selectors; loot/XP/progression; resources.

**Branches and aborts:**

No target, dead target, active wanted position, failed draw or close distance
abort charge admission before start effects. Collision attacks once and
clears charging regardless of damage; target loss or completed movement stops
the goal. Random flight accepts the first Air candidate and otherwise exhausts
three attempts. Unlimited life skips the counter entirely. A null summoned
factory skips finalization, owner, team, insertion and event for only that
attempt; nonnull insertion cannot suppress the later placement event.

**Boundary cases and quirks:**

The two one-in-four Move draws are independent, despite the source constant
being `7`. Phasing is enabled only around the inherited tick and then reset;
no-gravity is written afterward. Block effects remain enabled. Random flight
is bound-centered but not bound-limited.

Owner copying ignores sight and invisibility, whereas the independent Player
selector must see. A saved `life_ticks=0` is limited, while an absent field is
unlimited. Limited life neither prevents distance despawn nor survives
`restoreFrom`; owner alone does.

Charge collision clears metadata but leaves the Move-Control request. The
damage result is ignored. Evoker insertion is followed by a placement game
event without success admission.

**Failure semantics:**

Rejected melee damage still clears charging. Rejected Starve damage still
resets the life counter to `20`. Failed equipment enchantment or generic
finalization retains its owners. Failed/ignored summoned insertion does not
roll back finalization, owner, bound, lifetime or scoreboard mutation and
does not suppress `ENTITY_PLACE`. Empty loot is a successful zero-result
evaluation.

**Client/server authority split:**

The server owns goals, owner resolution, target state, phasing/no-gravity,
movement requests and velocity, charging metadata, melee, lifetime damage,
equipment, summon production, persistence, loot and progression. Clients
consume slot `16`, equipment, movement and resources; they choose the
charging texture/arm pose and animate wings continuously. Client presentation
cannot initiate or complete a charge.

**Observability:**

Observe registration/raw/final attributes, equipment/drop chance and slot
`16`; every NBT presence/type/value and owner-resolution state; goal
registration and selector precedence; both reduced-delay draws and random
candidate RNG; charge start/retarget/intersection/stop; noPhysics/noGravity/
block-effect order and exact acceleration/arrival/yaw; limited-life
predecrement/reset/damage; finalization order; Evoker count probability and
all three create/finalize/owner/bound/life/team/insert/event paths; zero
production rows, empty loot, XP, criteria/Egg/compatibility; four species
sounds, Parrot and exact armed/two-texture/wing projection.

**Persistence and reload:**

Generic entity/Mob/equipment/attribute state, owner, nullable bound origin
and optional life counter persist. Charging, target, goals and Move-Control
request do not. Missing `life_ticks` reconstructs unlimited life; present
values reconstruct limited life. Loot and advancements reload server-side.
Language, sounds, model and textures reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.Entity`;
`net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.Vex`;
`net.minecraft.world.entity.monster.Vex$VexChargeAttackGoal`;
`net.minecraft.world.entity.monster.Vex$VexRandomMoveGoal`;
`net.minecraft.world.entity.monster.Vex$VexMoveControl`;
`net.minecraft.world.entity.monster.Vex$VexCopyOwnerTargetGoal`;
`net.minecraft.world.entity.monster.illager.Evoker`;
`net.minecraft.world.entity.monster.illager.Evoker$EvokerSummonSpellGoal`;
`net.minecraft.world.entity.ai.goal.target.HurtByTargetGoal`;
`net.minecraft.world.entity.OwnableEntity`;
`net.minecraft.world.entity.EntityReference`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.fixes.StatsCounterFix`;
`net.minecraft.util.datafix.schemas.V705`; `V1460`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.VexRenderer`;
`net.minecraft.client.renderer.entity.state.VexRenderState`;
`net.minecraft.client.model.monster.vex.VexModel`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,
loot_table,worldgen/biome,advancement}`;
`reports/minecraft/components/item/{iron_sword,vex_spawn_egg}.json`;
`data/minecraft/loot_table/entities/vex.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/textures/entity/illager/{vex,vex_charging}.png`;
`assets/minecraft/sounds.json`; `assets/minecraft/lang/en_us.json`;
`ENT-EVOKER-001`; `MOB-DESPAWN-001`; `MOB-RAID-001`;
`ITM-IRON-MATERIAL-001`; `ITM-ENCHANT-001`;
`WGEN-STRUCTURE-WOODLAND-MANSION-001`; `CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-038` across raw/finalized/Evoker-summoned/loaded Vexes; every
metadata/NBT/owner/bound/life state; all goal/target and reduced-delay
branches; random candidates; complete charge and collision result; physics/
gravity/block-effect ordering and Move-Control math; starvation and despawn;
finalization/equipment; every Evoker count/create/team/insertion result; zero
biome/Trial/template production; empty loot, XP, criteria/Egg/compatibility;
all sounds and exact armed/two-texture/wing states.

**Limits:**

Generic lifecycle, metadata, target/goal scheduling, damage/death,
invulnerability, equipment/enchantment, Monster despawn, Evoker common spell
state, entity insertion, scoreboard, loot, advancements, Spawn-Egg
interaction and renderer submission retain their cited owners. This leaf
owns Vex selectors, overrides, constants and their exact composition.
