# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-DOLPHIN-001` — Dolphins trade fish for treasure searches while balancing air, moisture, play and player-swim goals

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`MOB-BREED-001`, `ITM-COD-001`, `ITM-SALMON-001`,
`ITM-TROPICAL-FISH-001`, `ITM-PUFFERFISH-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Dolphin`,
`AgeableWaterCreature`, breath, jump and three Dolphin-specific goal paths,
placement/category code, all 66 biomes, three direct entity tags, six Fish
items, four treasure structures, loot, Spawn Egg, entity/effect migrations,
all 1,212 templates and exact adult/baby client resources close protocol
entity ID `35`.

**Applies when:**

`minecraft:dolphin` is constructed, finalized, naturally selected, spawned
by an Egg, spawner, command or custom selector, loaded, fed, age-locked,
leashed, breathing, drying, flopping, following a swimmer, seeking treasure,
jumping, playing with an item, retaliating, killed, synchronized or rendered.

**Authoritative state:**

Protocol entity ID `35` constructs `Dolphin` in `WATER_CREATURE`.
Registration fixes adult width/height `0.9×0.6`, eye height `0.3`,
builder-default client tracking range `5` and update interval `3`. Dolphin is
Peaceful-compatible. Attributes are maximum health `10`, movement speed
`1.2000000476837158`, attack damage `3` and inherited follow range `16`.
Movement emission is the Entity default `ALL`, sound volume is `1`, gravity
is `0.08`, Water path malus is `0`, it can be leashed and it is not pushed
by fluid.

Baby collision dimensions are the adult box scaled by `0.65`, hence
`0.585×0.39`, but eye height is replaced with `0.09375`. Its passenger
attachment is `(0,0.3125,0)`. Adult dimensions retain the registration
attachment defaults. `getAgeScale` likewise returns `0.65` for babies and
`1` for adults.

Entity, Living-Entity and Mob state occupies synchronized metadata slots
`0..15`. `AgeableMob` adds slot `16` baby and slot `17` age-locked, both
serializer ID `8` (`BOOLEAN`) and default `false`. Dolphin adds slot `18`
`GotFish`, also Boolean/default false, and slot `19` `Moistness`, serializer
ID `1` (`INT`), default `2400`.

Signed `Age`, signed `ForcedAge`, Boolean `AgeLocked`, Boolean `GotFish` and
signed `Moistness` persist. Missing or wrong-type Dolphin keys use
`false/2400`; equipment, air and generic Mob state retain their owners.
`treasurePos`, the treasure-goal `stuck` bit, its three goal-local player/item
fields, jump `breached` bit and play cooldown are transient and
unsynchronized.

Construction installs `SmoothSwimmingMoveControl(85,10,0.02,0.1,true)`,
`SmoothSwimmingLookControl(10)` and enables loot pickup. Water-Bound
navigation and maximum head X/Y rotations of only `1/1` degree complete the
direct movement configuration.

**Transition and ordering:**

### Air, moisture, land flop and water travel

The Dolphin override of the ageable-water dry-air handler is empty. Its air
instead follows the generic Living-Entity eye-fluid path with maximum
`4800`: while its eye is in Water, not a Bubble Column, and no generic
underwater-breathing exemption applies, air decrements by one per tick
subject to Oxygen-Bonus RNG. At `-20` or below it resets to zero, broadcasts
drowning event `67` and offers `2` Drown damage. Leaving that path invokes
Dolphin's `increaseAirSupply`, which returns `4800` immediately rather than
adding four. Spawn finalization also sets air to `4800` and pitch to zero
before inherited group finalization.

Priority-zero `BreathAirGoal` begins at air strictly below `140`, is
non-interruptible and claims Move/Look. It stops navigation, scans the
current X/Z column from current Y through Y+8 for the first land-pathfindable
empty-fluid or Bubble-Column cell, falling back to the cell eight above,
then moves toward `(x,y+1,z)` at speed `1`. It repeats that search and adds
relative movement `0.02` every goal tick.

Every Dolphin tick completes its superclass first. `NoAI` then resets air to
`4800` and returns without changing moisture, flopping or emitting Dolphin
trail particles. Otherwise Water or rain resets slot `19` to `2400`; a dry
tick decrements it first and, when the result is at most zero, offers `1`
Dry-Out damage every tick.

A dry on-ground Dolphin flops regardless of remaining moisture: it consumes
three floats, adds X/Z `(2*nextFloat-1)*0.2` and Y `0.5` to current velocity,
sets yaw to `nextFloat*360`, clears on-ground state and requests motion
synchronization.

In water, travel applies current speed to the input, moves `SELF`, multiplies
the resulting velocity by `0.9`, and adds Y `-0.005` only when no attack
target exists. The smooth controls own steering and buoyancy; generic
out-of-water travel retains its owner.

On a client, after moisture/flop handling, an in-water Dolphin with total
velocity squared strictly above `0.03` consumes one entity float
`r=1.2-0.7*nextFloat` and emits four local Dolphin particles in two symmetric
pairs. The common Y is `y-viewY`; X/Z are
`position-viewHorizontal*r ± (cos(yaw),sin(yaw))*0.3`. All four requested
velocities are zero.

Dolphin particle protocol ID is `83` and its atlas is one `generic_0` frame.
The client provider fixes color `(0.3,0.5,1)`, alpha
`1-0.7*nextFloat`, scales the quad by `0.5+0.6*nextFloat`, multiplies input
velocity by `0.019999999552965164`, halves
`floor(20/(0.2+0.8*nextFloat))` lifetime and then moves with drag `0.99`.

### Goal graph, retaliation and jumping

Dolphin registers this exact goal graph:

| Priority | Goal and direct configuration |
|---:|---|
| `0` | Breath Air; Try Find Water |
| `1` | non-interruptible Dolphin Swim To Treasure, Move/Look |
| `2` | Dolphin Swim With Player, speed `4`, Move/Look |
| `4` | Random Swimming, speed `1`, interval `10`; Random Look Around |
| `5` | Look At Player within `6`; non-interruptible Dolphin Jump, interval adjusted `10` |
| `6` | Melee Attack, speed `1.2000000476837158`, follow through lost sight |
| `8` | Play With Items; Follow Player Ridden Boat; Follow Player Ridden Nautilus |
| `9` | Avoid Guardian within `8`, near/far speeds `1/1` |

Try Find Water admits an on-ground Dolphin whose current cell is not Water,
scans the fixed nearby `5×3×5` iteration for the first Water-tag fluid and
sets that cell as a speed-`1` move target. Jump declares no control flag, so
it can coexist with flagged goals. Item play claims Move.

Target priority `1` is Hurt By Target, configured to ignore Guardian damage
and alert other eligible Dolphins. Babies fail Dolphin's `canAttack`
override; an adult must also pass the inherited attack predicate. An
admitted melee hit invokes Dolphin Attack at volume/pitch `1/1`.

Jump admission first consumes `nextInt(reducedTickDelay(10))` and requires
zero. Along current horizontal motion direction, steps
`[0,1,4,5,6,7]` must each have Water-tag, non-motion-blocking current cells
and air at one and two cells above. Start adds
`(stepX*0.6,0.7,stepZ*0.6)` to velocity and stops navigation.

Continuation requires not on ground and rejects the return-to-water state
where `dy²<0.029999999329447746`, pitch is nonzero with absolute value below
`10`, and the Dolphin is in water. Stop sets pitch to zero. Each tick, small
vertical velocity lerps pitch toward zero by `0.2`; otherwise nontrivial
velocity sets pitch to
`atan2(-dy,horizontalSpeed)*57.2957763671875`.

The goal's `breached` bit is never reset on later starts. While it is false,
a tick assigns current-cell Water membership to it and plays Dolphin Jump
only on the false-to-true transition. Consequently this goal instance can
produce that direct jump sound at most once over the Dolphin's lifetime.

### Swimming players and Dolphin's Grace

The swim goal queries the nearest player through noncombat targeting at
range `10` with line of sight ignored. Admission needs that player to be
swimming and not equal to the Dolphin's current attack target. Continuation
needs the retained player still swimming at squared distance strictly below
`256`, so an admitted player may remain followed out to distance `16`.

Start gives that player Dolphin's Grace for `100` ticks, amplifier `0`, with
the Dolphin as source. Every tick looks at the player with yaw/pitch limits
`21/1`; below squared distance `6.25` navigation stops, otherwise it moves
to the player at speed `4`. It then consumes the level RNG's `nextInt(6)`
and refreshes the same 100-tick effect only on zero. Stop clears the retained
player and navigation.

Dolphin's Grace is beneficial mob-effect protocol ID `29`, color `8954814`.
While present, the generic water-travel path replaces horizontal water drag
with `0.96`; effect merge, visibility and expiration remain owned by
`ENT-EFFECT-001`. Its `18×18`, `194`-byte icon has SHA-256
`7b346976b32630fb6505aef9e549fcbdc1e5934d4485ea42862f4070187012a9`.
The locked `nether/all_effects` advancement names the effect, not the Dolphin
entity type.

### Fish feeding and treasure search

The Fish item tag contains exactly raw/cooked Cod, raw/cooked Salmon,
Pufferfish and Tropical Fish. A nonempty tagged stack takes precedence over
the inherited Golden-Dandelion interaction. Dolphin Eat plays server-side
at volume/pitch `1/1`.

For an unlocked baby, one fish is consumed and age advances by

`20*floor(0.1*floor((-Age)/20))` ticks,

with forced-age accounting and a 40-tick growth-particle timer. An adult or
age-locked baby instead sets `GotFish=true` and consumes one. Thus an
age-locked baby can arm treasure search rather than grow. All successful
fish interactions return Success; repeated adult feeding consumes another
fish even when `GotFish` is already true.

Golden Dandelion remains available through inherited interaction because it
is not a Fish. Baby/timer/tag admission, lock toggle, age reset to `-24000`,
40-tick lock timer, conditional custom persistence, use/unuse sound and
twenty client particles match the ageable path. Dolphin is not an `Animal`,
base `canBreed()` is false and it registers no breeding goal. Its offspring
factory can still construct a Dolphin with reason `BREEDING` for an external
caller.

Treasure search can start while `GotFish=true` and air is at least `100`.
Start is server-only, clears `stuck`, stops navigation and finds the nearest
structure in reloadable `dolphin_located`, search radius `50`, with known
structures not skipped. The tag expands to exactly Ocean Ruin Cold/Warm and
Shipwreck/Shipwreck Beached. No result sets `stuck=true`; a result stores its
position and broadcasts event `38`.

Continuation requires a stored position, air at least `100`, not stuck and
horizontal distance at least `4`: it compares the Dolphin with
`(treasureX,currentY,treasureZ)`. Stop clears `GotFish` only after reaching
that horizontal radius or becoming stuck. An external stop before either
condition preserves `GotFish`.

When navigation is done or its current navigation target is within `12`,
tick chooses a waypoint toward the treasure center. It tries radius/vertical
range/angle `(16,1,pi/8)`, then `(8,4,pi/2)` when null. A nonnull result that
is not both Water-tag and Water-pathfindable is replaced by a third
`(8,5,pi/2)` sample without a second validity test. Null after that marks
stuck. Otherwise look targets the waypoint with limits `21/1`, navigation
moves at `1.3`, and level RNG result zero from
`nextInt(adjustedTickDelay(80))` broadcasts event `38`.

Event `38` creates exactly seven local Happy-Villager particles, each with
three Gaussian velocity samples scaled by `0.01` and independently randomized
body-relative X/Y/Z. The event occurs once on a successful structure lookup
and may recur during waypoint selection.

### Item pickup, play and guaranteed recovery

The play goal's eligible Item Entity predicate requires no pickup delay,
alive state and current Water membership; item identity is unrestricted.
It queries the Dolphin box inflated by `8` on every admission/start/tick.
Admission waits until absolute `cooldown<=tickCount` and requires an eligible
entity or a nonempty main hand.

Start navigates toward the first returned item at speed
`1.2000000476837158` and plays Dolphin Play when a candidate exists, then
sets cooldown to zero. A tick holding an item throws the entire stack and
clears the hand; otherwise it navigates to the first candidate. Stop also
throws/clears a held stack, then sets cooldown to
`tickCount+nextInt(100)`. Stopping with an empty hand does not arm cooldown.

A thrown stack appears at `(x,eyeY-0.30000001192092896,z)`, receives pickup
delay `40` and records the Dolphin as thrower. It consumes two floats:
`a=nextFloat*2*pi`, `j=0.02*nextFloat`. With yaw `y` and pitch `p` in
radians its velocity is

`(-0.3*sin(y)*cos(p)+cos(a)*j, 0.45*sin(p),
  0.3*cos(y)*cos(p)+sin(a)*j)`.

The middle expression is the code's `0.3*sin(p)*1.5`. Entity insertion
failure is ignored.

Generic pickup requires server side, live state, enabled loot pickup and
`mobGriefing=true`. Dolphin then accepts only while its main hand is empty
and generic `canHoldItem` passes, moves the entire stack into main hand,
marks that slot guaranteed-drop, records/takes the full count and discards
the Item Entity. Pickup does not itself set custom persistence. Dispensers
may likewise equip only the main-hand slot while loot pickup is enabled.
The carrying item is saved by generic equipment persistence and is
guaranteed in the equipment-drop path if death occurs before it is thrown.

### Placement, group finalization and natural selection

Dolphin registers placement `IN_WATER` with heightmap
`MOTION_BLOCKING_NO_LEAVES` and the same surface-ageable-water predicate as
Squid. The placement-type gate requires non-null type, world border,
Water-tag candidate fluid and nonconducting block above. The species
predicate consumes no RNG and requires candidate Y inclusively in
`[seaLevel-13,seaLevel]`, Water-tag fluid below and exactly `Blocks.WATER`
above. A waterlogged candidate can therefore pass. Spawn obstruction later
requires the constructed Dolphin to be unobstructed.

Exactly five of 66 locked biomes select Dolphin in `water_creature`:

| Biomes | Weight | Group |
|---|---:|---:|
| Deep Ocean, Ocean | `1` | `1..2` |
| Deep Lukewarm Ocean, Lukewarm Ocean, Warm Ocean | `2` | `1..2` |

Null spawn-group data is replaced by `AgeableMobGroupData(0.1)`. The first
member is adult; each later member becomes age `-24000` when one level float
is at most `0.1`. A compatible supplied group retains its chance; an
incompatible non-null object follows inherited cast/failure semantics.
Maximum spawn cluster is inherited `4`.

`WATER_CREATURE` has global cap `5`, is friendly and not
category-persistent, with no-despawn/despawn distances `32/128`. `GotFish`,
active treasure navigation and held guaranteed-drop items do not by
themselves suppress distance removal. No bundled structure or other direct
baseline producer creates a Dolphin.

### Loot, tags, sounds, item and migration projection

The entity loot table has type `entity`, random sequence
`minecraft:entities/dolphin` and one roll. It emits Cod raw item ID `1086`
with base uniform integer count `0..1`, plus the Looting enchanted-count
increase with uniform `0..1`. Furnace Smelt converts an emitted item to
Cooked Cod raw ID `1090` when the Dolphin is on fire or the direct attacker's
main hand matches `smelts_loot`. Eligible generic death also gives XP
`1+nextInt(3)`; guaranteed held equipment is a separate drop path.

Dolphin belongs directly to exactly three entity-type tags:

- `aquatic`, transitively selecting `sensitive_to_impaling`;
- `cannot_be_pushed_onto_boats`, preventing the boat collision loop from
  auto-mounting it while retaining physical push; and
- `not_scary_for_pufferfish`.

It is deliberately absent from `can_breathe_under_water`. No locked
advancement names the exact entity type. Dolphin Spawn Egg is raw item ID
`1182`, stack size `64`, with `entity_data.id=minecraft:dolphin`; generic
Egg construction, component patch, naming, finalization and insertion retain
their owners.

The ten consecutive sound protocol IDs and locked resources are:

| Event | ID | Clips | Entry-volume exceptions | English subtitle |
|---|---:|---:|---|---|
| ambient dry | `519` | `8` | none | Dolphin chirps |
| ambient water | `520` | `10` | clips 1/10 `0.8`, 7/8 `0.75` | Dolphin whistles |
| attack | `521` | `3` | none | Dolphin attacks |
| death | `522` | `2` | none | Dolphin dies |
| eat | `523` | `3` | all `0.75` | Dolphin eats |
| hurt | `524` | `3` | none | Dolphin hurts |
| jump | `525` | `3` | all `0.75` | Dolphin jumps |
| play | `526` | `2` | none | Dolphin plays |
| splash | `527` | `3` | none | Dolphin splashes |
| swim | `528` | `7` | none | Dolphin swims |

Ambient selection uses Water membership and inherited interval `120`.
Exact UTF scanning of all `1,212` structure templates finds zero
`minecraft:dolphin` occurrence.

Three entity compatibility contexts and one directly coupled effect context
own migration:

- `EntityUUIDFix` includes `minecraft:dolphin` in Mob UUID migration;
- `V1470` introduces its modern Mob schema;
- `V705` maps Dolphin Spawn Egg to the entity schema; and
- `MobEffectIdFix` maps legacy numeric effect `30` to
  `minecraft:dolphins_grace`.

Legacy effect ID `30` is unrelated to current effect protocol ID `29` and
entity protocol ID `35`. No fix rewrites `GotFish`, `Moistness`,
`treasurePos`, jump state or play cooldown.

### Client model, carrying layer and textures

`EntityRenderers` binds Dolphin to `DolphinRenderer`, with dedicated adult
and baby model layers, `DolphinRenderState`, shadow radius `0.7` and a
carrying-item layer. Render extraction copies the main-hand item and sets
`isMoving` only when horizontal velocity squared is strictly above `1e-7`.

Both model atlases are `64×64`. Adult geometry has body `8×7×13`, head
`8×7×6`, nose `2×2×4`, paired `1×4×7` side fins, back fin `1×4×5`, tail
`4×5×11` and tail fin `10×1×6`. Dedicated baby geometry has body `6×5×8`,
head `6×5×4`, nose `2×2×2`, paired `1×3×6` side fins, back fin `1×3×4`,
tail `4×3×7` and tail fin `8×1×4`.

Model setup copies state pitch/yaw in radians to the body. While moving it
adds `-0.05-0.05*cos(ageInTicks*0.3)` to body X rotation and sets tail/tail
fin X rotations to `-0.1*cos(ageInTicks*0.3)` and
`-0.2*cos(ageInTicks*0.3)`. Otherwise their baked rotations remain.

For a nonempty held item let `f=abs(pitchDegrees)/60`. Negative pitch
translates the item by `(0,1-0.5f,-1+0.5f)`; nonnegative pitch uses
`(0,1+0.8f,-1+0.2f)`. The current item model renders with ordinary packed
light, no overlay and the entity outline color.

Texture selection follows synchronized baby state:

- adult `textures/entity/dolphin/dolphin.png` is `64×64`, `550` bytes,
  SHA-256
  `3120b2aba9c71e85e47f470117e8d0115d4cfc6fc18f11bb483256c14a90addb`;
  and
- baby `textures/entity/dolphin/dolphin_baby.png` is `64×64`, `441` bytes,
  SHA-256
  `521b147f669eebbdb78320ad3e013978e180c8edb12b02927c7ce3c9d1325371`.

The renderer has no water, moisture, fish, treasure or effect texture
branch and uses ordinary world lighting. English names are `Dolphin` and
`Dolphin Spawn Egg`. The generated Egg model selects its same-named
`16×16`, `241`-byte texture, SHA-256
`b6c3e395c942acd7486decf3ffcda54ee65f8d66b43e86638b2e9a502c6c7d73`.

**Branches and aborts:**

- `NoAI` refreshes air and freezes moisture before Dolphin-local work.
- Water/rain resets moisture; air uses eye Water and Bubble-Column/effect
  gates.
- An age-locked baby eats Fish into `GotFish` rather than growth.
- A missing/failed treasure search clears `GotFish` on stop; an unrelated
  interruption may preserve it.
- The third treasure waypoint is not revalidated for Water/pathfindability.
- Item-goal navigation can run while `mobGriefing=false`, but pickup cannot.
- Jump admission consumes cadence before all six water/surface checks.
- The jump sound latch is never reset across goal runs.

**Constants and randomness:**

Entity/Egg/Cod/Cooked-Cod IDs `35/1182/1086/1090`; adult/baby dimensions
`0.9×0.6/0.585×0.39`, eyes `0.3/0.09375`; range/update `5/3`;
health/speed/attack/follow `10/1.2000000476837158/3/16`; slots
`18 BOOLEAN/19 INT`; air `4800/140/-20/2`; moisture `2400/1`; flop
`0.2/0.5/360`; trail square threshold `0.03`, four particles, particle ID
`83`; swim-player range/continuation/near `10/256/6.25`, effect
`100`, refresh `1/6`, effect ID `29`, drag `0.96`; treasure air/radius/search
`100/4/50`, waypoints `(16,1,pi/8)/(8,4,pi/2)/(8,5,pi/2)`, event cadence
adjusted `80`; jump cadence adjusted `10`, steps `[0,1,4,5,6,7]`, impulse
`0.6/0.7`; item search `8`, throw delay `40`, cooldown `0..99`; baby chance
`<=0.1`; spawn five biomes, groups `1..2`, category `5/32/128`; loot
`0..1` plus uniform Looting increase, XP `1..3`; tags/templates
`3/0 of 1212`; sounds `519..528`; shadow `0.7`.

**Side effects:**

Age/lock/fish/moisture/air/equipment persistence and metadata; transient
treasure/player/item/jump/cooldown state; RNG cursors, navigation, movement,
rotations and leash; damage, attack target, loot/XP and guaranteed item;
Fish/Golden-Dandelion consumption; structure search; effect merge; sounds,
entity events and local particles; spawn selection/finalization/despawn;
renderer age/model/texture/item state.

**Gates:**

Logical side and `NoAI`; eye/current Water, rain, Bubble Column, air,
moisture and ground; age/lock/item tag; fish bit/structure availability/
horizontal radius/waypoint; player targeting/swimming/distance/current
target; jump cadence/direction/water/air/ground; item delay/liveness/water/
main hand/`mobGriefing`; adult attackability; border/Y/three Water positions/
obstruction; biome/category cap; death/Looting/smelting; tags, Egg,
migrations and client baby/velocity/item state.

**Boundary cases and quirks:**

Dry land flopping begins before moisture expires. Moisture damage is offered
every expired dry tick. Any air-refill branch restores all `4800` at once.
Age-locked babies can seek treasure. Treasure continuation ignores vertical
distance and can retain `GotFish` after an external stop. The final waypoint
is accepted without fluid validation. Jump sound state persists for the
entity lifetime. Item-play stop arms cooldown only while a held stack still
exists. A held item is guaranteed-drop but does not imply persistence.

**Failure semantics:**

Rejected placement prevents natural construction/insertion. Generic
insertion failure does not undo finalization. Missing treasure marks stuck;
null waypoint does likewise, while navigation return values are ignored.
Failed thrown-item insertion loses the newly constructed entity without
restoring the held stack. Generic damage, effect, loot, equipment, Egg and
spawn owners retain their commit/rollback rules.

**Client/server authority split:**

The server owns age/lock/fish/moisture/air, goals, targets, navigation,
feeding, item pickup/throw, treasure queries, effect application, spawning,
damage, loot and XP. The client receives metadata/equipment/motion and
events, speculates interaction consumption through generic stack handling,
emits velocity trails and event particles locally, and selects/animates the
adult or baby model, texture and carried item. Server corrections remain
authoritative.

**Observability:**

Observe registration and both collision boxes, four metadata slots and NBT,
air versus moisture clocks, `NoAI`, flop/trail RNG, all goal flags/priorities,
jump latch/pitch, swimmer effect renewal, every Fish and age-lock branch,
treasure tag/search/waypoint/event behavior, item pickup/throw/cooldown,
placement and five-biome census, group babies/caps/despawn, loot/equipment/
XP/tags/sounds, zero-template and migration closure, and both client
models/textures/item transforms.

**Persistence and reload:**

Generic state plus `Age`, `ForcedAge`, `AgeLocked`, `GotFish`, `Moistness`,
air and held equipment persist. Treasure position, stuck/player/item goal
state, jump latch, play cooldown and render interpolation do not. Code fixes
registration, goals, placement and schemas. Biomes, entity/item/structure
tags, loot, Egg components, effect and advancement reload through their
owners; language, sounds, particle atlas, icon and model textures are client
resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.AgeableMob`;
`net.minecraft.world.entity.animal.AgeableWaterCreature`;
`net.minecraft.world.entity.animal.dolphin.Dolphin` and all three inner
goals; `BreathAirGoal`, `TryFindWaterGoal`, `DolphinJumpGoal`,
`SmoothSwimmingMoveControl`, `SmoothSwimmingLookControl`, `LivingEntity`;
`MobEffects`, `AbstractBoat`, `AbstractNautilus`, `Guardian`;
`EntityRenderers`, `DolphinRenderer`, `DolphinRenderState`,
`DolphinCarryingItemLayer`, `DolphinModel`, `BabyDolphinModel`,
`SuspendedTownParticle`, `LayerDefinitions`; `EntityUUIDFix`,
`MobEffectIdFix`, `V1470`, `V705`; reports, tags, loot, advancement, all 66
biomes, all 1,212 templates, Egg resources, effect icon, particle resource,
two entity textures, locked sounds and language. Complete compiled/data
identity searches find no other exact runtime path.

**Test vectors:**

Run `EXP-ENT-021` across air/moisture/NoAI/flop/trail states, every goal and
RNG boundary, adult/unlocked-baby/locked-baby feeding, all treasure search
and event outcomes, pickup/play/throw/guaranteed-drop paths, placement,
groups/biomes/caps, loot/smelting/XP/tags/Egg, migrations/templates/sounds,
Dolphin's Grace and exact adult/baby/carried-item client projection.

**Limits:**

Generic lifecycle, age locking, goal scheduling, path search, damage/death,
natural spawning, despawn, loot/equipment evaluation, Spawn Egg, metadata,
effects, particles and rendering retain their owners. Fish item behavior,
structure generation and Impaling retain their leaves. This leaf fixes exact
Dolphin dispatch and every direct join selecting it.
