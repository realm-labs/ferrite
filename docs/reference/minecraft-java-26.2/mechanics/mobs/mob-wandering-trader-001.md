# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-WANDERING-TRADER-001` — Persisted delay and escalating chance admit one trader with up to two llamas

**Parent:** `MOB-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — installation, pausable timer, persisted state, both RNG streams, player and
meeting-point selection, placement, construction, llama attachment and failure boundaries are
explicit in locked source and bundled data.

**Applies when:**

The nondebug Overworld chunk source runs normally, `spawn_mobs` admits custom spawners, and its
single `WanderingTraderSpawner` reaches an enabled `spawn_wandering_traders` tick.

**Authoritative state:**

The spawner's nonpersisted `tickDelay` and independent RNG; persisted `wandering_trader`
`spawn_delay` and `spawn_chance`; live gamerules; alive level players and level RNG; POI, heightmap,
world-border, block, fluid, biome and collision state; and constructed trader/llama entity state.

**Transition and ordering:**

**Installation and caller gates:** `MinecraftServer#createLevels` installs one wandering-trader
spawner in the five-entry custom-spawner list passed only to the Overworld; other levels receive an
empty list. Debug or frozen chunk ticking does not call it. Normal custom spawning occurs after
block/random chunk ticks and only while `spawn_mobs` is true. The spawner ignores its cached
hostile-policy Boolean, so `spawn_monsters` does not gate or pause it. It first reads the
`SPAWNING` Boolean `spawn_wandering_traders`, whose default is true; false returns before timer,
saved-data load or RNG.

**Two-level timer and persistence:** A new spawner has nonpersisted `tickDelay=1200`. Each admitted
call predecrements it; a positive result returns, while a nonpositive result is replaced with 1200
before saved state is touched. The first due call lazily loads or creates saved-data type
`minecraft:wandering_trader`. Its codec has optional integer `spawn_delay` and `spawn_chance`
fields, defaulting to 24000 and 25. A setter marks the record dirty only when the value changes.

On every due call, subtract 1200 from persisted delay and store the result. A positive result
returns. Otherwise store delay 24000, capture the old chance, then store
`clamp(oldChance+25,25,75)` before any chance draw or spawn work. Consume the spawner RNG's
`nextInt(100)` and proceed exactly when `draw <= oldChance`. Consequently ordinary old chances
25/50/75 admit 26/51/76 outcomes out of 100, not 25/50/75. Arbitrary decoded integers are not
validated before this comparison; the next stored chance is still clamped. A failed comparison or
failed spawn keeps the already increased chance. A successful spawn resets it to 25.

With uninterrupted gates and ordinary state, saved delay reaches an attempt every twenty due calls,
or 24000 custom-spawner calls. Disabling either `spawn_mobs` or this rule pauses both timer layers.
Restart preserves saved delay/chance but reconstructs the 1200-call phase and independent RNG.

**Player and meeting target:** On an admitted chance, `ServerLevel#getRandomPlayer` filters the
level player list only by `LivingEntity#isAlive`. No alive player returns null, and `spawn` returns
true immediately: no 1-in-10 draw or placement occurs, but the caller resets chance to 25. Otherwise
the level RNG selects one alive entry uniformly; spectators are not excluded. Then the spawner RNG
must return zero from `nextInt(10)` or the spawn fails.

From the selected player's block position, `PoiManager#find` searches meeting POIs with occupancy
`ANY`, an always-true position predicate and inclusive Euclidean radius 48. It selects the first
position in `getInRange` encounter order, not the closest; absence falls back to the player
position. This meeting-or-player position is the target used below.

**Trader candidate:** Make at most ten attempts using the spawner RNG. Each draws X then Z as
`target + nextInt(2*radius)-radius`; radius 48 therefore produces offsets `-48..47`. Y comes from
the wandering trader's `MOTION_BLOCKING_NO_LEAVES` heightmap. The helper deliberately uses the
wandering trader's registered `ON_GROUND` placement type and entity type. That placement requires
the candidate inside the world border, a valid-spawn block below, and
`NaturalSpawner#isValidEmptySpawnBlock` at both the candidate and the block above. The first valid
candidate wins; ten failures return null. The registered spawn predicate is not called, and there
is no entity AABB, nearby-player or full spawn-rule check.

For a trader candidate only, every block collision shape in the inclusive box from the candidate
through `offset(1,2,1)` must be empty: twelve positions in a 2-by-3-by-2 volume. This additional
test does not inspect entities or use the trader AABB. Finally reject a biome in
`minecraft:without_wandering_trader_spawns`; locked data contains exactly `minecraft:the_void`.

**Trader construction and insertion:** Spawn `WANDERING_TRADER` with reason `EVENT`. The selected
overload disables Y adjustment: it creates the entity, snaps to `(x+0.5,y,z+0.5)` with
level-RNG yaw and pitch zero, aligns head/body yaw, and finalizes at current regional difficulty
with null group data. WanderingTrader inherits Mob finalization, including the follow-range random
bonus and left-handed trial. The helper then calls void `addFreshEntityWithPassengers`, plays the
ambient sound and returns the nonnull object without observing whether insertion retained it.

Null construction fails. A nonnull trader proceeds even after ignored insertion failure. After
both llama attempts below, set trader despawn delay 48000, wander target to the meeting/fallback
position and Mob home to the same position with radius 16. These target fields use the POI/player
position, not the sampled spawn position.

**Two independent llamas:** Call the llama helper exactly twice in sequence. Each independently
searches up to ten positions at radius 4 around the trader object's current block position, with
X/Z offsets `-4..3`. Crucially it again uses the wandering trader's heightmap and `ON_GROUND`
placement checks, even though `TRADER_LLAMA` is registered as `NO_RESTRICTIONS`. Llamas receive no
2-by-3-by-2 space check and no biome-tag check.

A found position spawns `TRADER_LLAMA` with reason `EVENT` through the same unchecked insertion
helper. Its constructor starts despawn delay at 47999; EVENT finalization forces adult age, creates
fresh ageable group data, then Llama finalization chooses random strength and variant before the
remaining superclass/Mob initialization. A nonnull object is leashed to the trader with broadcast
enabled, again regardless of whether either insertion was retained. Null candidate/construction
skips only that llama, so zero, one or two llama objects may be produced.

**Branches and aborts:**

Absent Overworld/custom-spawner call; debug/frozen tick; false `spawn_mobs` or
`spawn_wandering_traders`; positive timer layer; positive saved delay; inclusive chance failure;
1-in-10 failure; ten invalid candidates; trader space or void-biome rejection; null entity
construction. No-alive-player is the exceptional successful return. Trader failure keeps elevated
chance; llama failures do not roll back the trader or the other llama.

**Constants and randomness:**

Timer 1200, saved delay 24000, chance step/minimum 25 and maximum 75, inclusive `nextInt(100)`
comparison, independent `nextInt(10)==0`, ten position attempts, target radius 48, llama radius 4,
trader space 2-by-3-by-2, two llama calls, despawn delays 48000/47999 and home radius 16. The
spawner RNG owns chance, 1-in-10 and every X/Z sample. Level RNG owns player selection and reached
entity yaw/finalization/sound work; calls interleave in the transaction order above.

**Side effects:**

Nonpersisted timer/RNG advancement; dirty persisted delay/chance; POI/chunk data access; level RNG
advancement; trader and zero-to-two llama construction, finalization, insertion attempts, ambient
sounds, leash state and trader target/home/despawn initialization. The leaf does not own later
offers, trading, AI or despawn ticks.

**Gates:**

Overworld installation, normal chunk ticking, `spawn_mobs`, `spawn_wandering_traders`, both timers,
the inclusive chance and one-in-ten trials, alive-player selection, POI fallback, exact placement,
space, biome, construction and per-llama conditions above.

**Boundary cases and quirks:**

The default chance is off by one because comparison is inclusive. An empty alive-player set counts
as success and resets chance. Spectators can anchor attempts. Meeting selection is encounter-first,
not closest. Radius samples exclude the positive endpoint. Llama searches use trader placement,
and unchecked insertion does not prevent subsequent llama/leash/target state or success.

**Evidence:**

`OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.server.MinecraftServer#createLevels`;
`net.minecraft.server.level.ServerChunkCache#tickChunks`;
`net.minecraft.server.level.ServerLevel#tickCustomSpawners`, `#getRandomPlayer`;
`net.minecraft.world.entity.npc.wanderingtrader.WanderingTraderSpawner#tick`, `#spawn`,
`#findSpawnPositionNear`, `#hasEnoughSpace`, `#tryToSpawnLlamaFor`;
`net.minecraft.world.level.saveddata.WanderingTraderData`;
`net.minecraft.world.entity.ai.village.poi.PoiManager#find`, `#findAll`, `#getInRange`;
`net.minecraft.world.entity.SpawnPlacements`; `net.minecraft.world.entity.SpawnPlacementTypes$1`;
`net.minecraft.world.entity.EntityType#spawn`, `#create`;
`net.minecraft.world.entity.Mob#finalizeSpawn`, `#setHomeTo`;
`net.minecraft.world.entity.npc.wanderingtrader.WanderingTrader#setDespawnDelay`, `#setWanderTarget`;
`net.minecraft.world.entity.animal.equine.TraderLlama#finalizeSpawn`;
`net.minecraft.world.entity.Leashable#setLeashedTo`;
`net.minecraft.world.entity.animal.equine.Llama#finalizeSpawn`;
`data/minecraft/tags/worldgen/biome/without_wandering_trader_spawns.json`;
`MOB-HOSTILE-GATE-001`; `WGEN-DIMENSION-001`; `EXP-MOB-008`.

**Test vectors:**

Cross first/subsequent/restarted timer phases, persisted delay around 0/1200/24000, ordinary and
out-of-range decoded chances, both pausing rules and inclusive draws 24/25/26, 49/50/51 and
74/75/76. Cover no/alive/spectator/multiple players, exact player-RNG selection, 1-in-10, absent and
multiple meeting POIs with encounter order, every radius endpoint and all ten placement failures.
Cross world border, below/candidate/above block-fluid checks, each of twelve collision cells,
`the_void`, null creation and ignored insertion. For both llama calls independently, cover
candidate/construction failure, trader-placement-versus-llama-registration divergence,
finalization, leash broadcast and retained/unretained objects. Assert both RNG cursors, saved dirty
state, entity state/count, target/home/despawn values, sounds and nonrollback.
