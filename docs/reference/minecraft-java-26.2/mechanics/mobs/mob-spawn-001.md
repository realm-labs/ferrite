# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-SPAWN-001` — Natural spawning is a category-cap, chunk, position, and mob-rule pipeline

**Parent:** `MOB-001`, `MOB-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — cap arithmetic, candidate selection, pack termination, structure overrides,
and per-species spawn rules remain unexpanded.

**Applies when:**

The server chunk source runs natural spawning for eligible chunks and mob categories.

**Authoritative state:**

Eligible/ticking chunks, non-spectator players, per-category counts/caps, spawn potentials/biome
structure data, difficulty, gamerules, local light/fluid/block state and RNG.

**Transition and ordering:**

Build the spawn state/counts; for each eligible chunk and category present in the caller-supplied
category list and below its scaled cap, choose candidate positions and biome/structure spawn
entries; perform pack attempts; validate distance from players/world spawn, category placement,
collision, light and entity-specific spawn rules; create/finalize/add accepted mobs; update counts
so later attempts see them. Anchor:
`net.minecraft.world.level.NaturalSpawner#spawnForChunk(net.minecraft.server.level.ServerLevel,net.minecraft.world.level.chunk.LevelChunk,net.minecraft.world.level.NaturalSpawner$SpawnState,java.util.List)`.

**Branches and aborts:**

Category cap met; no eligible players/chunk; gamerule false; peaceful; invalid
Y/block/fluid/light/collision; too near/far; weighted entry fails; pack budget reached; entity
finalization/addition rejected. Failed attempts consume only the RNG reached by that branch.

**Constants and randomness:**

Category base caps, eligible chunk scaling, attempt/pack limits and distance thresholds are source
constants. Weighted selection and candidate offsets consume the level RNG in control-flow order.
Exact constants/RNG trace: `EXP-MOB-001`.

**Side effects:**

Entity creation/finalization/equipment, group data, jockey/passenger entities, category counts, game
events and later tracking. Failed attempts do not reserve cap.

**Gates:**

`doMobSpawning`, difficulty, chunk block/entity ticking eligibility, player spectator
status/distance, category cap, biome/structure spawn data, placement rules, local conditions and
entity feature flags.

**Boundary cases and quirks:**

Caps are global-per-level scaled by eligible chunks, not per chunk. Spawn chunks and simulation
distance affect eligibility differently. Some mobs use special spawners outside this pipeline.

**Evidence:**

`Confirmed` pipeline; exact iteration/RNG `Cross-checked`; `OFF-SERVER-001`, `OFF-DATA-001`; locator
above; `EXP-MOB-001`.

**Test vectors:**

One/two players with overlapping and separate eligible chunks; cap boundary; spectator only;
peaceful; biome/structure override; fail collision then succeed; fixed seed trace of pack attempts.
