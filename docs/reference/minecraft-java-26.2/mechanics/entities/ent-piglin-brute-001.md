# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-PIGLIN-BRUTE-001` - Piglin Brutes bind HOME-centered brain combat to bastion production and zombification

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-005`, `ENT-DAMAGE-001`, `ENT-BLOCK-001`,
`ENT-DAMAGE-REDUCE-001`, `ENT-KNOCKBACK-001`, `ENT-006`,
`ENT-EFFECT-001`, `ENT-007`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`,
`MOB-UNIVERSAL-ANGER-001`, `ITM-001`, `ITM-ENCHANT-001`,
`ITM-ADVANCEMENT-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-DIMENSION-001`, `WGEN-JIGSAW-BASTION-001`,
`WGEN-PORTAL-001`, `CLI-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` - locked registration, `AbstractPiglin`, `PiglinBrute`,
the complete Brute brain/sensor classes, shared Piglin retaliation,
the already decoded bastion templates, entity loot and advancement records,
seven sound joins and the complete client renderer close protocol entity
ID `102`.

**Applies when:**

`minecraft:piglin_brute` is constructed, finalized, placed from a bastion
template or another production path, sensing or choosing a target, idling
around HOME, fighting, hurt, sharing anger, picking up equipment, crossing
a zombifying environment, saved, killed, synchronized, heard, imitated by
a Parrot or rendered.

**Authoritative state:**

Protocol entity ID `102` constructs `PiglinBrute` in `MONSTER`, and
registration excludes it from Peaceful. Registration fixes width/height
`0.6x1.95`, eye height `1.79`, passenger attachment `2.0125`, riding
offset `-0.7`, client tracking range `8` and the default update interval
`3`.

The Monster attribute supplier is overridden to maximum health `50`,
movement speed `0.3499999940395355`, attack damage `7` and follow range
`12`. Other Monster defaults remain inherited. XP reward is `20`.

Piglin Brute defines no subtype metadata. It inherits Abstract Piglin
metadata slot `16`, serializer `BOOLEAN`, default false, as
`immune_to_zombification`; inherited Entity, Living-Entity and Mob data
occupy slots `0..15`. It also inherits loot pickup enabled, ground
navigation that can open doors, path malus `16` for
`FIRE_IN_NEIGHBOR`, and path malus `-1` for `FIRE`.

Server persistence supplements generic Mob data with
`IsImmuneToZombification` and `TimeInOverworld`. Loading
`CanPickUpLoot` defaults it to true when absent, while the two conversion
fields default to false and zero. Brain persistence remains owned by the
generic Mob/Brain transaction.

Finalization first writes `HOME` to the Brute's current dimension and
block position, then places one Golden Axe in the main hand. Generic Mob
finalization subsequently adds a permanent follow-range multiplier drawn
from `triangle(0,0.11485000000000001)` when that modifier is absent, and
sets left-handed state from `nextFloat < 0.05`.

**Transition and ordering:**

### Brain inventory and activities

The Brute provider explicitly adds
`NEAREST_VISIBLE_ADULT_PIGLINS` and installs the standard nearest-living,
nearest-player, nearest-item and hurt-by sensors plus the
Piglin-Brute-specific sensor. Its activities are exactly `CORE` priority
`0`, `IDLE` priority `10` and `FIGHT` priority `10`.

`CORE` runs, in order:

1. `LookAtTargetSink(45,90)`;
2. `MoveToTargetSink`;
3. `InteractWithDoor`; and
4. `StopBeingAngryIfTargetDead`.

`IDLE` runs target acquisition, one weighted look behavior, one weighted
movement behavior, and Player look/interaction within distance `4`.
The look selector has five equal-weight choices: Player within `8`,
Piglin within `8`, Piglin Brute within `8`, any entity within `8`, or
`DoNothing(30,60)`.

The movement selector has weights `2/2/2/2/2/1`:

| Weight | Behavior |
|---:|---|
| `2` | random stroll at speed multiplier `0.6` |
| `2` | interact with Piglin within `8`, speed `0.6`, close distance `2` |
| `2` | interact with Piglin Brute within `8`, speed `0.6`, close distance `2` |
| `2` | stroll toward HOME at speed `0.6`, close distance `2`, too-far distance `100` |
| `2` | stroll around HOME at speed `0.6`, radius `5` |
| `1` | `DoNothing(30,60)` |

`FIGHT` requires `ATTACK_TARGET`. It stops attacking when the current
target is no longer the target returned by the exact acquisition order,
walks toward an out-of-reach target at speed multiplier `1`, and performs
the shared melee behavior with cooldown `20`.

After each Brain tick, activity selection chooses the first valid of
`FIGHT`, then `IDLE`, and synchronized Mob aggression is set to whether
`ATTACK_TARGET` is present. A transition into FIGHT plays the angry
sound. Independently, every server AI tick consumes `nextFloat`; a result
below `0.0125` asks the active non-core activity to play its sound, which
only FIGHT maps to the angry sound.

### Sensors, target selection and retaliation

The Brute-specific sensor selects the closest visible Wither Skeleton or
Wither Boss as `NEAREST_VISIBLE_NEMESIS`, and refreshes
`NEARBY_ADULT_PIGLINS` through the shared Piglin scan.

Target acquisition uses the first available candidate:

1. the living entity named by `ANGRY_AT`, but only while attackable when
   line of sight is ignored;
2. `NEAREST_VISIBLE_ATTACKABLE_PLAYER`; or
3. `NEAREST_VISIBLE_NEMESIS`.

Setting a Brute anger target erases
`CANT_REACH_WALK_TARGET_SINCE` and stores the target UUID in `ANGRY_AT`
for `600` ticks. The FIGHT stop condition recomputes the ordered choice,
so a higher-priority candidate can invalidate a current lower-priority
target.

After inherited damage succeeds, a living attacker is offered to
retaliation. Every `AbstractPiglin` attacker is ignored. Other attackers
enter the shared `PiglinAi.maybeRetaliate` path, including attackability,
avoidance and current-target-distance gates, adult-Piglin broadcast, and
the `universal_anger` Player redirection specified by
`MOB-UNIVERSAL-ANGER-001`.

`canHunt` is always false. The Brute therefore neither starts a Hoglin
hunting cooldown nor accepts a broadcast Hoglin hunting target, while the
remaining shared retaliation branches still apply.

### Equipment and combat pose

The Brute considers only an exact Golden Axe item for pickup; every other
item is rejected before generic equipment admission. A Golden Axe still
must pass the inherited hold/equipment-slot and current-weapon comparator.
An accepted stack follows the generic single-slot equip transaction:
possible old-stack emission, source-stack shrink, drop-on-kill chance,
and persistence-required state. Those comparison and emission details
remain with the generic Mob and item owners.

The visible arm pose is `ATTACKING_WITH_MELEE_WEAPON` exactly when Mob
aggression is true and the main-hand stack has a `TOOL` data component.
It is `DEFAULT` otherwise. This is a component check, not an exact
Golden-Axe check, so command-replaced tool-bearing main-hand items can
retain the attack pose even though the Brute will not pick them up.

### Zombification

Conversion is active exactly while all three conditions hold:

- slot `16` is false;
- NoAI is false; and
- the `piglins_zombify` environment attribute is true at the Brute's
  position.

Each active server AI step increments `TimeInOverworld`; any inactive step
resets it to zero. Conversion occurs only when the incremented value is
greater than `300`, hence on the 301st consecutive active step.

At conversion, non-Peaceful difficulty first plays the Brute conversion
sound. Peaceful suppresses that sound but does not suppress conversion.
The Brute then converts singly to `minecraft:zombified_piglin`, keeping
equipment, preserving loot-pickup state and retaining its team through
the generic conversion transaction. The result receives Nausea for
`200` ticks at amplifier `0`.

The client derives conversion shaking from the same synchronized immunity
state plus its environment view; `TimeInOverworld` itself is not metadata.
The environment-attribute values, generic entity replacement and Nausea
effect remain with their cited owners.

### Production, loot and progression

Piglin Brute has no `SpawnPlacements` registration and no baseline biome
spawn row. Generic lookup therefore supplies the default unrestricted
placement, `MOTION_BLOCKING_NO_LEAVES` heightmap and always-true placement
predicate; that lookup alone does not produce a natural spawn entry.

Ordinary baseline production is instead the closed
`WGEN-JIGSAW-BASTION-001` transaction. Both
`bastion/mobs/melee_piglin` and
`bastion/mobs/melee_piglin_always` contain a health-`50` persistent
Piglin Brute with loot pickup enabled. Template placement constructs with
STRUCTURE reason, applies transform and UUID removal, finalizes, then
inserts with passengers. The nominal ordinary-Piglin pool can therefore
select a Brute, while the melee pool can select either Brute template.

The named entity loot table declares the `entity` parameter set and random
sequence `minecraft:entities/piglin_brute` but has no pools. It emits no
table-selected items; inherited equipment drops and XP remain separate
generic death consequences.

Both `adventure/kill_a_mob` and `adventure/kill_all_mobs` contain Piglin
Brute criteria. The Spawn Egg is item protocol ID `1239`, common, maximum
stack `64`, with `entity_data.id = minecraft:piglin_brute`.
No direct entity-type tag names Piglin Brute.

### Sounds and client projection

The locked species sound joins are:

| Protocol ID | Event | English subtitle |
|---:|---|---|
| `1290` | Piglin Brute Ambient | "Piglin Brute snorts" |
| `1291` | Piglin Brute Angry | "Piglin Brute snorts angrily" |
| `1292` | Piglin Brute Death | "Piglin Brute dies" |
| `1293` | Piglin Brute Hurt | "Piglin Brute hurts" |
| `1294` | Piglin Brute Step | "Piglin Brute steps" |
| `1295` | Piglin Brute Converted to Zombified | "Piglin Brute converts to Zombified Piglin" |

Ambient playback is admitted only while the shared Piglin idle predicate
holds. Step sound uses volume `0.15` and pitch `1`. Parrot imitation maps
Piglin Brute to sound-event ID `1234`,
`entity.parrot.imitate.piglin_brute`, subtitle "Parrot snorts".

The client registers `PiglinRenderer` with `PIGLIN_BRUTE` for both adult
and baby base-model arguments, and `PIGLIN_BRUTE_ARMOR` for both armor
arguments. The shared humanoid renderer has shadow radius `0.5`, adds the
armor layer and uses custom-head Z transform `1.0019531`.

Render extraction marks the state as a Brute by exact entity type, copies
arm pose and conversion state, and selects
`textures/entity/piglin/piglin_brute.png`. Conversion adds renderer
shaking. The dedicated texture is `64x64`, `1,131` bytes, SHA-256
`6c93d3cc51c8ef44895db795a58a5c365200537dfee44d8b73deeb889a0678c2`.
English labels are "Piglin Brute" and "Piglin Brute Spawn Egg".

**Branches and aborts:**

- Registration excludes ordinary live Brutes in Peaceful.
- A missing or invalid Brain candidate falls through in the exact
  anger, Player, nemesis order.
- Damage by any Abstract Piglin never starts Brute retaliation.
- Shared retaliation can reject avoidance, attackability and farther-target
  cases before changing memory or broadcasting.
- Every non-Golden-Axe pickup is rejected before equipment comparison.
- Immunity, NoAI or a false environment value resets conversion progress
  to zero on that server step.
- Peaceful suppresses only conversion sound, not the replacement.
- Empty loot-table pools never consume an item-producing loot draw.
- Template decode, construction and insertion failures retain the bastion
  owner's nontransactional behavior.

**Invariants:**

- Slot `16` is the sole synchronized Brute-specific inherited flag.
- FIGHT is active only with `ATTACK_TARGET`; aggression mirrors that
  memory.
- HOME is the finalization position and is not periodically recentered.
- Target priority is anger, visible attackable Player, then visible
  Wither nemesis.
- Brutes never hunt Hoglins.
- Conversion requires 301 consecutive active server AI steps.
- Baseline biome lists contain no Piglin Brute row.
- The entity loot record contains zero pools.

**Constants and randomness:**

Entity/Egg IDs `102/1239`; dimensions/eye/passenger/riding
`0.6x1.95/1.79/2.0125/-0.7`; range/update `8/3`; health/speed/attack/follow
`50/0.3499999940395355/7/12`; XP `20`; core/idle/fight priorities
`0/10/10`; look sink `45..90`; look range `8`; Player interaction `4`;
idle speed `0.6`; interaction range/close distance `8/2`; HOME close/far/
radius `2/100/5`; idle waits `30..60`; fight speed/cooldown `1/20`;
anger `600`; random angry probability `0.0125` per server AI tick;
conversion threshold `>300`; Nausea `200/0`; follow-range triangle
`0/0.11485000000000001`; left-handed probability `0.05`; step
volume/pitch `0.15/1`; shadow `0.5`; head Z `1.0019531`.

**Side effects:**

HOME and combat memories; active activity and Mob aggression; look,
navigation, door and interaction state; attacks and damage; anger
broadcast; equipment, item entities, drop chance and persistence;
conversion counter, metadata and replacement entity; Nausea; loot/XP and
advancement progress; sounds, shaking, model pose, armor and texture
projection.

**Gates:**

Logical side, Peaceful, NoAI and persistence; Brain memory/activity
requirements; sensor visibility and attackability; target priority;
retaliation distance/avoidance/universal-anger rules; equipment comparator;
environment attribute and conversion counter; template and insertion
admission; death attribution; resources.

**Boundary cases and quirks:**

The conversion check follows the increment and uses strict `>300`.
Persisted `TimeInOverworld = 300` therefore converts on the next active
step, while one inactive step resets it to zero. Peaceful is tested only
around sound playback after the threshold is crossed.

The target validity behavior does not merely ask whether the current
target remains attackable. It recomputes the ordered candidate and stops
if that result is a different entity.

The renderer accepts a baby-model argument but Brute registration and
normal finalization do not create a baby subtype. Both renderer arguments
deliberately name the same Brute layer.

**Failure semantics:**

An equipment candidate can pass exact-item pickup admission and still
fail the generic comparator without changing either stack. Failed entity
replacement retains the generic conversion owner's outcome; prior counter
and sound effects are not rolled back here. Failed loot emission,
advancement grant or client resource load does not roll back death or
server AI state.

**Client/server authority split:**

The server owns Brain state, aggression, navigation, combat, equipment,
conversion progress and replacement, death and progression. Clients
consume inherited metadata, equipment and movement, derive arm pose and
conversion shaking, and own model, armor, texture, subtitle and sound
projection.

**Observability:**

Observe registration, attributes and slot `16`; finalization RNG and HOME;
every activity/weighted selector and target-priority transition; nemesis
sensor and retaliation/broadcast gates; Golden-Axe admission/comparison;
conversion continuity, reset, save/reload and tick-301 boundary in every
difficulty; both bastion templates; empty loot, two advancements and Egg;
six species sounds, Parrot imitation, renderer pose/shake/armor, texture
hash and labels.

**Persistence and reload:**

`IsImmuneToZombification` and `TimeInOverworld` supplement generic Mob
state; `CanPickUpLoot` has the Abstract-Piglin true-on-absence load
default. HOME and other codec-backed Brain memories follow generic Brain
persistence. Activity execution, sensor cadence and transient behavior
state rebuild from the provider. Bastion/template data, advancements and
loot reload server-side; texture, model and language resources reload
client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.ConversionParams`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.piglin.AbstractPiglin`;
`net.minecraft.world.entity.monster.piglin.PiglinBrute`;
`net.minecraft.world.entity.monster.piglin.PiglinBruteAi`;
`net.minecraft.world.entity.monster.piglin.PiglinAi`;
`net.minecraft.world.entity.ai.sensing.PiglinBruteSpecificSensor`;
`net.minecraft.world.entity.ai.behavior.StartAttacking`;
`net.minecraft.world.entity.ai.behavior.StopAttackingIfTargetInvalid`;
`net.minecraft.world.entity.ai.behavior.MeleeAttack`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.advancements.packs.VanillaAdventureAdvancements`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.PiglinRenderer`;
`net.minecraft.client.model.monster.piglin.AdultPiglinModel`;
`net.minecraft.client.model.geom.ModelLayers`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,
mob_effect,loot_table,advancement,environment_attribute,game_rule,
worldgen/template_pool}`;
`reports/minecraft/components/item/piglin_brute_spawn_egg.json`;
`data/minecraft/loot_table/entities/piglin_brute.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/structure/bastion/mobs/{melee_piglin,
melee_piglin_always}.nbt`;
`assets/minecraft/textures/entity/piglin/piglin_brute.png`;
`assets/minecraft/lang/en_us.json`;
`WGEN-JIGSAW-BASTION-001`; `MOB-UNIVERSAL-ANGER-001`;
`ENT-EFFECT-001`; `ENT-DEATH-001`; `ITM-ENCHANT-001`; `CLI-006`.

**Test vectors:**

Run `EXP-ENT-031` across construction/finalization/metadata/persistence,
all Brain activities and weighted choices, candidate-priority changes,
Wither sensing, damage and universal-anger broadcasts, equipment
admission/comparison, every conversion gate and counter boundary, both
bastion templates, loot/advancement/Egg joins, seven sounds, texture and
render pose/shaking/armor.

**Limits:**

Generic lifecycle, Brain scheduling, behavior implementations, pathing,
melee damage, equipment comparison/drop, conversion replacement, Nausea,
death, advancements, Spawn Egg interaction and client entity transport
retain their cited owners. `WGEN-JIGSAW-BASTION-001` remains the sole
owner of template/pool selection and entity insertion, while
`MOB-UNIVERSAL-ANGER-001` remains the sole owner of the shared game-rule
transaction.
