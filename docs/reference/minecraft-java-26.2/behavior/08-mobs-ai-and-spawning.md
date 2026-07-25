# 08 — Mob Spawning, Despawning, AI, and Husbandry

Concrete mobs' spawn placement, biome list, goals, sensors, memories, breeding items, and taming
chance come from type implementations and `OFF-DATA-001`. This page does not flatten every mob into
one AI.

## `MOB-001` Natural spawning applies both global category and per-player local caps

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.NaturalSpawner#createState(int,java.lang.Iterable,net.minecraft.world.level.NaturalSpawner$ChunkGetter,net.minecraft.world.level.LocalMobCapCalculator)`;
`net.minecraft.world.level.NaturalSpawner#getFilteredSpawningCategories(net.minecraft.world.level.NaturalSpawner$SpawnState,boolean,boolean)`;
`net.minecraft.world.level.LocalMobCapCalculator#canSpawn(net.minecraft.world.entity.MobCategory,net.minecraft.world.level.ChunkPos)`;
`net.minecraft.world.entity.MobCategory#getMaxInstancesPerChunk()`; `COM-WIKI-MOB-001`

### Applies when

The server's active-chunk phase considers natural spawning for a `MobCategory`.

### Behavior and timing

Each round first counts existing entities into `SpawnState`. The global category cap scales its
per-category max by spawnable chunks relative to `289`; a category at cap leaves the round's
candidate list. A local cap also counts per non-spectator player near a candidate chunk. Each
successful spawn immediately increments relevant counts and affects later attempts in the same
round.

### Boundaries and quirks

Persistent categories, misc entities, structures, spawners, and chunk-generation spawning need not
use the same cap. Overlapping player regions affect multiple local counts. One world-wide
`mob_count` is insufficient.

### Verification

**Owners:** `MOB-SPAWN-001`, `MOB-HOSTILE-GATE-001`, `MOB-PATROL-001`,
`MOB-PHANTOM-SPAWN-001`, `MOB-WANDERING-TRADER-001`, `MOB-WARDEN-SPAWN-001`, `MOB-RAID-001`;
`EXP-MOB-*`

Lock spawnable-chunk boundary, rounding formula, overlapping-player local counts, and whether
same-tick removal enters the initial snapshot. Also lock the startup/live `spawnEnemies` projection,
its natural-category effect and custom-spawner consumers. `MOB-RAID-001` owns the distinct
event-spawned raider waves, membership and completion lifecycle rather than natural caps.

## `MOB-002` Natural spawning makes pack attempts and fully validates every individual

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.NaturalSpawner#spawnForChunk(net.minecraft.server.level.ServerLevel,net.minecraft.world.level.chunk.LevelChunk,net.minecraft.world.level.NaturalSpawner$SpawnState,java.util.List)`;
`net.minecraft.world.level.NaturalSpawner#spawnCategoryForChunk(net.minecraft.world.entity.MobCategory,net.minecraft.server.level.ServerLevel,net.minecraft.world.level.chunk.LevelChunk,net.minecraft.world.level.NaturalSpawner$SpawnPredicate,net.minecraft.world.level.NaturalSpawner$AfterSpawnCallback)`;
`net.minecraft.world.level.NaturalSpawner#isValidSpawnPostitionForType(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.MobCategory,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.world.level.biome.MobSpawnSettings$SpawnerData,net.minecraft.core.BlockPos$MutableBlockPos,double)`;
`net.minecraft.world.level.NaturalSpawner#isValidEmptySpawnBlock(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState,net.minecraft.world.entity.EntityType)`

### Applies when

`MOB-001` allows a category to attempt in a selected natural-spawn chunk.

### Behavior and timing

A random chunk position starts several offset candidates for a pack. Each candidate reselects
biome/structure spawn entry and checks player/world-spawn distance, chunk/border, block/fluid
emptiness, entity-type placement, collision, light/difficulty, and the mob's own
`checkSpawnRules`/`checkSpawnObstruction`. Success invokes `finalizeSpawn`, adds the entity, and
updates caps. Pack/cap limits terminate attempts.

### Boundaries and quirks

Generic natural spawning rejects candidates within `24` blocks of the nearest player or world spawn
and constrains candidates to at most `8` chunks / `128` blocks; structure overrides and concrete
types still alter the spawn list. Other spawn reasons bypass different subsets.

### Verification

**Owners:** `MOB-SPAWN-001`, `BLK-TINTED-GLASS-001`, `BLK-GLASS-001`,
`BLK-STAINED-GLASS-001`, `BLK-CONCRETE-001`, `BLK-TERRACOTTA-001`,
`BLK-GLAZED-TERRACOTTA-001`, `BLK-QUARTZ-001`, `BLK-SANDSTONE-001`,
`BLK-STONE-VARIANT-001`, `BLK-STONE-BRICK-001`, `BLK-BEACON-STORAGE-001`,
`BLK-RAW-STORAGE-001`, `BLK-LAPIS-BLOCK-001`, `BLK-DEEPSLATE-001`, `BLK-SLIME-001`,
`BLK-HONEY-001`, `BLK-HONEYCOMB-BLOCK-001`, `BLK-BRICKS-001`, `BLK-PACKED-MUD-001`, `BLK-MUD-BRICKS-001`, `BLK-PURPUR-BLOCK-001`, `BLK-RED-NETHER-BRICKS-001`, `BLK-NETHER-WART-BLOCK-001`, `BLK-WARPED-WART-BLOCK-001`, `BLK-NETHER-SPROUTS-001`, `BLK-NETHER-ROOTS-001`, `BLK-NETHER-STEM-001`, `BLK-CORAL-BLOCK-001`, `BLK-CORAL-PLANT-001`, `BLK-FLOWER-POT-001`, `BLK-COPPER-FULL-001`, `BLK-SAPLING-001`, `BLK-BAMBOO-001`, `BLK-ANCIENT-DEBRIS-001`, `BLK-STEM-CROP-001`, `BLK-SOUL-SAND-001`, `BLK-MAGMA-001`,
`BLK-LAVA-CAULDRON-001`, `BLK-TORCHFLOWER-CROP-001`;
`EXP-MOB-*`, `EXP-BLK-033`, `EXP-BLK-034`, `EXP-BLK-035`, `EXP-BLK-036`, `EXP-BLK-037`,
`EXP-BLK-038`, `EXP-BLK-039`, `EXP-BLK-040`, `EXP-BLK-041`, `EXP-BLK-042`, `EXP-BLK-043`,
`EXP-BLK-044`, `EXP-BLK-045`, `EXP-BLK-046`, `EXP-BLK-047`, `EXP-BLK-048`, `EXP-BLK-049`,
`EXP-BLK-050`, `EXP-BLK-055`, `EXP-BLK-058`, `EXP-BLK-059`, `EXP-BLK-060`, `EXP-BLK-061`, `EXP-BLK-062`, `EXP-BLK-063`, `EXP-BLK-064`, `EXP-BLK-065`, `EXP-BLK-066`, `EXP-BLK-067`, `EXP-BLK-069`, `EXP-BLK-070`, `EXP-BLK-071`, `EXP-BLK-072`, `EXP-BLK-073`, `EXP-BLK-074`, `EXP-BLK-075`, `EXP-BLK-076`, `EXP-BLK-077`, `EXP-BLK-079`

Extract attempts/pack termination, surface selection, and special-structure overrides per
category/type into fixtures.
`BLK-TINTED-GLASS-001` fixes its copied never-spawn support predicate and the separate exact
`LEGACY_IRON_GOLEM` floor rejection used by villager summon searches; generic candidate traversal,
above-cell checks and summon commit stay with the mob owners.
`BLK-GLASS-001` fixes the corresponding registered never-spawn predicate and the strategy's
separate exact plain-glass rejection under the same traversal and commit owners.
`BLK-STAINED-GLASS-001` fixes the never-spawn predicate for all sixteen colors and the strategy's
class-wide `StainedGlassBlock` rejection before above-cell/solidity checks.
`BLK-CONCRETE-001` fixes the opposite ordinary-block boundary for all sixteen colors: the full
sturdy upper face and emission 0 pass the default spawn-support predicate, while entity-specific
placement checks remain here.
`BLK-TERRACOTTA-001` fixes the same generic full-sturdy/emission-0 support for all seventeen
identities. Armadillo's additional floor tag accepts only plain, white, yellow, orange, red, brown
and light-gray terracotta through `badlands_terracotta`, then independently requires sufficient
brightness; caps, biome selection, pack traversal and insertion remain here.
`BLK-GLAZED-TERRACOTTA-001` fixes ordinary full-sturdy/emission-0 spawn support for every color and
facing. Its grouping tag adds no entity-specific placement exception; those predicates remain with
the entity-type owners.
`BLK-QUARTZ-001` fixes the same ordinary full-sturdy/emission-0 support for all seven full-cube
states. Pillar axis and the five items' slow-bouncy sulfur-cube membership do not alter the
world-block spawn-support predicate; entity-specific gates remain with entity owners.
`BLK-SANDSTONE-001` fixes the same ordinary full-sturdy/emission-0 support for all eight
full-cube states. Color, strength profile and the items' slow-bouncy sulfur-cube membership do not
alter the world-block spawn-support predicate; entity-specific gates remain with entity owners.
`BLK-STONE-VARIANT-001` fixes the same ordinary full-sturdy/emission-0 support for states 2..7.
Only raw granite, diorite and andesite additionally enter `bats_spawnable_on` through
`base_stone_overworld`; bat height, random, brightness and generic mob gates remain with their
owners.
`BLK-DEEPSLATE-001` fixes the same ordinary full-sturdy/emission-0 support for states 30416..30418
and enters `bats_spawnable_on` through `base_stone_overworld`. Axis does not affect support; bat
height, random, brightness and generic mob gates remain with their owners.
`BLK-STONE-BRICK-001` fixes ordinary full-sturdy/emission-0 support for all four states. Matching
infested hosts can spawn silverfish under `BLK-BREAK-HOOK-001`, but that separate block callback
does not alter the ordinary hosts' spawn-support predicate.
`BLK-BEACON-STORAGE-001` fixes iron block as the four-cell body of every oriented iron-golem
pattern and gold block as the guarded block/loved nonbarter item. It also fixes the five items'
slow-flat/slow-bouncy sulfur-archetype memberships; generic spawn admission, piglin anger/brain
arbitration and sulfur-cube contact/knockback remain with their owning rules.
`BLK-RAW-STORAGE-001` fixes raw gold block as a guarded block and loved nonbarter item, while all
three raw-storage items select slow-flat. Their full-sturdy/emission-0 states otherwise use
ordinary spawn support; piglin anger/brain arbitration and sulfur-cube contact/knockback remain
with their owning rules.
`BLK-LAPIS-BLOCK-001` fixes ordinary full-sturdy/emission-0 spawn support for state 565 and the
item's slow-bouncy membership. That item selector does not alter the world-block support predicate;
sulfur-cube admission, contact and knockback remain with their owning rules.
`BLK-SLIME-001` fixes the opposite inherited boundary: its full sturdy top face and zero emission
pass the default support predicate, with entity-specific placement checks remaining here.
`BLK-HONEY-001` fixes a reduced support shape that fails the default full-top-face spawn predicate;
its snow-layer override is a separate tag consumer and does not grant entity spawn support.
`BLK-HONEYCOMB-BLOCK-001` fixes ordinary full-sturdy/emission-0 support for state 21817. Its
item's sticky sulfur-cube membership affects equipment matching only; entity-specific placement,
contact and knockback gates remain with their owning rules.
`BLK-BRICKS-001` fixes ordinary full-sturdy/emission-0 support for state 2340. Its item's
slow-bouncy membership affects equipment matching only; entity-specific placement, contact and
knockback gates remain with their owning rules.
`BLK-PURPUR-BLOCK-001` fixes ordinary full-sturdy/emission-0 support for state 14712; its
slow-bouncy membership affects equipment matching only.
`BLK-RED-NETHER-BRICKS-001` fixes ordinary full-sturdy/emission-0 support for state 14847; its
slow-bouncy membership affects equipment matching only.
`BLK-NETHER-WART-BLOCK-001` fixes ordinary full-sturdy/emission-0 support for state 14846, then
owns three stronger exact-identity vetoes: Hoglin and Piglin reject this block immediately below,
and Zombified Piglin requires non-Peaceful before the same rejection. Those predicates use no RNG;
the remaining natural-spawn pipeline stays with `MOB-SPAWN-001`.
`BLK-WARPED-WART-BLOCK-001` fixes the contrasting identity boundary: state 20959 has ordinary
full-sturdy/emission-0 support, but none of those three predicates rejects it because each compares
only exact Nether wart block.
`BLK-NETHER-SPROUTS-001` fixes the non-support boundary: state 20961 has empty collision and no
sturdy upper face, so it cannot serve as an ordinary spawn floor. Its AIR-pathfindable vegetation
classification does not bypass entity-specific spawn or obstruction gates.
`BLK-NETHER-ROOTS-001` fixes that same non-support boundary for root states 20960/21031 and the
non-full support boundary for their small potted forms. Root membership in `enderman_holdable`
instead affects an Enderman goal and does not make either identity a spawn floor.
`BLK-NETHER-STEM-001` fixes ordinary full-sturdy/emission-0 support for all 24 axis states.
Nested `parrots_spawnable_on` membership additionally admits a parrot above any of the eight
identities when the independent brightness and generic spawn gates pass; axis and stripped state
do not alter that test.
`BLK-CORAL-BLOCK-001` fixes ordinary full-sturdy/emission-0 support for all ten states. Live/dead
identity, adjacent-water truth and the items' fast-flat equipment membership do not alter the
world-block support predicate; entity-specific placement gates remain with their owners.
`BLK-CORAL-PLANT-001` fixes empty collision and no sturdy face for every upright and fan state, so
none supplies an ordinary natural-spawn floor. Their own support predicates do not grant mob-floor
admission.
`BLK-FLOWER-POT-001` fixes the same spawn-floor rejection for its partial 6-by-6-by-6 collision:
no state exposes a full sturdy upper face, regardless of contained plant.
`BLK-COPPER-FULL-001` fixes ordinary full-sturdy, emission-zero spawn-floor support for all 24
states. Only the eight full-block identities are in `copper`: a carved pumpkin or jack o'lantern
above the locked two-block copper pattern clears both cached cells with flags 2 and break event
2001, snaps and attempts to add a copper golem, awards nearby-player criteria, updates both air
neighbors and replaces the captured source with an age/wax-derived copper chest. Every clear,
entity-add and chest-write result is ignored; adjacent chest pairing may unwind source wax, while
the spawned golem receives only the captured weather age and never waxed state.
`BLK-SAPLING-001` fixes spawn-floor rejection for all sixteen states: empty collision and no sturdy
upper face prevent ordinary ground support. Its block `saplings` tag has no production mob/AI
consumer, and the saplings themselves add no spawn or perception callback.
`BLK-BAMBOO-001` fixes the sole `panda_food` item member. Panda food checks and held-item
targeting therefore admit bamboo and join generic panda feeding, ageing, breeding, sitting and
eating behavior; neither bamboo block provides an ordinary full spawn floor.
`BLK-ANCIENT-DEBRIS-001` fixes ordinary full-sturdy, emission-zero spawn-floor support for state
21819 and direct slow-flat item membership. It adds no spawn veto or AI callback; sulfur-cube
matching, installed movement values and contact handling remain with their owners.
`BLK-STEM-CROP-001` fixes both seeds as direct `chicken_food` and `parrot_food` members. Chickens
therefore admit the generic temptation and breeding transaction; parrots consume a seed for their
generic one-in-ten taming attempt but remain nonbreedable. The nonsturdy, emission-zero stem forms
provide no ordinary spawn floor or special AI callback.
`BLK-OVERWORLD-CROP-001`/`EXP-BLK-078` fixes the exact crop/seed animal-tag closure, Ravager
mob-griefing destruction and farmer-villager harvesting. `HarvestFarmland` scans a 3x3x3 candidate
volume, destroys a mature crop, then on a later tick replants from the first tagged block-item
slot; its cached state prevents same-tick replant, and the replant event/sound/shrink still occur
after an ignored write result. Carrot food also reaches the scoped equine consumer effects.
`BLK-TORCHFLOWER-CROP-001`/`EXP-BLK-079` fixes seeds as chicken/parrot/sniffer food and villager
plantable seeds. Farmers can pick up and plant them but never harvest the family: both stored crop
states are below logical max age and the mature replacement is not a crop. Sniffer digging chooses
between seeds and pitcher pod at equal default weight.
`BLK-PITCHER-CROP-001`/`EXP-BLK-080` fixes pods as chicken/parrot food and villager-plantable
seeds, but not sniffer food. Farmers pick up and directly plant lower age zero, emit/place/shrink in
that order, then never harvest it because pitcher crop is not a `CropBlock`. Mature plant
bee-food/flower membership drives generic bee attraction and pollination.
`BLK-SWEET-BERRY-BUSH-001`/`EXP-BLK-081` fixes direct bee growth, fox harvest/food/immunity and
careful-Ghast traversal. A qualified nectared bee can advance bushes one and two cells below,
event before ignored write and counter increment after it. A fox waits 40 ticks, requires
`mob_griefing`, takes 1..2 or 2..3 berries into an empty hand then drops the remainder, resets age
one and emits block change. Foxes and bees bypass bush contact entirely; careful Ghast movement
rejects the tagged cell.
`BLK-CAVE-VINES-001`/`EXP-BLK-082` fixes both segments as bee-growable only while unlit. For each
qualified scan cell, the bee performs the first flags-2 berry write, rereads state, emits event
2011/15, redundantly offers that state and then increments its counter; it can light both depths
without extending either vine. Glow berries' `fox_food` membership drives generic fox temptation
and breeding but adds no fox vine-harvest goal.
`BLK-CHORUS-001`/`EXP-BLK-083` fixes flower membership in `bee_attractive` plus flower-item
membership in `bee_food`, so the generic bee attraction and held-item food consumers can select it.
Neither chorus identity belongs to `bee_growables`, and chorus fruit is not fox food; no bee or fox
callback mutates a live chorus structure.
`BLK-PACKED-MUD-001` fixes ordinary full-sturdy/emission-0 support for state 7758. Its item's
buoyant regular membership affects equipment matching only; entity-specific buoyancy, placement,
contact and knockback gates remain with their owning rules.
`BLK-MUD-BRICKS-001` fixes ordinary full-sturdy/emission-0 support for state 7759. Its item's
slow-bouncy membership affects equipment matching only; entity-specific placement, contact and
knockback gates remain with their owning rules.
`BLK-SOUL-SAND-001` deliberately registers an always-true spawn predicate despite its shortened
collider. Entity-type placement, light, collision and category-specific admission remain here.
`BLK-MAGMA-001` has a full support cube but its registered spawn predicate admits only fire-immune
entity types; every remaining placement, collision, light and category gate remains here.
`BLK-LAVA-CAULDRON-001` has only a rim at the top and keeps the default spawn predicate, so it
does not provide a full sturdy upper face; remaining entity/category admission stays here.

## `MOB-003` Despawning combines persistence, player distance, category ranges, and random checks

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.entity.Mob#checkDespawn()`;
`net.minecraft.world.entity.Mob#requiresCustomPersistence()`;
`net.minecraft.world.entity.Mob#isPersistenceRequired()`;
`net.minecraft.world.entity.Mob#removeWhenFarAway(double)`;
`net.minecraft.world.entity.MobCategory#getNoDespawnDistance()`;
`net.minecraft.world.entity.MobCategory#getDespawnDistance()`; `COM-WIKI-MOB-001`

### Applies when

A non-player mob's server AI tick checks natural despawn.

### Behavior and timing

Peaceful difficulty may first remove hostile mobs that should not exist. Required or type-specific
persistence bypasses ordinary despawn. Otherwise nearest-player distance is used: beyond the
category hard despawn distance and when the type permits, remove immediately; beyond the fixed
`32`-block no-despawn distance, `noActionTime` plus random chance may despawn; proximity resets
idleness.

### Boundaries and quirks

Naming, taming, breeding state, riding/passengers, held/equipped items, and special spawn reasons
may require persistence. Chunk unload into storage is not natural despawn.

### Verification

**Owners:** `MOB-DESPAWN-001`; `EXP-MOB-003`

Audit `requiresCustomPersistence` and `removeWhenFarAway` overrides per type; lock random frequency
and exact-threshold positions.

## `MOB-004` GoalSelector and Brain are composable but distinct AI schedulers

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.entity.Mob#serverAiStep()`;
`net.minecraft.world.entity.ai.goal.GoalSelector#addGoal(int,net.minecraft.world.entity.ai.goal.Goal)`;
`net.minecraft.world.entity.ai.goal.GoalSelector#tick()`;
`net.minecraft.world.entity.ai.Brain#tick(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity)`;
`net.minecraft.world.entity.ai.Brain#setActiveActivityIfPossible(net.minecraft.world.entity.schedule.Activity)`;
`COM-WIKI-MOB-001`

### Applies when

A mob's server AI step runs and its type registered goals and/or Brain behaviors.

### Behavior and timing

`GoalSelector` uses priority and mutually exclusive control flags to stop goals that cannot
continue, replace eligible incumbents, start candidates whose conditions pass, and tick running
goals. `Brain` advances memory expiry and sensors, then starts/stops/ticks behaviors from
core/non-core activity and memory preconditions. A mob may combine navigation, target selector, and
Brain, but their state is not interchangeable.

### Boundaries and quirks

Smaller priority number, equal-priority registration order, non-interruptible goals, disabled flags,
and reduced AI cadence are observable. Ferrite's ECS may differ internally but must preserve
arbitration results.

### Verification

**Owners:** `MOB-AI-001`, `MOB-UNIVERSAL-ANGER-001`, `ITM-ENDER-CHEST-001`,
`ITM-BARREL-001`, `BLK-BELL-001`, `BLK-LAVA-CAULDRON-001`, `BLK-NETHER-ROOTS-001`,
`ITM-STEW-001`, `ITM-HARNESS-001`, `ITM-STEERING-STICK-001`, `ITM-SPEAR-001`,
`ITM-NAUTILUS-ARMOR-001`, `ITM-EGG-001`; `EXP-MOB-002`, `EXP-MOB-010`, `EXP-ITM-008`, `EXP-ITM-009`,
`EXP-ITM-021`, `EXP-ITM-023`, `EXP-ITM-024`, `EXP-ITM-025`,
`EXP-ITM-026`,
`EXP-BLK-009`, `EXP-BLK-039`,
`EXP-BLK-067`, `EXP-ITM-016`

The content leaves fix guarded-container piglin anger and bell `HEARD_BELL_TIME` ingress with exact
memory inputs. Lock the remaining equal-priority traversal, every-tick/reduced goal cadence, Brain
behavior ordering, and recovery after inactive-chunk gating.
`MOB-UNIVERSAL-ANGER-001` fixes the live revenge/reset arbitration and both classic-neutral and
Piglin target/memory models without generalizing their different persistence or toggle behavior.
`BLK-HONEY-001` fixes generic and breeze long-jump startup rejection on exact honey, including the
generic half-sampled cooldown write; scheduler admission and later jump phases remain here.
`BLK-NETHER-ROOTS-001` fixes both root identities' exact Enderman take/leave goal predicates,
reduced bounds, sampled positions, sight ray, drop-suppressed removal, neighbor-shape transform,
placement/discard gates, events and carried-state mutation. Priority arbitration, persistence and
death behavior remain with the mob/entity owners.
`ITM-STEW-001` fixes the interaction-side mob joins rather than scheduler arbitration. An adult
mooshroom resolves bowl milking before flower charge; only an adult brown uncharged variant accepts
an effect flower and persists its ordered component until the next suspicious-stew result clears
it. A tamed injured wolf accepts direct `wolf_food` rabbit stew, heals 20 and consumes one without
running player item-use completion. Generic goal, navigation and inherited interaction behavior
remain with their owners.
`ITM-HARNESS-001` fixes Happy Ghast temptation inputs around the scheduler. An unharnessed adult
uses the live temptation tag containing snowball plus all sixteen harnesses; a baby or validly
harnessed adult uses the food tag containing only snowball. Successful equip therefore removes
harness temptation immediately, and allowed-entity or temptation-tag reload changes future goal
predicates without rewriting the body stack. Goal priority, navigation and sensing remain here.
`ITM-STEERING-STICK-001` fixes distinct pig/strider lure selectors. Pig registers exact carrot on
a stick at priority four/speed 1.2 alongside its independent live food goal. Strider uses one
priority-three/speed-1.4 predicate over live `strider_tempt_items`, which expands strider food and
adds warped fungus on a stick. Reload can therefore remove warped-stick temptation without
changing its code-built mounted controller; neither stick is breeding food.
`ITM-SPEAR-001` fixes kinetic-component selection in zombie/zombified-piglin goal AI and piglin
brain AI. Wielders approach from radius 10, engage for delay plus the tier damage window, charge
and reposition at speed 1, and retreat through the `6..7`/`9..11` distance bands extended by 2
while mounted. Nonplayers use root-vehicle velocity when mounted and multiply speed thresholds by
`0.2`. Zombie, zombie-horse, husk camel-jockey, zombified-piglin and piglin spawn equipment select
iron or golden spear at their locked code-built probabilities.
`ITM-NAUTILUS-ARMOR-001` fixes zombie-nautilus sunlight protection outside goal arbitration. A live
`burn_in_daylight` member that passes the monster-burn, light/RNG, weather/fluid and sky gates uses
BODY as its protection slot. Any scoped armor is nondamageable, so a nonempty stack suppresses the
eight-second ignition without consuming the protector or drawing damage; normal nautilus never
enters this path. Removing allowed-entity membership blocks later insertion but does not remove
stored armor, its attributes, rendering or sunlight protection.
`ITM-EGG-001` fixes chicken laying outside goal arbitration. A new chicken seeds `EggLayTime`
uniformly over `6000..11999`; only an alive adult non-jockey server tick decrements it. Expiry
evaluates one ordered gift-table alternative by live variant: temperate emits ordinary egg, warm
brown, cold blue, and any other variant emits nothing. Success alone plays the two-float-pitched
lay sound and emits `ENTITY_PLACE`, but success and failure both consume a fresh interval draw and
reset the persisted timer.

## `MOB-005` Perception caches and paths are consumed incrementally by AI ticks

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Cross-checked`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.entity.ai.sensing.Sensing#tick()`;
`net.minecraft.world.entity.ai.sensing.Sensing#hasLineOfSight(net.minecraft.world.entity.Entity)`;
`net.minecraft.world.entity.ai.navigation.PathNavigation#recomputePath()`;
`net.minecraft.world.entity.ai.navigation.PathNavigation#moveTo(net.minecraft.world.level.pathfinder.Path,double)`;
`net.minecraft.world.entity.ai.navigation.PathNavigation#tick()`; `COM-WIKI-MOB-001`

### Applies when

AI tests target visibility or moves along a path.

### Behavior and timing

`Sensing` clears seen/unseen caches each mob AI tick and fills them lazily with line-of-sight clips.
`PathNavigation` creates a discrete path from node evaluator/pathfinder, stores speed and current
node, then advances, detects stalls, recomputes or stops over later ticks before handing a movement
target to move control/entity physics.

### Boundaries and quirks

Doors, fluids, danger malus, size, chunk boundaries, and dynamic blocks alter node feasibility.
Vanilla compute budgets and tie-breaks may create quirks, but Ferrite targets equivalent
player-visible route, reachability, and response timing rather than an identical internal open set.

### Verification

**Owners:** `MOB-AI-001`, `BLK-HONEY-001`, `BLK-NETHER-STEM-001`, `BLK-CORAL-BLOCK-001`,
`BLK-FLOWER-POT-001`, `BLK-COPPER-FULL-001`,
`BLK-SOUL-SAND-001`, `BLK-MAGMA-001`,
`BLK-LAVA-CAULDRON-001`, `ITM-HARNESS-001`, `ITM-STEERING-STICK-001`, `ITM-SPEAR-001`,
`ITM-NAUTILUS-ARMOR-001`; `EXP-MOB-002`, `EXP-ITM-021`, `EXP-ITM-023`, `EXP-ITM-024`,
`EXP-ITM-025`,
`EXP-BLK-036`, `EXP-BLK-037`, `EXP-BLK-038`,
`EXP-BLK-039`, `EXP-BLK-069`, `EXP-BLK-070`, `EXP-BLK-072`, `EXP-BLK-073`

The honey leaf fixes exact `STICKY_HONEY` classification, default malus 8.0 and the no-extra-step
branch. This parent retains node expansion, mob overrides, path selection and incremental use.
The magma leaf fixes exact burning-block recognition and careful Ghast rejection through
`happy_ghast_avoids`; this parent retains path type/malus assignment, traversal and route choice.
The lava-cauldron leaf fixes three distinct inputs: every path type is rejected, exact lava
cauldron is burning, and the reloadable `cauldrons` tag lifts current/eligible following path nodes
by one. Its hardcoded state also belongs to the leatherworker POI; navigation, job claiming and
profession transitions remain with this parent.
The soul-sand leaf makes every queried path-computation type return false at the block hook; node
expansion, entity overrides and route selection remain with this parent.
The harness leaf fixes which live tag supplies Happy Ghast temptation targets before path
selection: only an adult without valid body equipment includes harnesses, while babies and
validly equipped adults follow snowball alone. This parent retains target scans, path construction
and incremental pursuit.
The Nether-stem leaf fixes the parrot wander goal's `logs` candidate: an air destination may be
selected above any of the eight blocks only when the destination and its upper neighbor are empty.
Its item also selects the bouncy sulfur archetype with powers 0.4125/0.105; goal traversal,
equipment matching and knockback application remain with their owners.
The coral-block leaf fixes all ten items' direct fast-flat sulfur-archetype membership with
horizontal/vertical powers 0.9125/0.09 and hit sound `entity.sulfur_cube.fast_flat.hit`;
equipment matching, contact admission and knockback application remain with their owners.
The flower-pot leaf fixes the sole AI-specific filled form. A default-period hoglin sensor searches
8 horizontally and 4 vertically for `hoglin_repellents`; potted warped fungus can set the nearest
memory, erase attack target, pacify for 200 ticks and request a speed-1 walk target 8 blocks away.
The other pot states, and the piglin-repellent tag, do not take this branch.
The full-copper leaf fixes direct `slow_flat` sulfur-cube-archetype membership for all 24 items:
horizontal/vertical powers 0.4125/0.105, cooldown 0.9, threshold 0.03, resistance 0.5/0.5,
bounciness 0.4000000059604645, friction -0.5999999940395355 and air drag
-0.8999999985098839. Equipment matching, contact admission and knockback application retain their
generic owners.

Define allowed route divergence and add reachability cases for doors/water/narrow spaces, dynamic
blockage, moving targets, and unavailable chunks.

## `MOB-006` Breeding and taming commit type-validated persistent state transitions

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`

### Primary evidence

`OFF-SERVER-001`; `OFF-DATA-001`;
`net.minecraft.world.entity.animal.Animal#canMate(net.minecraft.world.entity.animal.Animal)`;
`net.minecraft.world.entity.animal.Animal#spawnChildFromBreeding(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.animal.Animal)`;
`net.minecraft.world.entity.animal.Animal#finalizeSpawnChildFromBreeding(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.animal.Animal,net.minecraft.world.entity.AgeableMob)`;
`net.minecraft.world.entity.TamableAnimal#tame(net.minecraft.world.entity.player.Player)`;
`net.minecraft.world.entity.TamableAnimal#setOwner(net.minecraft.world.entity.LivingEntity)`;
`COM-WIKI-MOB-001`

### Applies when

Two compatible animals in love can approach, or a player uses a type-approved item for a taming
attempt.

### Behavior and timing

Breeding validates both ages, love causes, type compatibility, and `canMate`, then creates
offspring, sets parent age/cooldown, clears love, attributes the player criterion, and emits
XP/events. A concrete mob interaction chooses taming chance and consumption; success calls `tame` to
persist owner reference, tame flags, and related AI/events.

### Boundaries and quirks

Variant inheritance, crossbreeding, player disconnect, mob griefing, sit commands, and failed taming
are concrete extensions. Feeding to heal and feeding to enter love are not one generic action.

### Verification

**Owners:** `MOB-BREED-001`; `EXP-MOB-004`

Generate a condition table from source/data for every breedable/tamable type. Generic commit
ordering is cross-checked, but content coverage is incomplete.
