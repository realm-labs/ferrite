# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-PATROL-001` — Patrol spawning advances a pausable timer into one player-relative pillager group attempt

**Parent:** `MOB-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — installation scope, every admission gate, timer mutation, RNG draw, candidate
walk, member failure and pillager finalization boundary are explicit in locked source and data.

**Applies when:**

The nondebug Overworld chunk source runs normally, `spawn_mobs` has admitted custom spawners, and
its single `PatrolSpawner` receives the cached hostile-spawn Boolean.

**Authoritative state:**

The spawner's nonpersisted signed `nextTick`, the `spawn_patrols` gamerule, cached hostile policy,
outside brightness, ordered level player list, level RNG, nearby-village/chunk state, positional
`CAN_PILLAGER_PATROL_SPAWN`, regional difficulty, heightmap/block/light/support state, each created
pillager's RNG and insertion/finalization state.

**Transition and ordering:**

**Installation and caller gates:** `MinecraftServer#createLevels` constructs one patrol spawner in
the five-entry custom-spawner list passed only to the Overworld; every other level receives an
empty list. A debug level or frozen tick-rate manager never reaches custom spawning. Within a
normal chunk tick, custom spawners run after block/random chunk ticks and only while `spawn_mobs`
is true. Patrol then returns first when cached hostile policy is false and second when the live
`spawn_patrols` rule is false. Both paths leave `nextTick` and RNG unchanged. The rule is a
`SPAWNING` Boolean whose default is true.

**Timer and attempt admission:** With both gates true, predecrement `nextTick`; a positive result
returns without RNG. A nonpositive result first adds `12000+nextInt(1200)` to the decremented
value, committing the next schedule before any later failure. The newly constructed spawner starts
at zero, so its first admitted call stores `11999..13198`; an ordinary later expiry reaches zero
after decrement and stores `12000..13199`. This field is neither saved nor reloaded, so a server
restart creates the zero state again.

After scheduling, require `isBrightOutside()`, then consume `nextInt(5)` and continue only on zero.
Read the player-list size and return at zero; otherwise consume `nextInt(size)` and choose exactly
that entry. A selected spectator aborts without retry, while creative status is not rejected.
Reject a player for which `isCloseToVillage(player.blockPosition(),2)` is true.

For X and Z independently, consume `nextInt(24)` and `nextBoolean`, producing signed offset
`±(24..47)` from the selected player's block position. Require all chunks intersecting the
inclusive block rectangle from candidate X/Z minus 10 through plus 10. Next sample the positional
`minecraft:gameplay/can_pillager_patrol_spawn` attribute at that initial candidate. Its registered
default is true; the Overworld `early_game` timeline combines by `and` as false from clock tick 0
until the keyframe at 120000, while the mushroom-fields biome supplies a false positional layer.
These late gates do not roll back the timer, chance or player/offset draws already consumed.

**Group walk:** Set attempt count to `ceil(effectiveDifficulty(candidate))+1`. For each index,
replace mutable Y with `MOTION_BLOCKING_NO_LEAVES` height at its current X/Z. Index zero is the
leader: failure to spawn it ends the entire group before horizontal walk RNG. Every later member's
Boolean result is ignored. After every successful leader or attempted follower, mutate X by
`nextInt(5)-nextInt(5)` and then Z by the same two-draw formula. Y is resampled on the next index.
The initial 21-by-21 loaded-block test is not repeated after these walks.

**Member transaction:** At the current integer position, first require
`NaturalSpawner#isValidEmptySpawnBlock` for a pillager. Then require block light at most 8 and the
block below to be a valid pillager spawn support; `PATROL` is not a spawner reason. This direct
predicate path has no separate Peaceful-difficulty, sky-light, `SpawnPlacements`, AABB collision,
obstruction or nearby-player check. Failure returns false without construction.

Success constructs one pillager with reason `PATROL`. Null construction returns false. For the
leader, `setPatrolLeader(true)` first marks it patrolling, then `findPatrolTarget` consumes two draws
from the pillager's own RNG and offsets its current pre-placement block position by
`-500+nextInt(1000)` on X/Z. Only afterward does the caller set the pillager to the candidate's
exact integer X/Y/Z. Thus the leader target is based on its constructed pre-placement position,
not the eventual spawn position.

Finalization uses current regional difficulty and `PATROL`: Pillager equips a crossbow and runs
spawn-enchantment logic; the already marked leader receives an ominous banner in its head slot
with drop chance `2`, and every member becomes patrolling. The caller then invokes
`addFreshEntityWithPassengers` and returns true without an insertion-result rollback. A leader
success therefore permits followers even if insertion did not retain it; an individual follower
failure does not shorten the configured loop.

**Branches and aborts:**

Absent Overworld/custom-spawner call; debug/frozen tick; false `spawn_mobs`, cached hostile policy
or `spawn_patrols`; positive countdown; dark/fixed-time outside; four-in-five chance failure; empty
players; selected spectator; nearby village; unloaded initial square; false timeline/biome
attribute; leader/member empty-block, light or support failure; null entity construction. Every
failure preserves schedule and consumes only draws already reached.

**Constants and randomness:**

Timer addition `12000+nextInt(1200)`; attempt chance `1/5`; player offset `±24..47`; initial loaded
margin 10; village argument 2; group count `ceil(effectiveDifficulty)+1`; member walk four
`nextInt(5)` draws; block-light maximum 8; leader target offsets `-500..499` from two entity-RNG
draws. Level and pillager RNG streams are distinct.

**Side effects:**

Ephemeral timer mutation; level RNG advancement; pillager construction, patrol leader/target and
patrolling state; crossbow/enchantment/banner equipment; entity insertion. This rule neither
persists the cadence nor owns later patrol AI, raid conversion, combat, despawn or item drops.

**Gates:**

Overworld installation, normal nondebug chunk ticking, `spawn_mobs`, cached hostile policy,
`spawn_patrols`, timer, outside brightness, chance, selected player, village/chunk/environment
state and the exact member predicates above.

**Boundary cases and quirks:**

Disabling either spawning rule pauses the timer. Every post-expiry failure nevertheless schedules
the next attempt. The single selected player is not retried when spectator/near-village. The early
game and mushroom-field attribute gates occur after schedule/chance/player/offset work. Peaceful is
not an admission gate here. Leader target selection precedes placement, follower failure is
nonterminal, and insertion is not checked.

**Evidence:**

`OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.server.MinecraftServer#createLevels`;
`net.minecraft.server.level.ServerChunkCache#tickChunks`;
`net.minecraft.server.level.ServerLevel#tickCustomSpawners`, `#isCloseToVillage`,
`#hasChunksAt`, `#getHeightmapPos`;
`net.minecraft.world.level.levelgen.PatrolSpawner#tick`, `#spawnPatrolMember`;
`net.minecraft.world.level.NaturalSpawner#isValidEmptySpawnBlock`;
`net.minecraft.world.entity.monster.PatrollingMonster#checkPatrollingMonsterSpawnRules`,
`#setPatrolLeader`, `#findPatrolTarget`, `#finalizeSpawn`;
`net.minecraft.world.entity.monster.illager.Pillager#finalizeSpawn`;
`net.minecraft.world.attribute.EnvironmentAttributes`;
`WGEN-DIMENSION-001`; `MOB-HOSTILE-GATE-001`; `EXP-MOB-006`.

**Test vectors:**

Cross first/subsequent expiry endpoints and restart; pause/resume each caller/rule gate; force all
post-schedule failures; use empty/mixed/all-spectator player lists with deterministic indices;
cross ±24/47 offsets, loaded margin and village argument; sample early-game ticks 0/119999/120000
and mushroom-field boundaries; sweep effective-difficulty ceilings, heightmap/light/support
states, leader failure, follower failure, null creation and ignored insertion. Assert both RNG
streams, timer, loop length, positions, target, equipment and patrolling state.
