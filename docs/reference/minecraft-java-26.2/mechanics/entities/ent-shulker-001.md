# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-SHULKER-001` - Shulkers bind shell expansion and surface attachment to teleporting Bullet duplication

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`,
`MOB-002`, `MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`,
`ITM-SHULKER-SHELL-001`, `ITM-ADVANCEMENT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-STRUCTURE-END-CITY-001`,
`WGEN-PORTAL-001`, `CLI-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` - locked registration, `Shulker`, all six nested
goal/control classes, the already specified Shulker-Bullet runtime,
End-City marker production, loot, advancements, tags, migrations, sounds,
color/component projection and client renderer close protocol entity ID
`112`.

**Applies when:**

`minecraft:shulker` is constructed, finalized, created by an End-City
marker, Egg, spawner, command or Shulker-Bullet duplication, attached,
opened, closed, displaced, teleported, targeted, hit, duplicated, saved,
loaded, killed, synchronized, heard, imitated by a Parrot or rendered.

**Authoritative state:**

Protocol entity ID `112` constructs `Shulker` in `MONSTER`, but the entity
type is not Peaceful-excluded. It is fire immune and may spawn far from a
Player. Registration fixes dimensions `1x1`, eye height `0.5`, client
tracking range `10` and default update interval `3`.

The Mob attribute supplier is overridden only to maximum health `30`;
follow range remains `16` and base armor remains zero. Construction fixes
XP reward `5`, installs Shulker-specific look/body controls and never
despawns for distance. The direct `minecraft:fall_damage_immune` tag
suppresses fall damage. The entity emits no movement event, reports zero
delta movement, ignores every delta-movement write and direct push, and is
collidable while alive.

Shulker adds three metadata entries after generic slots `0..15`:

| Slot | Serializer | Meaning | Default |
|---:|---|---|---|
| `16` | Direction | attachment face, pointing from body to support | Down |
| `17` | Byte | raw peek target | `0` |
| `18` | Byte | color ID | `16` (uncolored) |

Persistence stores `AttachFace` through the legacy Direction codec and
stores `Peek` and `Color` as bytes. Missing or invalid attachment defaults
Down; missing peek defaults `0`; missing color defaults `16`. Loading
writes all three metadata entries directly rather than calling the private
peek transition.

Color IDs `0..15` select the corresponding Dye-Color texture and component.
ID `16` and every positive byte above `15` are uncolored. Negative bytes
reach `DyeColor.byId`, whose locked out-of-bounds strategy is Zero, and
therefore project as White. `minecraft:shulker_color` is exposed as an
implicit entity component; applying a present Dye Color writes its
`0..15` ID. Bullet-created offspring copy the parent's optional variant,
including uncolored.

Finalization sets body yaw to zero, copies that yaw to head yaw, snapshots
old position/rotation and then performs generic Mob finalization:
follow-range `triangle(0,0.11485000000000001)` when absent and
left-handed from `nextFloat<0.05`. End-City markers and Bullet duplication
do not call finalization.

**Transition and ordering:**

### Shell state and covered armor

The private peek setter first removes permanent Armor modifier
`minecraft:covered`. For input exactly zero it adds that modifier back as
`+20` Add-Value, plays Shulker Close at `1/1`, emits Container-Close and
writes the input as a byte. For every nonzero input it instead plays
Shulker Open, emits Container-Open and writes the byte.

Construction, persistence load, position changes and teleport close the
Shulker by writing slot `17` directly. Those routes do not remove or add
the covered modifier and do not play a sound or emit a container event.
Consequently visual closure (`rawPeek==0`) and covered armor are separate
authorities:

- a fresh Shulker is closed with armor `0`;
- a private-setter close has armor `20`;
- a private-setter open has armor `0`;
- a directly written close retains whatever armor state existed before
  the write; and
- the permanent modifier itself persists through generic attribute
  serialization, while the direct Peek byte persists separately.

This permits closed armor `0` and `20`, and a directly closed Shulker can
retain `0` after teleport until a later AI open/close transition.

Every tick copies `currentPeekAmount` to its old value, converts the signed
raw byte to target `raw*0.01`, and approaches by `0.05`. Decreasing uses
`clamp(current-0.05,target,1)`; increasing uses
`clamp(current+0.05,0,target)`. Ordinary goals use `0`, `30` and `100`, so
their targets are `0`, `0.3` and `1`, reached in at most `20` ticks.
Command-supplied negative and greater-than-100 bytes retain the exact
signed target and clamp behavior.

Physical lid progress for animated value `p` is

`0.5 - sin((0.5+p)*pi)*0.5`.

The bounding box is a scale-sized base expanded from `-1` to that physical
progress along the direction opposite the attachment face. When attached
Down and animation `p>0`, default dimensions additionally scale height by
`1+p`; other faces retain base default dimensions while their explicit
bounding box expands sideways or downward.

On every animation change, position and dimensions refresh. Only a
positive physical-progress delta pushes entities in the newly occupied
slice by `delta*scale` along the opening direction with mover type
`SHULKER`. Spectators, other Shulkers, no-physics entities and passengers
of the same vehicle are excluded. Closing never pulls entities.

Scale sanitization caps the generic scale at `3`, so expansion and push
distance use at most that scale.

### Position, attachment and teleport

While not a passenger, every `setPos(x,y,z)` snaps to
`(floor(x)+0.5, floor(y+0.5), floor(z)+0.5)`. A passenger delegates exact
coordinates. After the first tick, a changed block position directly
closes slot `17`, sets the generic synchronization flag, and on the client
starts a six-step old-to-new render interpolation.

On each server tick, a nonpassenger that cannot remain on its current face
searches Direction enum order, Down, Up, North, South, West, East, and
selects the first viable face. A viable attachment requires:

1. the Shulker's block is air, or is a Moving Piston at its exact current
   block;
2. the neighbor in the candidate face is loaded and can support the entity
   on its opposite face; and
3. the fully open, scale-aware Shulker box deflated by `1e-6` has no
   collision.

If no current-block face works, the Shulker tries to teleport. Teleport is
rejected while NoAI or dead. Up to five attempts each consume three entity
RNG draws, independently selecting X/Y/Z offsets uniformly from `-8..8`.
A candidate must be strictly above minimum build Y, be an empty block, be
inside the world border, have a unit block AABB deflated by `1e-6` free of
collision and have at least one attachable surface by the same ordered
test.

The first success dismounts, installs the selected face, plays Shulker
Teleport at `1/1`, snaps to candidate `(x+0.5,y,z+0.5)`, emits Teleport at
the old block position with the Shulker as context, directly closes slot
`17`, clears the target and returns true. Five rejected candidates return
false without moving or clearing the target.

Movement with `MoverType.SHULKER_BOX` ignores its vector and merely attempts
this teleport. Other mover types delegate, but the subtype still reports
zero velocity, ignores velocity assignment and ignores pushes.

Riding first clears client interpolation, forces attachment Down and then
delegates admission. Client-side dismount records the current block as its
old attachment position and both sides reset body yaw old/current to zero.
Ordinary entity interpolation is disabled by returning no interpolation
handler.

### Goal graph and targeting

The goal selector is exactly:

| Priority | Goal |
|---:|---|
| `1` | look at Player within `8`, probability `0.02`, horizontal-only |
| `4` | Shulker Attack |
| `7` | Shulker Peek |
| `8` | random look around |

The target selector is:

1. priority `1` Hurt-By-Target, excluding the Shulker's own runtime class
   as attacker and alerting eligible allies;
2. priority `2` nearest Player with sight required, disabled in Peaceful;
3. priority `3` nearest living Enemy with sight required, no reach
   requirement and random interval `10`, enabled only when the Shulker has
   a scoreboard team.

Nearest searches are attachment-aware. With inherited follow distance
`d=16`, an X-axis attachment inflates the search box by `(4,d,d)`, a
Z-axis attachment by `(d,d,4)`, and a Y-axis attachment by `(d,4,d)`.

The Shulker body-rotation control performs no client tick and look control
does not clamp head rotation to the body. Desired X rotation is always
zero. Desired Y rotation projects the wanted direction into the plane
normal to the opposite attachment face and returns
`atan2(-crossDot,forwardDot)*57.2957763671875` when either dot magnitude
exceeds `1e-5`. Head X/Y limits are both `180`.

### Attack and idle peek

Shulker Attack requests Move and Look. It starts only with a live target
outside Peaceful, stores attack time `20`, and private-sets peek to `100`.
It updates every tick, decrements attack time and looks at the target with
limits `180/180`.

While squared distance is strictly below `400`, an attack time at or below
zero:

1. stores `20 + nextInt(10)*10`, one of `20..110` in steps of `10`;
2. constructs protocol entity `113`, Shulker Bullet, with the Shulker as
   owner, current target and the attachment face's axis as the initially
   excluded homing axis;
3. ignores the insertion result; and
4. plays Shulker Shoot at volume `2` and pitch
   `1+(nextFloat-nextFloat)*0.2`.

At squared distance exactly `400` or greater it clears the target without
firing. Stop private-sets peek to zero, thereby installing covered armor
and producing Close effects.

The independently specified Bullet selects homing legs of
`10/20/30/40/50` ticks, deals `4`, and on a successful living hit applies
Levitation for `200` ticks. Its pathing, collision, destruction, sounds
and client spark remain `ENT-PROJECTILE-001`.

Idle Peek has no goal flags. It is admitted only without a target, with
`nextInt(reducedTickDelay(40))==0`, and while the current attachment is
still viable. Start stores
`adjustedTickDelay(20*(1+nextInt(3)))`, hence `10/20/30` goal ticks for
this non-every-tick goal, and private-sets peek to `30`. Its tick
decrements the counter; it continues only while targetless and positive.
Stop private-closes only if a target is still absent, allowing an attack
transition to own the subsequent peek state.

### Damage, emergency teleport and duplication

A closed Shulker rejects damage whose direct entity is any Abstract Arrow
before generic damage, including arrows whose Damage Type is otherwise not
tagged projectile. Other sources delegate to generic Golem/Living damage.

After successful damage, post-damage health strictly below half of maximum
first consumes `nextInt(4)`. A zero result attempts teleport and then skips
all Bullet-duplication logic whether teleport succeeds or fails. At half
health or above there is no such draw; below half, nonzero results continue.

The continuation recognizes only a Damage Source in
`minecraft:is_projectile` whose nonnull direct entity is exactly a Shulker
Bullet. It then duplicates only if the Shulker is open and its own
teleport succeeds. The original position and box are captured before that
teleport.

After success, all alive Shulkers in the old box inflated by `8` are
counted. The teleported original remains inside this query at every
inclusive `-8..8` candidate endpoint, so

`density = (count-1)/5 = nearbyOtherShulkers/5`.

The level RNG consumes `nextFloat`; a result below density aborts.
Otherwise a new Shulker is created with reason `BREEDING`, copies the
parent's optional color, snaps to the parent's old position and is inserted
without finalization or rollback on a false insertion result. Duplication
chance is therefore `1 - otherCount/5` for `0..5` other Shulkers and zero
from five onward.

A closed Shulker Bullet can still deal its ordinary damage because it is
not an Abstract Arrow, but the duplication helper returns while closed.
Below half health, the one-in-four emergency branch suppresses duplication
even when its teleport attempt fails.

### Production, loot and progression

Spawn placement is `NO_RESTRICTIONS` with
`MOTION_BLOCKING_NO_LEAVES` and the shared Mob predicate, but all `66`
biome files contain zero Shulker spawn rows. Monster caps and pack walking
therefore provide no baseline production.

`WGEN-STRUCTURE-END-CITY-001` owns nine `Sentry*` data-marker cells across
the locked End-City templates. Each reached marker in spawnable bounds
creates a Shulker with reason `STRUCTURE`, positions it at
`(x+0.5,y,z+0.5)` and ignores insertion failure. It does not finalize, set
persistence or consume caller RNG. Shulker distance-despawn is always
false, so accepted marker entities remain without a persistence flag.

Bullet duplication is the renewable producer described above. Spawn Egg
protocol item ID `1246` carries
`minecraft:entity_data={id:"minecraft:shulker"}` and uses the generic Egg
transaction; Shulker Color can be supplied by implicit entity components.
Commands and spawners retain their generic finalization distinctions.

The sole entity-loot pool is already owned by
`ITM-SHULKER-SHELL-001`. It rolls once with sequence
`minecraft:entities/shulker`; Shell chance is `0.5` without positive
Looting and `0.5+0.0625L` for Looting level `L>0`, consuming one float and
passing on strict less-than. No player-kill condition exists. XP is `5`.

Shulker is directly tagged only `minecraft:fall_damage_immune`. It
participates in `adventure/kill_a_mob` and
`adventure/kill_all_mobs`. A Bullet-applied Levitation effect can advance
`end/levitate` once the Player's vertical distance reaches at least `50`;
that challenge grants `50` XP and remains an effect/progression join rather
than a Shulker kill condition.

### Client projection

Species sound events are Ambient `1463`, Close `1468`, Death `1469`, Hurt
`1470`, Hurt-Closed `1471`, Open `1472`, Shoot `1473` and Teleport `1474`.
Ambient playback is suppressed while raw peek is zero. Parrot imitation is
`1237`; Bullet Hit/Hurt are `1466/1467`.

When a nonpassenger client changes block after tick zero, it stores the old
block and sets interpolation steps to `6`. Each client tick decrements the
steps, then clears the old position after expiry. At partial tick `a`, the
render offset is the negative block delta multiplied by

`(((steps-a)/6)^2)*scale`.

The model therefore begins at the old block and converges quadratically to
the new block. Riding clears interpolation. If normal frustum testing
fails but an old position exists, the renderer tests the swept AABB between
that interpolated position and the current block center, inflated by entity
half-width/half-height.

`ShulkerRenderer` uses `ModelLayers.SHULKER`, shadow radius zero and the
texture selected by color. It adds the inherited entity rotation, then
rotates around `(0,0.5,0)` by the quaternion of the attachment face's
opposite. The render state carries offset, color, interpolated peek, head
and body yaw and attachment.

With `f=(0.5+peek)*pi` and `g=-1+sin(f)`, the model sets lid Y to
`16+sin(f)*8+sin(age*0.1)*0.7` only for the last wobble term when `f>pi`.
For peek above `0.3`, lid yaw is `g^4*pi*0.125`; otherwise it is zero.
Head pitch is render X rotation in radians and head yaw is
`(yHeadRot-180-yBodyRot)` in radians.

The uncolored texture plus sixteen dye textures are all `64x64`, total
`18,619` bytes. Their sorted-filename payload concatenation has SHA-256
`68a317be1eeeaaf3981cfe397ce5de4c020d60f308bd32010505be20b4b17ab1`.
The Spawn Egg texture is `16x16`, `206` bytes, SHA-256
`78477bfa8d321eb346a94ed29a2932251ac8ddf24066eb7691bd5e9b790e5730`.

**Branches and aborts:**

Peaceful attack suppression without entity removal; NoAI/death teleport
gates; passenger versus snapped positioning; blocked body, support face,
collision, build Y and border; five teleport candidates; private versus
direct peek writes, signed byte animation and positive-only shell pushes;
goal flags, target/team/sight/distance/cooldown; closed Abstract-Arrow
immunity; successful damage, strict half health, emergency draw, projectile
tag/direct Bullet/open/teleport/density/creation; structure bounds,
placement with zero biome rows, loot and client resource gates.

**Constants and randomness:**

Dimensions/eye/range/update `1/0.5/10/3`; health/follow/base armor/covered
armor/XP `30/16/0/20/5`; metadata `16/17/18`; peek step `0.05`, raw goals
`0/30/100`, scale cap `3`; snap `floor(x)+0.5/floor(y+0.5)/floor(z)+0.5`;
attachment order six, collision epsilon `1e-6`; teleport attempts/range
`5/-8..8`; render steps `6`; goal ranges `8/16/4`, probabilities
`0.02`, `1/reducedTickDelay(40)`, defense interval `10`; attack initial
`20`, radius squared `400`, cooldown `20..110` step `10`, sound
`2/[0.8,1.2)` triangular; Bullet legs `10/20/30/40/50`, damage/effect
`4/200`; emergency `nextInt(4)==0`, duplication radius/limit `8/5`;
Shell `0.5+0.0625L`; End-City markers `9`; textures `17`.

**Side effects:**

Attachment, position, target, goal timers and RNG cursors; peek metadata,
permanent armor, sounds, game events, bounding boxes, dimensions and pushed
entities; teleports/dismounts; Bullets, damage, Levitation and offspring;
color components, persistence, loot, XP and advancements; client
interpolation, culling, rotations, animation and textures.

**Gates:**

Logical side, NoAI, liveness, passenger and Peaceful attack state; support,
collision, world border and build bounds; raw peek and armor authority;
target liveness/team/sight/range and goal cadence; direct-entity class,
Damage-Type tag, post-damage health, open state, teleport and density;
structure marker, Spawn Egg/spawner/command admission, loot and resources.

**Boundary cases and quirks:**

Shulkers survive Peaceful but cannot acquire Players or run their attack
goal there. Fresh closed Shulkers have zero armor because metadata default
does not invoke the private setter. Teleport and block-position changes can
visually close an open Shulker without restoring armor. Direct closure also
produces no Close sound/event. Closed immunity tests direct Abstract-Arrow
class rather than the projectile Damage-Type tag. At low health the
one-in-four emergency branch suppresses Bullet duplication even if
teleport fails. Duplication first moves the parent, then creates an
unfinalized child at the abandoned position and ignores insertion failure.
End-City marker Shulkers also skip finalization. Shell opening pushes but
closing never pulls. All ordinary displacement and push inputs are
neutralized, while Shulker-Box displacement becomes a teleport request.

**Failure semantics:**

A five-attempt teleport failure preserves face, position, peek and target.
Failed Bullet insertion still commits cooldown and Shoot sound. Failed
offspring creation stops after the already committed parent teleport and
density draw; failed insertion is not rolled back. Rejected shell pushes
follow each moved entity's collision behavior without rolling back the lid.
Generic damage failure performs no emergency or duplication work. Loot
failure emits no Shell and does not affect death/XP.

**Client/server authority split:**

The server owns attachment validation, snapped position, teleports,
peek/armor transitions, goals, targets, Bullets, damage, duplication,
color/component state, persistence, loot and progression. Both sides
animate peek and may evaluate expansion movement, but authoritative entity
positions come from the server. Clients own six-step render interpolation,
swept culling, attachment rotation, model animation, sound playback and
textures. Client metadata cannot add server armor, teleport, fire a Bullet
or create an offspring.

**Observability:**

Observe registration/Peaceful/fire/fall/despawn state; all metadata bytes,
components, NBT and every direct/private peek divergence; armor across
fresh, open, close, move, teleport and reload; signed-byte lid animation,
boxes and push order; snap/riding and every attachment/teleport candidate;
complete goals, anisotropic target searches, controls and Bullet cadence;
closed Arrow immunity, post-damage half-health branches and density
duplication; nine End-City markers versus zero biome rows, Shell/XP,
advancements, Egg/tag, eleven sound joins, seventeen color textures and
exact interpolation/culling/model projection.

**Persistence and reload:**

Generic entity/Mob state, attributes, scale and optional team save;
`AttachFace`, `Peek` and `Color` supplement it. Covered armor persists as
an attribute modifier independently of Peek, and load does not reconcile
them. Current/old animated peek, client old attachment/interpolation,
targets, goals, timers, controls and failed candidate history do not save.
Loot, tags, End-City data and advancements reload through their owners;
models, language and textures reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.animal.golem.AbstractGolem`;
`net.minecraft.world.entity.monster.Shulker`;
`net.minecraft.world.entity.monster.Shulker$ShulkerAttackGoal`;
`net.minecraft.world.entity.monster.Shulker$ShulkerPeekGoal`;
`net.minecraft.world.entity.monster.Shulker$ShulkerNearestAttackGoal`;
`net.minecraft.world.entity.monster.Shulker$ShulkerDefenseAttackGoal`;
`net.minecraft.world.entity.monster.Shulker$ShulkerLookControl`;
`net.minecraft.world.entity.monster.Shulker$ShulkerBodyRotationControl`;
`net.minecraft.world.entity.projectile.ShulkerBullet`;
`net.minecraft.world.entity.ai.goal.LookAtPlayerGoal`;
`net.minecraft.world.entity.ai.goal.target.HurtByTargetGoal`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.item.DyeColor`;
`net.minecraft.world.level.levelgen.structure.structures.EndCityStructure`;
`net.minecraft.world.level.levelgen.structure.structures.EndCityPieces`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.advancements.packs.VanillaAdventureAdvancements`;
`net.minecraft.util.datafix.schemas.V704`; `V705`; `V808`; `V1458`;
`V1460`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.EntityShulkerColorFix`;
`net.minecraft.util.datafix.fixes.ColorlessShulkerEntityFix`;
`net.minecraft.util.datafix.fixes.EntityShulkerRotationFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.ItemStackComponentizationFix`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.ShulkerRenderer`;
`net.minecraft.client.renderer.entity.state.ShulkerRenderState`;
`net.minecraft.client.model.monster.shulker.ShulkerModel`;
`net.minecraft.client.model.geom.ModelLayers`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,loot_table,advancement}`;
`reports/minecraft/components/item/shulker_spawn_egg.json`;
`data/minecraft/tags/entity_type/fall_damage_immune.json`;
`data/minecraft/loot_table/entities/shulker.json`;
`data/minecraft/advancement/{adventure/kill_a_mob,adventure/kill_all_mobs,end/levitate}.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/structure/end_city/*.nbt`;
`assets/minecraft/textures/entity/shulker/shulker*.png`;
`assets/minecraft/textures/item/shulker_spawn_egg.png`;
`assets/minecraft/lang/en_us.json`;
`ENT-PROJECTILE-001`; `ENT-DAMAGE-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-SPAWN-001`;
`MOB-DESPAWN-001`; `ITM-SHULKER-SHELL-001`;
`ITM-ADVANCEMENT-001`; `WGEN-STRUCTURE-END-CITY-001`;
`CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-033` across raw/finalized/marker/duplicated/loaded production;
all Direction, signed Peek/Color byte, scale, armor and component states;
every support/collision/snap/riding/movement/teleport candidate; all goals,
target axes, controls, ranges and Bullet cooldowns; closed/open Arrow and
Bullet damage at every half-health/draw/teleport/density boundary; End-City
markers, zero biome rows, loot/XP/advancements/Egg/tag/sounds; all color
textures, interpolation steps, swept culling, attachment rotations and
model animation thresholds.

**Limits:**

Generic entity/Mob lifecycle, attribute serialization, collision movement,
targeting, damage/death, effect application, Shulker-Bullet homing/hit
runtime, spawn selection, End-City assembly, loot, advancement and render
submission retain their cited owners. This leaf owns the Shulker's exact
inputs, overrides, joins and observable subtype transaction.
